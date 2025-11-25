//! Keyboard mode management for vim-like window control
//!
//! This module defines the modal keyboard system that allows full keyboard-only
//! control of windows. Press backtick (`) to toggle Window Mode.

use std::fmt;
use std::time::Instant;

/// The main application keyboard mode
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum KeyboardMode {
    /// Normal mode - keyboard input goes to focused terminal
    #[default]
    Normal,
    /// Window management mode - keyboard controls windows
    WindowMode(WindowSubMode),
}

impl KeyboardMode {
    /// Check if currently in any window mode
    pub fn is_window_mode(&self) -> bool {
        matches!(self, KeyboardMode::WindowMode(_))
    }

    /// Get the current sub-mode if in window mode
    #[allow(dead_code)]
    pub fn sub_mode(&self) -> Option<WindowSubMode> {
        match self {
            KeyboardMode::WindowMode(sub) => Some(*sub),
            KeyboardMode::Normal => None,
        }
    }

    /// Toggle between Normal and WindowMode (Navigation)
    pub fn toggle(&mut self) {
        *self = match self {
            KeyboardMode::Normal => KeyboardMode::WindowMode(WindowSubMode::Navigation),
            KeyboardMode::WindowMode(_) => KeyboardMode::Normal,
        };
    }

    /// Enter a specific sub-mode (only if already in WindowMode)
    pub fn enter_sub_mode(&mut self, sub_mode: WindowSubMode) {
        if self.is_window_mode() {
            *self = KeyboardMode::WindowMode(sub_mode);
        }
    }

    /// Return to Navigation sub-mode (from Move or Resize)
    pub fn return_to_navigation(&mut self) {
        if self.is_window_mode() {
            *self = KeyboardMode::WindowMode(WindowSubMode::Navigation);
        }
    }

    /// Exit to Normal mode
    pub fn exit_to_normal(&mut self) {
        *self = KeyboardMode::Normal;
    }
}

impl fmt::Display for KeyboardMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyboardMode::Normal => write!(f, ""),
            KeyboardMode::WindowMode(sub) => write!(f, "{}", sub),
        }
    }
}

/// Sub-modes within Window Mode
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum WindowSubMode {
    /// Default window mode - navigation and quick actions
    #[default]
    Navigation,
    /// Move mode - h/j/k/l moves the focused window
    Move,
    /// Resize mode - h/j/k/l resizes the focused window
    Resize(ResizeDirection),
}

impl fmt::Display for WindowSubMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowSubMode::Navigation => write!(f, "[WIN]"),
            WindowSubMode::Move => write!(f, "[WIN:MOVE]"),
            WindowSubMode::Resize(_) => write!(f, "[WIN:SIZE]"),
        }
    }
}

/// Resize direction (kept for API compatibility, but Shift modifier now controls edge)
/// Without Shift: resize from right/bottom edges
/// With Shift: resize from left/top edges (push)
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum ResizeDirection {
    #[default]
    Default,
}

/// Snap positions using numpad-style layout:
/// ```text
/// 7 8 9   TopLeft    TopCenter    TopRight
/// 4 5 6   MiddleLeft Center       MiddleRight
/// 1 2 3   BottomLeft BottomCenter BottomRight
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapPosition {
    // Numpad positions (1-9)
    BottomLeft,   // 1
    BottomCenter, // 2
    BottomRight,  // 3
    MiddleLeft,   // 4
    Center,       // 5
    MiddleRight,  // 6
    TopLeft,      // 7
    TopCenter,    // 8
    TopRight,     // 9

    // Full half positions (Shift + h/j/k/l)
    FullLeft,   // Left half of screen
    FullRight,  // Right half of screen
    FullTop,    // Top half of screen
    FullBottom, // Bottom half of screen
}

impl SnapPosition {
    /// Create from numpad key (1-9)
    #[allow(dead_code)]
    pub fn from_numpad(key: char) -> Option<Self> {
        match key {
            '1' => Some(SnapPosition::BottomLeft),
            '2' => Some(SnapPosition::BottomCenter),
            '3' => Some(SnapPosition::BottomRight),
            '4' => Some(SnapPosition::MiddleLeft),
            '5' => Some(SnapPosition::Center),
            '6' => Some(SnapPosition::MiddleRight),
            '7' => Some(SnapPosition::TopLeft),
            '8' => Some(SnapPosition::TopCenter),
            '9' => Some(SnapPosition::TopRight),
            _ => None,
        }
    }

    /// Calculate the window rectangle for this snap position
    /// Returns (x, y, width, height)
    /// `top_bar_y` is typically 1 (row 0 is top bar)
    pub fn calculate_rect(
        &self,
        buffer_width: u16,
        buffer_height: u16,
        top_bar_y: u16,
    ) -> (u16, u16, u16, u16) {
        let usable_height = buffer_height.saturating_sub(top_bar_y + 1); // -1 for bottom bar
        let half_width = buffer_width / 2;
        let half_height = usable_height / 2;
        let third_width = buffer_width / 3;
        let third_height = usable_height / 3;

        match self {
            // Corner positions (quarters)
            SnapPosition::TopLeft => (0, top_bar_y, half_width, half_height),
            SnapPosition::TopRight => (
                half_width,
                top_bar_y,
                buffer_width - half_width,
                half_height,
            ),
            SnapPosition::BottomLeft => (
                0,
                top_bar_y + half_height,
                half_width,
                usable_height - half_height,
            ),
            SnapPosition::BottomRight => (
                half_width,
                top_bar_y + half_height,
                buffer_width - half_width,
                usable_height - half_height,
            ),

            // Edge centers (thirds width or height)
            SnapPosition::TopCenter => (third_width, top_bar_y, third_width, half_height),
            SnapPosition::BottomCenter => (
                third_width,
                top_bar_y + half_height,
                third_width,
                usable_height - half_height,
            ),
            SnapPosition::MiddleLeft => (0, top_bar_y + third_height, half_width, third_height),
            SnapPosition::MiddleRight => (
                half_width,
                top_bar_y + third_height,
                half_width,
                third_height,
            ),

            // Center (third of each dimension)
            SnapPosition::Center => (
                third_width,
                top_bar_y + third_height,
                third_width,
                third_height,
            ),

            // Full halves
            SnapPosition::FullLeft => (0, top_bar_y, half_width, usable_height),
            SnapPosition::FullRight => (
                half_width,
                top_bar_y,
                buffer_width - half_width,
                usable_height,
            ),
            SnapPosition::FullTop => (0, top_bar_y, buffer_width, half_height),
            SnapPosition::FullBottom => (
                0,
                top_bar_y + half_height,
                buffer_width,
                usable_height - half_height,
            ),
        }
    }
}

/// Direction for spatial window navigation
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Tracks movement/resize timing for adaptive step size acceleration
#[derive(Clone, Debug, Default)]
pub struct MovementState {
    /// Timestamp of last key press
    last_press: Option<Instant>,
}

impl MovementState {
    /// Create a new movement state
    pub fn new() -> Self {
        Self { last_press: None }
    }

    /// Get the step size based on timing since last press
    /// Returns adaptive step: 1 (slow) -> 2 -> 4 -> 8 (rapid)
    pub fn get_step(&mut self) -> u16 {
        let now = Instant::now();
        let step = match self.last_press {
            None => 1, // First press: precise
            Some(last) => {
                let elapsed = now.duration_since(last).as_millis();
                match elapsed {
                    0..=50 => 8,    // Rapid
                    51..=100 => 4,  // Fast
                    101..=200 => 2, // Medium
                    _ => 1,         // Slow/reset to precise
                }
            }
        };
        self.last_press = Some(now);
        step
    }

    /// Reset the timing state
    pub fn reset(&mut self) {
        self.last_press = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_mode_toggle() {
        let mut mode = KeyboardMode::Normal;
        assert!(!mode.is_window_mode());

        mode.toggle();
        assert!(mode.is_window_mode());
        assert_eq!(mode.sub_mode(), Some(WindowSubMode::Navigation));

        mode.toggle();
        assert!(!mode.is_window_mode());
    }

    #[test]
    fn test_sub_mode_transitions() {
        let mut mode = KeyboardMode::WindowMode(WindowSubMode::Navigation);

        mode.enter_sub_mode(WindowSubMode::Move);
        assert_eq!(mode.sub_mode(), Some(WindowSubMode::Move));

        mode.return_to_navigation();
        assert_eq!(mode.sub_mode(), Some(WindowSubMode::Navigation));
    }

    #[test]
    fn test_snap_position_from_numpad() {
        assert_eq!(
            SnapPosition::from_numpad('1'),
            Some(SnapPosition::BottomLeft)
        );
        assert_eq!(SnapPosition::from_numpad('5'), Some(SnapPosition::Center));
        assert_eq!(SnapPosition::from_numpad('9'), Some(SnapPosition::TopRight));
        assert_eq!(SnapPosition::from_numpad('0'), None);
    }

}
