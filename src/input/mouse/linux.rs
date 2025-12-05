//! Linux-specific raw mouse input handling
//!
//! Supports both /dev/input/mice (PS/2 protocol) and /dev/input/event* (evdev protocol)

use super::types::{MouseButtons, Protocol, RawMouseEvent};
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::io::AsRawFd;
use std::path::Path;

// Event types
const EV_REL: u16 = 0x02;
const EV_KEY: u16 = 0x01;

// Relative axis codes
const REL_X: u16 = 0x00;
const REL_Y: u16 = 0x01;
const REL_WHEEL: u16 = 0x08;
const REL_HWHEEL: u16 = 0x06;

// Button codes
const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;
const BTN_MIDDLE: u16 = 0x112;

#[cfg(target_pointer_width = "64")]
const INPUT_EVENT_SIZE: usize = 24;
#[cfg(target_pointer_width = "32")]
const INPUT_EVENT_SIZE: usize = 16;

/// Linux raw mouse input reader
pub struct RawMouseInput {
    file: File,
    protocol: Protocol,
    dx_accumulator: i32,
    dy_accumulator: i32,
    scroll_accumulator: i32,
    scroll_h_accumulator: i32,
    buttons: MouseButtons,
    button_changed: bool,
}

impl RawMouseInput {
    /// Open the mouse input device
    pub fn new(device_path: Option<&str>) -> io::Result<Self> {
        if let Some(path) = device_path {
            let file = File::open(path)?;
            let protocol = if path.contains("mice") {
                Protocol::Ps2
            } else {
                Protocol::InputEvent
            };
            Self::setup_device(file, protocol)
        } else if let Ok(input) = Self::find_mouse_event_device() {
            // Prefer evdev for better scroll wheel support
            Ok(input)
        } else if let Ok(file) = File::open("/dev/input/mice") {
            // Fallback to PS/2 multiplexer (may not have scroll wheel)
            Self::setup_device(file, Protocol::Ps2)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No mouse input device found",
            ))
        }
    }

    /// Find a mouse event device that supports scroll wheel (REL_WHEEL)
    fn find_mouse_event_device() -> io::Result<Self> {
        // EVIOCGBIT ioctl to get device capabilities
        // _IOC(IOC_READ, 'E', 0x20 + ev_type, len)
        // For EV_REL (0x02): 0x20 + 0x02 = 0x22
        // Note: ioctl request type varies by platform:
        // - c_int on musl and Android (Bionic)
        // - c_ulong on glibc/BSD
        #[cfg(any(target_env = "musl", target_os = "android"))]
        const EVIOCGBIT_REL: libc::c_int = 0x80084522u32 as libc::c_int;
        #[cfg(not(any(target_env = "musl", target_os = "android")))]
        const EVIOCGBIT_REL: libc::c_ulong = 0x80084522;

        for i in 0..16 {
            let path = format!("/dev/input/event{}", i);
            if !Path::new(&path).exists() {
                continue;
            }

            if let Ok(file) = File::open(&path) {
                let fd = file.as_raw_fd();

                // Query REL capabilities (need at least 2 bytes for REL_WHEEL which is bit 8)
                let mut rel_bits = [0u8; 2];
                let ret = unsafe { libc::ioctl(fd, EVIOCGBIT_REL, rel_bits.as_mut_ptr()) };

                if ret >= 0 {
                    // Check if device has REL_X (bit 0), REL_Y (bit 1), and REL_WHEEL (bit 8)
                    let has_rel_x = (rel_bits[0] & (1 << REL_X)) != 0;
                    let has_rel_y = (rel_bits[0] & (1 << REL_Y)) != 0;
                    let has_rel_wheel = (rel_bits[1] & (1 << (REL_WHEEL - 8))) != 0;

                    if has_rel_x && has_rel_y && has_rel_wheel {
                        if let Ok(input) = Self::setup_device(file, Protocol::InputEvent) {
                            return Ok(input);
                        }
                    }
                }
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No mouse event device with scroll wheel found",
        ))
    }

    fn setup_device(file: File, protocol: Protocol) -> io::Result<Self> {
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
            protocol,
            dx_accumulator: 0,
            dy_accumulator: 0,
            scroll_accumulator: 0,
            scroll_h_accumulator: 0,
            buttons: MouseButtons::default(),
            button_changed: false,
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
        match self.protocol {
            Protocol::Ps2 => self.read_ps2_event(),
            Protocol::InputEvent => self.read_input_event(),
        }
    }

    fn read_ps2_event(&mut self) -> io::Result<Option<RawMouseEvent>> {
        let mut buf = [0u8; 4];
        match self.file.read(&mut buf) {
            Ok(n) if n >= 3 => {
                let buttons = MouseButtons {
                    left: (buf[0] & 0x01) != 0,
                    right: (buf[0] & 0x02) != 0,
                    middle: (buf[0] & 0x04) != 0,
                };
                let dx = buf[1] as i8;
                let dy = buf[2] as i8;
                let scroll = if n == 4 { -(buf[3] as i8) } else { 0 };

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

    fn read_input_event(&mut self) -> io::Result<Option<RawMouseEvent>> {
        let mut buf = [0u8; INPUT_EVENT_SIZE];

        #[cfg(target_pointer_width = "64")]
        const TYPE_OFFSET: usize = 16;
        #[cfg(target_pointer_width = "32")]
        const TYPE_OFFSET: usize = 8;

        loop {
            match self.file.read(&mut buf) {
                Ok(n) if n == INPUT_EVENT_SIZE => {
                    let type_ = u16::from_ne_bytes([buf[TYPE_OFFSET], buf[TYPE_OFFSET + 1]]);
                    let code = u16::from_ne_bytes([buf[TYPE_OFFSET + 2], buf[TYPE_OFFSET + 3]]);
                    let value = i32::from_ne_bytes([
                        buf[TYPE_OFFSET + 4],
                        buf[TYPE_OFFSET + 5],
                        buf[TYPE_OFFSET + 6],
                        buf[TYPE_OFFSET + 7],
                    ]);

                    match type_ {
                        EV_REL => match code {
                            REL_X => self.dx_accumulator += value,
                            // evdev REL_Y: positive = down (screen direction)
                            // PS/2 dy: positive = up (toward user)
                            // Negate to match PS/2 convention used by CursorTracker
                            REL_Y => self.dy_accumulator -= value,
                            REL_WHEEL => self.scroll_accumulator += value,
                            REL_HWHEEL => self.scroll_h_accumulator += value,
                            _ => {}
                        },
                        EV_KEY => {
                            let old_buttons = self.buttons;
                            match code {
                                BTN_LEFT => self.buttons.left = value != 0,
                                BTN_RIGHT => self.buttons.right = value != 0,
                                BTN_MIDDLE => self.buttons.middle = value != 0,
                                _ => {}
                            }
                            if self.buttons != old_buttons {
                                self.button_changed = true;
                            }
                        }
                        _ => {}
                    }

                    let has_data = self.dx_accumulator != 0
                        || self.dy_accumulator != 0
                        || self.scroll_accumulator != 0
                        || self.scroll_h_accumulator != 0
                        || self.button_changed;

                    if has_data {
                        return Ok(Some(self.flush_accumulators()));
                    }
                }
                Ok(_) => return Ok(None),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    let has_data = self.dx_accumulator != 0
                        || self.dy_accumulator != 0
                        || self.scroll_accumulator != 0
                        || self.scroll_h_accumulator != 0
                        || self.button_changed;

                    if has_data {
                        return Ok(Some(self.flush_accumulators()));
                    }
                    return Ok(None);
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn flush_accumulators(&mut self) -> RawMouseEvent {
        let event = RawMouseEvent {
            dx: self.dx_accumulator.clamp(-127, 127) as i8,
            dy: self.dy_accumulator.clamp(-127, 127) as i8,
            buttons: self.buttons,
            scroll: self.scroll_accumulator.clamp(-127, 127) as i8,
            scroll_h: self.scroll_h_accumulator.clamp(-127, 127) as i8,
        };
        self.dx_accumulator = 0;
        self.dy_accumulator = 0;
        self.scroll_accumulator = 0;
        self.scroll_h_accumulator = 0;
        self.button_changed = false;
        event
    }
}
