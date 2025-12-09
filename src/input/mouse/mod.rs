//! Unified mouse input module
//!
//! Provides mouse input support across multiple platforms and scenarios:
//! 1. Terminal emulator (xterm, etc.) - Uses crossterm ANSI mouse protocol
//! 2. TTY direct (Linux/BSD console) - Uses raw device input with inverted cell cursor
//! 3. Framebuffer mode (Linux only) - Uses raw /dev/input with sprite cursor rendering

mod cursor;
mod types;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
mod bsd;

// Re-export public types
pub use cursor::CursorTracker;
pub use types::{MouseButtons, MouseInputMode, RawMouseEvent};

#[cfg(target_os = "linux")]
pub use linux::RawMouseInput;

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
pub use bsd::RawMouseInput;

use crossterm::event::MouseEvent;
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use crossterm::event::{KeyModifiers, MouseButton, MouseEventKind};
use std::collections::VecDeque;
use std::io;

/// Unified mouse input manager
///
/// Handles mouse input across all modes (terminal emulator, TTY, framebuffer)
/// and converts raw events to crossterm MouseEvent format.
pub struct MouseInputManager {
    mode: MouseInputMode,
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    raw_input: Option<RawMouseInput>,
    cursor: CursorTracker,
    #[cfg_attr(
        not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )),
        allow(dead_code)
    )]
    prev_buttons: MouseButtons,
    event_queue: VecDeque<MouseEvent>,
    #[cfg_attr(
        not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )),
        allow(dead_code)
    )]
    swap_buttons: bool,
}

impl MouseInputManager {
    /// Create a new mouse input manager
    #[allow(clippy::too_many_arguments)]
    #[allow(unused_variables)] // device_path unused on non-Unix
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
        #[cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
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
            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
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
        #[cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            self.mode.uses_raw_input() && self.raw_input.is_some()
        }
        #[cfg(not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        {
            false
        }
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

        // Only poll raw input for TTY/Framebuffer modes on supported platforms
        #[cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
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

        #[cfg(not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        {
            Ok(None)
        }
    }

    /// Generate crossterm MouseEvents from a raw event
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
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
        // Only generate drag if button was already pressed (not just pressed this frame)
        // to prevent Down+Drag double-events on initial click with movement
        if raw.dx != 0 || raw.dy != 0 {
            if buttons.left && self.prev_buttons.left {
                self.event_queue.push_back(MouseEvent {
                    kind: MouseEventKind::Drag(MouseButton::Left),
                    column: col,
                    row,
                    modifiers: KeyModifiers::empty(),
                });
            } else if buttons.right && self.prev_buttons.right {
                self.event_queue.push_back(MouseEvent {
                    kind: MouseEventKind::Drag(MouseButton::Right),
                    column: col,
                    row,
                    modifiers: KeyModifiers::empty(),
                });
            } else if buttons.middle && self.prev_buttons.middle {
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

/// Backward compatible MouseEvent type for framebuffer module
/// This mirrors the old framebuffer::mouse_input::MouseEvent
#[cfg(unix)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Used by framebuffer backend on Linux only
pub struct FramebufferMouseEvent {
    pub dx: i8,
    pub dy: i8,
    pub buttons: MouseButtons,
    pub scroll: i8,
    pub scroll_h: i8,
}

#[cfg(unix)]
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

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
impl RawMouseInput {
    /// Backward compatible read_event that returns FramebufferMouseEvent
    /// This is used by the framebuffer backend
    #[allow(dead_code)] // Used by framebuffer backend on Linux only
    pub fn read_event_compat(&mut self) -> io::Result<Option<FramebufferMouseEvent>> {
        self.read_event().map(|opt| opt.map(Into::into))
    }
}
