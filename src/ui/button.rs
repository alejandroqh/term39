use crate::rendering::{Cell, Theme, VideoBuffer};

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
}

#[derive(Debug, Clone)]
pub struct Button {
    pub x: u16,
    pub y: u16,
    pub label: String,
    pub state: ButtonState,
    pub enabled: bool,
}

impl Button {
    /// Create a new button at the specified position with the given label
    pub fn new(x: u16, y: u16, label: String) -> Self {
        Self {
            x,
            y,
            label,
            state: ButtonState::Normal,
            enabled: true,
        }
    }

    /// Get the width of the button (includes brackets and spaces)
    /// Format: "[ Label ]"
    pub fn width(&self) -> u16 {
        (self.label.len() as u16) + 4 // "[ " + label + " ]"
    }

    /// Check if a point (x, y) is inside the button
    pub fn contains(&self, x: u16, y: u16) -> bool {
        if !self.enabled {
            return false;
        }
        x >= self.x && x < self.x + self.width() && y == self.y
    }

    /// Set the button state
    pub fn set_state(&mut self, state: ButtonState) {
        self.state = state;
    }

    /// Render the button to the video buffer
    pub fn render(&self, buffer: &mut VideoBuffer, theme: &Theme) {
        if !self.enabled {
            return;
        }

        let (fg_color, bg_color) = match self.state {
            ButtonState::Normal => (theme.button_normal_fg, theme.button_normal_bg),
            ButtonState::Hovered => (theme.button_hovered_fg, theme.button_hovered_bg),
            ButtonState::Pressed => (theme.button_pressed_fg, theme.button_pressed_bg),
        };

        let mut current_x = self.x;

        // Render "[ "
        // Use new_unchecked for performance - theme colors are pre-validated
        buffer.set(
            current_x,
            self.y,
            Cell::new_unchecked('[', fg_color, bg_color),
        );
        current_x += 1;
        buffer.set(
            current_x,
            self.y,
            Cell::new_unchecked(' ', fg_color, bg_color),
        );
        current_x += 1;

        // Render label
        for ch in self.label.chars() {
            buffer.set(
                current_x,
                self.y,
                Cell::new_unchecked(ch, fg_color, bg_color),
            );
            current_x += 1;
        }

        // Render " ]"
        buffer.set(
            current_x,
            self.y,
            Cell::new_unchecked(' ', fg_color, bg_color),
        );
        current_x += 1;
        buffer.set(
            current_x,
            self.y,
            Cell::new_unchecked(']', fg_color, bg_color),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_width() {
        let button = Button::new(0, 0, "Test".to_string());
        assert_eq!(button.width(), 8); // "[ Test ]"
    }

    #[test]
    fn test_button_contains() {
        let button = Button::new(5, 10, "Click".to_string());
        // Button renders as "[ Click ]" from x=5 to x=13 (width=9)
        assert!(button.contains(5, 10)); // Left edge '['
        assert!(button.contains(8, 10)); // Middle 'i'
        assert!(button.contains(13, 10)); // Right edge ']'
        assert!(!button.contains(14, 10)); // Past right edge
        assert!(!button.contains(4, 10)); // Before left edge
        assert!(!button.contains(5, 11)); // Wrong row
    }
}
