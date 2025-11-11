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

#![cfg(target_os = "linux")]

use libc::{c_int, c_short, c_ushort};
use std::ptr;

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
    pub event_type: c_int,
    pub clicks: c_int,
    pub margin: c_int,
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

// External C functions from libgpm
extern "C" {
    fn Gpm_Open(conn: *mut GpmConnect, flag: c_int) -> c_int;
    fn Gpm_Close() -> c_int;
    fn Gpm_GetEvent(event: *mut GpmEvent) -> c_int;
    fn Gpm_Getc(f: *mut libc::FILE) -> c_int;
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
        unsafe {
            let mut conn = GpmConnect {
                event_mask: (GPM_MOVE_MODE | GPM_DRAG_MODE | GPM_DOWN_MODE | GPM_UP_MODE),
                default_mask: 0,
                min_mod: 0,
                max_mod: 0,
            };

            let fd = Gpm_Open(&mut conn as *mut GpmConnect, 0);

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
    pub fn fd(&self) -> c_int {
        self.fd
    }

    /// Check if there's an event available (non-blocking)
    pub fn has_event(&self) -> bool {
        if !self.connected {
            return false;
        }

        // Use select() to check if data is available
        unsafe {
            let mut read_fds: libc::fd_set = std::mem::zeroed();
            libc::FD_ZERO(&mut read_fds);
            libc::FD_SET(self.fd, &mut read_fds);

            let mut timeout = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };

            let result = libc::select(
                self.fd + 1,
                &mut read_fds,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut timeout,
            );

            result > 0
        }
    }

    /// Read a GPM event (blocking)
    /// Returns None if no event is available or connection is closed
    pub fn get_event(&self) -> Option<GpmMouseEvent> {
        if !self.connected {
            return None;
        }

        unsafe {
            let mut event: GpmEvent = std::mem::zeroed();
            let result = Gpm_GetEvent(&mut event as *mut GpmEvent);

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
            let button = if (event.buttons as c_int & GPM_B_LEFT) != 0 {
                Some(GpmButton::Left)
            } else if (event.buttons as c_int & GPM_B_MIDDLE) != 0 {
                Some(GpmButton::Middle)
            } else if (event.buttons as c_int & GPM_B_RIGHT) != 0 {
                Some(GpmButton::Right)
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
            unsafe {
                Gpm_Close();
            }
            self.connected = false;
        }
    }

    /// Check if connected to GPM
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
pub fn is_gpm_available() -> bool {
    if let Some(mut conn) = GpmConnection::open() {
        conn.close();
        true
    } else {
        false
    }
}
