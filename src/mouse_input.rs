//! Unified mouse input module
//!
//! Provides mouse input support across three scenarios:
//! 1. Terminal emulator (xterm, etc.) - Uses crossterm ANSI mouse protocol
//! 2. TTY direct (Linux console) - Uses raw /dev/input/ with inverted cell cursor
//! 3. Framebuffer mode - Uses raw /dev/input/ with sprite cursor rendering

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// Mouse input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants used on Linux only
pub enum MouseInputMode {
    /// Terminal emulator - uses crossterm ANSI mouse protocol
    TerminalEmulator,
    /// Linux console TTY - uses raw /dev/input with inverted cell cursor
    Tty,
    /// Framebuffer mode - uses raw /dev/input with sprite cursor
    Framebuffer,
}

impl MouseInputMode {
    /// Detect the appropriate mouse input mode based on environment
    #[allow(dead_code)] // Used on Linux only
    pub fn detect(framebuffer_requested: bool) -> Self {
        let is_linux_console = std::env::var("TERM").map(|t| t == "linux").unwrap_or(false);

        if framebuffer_requested {
            MouseInputMode::Framebuffer
        } else if is_linux_console {
            MouseInputMode::Tty
        } else {
            MouseInputMode::TerminalEmulator
        }
    }

    /// Returns true if this mode uses raw mouse input (not crossterm)
    pub fn uses_raw_input(&self) -> bool {
        matches!(self, MouseInputMode::Tty | MouseInputMode::Framebuffer)
    }
}

/// Mouse button states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

/// Raw mouse event with deltas
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Fields used on Linux only
pub struct RawMouseEvent {
    pub dx: i8,
    pub dy: i8,
    pub buttons: MouseButtons,
    pub scroll: i8,
    pub scroll_h: i8,
}

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

/// Mouse input protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Protocol {
    Ps2,
    InputEvent,
}

/// Raw mouse input reader
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
        const EVIOCGBIT_REL: libc::c_ulong = 0x80084522; // Get REL capability bits

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

/// Calculate adaptive mouse sensitivity based on screen size
/// Larger screens need higher sensitivity to traverse quickly
/// Smaller screens need lower sensitivity for precision
fn calculate_sensitivity(cols: usize, rows: usize) -> f32 {
    // Reference: 80x25 = 2000 cells, feels good at ~0.35 sensitivity
    let reference_cells = 80.0 * 25.0; // 2000 cells
    let current_cells = (cols * rows) as f32;

    // Base sensitivity for reference screen
    let base_sensitivity = 0.35;

    // Scale: larger screens get proportionally higher sensitivity
    // Use sqrt to prevent extreme values
    let scale = (current_cells / reference_cells).sqrt();

    // Clamp to reasonable range (0.2 minimum for precision, 1.0 maximum)
    (base_sensitivity * scale).clamp(0.2, 1.0)
}

/// Cursor position tracker
/// Uses usize for pixel coordinates (compatible with framebuffer renderer)
pub struct CursorTracker {
    pub x: usize,
    pub y: usize,
    max_x: usize,
    max_y: usize,
    sensitivity: f32,
    invert_x: bool,
    invert_y: bool,
    // Accumulators for fractional movement (prevents jumpy behavior)
    accum_x: f32,
    accum_y: f32,
}

impl CursorTracker {
    pub fn new(max_x: usize, max_y: usize, invert_x: bool, invert_y: bool) -> Self {
        // Calculate adaptive sensitivity based on screen size
        let sensitivity = calculate_sensitivity(max_x, max_y);
        Self {
            x: max_x / 2,
            y: max_y / 2,
            max_x,
            max_y,
            sensitivity,
            invert_x,
            invert_y,
            accum_x: 0.0,
            accum_y: 0.0,
        }
    }

    pub fn update(&mut self, dx: i8, dy: i8) {
        let dx = if self.invert_x { -dx } else { dx };
        // PS/2 mouse reports positive dy as "up" (toward user), but screen Y increases downward
        // So we invert by default, and the invert_y flag un-inverts it
        let dy = if self.invert_y { dy } else { -dy };

        // Accumulate fractional movement
        self.accum_x += dx as f32 * self.sensitivity;
        self.accum_y += dy as f32 * self.sensitivity;

        // Only move when accumulator reaches a full cell (±1.0)
        let move_x = self.accum_x.trunc() as i32;
        let move_y = self.accum_y.trunc() as i32;

        // Keep the fractional part for next update
        self.accum_x -= move_x as f32;
        self.accum_y -= move_y as f32;

        // Apply movement
        let new_x = (self.x as i32 + move_x).max(0).min(self.max_x as i32 - 1);
        let new_y = (self.y as i32 + move_y).max(0).min(self.max_y as i32 - 1);

        self.x = new_x as usize;
        self.y = new_y as usize;
    }

    #[allow(dead_code)]
    pub fn position(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    /// Get position as u16 (for crossterm event compatibility)
    pub fn position_u16(&self) -> (u16, u16) {
        (self.x as u16, self.y as u16)
    }

    #[allow(dead_code)]
    pub fn set_bounds(&mut self, max_x: usize, max_y: usize) {
        self.max_x = max_x;
        self.max_y = max_y;
        self.x = self.x.min(max_x.saturating_sub(1));
        self.y = self.y.min(max_y.saturating_sub(1));
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, x: usize, y: usize) {
        self.x = x.min(self.max_x.saturating_sub(1));
        self.y = y.min(self.max_y.saturating_sub(1));
    }

    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity.clamp(0.1, 5.0);
    }
}

/// Unified mouse input manager
///
/// Handles mouse input across all three modes (terminal emulator, TTY, framebuffer)
/// and converts raw events to crossterm MouseEvent format.
pub struct MouseInputManager {
    mode: MouseInputMode,
    raw_input: Option<RawMouseInput>,
    cursor: CursorTracker,
    prev_buttons: MouseButtons,
    event_queue: VecDeque<MouseEvent>,
    swap_buttons: bool,
}

impl MouseInputManager {
    /// Create a new mouse input manager
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: MouseInputMode,
        cols: u16,
        rows: u16,
        device_path: Option<&str>,
        invert_x: bool,
        invert_y: bool,
        swap_buttons: bool,
        sensitivity_override: Option<f32>,
    ) -> io::Result<Self> {
        let raw_input = if mode.uses_raw_input() {
            match RawMouseInput::new(device_path) {
                Ok(input) => Some(input),
                Err(e) => {
                    eprintln!("Warning: Could not open mouse device: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let mut cursor = CursorTracker::new(cols as usize, rows as usize, invert_x, invert_y);

        // Apply sensitivity override if provided
        if let Some(sens) = sensitivity_override {
            cursor.set_sensitivity(sens);
        }

        Ok(Self {
            mode,
            raw_input,
            cursor,
            prev_buttons: MouseButtons::default(),
            event_queue: VecDeque::new(),
            swap_buttons,
        })
    }

    /// Get the current mouse input mode
    #[allow(dead_code)]
    pub fn mode(&self) -> MouseInputMode {
        self.mode
    }

    /// Returns true if using raw mouse input (not crossterm)
    pub fn uses_raw_input(&self) -> bool {
        self.mode.uses_raw_input() && self.raw_input.is_some()
    }

    /// Get current cursor position
    pub fn cursor_position(&self) -> (u16, u16) {
        self.cursor.position_u16()
    }

    /// Update screen bounds (for resize)
    #[allow(dead_code)]
    pub fn set_bounds(&mut self, cols: u16, rows: u16) {
        self.cursor.set_bounds(cols as usize, rows as usize);
    }

    /// Poll for a mouse event
    /// Returns None if no event available or if using terminal emulator mode
    pub fn poll_event(&mut self) -> io::Result<Option<MouseEvent>> {
        // Return queued events first
        if let Some(event) = self.event_queue.pop_front() {
            return Ok(Some(event));
        }

        // Only poll raw input for TTY/Framebuffer modes
        let raw_input = match &mut self.raw_input {
            Some(input) => input,
            None => return Ok(None),
        };

        // Read raw event
        let raw_event = match raw_input.read_event()? {
            Some(e) => e,
            None => return Ok(None),
        };

        // Update cursor position
        self.cursor.update(raw_event.dx, raw_event.dy);
        let (col, row) = self.cursor.position_u16();

        // Convert to crossterm events
        self.generate_events(raw_event, col, row);

        // Return first event from queue
        Ok(self.event_queue.pop_front())
    }

    /// Generate crossterm MouseEvents from a raw event
    fn generate_events(&mut self, raw: RawMouseEvent, col: u16, row: u16) {
        let buttons = if self.swap_buttons {
            MouseButtons {
                left: raw.buttons.right,
                right: raw.buttons.left,
                middle: raw.buttons.middle,
            }
        } else {
            raw.buttons
        };

        // Check for button state changes
        // Down events
        if buttons.left && !self.prev_buttons.left {
            self.event_queue.push_back(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: col,
                row,
                modifiers: KeyModifiers::empty(),
            });
        }
        if buttons.right && !self.prev_buttons.right {
            self.event_queue.push_back(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Right),
                column: col,
                row,
                modifiers: KeyModifiers::empty(),
            });
        }
        if buttons.middle && !self.prev_buttons.middle {
            self.event_queue.push_back(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Middle),
                column: col,
                row,
                modifiers: KeyModifiers::empty(),
            });
        }

        // Up events
        if !buttons.left && self.prev_buttons.left {
            self.event_queue.push_back(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column: col,
                row,
                modifiers: KeyModifiers::empty(),
            });
        }
        if !buttons.right && self.prev_buttons.right {
            self.event_queue.push_back(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Right),
                column: col,
                row,
                modifiers: KeyModifiers::empty(),
            });
        }
        if !buttons.middle && self.prev_buttons.middle {
            self.event_queue.push_back(MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Middle),
                column: col,
                row,
                modifiers: KeyModifiers::empty(),
            });
        }

        // Drag events (movement while button held)
        if raw.dx != 0 || raw.dy != 0 {
            if buttons.left {
                self.event_queue.push_back(MouseEvent {
                    kind: MouseEventKind::Drag(MouseButton::Left),
                    column: col,
                    row,
                    modifiers: KeyModifiers::empty(),
                });
            } else if buttons.right {
                self.event_queue.push_back(MouseEvent {
                    kind: MouseEventKind::Drag(MouseButton::Right),
                    column: col,
                    row,
                    modifiers: KeyModifiers::empty(),
                });
            } else if buttons.middle {
                self.event_queue.push_back(MouseEvent {
                    kind: MouseEventKind::Drag(MouseButton::Middle),
                    column: col,
                    row,
                    modifiers: KeyModifiers::empty(),
                });
            } else {
                // Movement without button = Moved event
                self.event_queue.push_back(MouseEvent {
                    kind: MouseEventKind::Moved,
                    column: col,
                    row,
                    modifiers: KeyModifiers::empty(),
                });
            }
        }

        // Scroll events
        if raw.scroll > 0 {
            for _ in 0..raw.scroll {
                self.event_queue.push_back(MouseEvent {
                    kind: MouseEventKind::ScrollUp,
                    column: col,
                    row,
                    modifiers: KeyModifiers::empty(),
                });
            }
        } else if raw.scroll < 0 {
            for _ in 0..(-raw.scroll) {
                self.event_queue.push_back(MouseEvent {
                    kind: MouseEventKind::ScrollDown,
                    column: col,
                    row,
                    modifiers: KeyModifiers::empty(),
                });
            }
        }

        // Update previous button state
        self.prev_buttons = buttons;
    }
}

// Re-export for backward compatibility with framebuffer module
#[allow(unused_imports)]
pub use CursorTracker as MouseCursorTracker;
#[allow(unused_imports)]
pub use RawMouseInput as MouseInput;

/// Backward compatible MouseEvent type for framebuffer module
/// This mirrors the old framebuffer::mouse_input::MouseEvent
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Used by framebuffer backend on Linux only
pub struct FramebufferMouseEvent {
    pub dx: i8,
    pub dy: i8,
    pub buttons: MouseButtons,
    pub scroll: i8,
    pub scroll_h: i8,
}

impl From<RawMouseEvent> for FramebufferMouseEvent {
    fn from(raw: RawMouseEvent) -> Self {
        Self {
            dx: raw.dx,
            dy: raw.dy,
            buttons: raw.buttons,
            scroll: raw.scroll,
            scroll_h: raw.scroll_h,
        }
    }
}

impl RawMouseInput {
    /// Backward compatible read_event that returns FramebufferMouseEvent
    /// This is used by the framebuffer backend
    #[allow(dead_code)] // Used by framebuffer backend on Linux only
    pub fn read_event_compat(&mut self) -> io::Result<Option<FramebufferMouseEvent>> {
        self.read_event().map(|opt| opt.map(Into::into))
    }
}
