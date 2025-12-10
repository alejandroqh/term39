//! Simple inline input field for settings
//!
//! A minimal text input component for entering short values like interface names.

use crate::rendering::{Cell, Theme, VideoBuffer};

/// A simple inline text input field
pub struct SimpleInput {
    pub text: String,
    pub cursor_position: usize,
    pub max_length: usize,
}

impl SimpleInput {
    /// Create a new simple input with initial text
    pub fn new(initial_text: &str, max_length: usize) -> Self {
        let text = initial_text.to_string();
        let cursor_position = text.len();
        Self {
            text,
            cursor_position,
            max_length,
        }
    }

    /// Insert a character at cursor position
    pub fn insert_char(&mut self, c: char) {
        if self.text.len() < self.max_length && c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            self.text.insert(self.cursor_position, c);
            self.cursor_position += 1;
        }
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.text.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start
    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.text.len();
    }

    /// Get the current text
    #[allow(dead_code)]
    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Render the input field at the given position
    /// Returns the width used
    pub fn render(
        &self,
        buffer: &mut VideoBuffer,
        x: u16,
        y: u16,
        field_width: u16,
        theme: &Theme,
        focused: bool,
    ) {
        let bg = if focused {
            theme.slight_input_bg
        } else {
            theme.config_content_bg
        };
        let fg = if focused {
            theme.slight_input_fg
        } else {
            theme.config_content_fg
        };

        // Render opening bracket
        buffer.set(x, y, Cell::new('[', fg, theme.config_content_bg));

        // Render input field background
        let inner_width = field_width.saturating_sub(2) as usize; // -2 for brackets
        for i in 0..inner_width {
            let ch = self.text.chars().nth(i).unwrap_or(' ');
            let cell_bg = if focused && i == self.cursor_position {
                fg // Inverted for cursor
            } else {
                bg
            };
            let cell_fg = if focused && i == self.cursor_position {
                bg // Inverted for cursor
            } else {
                fg
            };
            buffer.set(x + 1 + i as u16, y, Cell::new(ch, cell_fg, cell_bg));
        }

        // Render closing bracket
        buffer.set(
            x + field_width - 1,
            y,
            Cell::new(']', fg, theme.config_content_bg),
        );
    }
}
