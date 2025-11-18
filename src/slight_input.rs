use crate::charset::Charset;
use crate::theme::Theme;
use crate::video_buffer::{Cell, VideoBuffer};

pub struct SlightInput {
    pub prompt_text: String,
    pub input_text: String,
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
    pub cursor_position: usize,
    pub input_col_start: u16,
}

impl SlightInput {
    pub fn new(buffer_width: u16, buffer_height: u16) -> Self {
        let prompt_text = "Enter a command to launch:".to_string();

        // Calculate dimensions (reduced height since no title)
        let width = 60;
        let height = 6;

        // Center on screen
        let x = (buffer_width.saturating_sub(width)) / 2;
        let y = (buffer_height.saturating_sub(height)) / 2;

        // Calculate input field position (absolute coordinates)
        let input_col_start = x + 3; // Border (2) + padding (1)

        SlightInput {
            prompt_text,
            input_text: String::new(),
            width,
            height,
            x,
            y,
            cursor_position: 0,
            input_col_start,
        }
    }

    pub fn render(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let content_width = self.width.saturating_sub(4); // Remove borders (2 on each side)

        // Fill entire popup with background
        for row in 0..self.height {
            for col in 0..self.width {
                let x = self.x + col;
                let y = self.y + row;
                buffer.set(x, y, Cell::new(' ', theme.slight_fg, theme.slight_bg));
            }
        }

        // Draw border
        // Top border
        buffer.set(
            self.x,
            self.y,
            Cell::new(
                charset.border_top_left,
                theme.slight_border,
                theme.slight_bg,
            ),
        );

        for col in 1..self.width - 1 {
            buffer.set(
                self.x + col,
                self.y,
                Cell::new(
                    charset.border_horizontal,
                    theme.slight_border,
                    theme.slight_bg,
                ),
            );
        }

        buffer.set(
            self.x + self.width - 1,
            self.y,
            Cell::new(
                charset.border_top_right,
                theme.slight_border,
                theme.slight_bg,
            ),
        );

        // Side borders
        for row in 1..self.height - 1 {
            buffer.set(
                self.x,
                self.y + row,
                Cell::new(
                    charset.border_vertical,
                    theme.slight_border,
                    theme.slight_bg,
                ),
            );

            buffer.set(
                self.x + 1,
                self.y + row,
                Cell::new(' ', theme.slight_fg, theme.slight_bg),
            );

            buffer.set(
                self.x + self.width - 2,
                self.y + row,
                Cell::new(' ', theme.slight_fg, theme.slight_bg),
            );

            buffer.set(
                self.x + self.width - 1,
                self.y + row,
                Cell::new(
                    charset.border_vertical,
                    theme.slight_border,
                    theme.slight_bg,
                ),
            );
        }

        // Bottom border
        buffer.set(
            self.x,
            self.y + self.height - 1,
            Cell::new(
                charset.border_bottom_left,
                theme.slight_border,
                theme.slight_bg,
            ),
        );

        for col in 1..self.width - 1 {
            buffer.set(
                self.x + col,
                self.y + self.height - 1,
                Cell::new(
                    charset.border_horizontal,
                    theme.slight_border,
                    theme.slight_bg,
                ),
            );
        }

        buffer.set(
            self.x + self.width - 1,
            self.y + self.height - 1,
            Cell::new(
                charset.border_bottom_right,
                theme.slight_border,
                theme.slight_bg,
            ),
        );

        // Render prompt text (centered)
        let prompt_y = self.y + 1;
        let prompt_start =
            self.x + 2 + (content_width.saturating_sub(self.prompt_text.len() as u16)) / 2;
        for (i, ch) in self.prompt_text.chars().enumerate() {
            if i < content_width as usize {
                buffer.set(
                    prompt_start + i as u16,
                    prompt_y,
                    Cell::new(ch, theme.slight_fg, theme.slight_bg),
                );
            }
        }

        // Render input field background
        let input_y = self.y + 3;
        let input_field_width = content_width.saturating_sub(2); // Add some padding
        for col in 0..input_field_width {
            buffer.set(
                self.input_col_start + col,
                input_y,
                Cell::new(' ', theme.slight_input_fg, theme.slight_input_bg),
            );
        }

        // Render input text
        for (i, ch) in self.input_text.chars().enumerate() {
            if i < input_field_width as usize {
                buffer.set(
                    self.input_col_start + i as u16,
                    input_y,
                    Cell::new(ch, theme.slight_input_fg, theme.slight_input_bg),
                );
            }
        }

        // Render cursor (if within visible area)
        if self.cursor_position < input_field_width as usize {
            let cursor_x = self.input_col_start + self.cursor_position as u16;
            let cursor_char = if self.cursor_position < self.input_text.len() {
                self.input_text
                    .chars()
                    .nth(self.cursor_position)
                    .unwrap_or(' ')
            } else {
                ' '
            };
            buffer.set(
                cursor_x,
                input_y,
                Cell::new(cursor_char, theme.slight_input_bg, theme.slight_input_fg),
            );
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.input_text.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.input_text.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_text.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input_text.len();
    }

    pub fn get_input(&self) -> String {
        self.input_text.clone()
    }
}
