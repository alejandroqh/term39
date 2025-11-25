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
const GPM_B_FOURTH: c_int = 16; // Scroll wheel up
const GPM_B_FIFTH: c_int = 32; // Scroll wheel down

// GPM modifier masks (from gpm.h)
const GPM_MOD_SHIFT: u8 = 1;
const GPM_MOD_CTRL: u8 = 4;
const GPM_MOD_META: u8 = 8; // ALT key

// GPM click type masks (from gpm.h) - combined with DOWN/UP events
const GPM_SINGLE: c_int = 16;
const GPM_DOUBLE: c_int = 32;
const GPM_TRIPLE: c_int = 64;

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

// Compile-time assertion to verify GpmEvent struct matches C library layout
// Expected size: 1+1+2+2+2+2+2+4+4+4+2+2 = 28 bytes
const _: () = assert!(
    std::mem::size_of::<GpmEvent>() == 28,
    "GpmEvent struct size mismatch - must match C library's Gpm_Event"
);

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
        // Try to load libgpm.so (common library names across distros)
        let lib_names = [
            "libgpm.so.2",
            "libgpm.so.1",
            "libgpm.so",
            "libgpm.so.2.1.0", // Some distros use full version
        ];

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
    /// File descriptor for GPM connection (-1 = not connected)
    fd: c_int,
    /// Track last pressed button for UP events that don't report button state
    last_button: Option<GpmButton>,
}

/// GPM mouse event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpmEventType {
    Move,
    Drag,
    Down,
    Up,
    ScrollUp,
    ScrollDown,
}

/// GPM mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpmButton {
    Left,
    Middle,
    Right,
}

/// Click count for mouse events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClickCount {
    #[default]
    Single,
    Double,
    Triple,
}

/// Simplified GPM event for application use
#[derive(Debug, Clone, Copy)]
pub struct GpmMouseEvent {
    pub x: u16,
    pub y: u16,
    pub event_type: GpmEventType,
    pub button: Option<GpmButton>,
    pub clicks: ClickCount,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Convert GPM 1-based coordinates to 0-based screen coordinates
#[inline]
fn gpm_to_screen_coords(x: c_short, y: c_short) -> (u16, u16) {
    ((x - 1).max(0) as u16, (y - 1).max(0) as u16)
}

impl GpmConnection {
    /// Open a connection to the GPM daemon
    ///
    /// # Arguments
    /// * `draw_cursor` - If true, GPM draws its cursor (for terminal mode).
    ///   If false, application draws cursor (for framebuffer mode).
    ///
    /// Returns None if GPM is not available or connection fails
    pub fn open(draw_cursor: bool) -> Option<Self> {
        // Try to load GPM library first
        let gpm_lib = get_gpm_lib()?;

        unsafe {
            // GPM event handling:
            // - event_mask: events we want to receive (button presses, etc.)
            // - default_mask: events GPM handles itself (cursor drawing, selection)
            //
            // For terminal mode (draw_cursor=true):
            //   Request all events and set default_mask to !0 so GPM draws its cursor.
            //   This matches the proven v0.10.3 configuration that works reliably.
            //
            // For framebuffer mode (draw_cursor=false):
            //   Capture all events with no defaults, we draw our own cursor.
            let event_mask = (GPM_MOVE | GPM_DRAG | GPM_DOWN | GPM_UP) as c_ushort;
            let default_mask = if draw_cursor {
                // Terminal mode: GPM handles all events by default (draws cursor)
                !0 as c_ushort
            } else {
                // Framebuffer mode: Application handles everything
                0 as c_ushort
            };

            let mut conn = GpmConnect {
                event_mask,
                default_mask,
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
                    last_button: None,
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
        if self.fd < 0 {
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
                poll_fds[0]
                    .revents()
                    .is_some_and(|r| r.contains(PollFlags::POLLIN))
            }
            _ => false,
        }
    }

    /// Read a GPM event (blocking)
    /// Returns None if no event is available or connection is closed
    pub fn get_event(&mut self) -> Option<GpmMouseEvent> {
        if self.fd < 0 {
            return None;
        }

        let gpm_lib = get_gpm_lib()?;

        unsafe {
            let mut event: GpmEvent = std::mem::zeroed();
            let result = (gpm_lib.gpm_get_event)(&mut event as *mut GpmEvent);

            if result <= 0 {
                return None;
            }

            let (x, y) = gpm_to_screen_coords(event.x, event.y);
            let buttons = event.buttons as c_int;

            // Extract modifier keys
            let shift = (event.modifiers & GPM_MOD_SHIFT) != 0;
            let ctrl = (event.modifiers & GPM_MOD_CTRL) != 0;
            let alt = (event.modifiers & GPM_MOD_META) != 0;

            // Check for scroll wheel events first (GPM uses buttons 4 and 5)
            if (buttons & GPM_B_FOURTH) != 0 {
                return Some(GpmMouseEvent {
                    x,
                    y,
                    event_type: GpmEventType::ScrollUp,
                    button: None,
                    clicks: ClickCount::Single,
                    shift,
                    ctrl,
                    alt,
                });
            }
            if (buttons & GPM_B_FIFTH) != 0 {
                return Some(GpmMouseEvent {
                    x,
                    y,
                    event_type: GpmEventType::ScrollDown,
                    button: None,
                    clicks: ClickCount::Single,
                    shift,
                    ctrl,
                    alt,
                });
            }

            // Convert GPM event type to our simplified format
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

            // Extract click count from event_type (GPM combines these with DOWN/UP)
            let clicks = if (event.event_type & GPM_TRIPLE) != 0 {
                ClickCount::Triple
            } else if (event.event_type & GPM_DOUBLE) != 0 {
                ClickCount::Double
            } else {
                ClickCount::Single
            };

            // Determine which button (if any)
            // Use ONLY the buttons field - event_type has overlapping bit values!
            // NOTE: GPM reports buttons swapped on most hardware - GPM_B_LEFT is actually
            // the right button and GPM_B_RIGHT is the left button. This has been verified
            // on multiple devices, so we swap them here to get correct behavior by default.
            let reported_button = if (buttons & GPM_B_LEFT) != 0 {
                Some(GpmButton::Right) // GPM_B_LEFT is actually the right button
            } else if (buttons & GPM_B_MIDDLE) != 0 {
                Some(GpmButton::Middle)
            } else if (buttons & GPM_B_RIGHT) != 0 {
                Some(GpmButton::Left) // GPM_B_RIGHT is actually the left button
            } else {
                None
            };

            // Track button state for proper UP event handling
            // On DOWN: record which button was pressed
            // On UP: use tracked button if GPM didn't report one, then clear
            // On DRAG: use tracked button if GPM didn't report one
            let button = match event_type {
                GpmEventType::Down => {
                    self.last_button = reported_button;
                    reported_button
                }
                GpmEventType::Up => {
                    let btn = reported_button.or(self.last_button);
                    self.last_button = None;
                    btn
                }
                GpmEventType::Drag => reported_button.or(self.last_button),
                _ => reported_button,
            };

            Some(GpmMouseEvent {
                x,
                y,
                event_type,
                button,
                clicks,
                shift,
                ctrl,
                alt,
            })
        }
    }

    /// Close the GPM connection
    pub fn close(&mut self) {
        if self.fd >= 0 {
            if let Some(gpm_lib) = get_gpm_lib() {
                unsafe {
                    (gpm_lib.gpm_close)();
                }
            }
            self.fd = -1;
        }
    }

    /// Check if connected to GPM
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.fd >= 0
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
    if let Some(mut conn) = GpmConnection::open(true) {
        conn.close();
        true
    } else {
        false
    }
}
