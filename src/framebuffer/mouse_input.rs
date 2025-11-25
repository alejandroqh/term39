//! Raw mouse input reader for framebuffer mode
//!
//! Reads mouse events directly from /dev/input/mice or /dev/input/event*
//! for cursor tracking when running in framebuffer mode on Linux console.

use std::fs::File;
use std::io::{self, Read};
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// Mouse button states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

/// Raw mouse event with deltas
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub dx: i8,
    pub dy: i8,
    pub buttons: MouseButtons,
    /// Scroll wheel movement (positive = up/away, negative = down/toward)
    #[allow(dead_code)]
    pub scroll: i8,
    /// Horizontal scroll movement (positive = right, negative = left)
    #[allow(dead_code)]
    pub scroll_h: i8,
}

// Event types
const EV_REL: u16 = 0x02; // Relative movement
const EV_KEY: u16 = 0x01; // Button press/release

// Relative axis codes
const REL_X: u16 = 0x00;
const REL_Y: u16 = 0x01;
const REL_WHEEL: u16 = 0x08; // Vertical scroll wheel
const REL_HWHEEL: u16 = 0x06; // Horizontal scroll wheel

// Button codes
const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;
const BTN_MIDDLE: u16 = 0x112;

// Size of struct input_event varies by architecture
// On 64-bit: 24 bytes (8 for timeval.tv_sec, 8 for tv_usec, 2 for type, 2 for code, 4 for value)
// On 32-bit: 16 bytes (4 for timeval.tv_sec, 4 for tv_usec, 2 for type, 2 for code, 4 for value)
#[cfg(target_pointer_width = "64")]
const INPUT_EVENT_SIZE: usize = 24;
#[cfg(target_pointer_width = "32")]
const INPUT_EVENT_SIZE: usize = 16;

/// Mouse input protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Protocol {
    Ps2,        // /dev/input/mice (PS/2 protocol)
    InputEvent, // /dev/input/eventX (Linux input event protocol)
}

/// Raw mouse input reader
pub struct MouseInput {
    file: File,
    protocol: Protocol,
    dx_accumulator: i32,
    dy_accumulator: i32,
    scroll_accumulator: i32,   // Vertical scroll accumulator
    scroll_h_accumulator: i32, // Horizontal scroll accumulator
    buttons: MouseButtons,
    button_changed: bool, // Track if button state changed
}

impl MouseInput {
    /// Open the mouse input device
    /// If device_path is provided, uses that specific device.
    /// Otherwise, tries /dev/input/mice first, then searches for event devices.
    pub fn new(device_path: Option<&str>) -> io::Result<Self> {
        if let Some(path) = device_path {
            // Use the specified device
            let file = File::open(path)?;

            // Determine protocol based on path
            let protocol = if path.contains("mice") {
                Protocol::Ps2
            } else {
                Protocol::InputEvent
            };

            Self::setup_device(file, protocol)
        } else {
            // Try /dev/input/mice first (PS/2 protocol)
            if let Ok(file) = File::open("/dev/input/mice") {
                Self::setup_device(file, Protocol::Ps2)
            } else {
                // Try to find a mouse event device
                Self::find_event_device()
            }
        }
    }

    /// Find and open a mouse event device
    fn find_event_device() -> io::Result<Self> {
        // Try common event device numbers first
        for i in 0..16 {
            let path = format!("/dev/input/event{}", i);
            if Path::new(&path).exists() {
                if let Ok(file) = File::open(&path) {
                    // Try to determine if this is a mouse device
                    // For now, we'll just try the first available event device
                    // A more sophisticated approach would query device capabilities
                    if let Ok(input) = Self::setup_device(file, Protocol::InputEvent) {
                        return Ok(input);
                    }
                }
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No mouse input device found (/dev/input/mice or /dev/input/event*)",
        ))
    }

    /// Setup a file descriptor for mouse input
    fn setup_device(file: File, protocol: Protocol) -> io::Result<Self> {
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
            protocol,
            dx_accumulator: 0,
            dy_accumulator: 0,
            scroll_accumulator: 0,
            scroll_h_accumulator: 0,
            buttons: MouseButtons {
                left: false,
                right: false,
                middle: false,
            },
            button_changed: false,
        })
    }

    /// Check if there's a mouse event available (non-blocking)
    pub fn has_event(&self) -> bool {
        let fd = self.file.as_raw_fd();
        let mut fds = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };

        unsafe {
            // Poll with 0 timeout (non-blocking check)
            libc::poll(&mut fds as *mut libc::pollfd, 1, 0) > 0 && (fds.revents & libc::POLLIN) != 0
        }
    }

    /// Read a mouse event (non-blocking)
    /// Returns None if no event is available
    pub fn read_event(&mut self) -> io::Result<Option<MouseEvent>> {
        if !self.has_event() {
            return Ok(None);
        }

        match self.protocol {
            Protocol::Ps2 => self.read_ps2_event(),
            Protocol::InputEvent => self.read_input_event(),
        }
    }

    /// Read PS/2 protocol event from /dev/input/mice
    fn read_ps2_event(&mut self) -> io::Result<Option<MouseEvent>> {
        let mut buf = [0u8; 4]; // 4 bytes for wheel mouse support

        // Try to read 4 bytes first (IntelliMouse with wheel)
        match self.file.read(&mut buf) {
            Ok(n) if n >= 3 => {
                // Parse PS/2 mouse protocol
                // Byte 0: [Y overflow][X overflow][Y sign][X sign][Always 1][Middle][Right][Left]
                // Byte 1: X movement (8-bit signed)
                // Byte 2: Y movement (8-bit signed)
                // Byte 3 (optional): Z/wheel movement (signed nibble in IntelliMouse protocol)

                let buttons = MouseButtons {
                    left: (buf[0] & 0x01) != 0,
                    right: (buf[0] & 0x02) != 0,
                    middle: (buf[0] & 0x04) != 0,
                };

                // Convert unsigned to signed
                let dx = buf[1] as i8;
                let dy = buf[2] as i8;

                // Scroll wheel (only if we got 4 bytes)
                let scroll = if n == 4 {
                    // IntelliMouse wheel data in byte 3 (signed 4-bit value)
                    let wheel_byte = buf[3] as i8;
                    // Typical values: -1 (down/toward), 1 (up/away)
                    // Negate to match convention (positive = up)
                    -wheel_byte
                } else {
                    0
                };

                Ok(Some(MouseEvent {
                    dx,
                    dy,
                    buttons,
                    scroll,
                    scroll_h: 0,
                }))
            }
            Ok(_) => {
                // Incomplete read, ignore
                Ok(None)
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Read Linux input event protocol from /dev/input/eventX
    fn read_input_event(&mut self) -> io::Result<Option<MouseEvent>> {
        let mut buf = [0u8; INPUT_EVENT_SIZE];

        // Byte offsets for type, code, value depend on architecture
        // 64-bit: timeval is 16 bytes, so type starts at offset 16
        // 32-bit: timeval is 8 bytes, so type starts at offset 8
        #[cfg(target_pointer_width = "64")]
        const TYPE_OFFSET: usize = 16;
        #[cfg(target_pointer_width = "32")]
        const TYPE_OFFSET: usize = 8;

        loop {
            match self.file.read(&mut buf) {
                Ok(n) if n == INPUT_EVENT_SIZE => {
                    // Parse input_event structure
                    let type_ = u16::from_ne_bytes([buf[TYPE_OFFSET], buf[TYPE_OFFSET + 1]]);
                    let code = u16::from_ne_bytes([buf[TYPE_OFFSET + 2], buf[TYPE_OFFSET + 3]]);
                    let value = i32::from_ne_bytes([
                        buf[TYPE_OFFSET + 4],
                        buf[TYPE_OFFSET + 5],
                        buf[TYPE_OFFSET + 6],
                        buf[TYPE_OFFSET + 7],
                    ]);

                    match type_ {
                        EV_REL => {
                            // Relative movement and scroll
                            match code {
                                REL_X => self.dx_accumulator += value,
                                REL_Y => self.dy_accumulator += value,
                                REL_WHEEL => self.scroll_accumulator += value,
                                REL_HWHEEL => self.scroll_h_accumulator += value,
                                _ => {}
                            }
                        }
                        EV_KEY => {
                            // Button press/release
                            let old_buttons = self.buttons;
                            match code {
                                BTN_LEFT => self.buttons.left = value != 0,
                                BTN_RIGHT => self.buttons.right = value != 0,
                                BTN_MIDDLE => self.buttons.middle = value != 0,
                                _ => {}
                            }
                            // Mark if button state changed
                            if self.buttons.left != old_buttons.left
                                || self.buttons.right != old_buttons.right
                                || self.buttons.middle != old_buttons.middle
                            {
                                self.button_changed = true;
                            }
                        }
                        _ => {}
                    }

                    // Check if we have accumulated movement, scroll, OR button state change to report
                    let has_data = self.dx_accumulator != 0
                        || self.dy_accumulator != 0
                        || self.scroll_accumulator != 0
                        || self.scroll_h_accumulator != 0
                        || self.button_changed;

                    if has_data {
                        // Clamp to i8 range
                        let dx = self.dx_accumulator.clamp(-127, 127) as i8;
                        let dy = self.dy_accumulator.clamp(-127, 127) as i8;
                        let scroll = self.scroll_accumulator.clamp(-127, 127) as i8;
                        let scroll_h = self.scroll_h_accumulator.clamp(-127, 127) as i8;

                        self.dx_accumulator = 0;
                        self.dy_accumulator = 0;
                        self.scroll_accumulator = 0;
                        self.scroll_h_accumulator = 0;
                        self.button_changed = false;

                        return Ok(Some(MouseEvent {
                            dx,
                            dy,
                            buttons: self.buttons,
                            scroll,
                            scroll_h,
                        }));
                    }
                }
                Ok(_) => {
                    // Incomplete read, ignore
                    return Ok(None);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No more events available
                    let has_data = self.dx_accumulator != 0
                        || self.dy_accumulator != 0
                        || self.scroll_accumulator != 0
                        || self.scroll_h_accumulator != 0
                        || self.button_changed;

                    if has_data {
                        let dx = self.dx_accumulator.clamp(-127, 127) as i8;
                        let dy = self.dy_accumulator.clamp(-127, 127) as i8;
                        let scroll = self.scroll_accumulator.clamp(-127, 127) as i8;
                        let scroll_h = self.scroll_h_accumulator.clamp(-127, 127) as i8;

                        self.dx_accumulator = 0;
                        self.dy_accumulator = 0;
                        self.scroll_accumulator = 0;
                        self.scroll_h_accumulator = 0;
                        self.button_changed = false;

                        return Ok(Some(MouseEvent {
                            dx,
                            dy,
                            buttons: self.buttons,
                            scroll,
                            scroll_h,
                        }));
                    }
                    return Ok(None);
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// Cursor position tracker
pub struct CursorTracker {
    pub x: usize,
    pub y: usize,
    max_x: usize,
    max_y: usize,
    sensitivity: f32,
    invert_x: bool,
    invert_y: bool,
}

impl CursorTracker {
    /// Create a new cursor tracker
    pub fn new(max_x: usize, max_y: usize, invert_x: bool, invert_y: bool) -> Self {
        Self {
            x: max_x / 2,
            y: max_y / 2,
            max_x,
            max_y,
            sensitivity: 1.0,
            invert_x,
            invert_y,
        }
    }

    /// Update cursor position with mouse deltas
    pub fn update(&mut self, dx: i8, dy: i8) {
        // Apply axis inversions if configured
        let dx = if self.invert_x { -dx } else { dx };
        let dy = if self.invert_y { -dy } else { dy };

        // Apply sensitivity and deltas
        let new_x = (self.x as i32 + (dx as f32 * self.sensitivity) as i32)
            .max(0)
            .min(self.max_x as i32 - 1);
        let new_y = (self.y as i32 + (dy as f32 * self.sensitivity) as i32)
            .max(0)
            .min(self.max_y as i32 - 1);

        self.x = new_x as usize;
        self.y = new_y as usize;
    }

    /// Set cursor position
    #[allow(dead_code)]
    pub fn set_position(&mut self, x: usize, y: usize) {
        self.x = x.min(self.max_x - 1);
        self.y = y.min(self.max_y - 1);
    }

    /// Update screen bounds (for resize)
    #[allow(dead_code)]
    pub fn set_bounds(&mut self, max_x: usize, max_y: usize) {
        self.max_x = max_x;
        self.max_y = max_y;
        self.x = self.x.min(max_x - 1);
        self.y = self.y.min(max_y - 1);
    }

    /// Set mouse sensitivity
    #[allow(dead_code)]
    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity.clamp(0.1, 10.0);
    }
}
