//! GPM (General Purpose Mouse) handler for Linux console
//!
//! This module provides FFI bindings to libgpm for mouse support in Linux console
//! (when running outside of X11/Wayland).
//!
//! # About GPM
//!
//! GPM (General Purpose Mouse) is a daemon that provides mouse support for Linux
//! virtual consoles. When running applications in a Linux console (TTY) without a
//! graphical environment, GPM allows mouse input to work properly.
//!
//! # Installation
//!
//! To use this feature, you need to:
//! 1. Install the GPM daemon on your Linux system:
//!    - Debian/Ubuntu: `sudo apt-get install gpm libgpm-dev`
//!    - Arch Linux: `sudo pacman -S gpm`
//!    - Fedora/RHEL: `sudo dnf install gpm gpm-devel`
//!
//! 2. Start the GPM service:
//!    ```bash
//!    sudo systemctl start gpm
//!    sudo systemctl enable gpm  # To start on boot
//!    ```
//!
//! 3. The application will automatically detect and use GPM if available.
//!
//! # Usage
//!
//! The GPM support is automatically enabled when running on Linux. The application
//! will try to connect to the GPM daemon at startup. If GPM is not available,
//! the application will fall back to standard terminal mouse support (crossterm).
//!
//! # Limitations
//!
//! - GPM is only available on Linux systems
//! - Requires the GPM daemon to be running
//! - Only works in Linux console (TTY), not in terminal emulators
//!
//! # Technical Details
//!
//! This module uses FFI (Foreign Function Interface) to communicate with the
//! libgpm C library. The implementation follows the standard GPM client protocol.

use libc::{c_int, c_short, c_ushort};
use nix::poll::{PollFd, PollFlags, poll};
use std::os::fd::BorrowedFd;

// GPM event types (from gpm.h)
const GPM_MOVE: c_int = 1;
const GPM_DRAG: c_int = 2;
const GPM_DOWN: c_int = 4;
const GPM_UP: c_int = 8;

// GPM button masks (from gpm.h)
const GPM_B_LEFT: c_int = 1;
const GPM_B_MIDDLE: c_int = 2;
const GPM_B_RIGHT: c_int = 4;

// Gpm_Event structure (from gpm.h)
// IMPORTANT: This must match the C structure exactly to avoid memory corruption
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpmEvent {
    pub buttons: c_uchar,
    pub modifiers: c_uchar,
    pub vc: c_ushort,
    pub dx: c_short,
    pub dy: c_short,
    pub x: c_short,
    pub y: c_short,
    pub event_type: c_int, // enum Gpm_Etype in C
    pub clicks: c_int,
    pub margin: c_int, // enum Gpm_Margin in C
    pub wdx: c_short,  // displacement since margin
    pub wdy: c_short,  // position on margin
}

use libc::c_uchar;

// Gpm_Connect structure (from gpm.h)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct GpmConnect {
    event_mask: c_ushort,
    default_mask: c_ushort,
    min_mod: c_ushort,
    max_mod: c_ushort,
}

// GPM connection mode flags
const GPM_MOVE_MODE: c_ushort = 1;
const GPM_DRAG_MODE: c_ushort = 2;
const GPM_DOWN_MODE: c_ushort = 4;
const GPM_UP_MODE: c_ushort = 8;

// Dynamic library handle and function pointers
use std::sync::OnceLock;

type GpmOpenFn = unsafe extern "C" fn(*mut GpmConnect, c_int) -> c_int;
type GpmCloseFn = unsafe extern "C" fn() -> c_int;
type GpmGetEventFn = unsafe extern "C" fn(*mut GpmEvent) -> c_int;

static GPM_LIB: OnceLock<Option<GpmLibrary>> = OnceLock::new();

struct GpmLibrary {
    _handle: *mut libc::c_void,
    gpm_open: GpmOpenFn,
    gpm_close: GpmCloseFn,
    gpm_get_event: GpmGetEventFn,
}

unsafe impl Send for GpmLibrary {}
unsafe impl Sync for GpmLibrary {}

/// Try to dynamically load libgpm.so at runtime
fn load_gpm_library() -> Option<GpmLibrary> {
    unsafe {
        // Try to load libgpm.so (common library names)
        let lib_names = ["libgpm.so.2", "libgpm.so.1", "libgpm.so"];

        for lib_name in &lib_names {
            let lib_cstr = std::ffi::CString::new(*lib_name).ok()?;
            let handle = libc::dlopen(lib_cstr.as_ptr(), libc::RTLD_LAZY);

            if handle.is_null() {
                continue; // Try next library name
            }

            // Clear any previous errors
            libc::dlerror();

            // Load function pointers
            let gpm_open_name = std::ffi::CString::new("Gpm_Open").ok()?;
            let gpm_close_name = std::ffi::CString::new("Gpm_Close").ok()?;
            let gpm_get_event_name = std::ffi::CString::new("Gpm_GetEvent").ok()?;

            let gpm_open_ptr = libc::dlsym(handle, gpm_open_name.as_ptr());
            let gpm_close_ptr = libc::dlsym(handle, gpm_close_name.as_ptr());
            let gpm_get_event_ptr = libc::dlsym(handle, gpm_get_event_name.as_ptr());

            // Check for errors
            let error = libc::dlerror();
            if !error.is_null()
                || gpm_open_ptr.is_null()
                || gpm_close_ptr.is_null()
                || gpm_get_event_ptr.is_null()
            {
                libc::dlclose(handle);
                continue;
            }

            // Convert dlsym results to function pointers
            // This is safe because we verified the pointers are not null
            let gpm_open: GpmOpenFn = std::mem::transmute_copy(&gpm_open_ptr);
            let gpm_close: GpmCloseFn = std::mem::transmute_copy(&gpm_close_ptr);
            let gpm_get_event: GpmGetEventFn = std::mem::transmute_copy(&gpm_get_event_ptr);

            return Some(GpmLibrary {
                _handle: handle,
                gpm_open,
                gpm_close,
                gpm_get_event,
            });
        }

        None
    }
}

/// Get the GPM library (loads it on first call)
fn get_gpm_lib() -> Option<&'static GpmLibrary> {
    GPM_LIB.get_or_init(load_gpm_library).as_ref()
}

/// GPM Connection handle
pub struct GpmConnection {
    fd: c_int,
    connected: bool,
}

/// GPM mouse event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpmEventType {
    Move,
    Drag,
    Down,
    Up,
}

/// GPM mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpmButton {
    Left,
    Middle,
    Right,
}

/// Simplified GPM event for application use
#[derive(Debug, Clone, Copy)]
pub struct GpmMouseEvent {
    pub x: u16,
    pub y: u16,
    pub event_type: GpmEventType,
    pub button: Option<GpmButton>,
}

impl GpmConnection {
    /// Open a connection to the GPM daemon
    /// Returns None if GPM is not available or connection fails
    pub fn open() -> Option<Self> {
        // Try to load GPM library first
        let gpm_lib = get_gpm_lib()?;

        unsafe {
            let event_mask = GPM_MOVE_MODE | GPM_DRAG_MODE | GPM_DOWN_MODE | GPM_UP_MODE;
            let mut conn = GpmConnect {
                event_mask,
                // Set default_mask to ~0 to force GPM to always draw cursor
                // This ensures cursor visibility regardless of our event handling
                default_mask: !0,
                min_mod: 0,
                max_mod: !0,
            };

            let fd = (gpm_lib.gpm_open)(&mut conn as *mut GpmConnect, 0);

            if fd < 0 {
                // GPM not available or connection failed
                None
            } else {
                Some(GpmConnection {
                    fd,
                    connected: true,
                })
            }
        }
    }

    /// Get the file descriptor for the GPM connection
    /// This can be used for polling
    #[allow(dead_code)]
    pub fn fd(&self) -> c_int {
        self.fd
    }

    /// Check if there's an event available (non-blocking)
    pub fn has_event(&self) -> bool {
        if !self.connected {
            return false;
        }

        // Use poll() to check if data is available (safer than select with fd_set)
        // SAFETY: self.fd is a valid file descriptor from GPM connection
        let mut poll_fds = unsafe {
            [PollFd::new(
                BorrowedFd::borrow_raw(self.fd),
                PollFlags::POLLIN,
            )]
        };

        // Timeout of 0ms for non-blocking check
        match poll(&mut poll_fds, 0u8) {
            Ok(n) if n > 0 => {
                // Check if our fd has data available
                if let Some(revents) = poll_fds[0].revents() {
                    revents.contains(PollFlags::POLLIN)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Read a GPM event (blocking)
    /// Returns None if no event is available or connection is closed
    pub fn get_event(&self) -> Option<GpmMouseEvent> {
        if !self.connected {
            return None;
        }

        let gpm_lib = get_gpm_lib()?;

        unsafe {
            let mut event: GpmEvent = std::mem::zeroed();
            let result = (gpm_lib.gpm_get_event)(&mut event as *mut GpmEvent);

            if result <= 0 {
                return None;
            }

            // Convert GPM event to our simplified format
            let event_type = if (event.event_type & GPM_DOWN) != 0 {
                GpmEventType::Down
            } else if (event.event_type & GPM_UP) != 0 {
                GpmEventType::Up
            } else if (event.event_type & GPM_DRAG) != 0 {
                GpmEventType::Drag
            } else if (event.event_type & GPM_MOVE) != 0 {
                GpmEventType::Move
            } else {
                return None;
            };

            // Determine which button (if any)
            // Use ONLY the buttons field - event_type has overlapping bit values!
            // Note: Some GPM configurations have left/right swapped, so we swap them here
            let button = if (event.buttons as c_int & GPM_B_LEFT) != 0 {
                Some(GpmButton::Right) // Swapped!
            } else if (event.buttons as c_int & GPM_B_MIDDLE) != 0 {
                Some(GpmButton::Middle)
            } else if (event.buttons as c_int & GPM_B_RIGHT) != 0 {
                Some(GpmButton::Left) // Swapped!
            } else {
                None
            };

            // Convert coordinates (GPM uses 1-based indexing)
            let x = (event.x - 1).max(0) as u16;
            let y = (event.y - 1).max(0) as u16;

            Some(GpmMouseEvent {
                x,
                y,
                event_type,
                button,
            })
        }
    }

    /// Close the GPM connection
    pub fn close(&mut self) {
        if self.connected {
            if let Some(gpm_lib) = get_gpm_lib() {
                unsafe {
                    (gpm_lib.gpm_close)();
                }
            }
            self.connected = false;
        }
    }

    /// Check if connected to GPM
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Drop for GpmConnection {
    fn drop(&mut self) {
        self.close();
    }
}

/// Check if GPM is available on the system
/// This tries to open a connection and immediately closes it
#[allow(dead_code)]
pub fn is_gpm_available() -> bool {
    if let Some(mut conn) = GpmConnection::open() {
        conn.close();
        true
    } else {
        false
    }
}
