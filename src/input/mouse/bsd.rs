//! BSD-specific raw mouse input handling
//!
//! Supports:
//! - FreeBSD: /dev/sysmouse (PS/2-like protocol)
//! - NetBSD: /dev/wsmouse0 (wscons protocol)
//! - OpenBSD: /dev/wsmouse (wscons protocol)

use super::types::{MouseButtons, RawMouseEvent};
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// BSD raw mouse input reader
pub struct RawMouseInput {
    file: File,
    #[cfg(target_os = "freebsd")]
    _protocol: FreeBsdProtocol,
    #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
    _protocol: WsconsProtocol,
    buttons: MouseButtons,
}

#[cfg(target_os = "freebsd")]
#[derive(Debug, Clone, Copy)]
enum FreeBsdProtocol {
    Sysmouse,
}

#[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
#[derive(Debug, Clone, Copy)]
enum WsconsProtocol {
    Wscons,
}

// wscons event types (NetBSD/OpenBSD)
#[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
mod wscons {
    pub const WSCONS_EVENT_MOUSE_DELTA_X: u32 = 0;
    pub const WSCONS_EVENT_MOUSE_DELTA_Y: u32 = 1;
    pub const WSCONS_EVENT_MOUSE_DELTA_Z: u32 = 2; // scroll
    pub const WSCONS_EVENT_MOUSE_DELTA_W: u32 = 3; // horizontal scroll
    pub const WSCONS_EVENT_MOUSE_ABSOLUTE_X: u32 = 4;
    pub const WSCONS_EVENT_MOUSE_ABSOLUTE_Y: u32 = 5;
    pub const WSCONS_EVENT_MOUSE_ABSOLUTE_Z: u32 = 6;
    pub const WSCONS_EVENT_MOUSE_ABSOLUTE_W: u32 = 7;
    pub const WSCONS_EVENT_MOUSE_UP: u32 = 8;
    pub const WSCONS_EVENT_MOUSE_DOWN: u32 = 9;

    // wscons event structure size
    // struct wscons_event { u_int type; int value; struct timespec time; }
    // On 64-bit: 4 + 4 + 16 = 24 bytes
    // On 32-bit: 4 + 4 + 8 = 16 bytes
    #[cfg(target_pointer_width = "64")]
    pub const WSCONS_EVENT_SIZE: usize = 24;
    #[cfg(target_pointer_width = "32")]
    pub const WSCONS_EVENT_SIZE: usize = 16;
}

impl RawMouseInput {
    /// Open the mouse input device
    #[cfg(target_os = "freebsd")]
    pub fn new(device_path: Option<&str>) -> io::Result<Self> {
        let path = device_path.unwrap_or("/dev/sysmouse");

        if !Path::new(path).exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Mouse device not found: {}", path),
            ));
        }

        let file = File::open(path)?;
        Self::setup_device(file, FreeBsdProtocol::Sysmouse)
    }

    /// Open the mouse input device
    #[cfg(target_os = "netbsd")]
    pub fn new(device_path: Option<&str>) -> io::Result<Self> {
        let path = device_path.unwrap_or("/dev/wsmouse0");

        if !Path::new(path).exists() {
            // Try alternative paths
            let alternatives = ["/dev/wsmouse", "/dev/wsmouse1"];
            let found_path = alternatives
                .iter()
                .find(|p| Path::new(p).exists())
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "No wscons mouse device found (/dev/wsmouse*)",
                    )
                })?;
            let file = File::open(found_path)?;
            return Self::setup_device(file, WsconsProtocol::Wscons);
        }

        let file = File::open(path)?;
        Self::setup_device(file, WsconsProtocol::Wscons)
    }

    /// Open the mouse input device
    #[cfg(target_os = "openbsd")]
    pub fn new(device_path: Option<&str>) -> io::Result<Self> {
        let path = device_path.unwrap_or("/dev/wsmouse");

        if !Path::new(path).exists() {
            // Try alternative paths
            let alternatives = ["/dev/wsmouse0", "/dev/wsmouse1"];
            let found_path = alternatives
                .iter()
                .find(|p| Path::new(p).exists())
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "No wscons mouse device found (/dev/wsmouse*)",
                    )
                })?;
            let file = File::open(found_path)?;
            return Self::setup_device(file, WsconsProtocol::Wscons);
        }

        let file = File::open(path)?;
        Self::setup_device(file, WsconsProtocol::Wscons)
    }

    #[cfg(target_os = "freebsd")]
    fn setup_device(file: File, protocol: FreeBsdProtocol) -> io::Result<Self> {
        // Set non-blocking mode
        unsafe {
            let flags = libc::fcntl(file.as_raw_fd(), libc::F_GETFL, 0);
            if flags < 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(Self {
            file,
            _protocol: protocol,
            buttons: MouseButtons::default(),
        })
    }

    #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
    fn setup_device(file: File, protocol: WsconsProtocol) -> io::Result<Self> {
        // Set non-blocking mode
        unsafe {
            let flags = libc::fcntl(file.as_raw_fd(), libc::F_GETFL, 0);
            if flags < 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
                return Err(io::Error::last_os_error());
            }
        }

        Ok(Self {
            file,
            _protocol: protocol,
            buttons: MouseButtons::default(),
        })
    }

    pub fn has_event(&self) -> bool {
        let fd = self.file.as_raw_fd();
        let mut fds = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        unsafe {
            libc::poll(&mut fds as *mut libc::pollfd, 1, 0) > 0 && (fds.revents & libc::POLLIN) != 0
        }
    }

    pub fn read_event(&mut self) -> io::Result<Option<RawMouseEvent>> {
        if !self.has_event() {
            return Ok(None);
        }

        #[cfg(target_os = "freebsd")]
        {
            self.read_sysmouse_event()
        }

        #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
        {
            self.read_wscons_event()
        }
    }

    /// Read FreeBSD sysmouse event (PS/2-like protocol)
    ///
    /// sysmouse uses a 5-byte protocol:
    /// - Byte 0: 0x80 | buttons (bit 0=left, bit 1=middle, bit 2=right) - buttons are ACTIVE LOW
    /// - Byte 1: X delta (signed)
    /// - Byte 2: Y delta (signed)
    /// - Byte 3-4: Z delta (scroll wheel, signed)
    #[cfg(target_os = "freebsd")]
    fn read_sysmouse_event(&mut self) -> io::Result<Option<RawMouseEvent>> {
        let mut buf = [0u8; 8]; // Read up to 8 bytes for extended protocol
        match self.file.read(&mut buf) {
            Ok(n) if n >= 5 => {
                // sysmouse buttons are active-low (0 = pressed)
                // Also note: bit mapping is different from Linux
                let btn_byte = buf[0];
                let buttons = MouseButtons {
                    left: (btn_byte & 0x04) == 0,   // bit 2 (active low)
                    middle: (btn_byte & 0x02) == 0, // bit 1 (active low)
                    right: (btn_byte & 0x01) == 0,  // bit 0 (active low)
                };

                let dx = buf[1] as i8;
                let dy = buf[2] as i8;

                // Scroll wheel in bytes 3-4 (signed, little-endian)
                let scroll = if n >= 5 {
                    let scroll_raw = i16::from_le_bytes([buf[3], buf[4]]);
                    scroll_raw.clamp(-127, 127) as i8
                } else {
                    0
                };

                Ok(Some(RawMouseEvent {
                    dx,
                    dy,
                    buttons,
                    scroll,
                    scroll_h: 0,
                }))
            }
            Ok(_) => Ok(None),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Read NetBSD/OpenBSD wscons event
    ///
    /// wscons uses an event-based protocol with discrete events for each axis/button
    #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
    fn read_wscons_event(&mut self) -> io::Result<Option<RawMouseEvent>> {
        use wscons::*;

        let mut buf = [0u8; WSCONS_EVENT_SIZE];
        let mut dx_accum: i32 = 0;
        let mut dy_accum: i32 = 0;
        let mut scroll_accum: i32 = 0;
        let mut scroll_h_accum: i32 = 0;
        let mut had_event = false;

        // Read all available events
        loop {
            match self.file.read(&mut buf) {
                Ok(n) if n == WSCONS_EVENT_SIZE => {
                    // Parse wscons_event structure
                    let event_type = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]);
                    let value = i32::from_ne_bytes([buf[4], buf[5], buf[6], buf[7]]);

                    match event_type {
                        WSCONS_EVENT_MOUSE_DELTA_X => {
                            dx_accum += value;
                            had_event = true;
                        }
                        WSCONS_EVENT_MOUSE_DELTA_Y => {
                            // wscons Y is already in screen coordinates (positive = down)
                            // Negate to match PS/2 convention (positive = up)
                            dy_accum -= value;
                            had_event = true;
                        }
                        WSCONS_EVENT_MOUSE_DELTA_Z => {
                            scroll_accum += value;
                            had_event = true;
                        }
                        WSCONS_EVENT_MOUSE_DELTA_W => {
                            scroll_h_accum += value;
                            had_event = true;
                        }
                        WSCONS_EVENT_MOUSE_DOWN => {
                            // Button pressed (value = button number, 0=left, 1=middle, 2=right)
                            match value {
                                0 => self.buttons.left = true,
                                1 => self.buttons.middle = true,
                                2 => self.buttons.right = true,
                                _ => {}
                            }
                            had_event = true;
                        }
                        WSCONS_EVENT_MOUSE_UP => {
                            // Button released (value = button number)
                            match value {
                                0 => self.buttons.left = false,
                                1 => self.buttons.middle = false,
                                2 => self.buttons.right = false,
                                _ => {}
                            }
                            had_event = true;
                        }
                        _ => {}
                    }
                }
                Ok(_) => break,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }

        if had_event {
            Ok(Some(RawMouseEvent {
                dx: dx_accum.clamp(-127, 127) as i8,
                dy: dy_accum.clamp(-127, 127) as i8,
                buttons: self.buttons,
                scroll: scroll_accum.clamp(-127, 127) as i8,
                scroll_h: scroll_h_accum.clamp(-127, 127) as i8,
            }))
        } else {
            Ok(None)
        }
    }
}
