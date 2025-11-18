use crate::charset::Charset;
use crate::theme::Theme;
use crate::video_buffer::{Cell, VideoBuffer};

pub struct ErrorDialog {
    pub error_message: String,
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
    button_x: u16,
    button_y: u16,
    button_width: u16,
}

impl ErrorDialog {
    pub fn new(buffer_width: u16, buffer_height: u16, error_message: String) -> Self {
        // Calculate dimensions based on message length
        let max_message_width = 60u16;
        let message_lines = Self::wrap_text(&error_message, max_message_width as usize);
        let message_height = message_lines.len() as u16;

        // Dialog dimensions: title bar (1) + padding (1) + message lines + padding (1) + button (1) + padding (1) + border (2)
        let width = (max_message_width + 4).min(buffer_width.saturating_sub(4));
        let height = 7 + message_height;

        // Center on screen
        let x = (buffer_width.saturating_sub(width)) / 2;
        let y = (buffer_height.saturating_sub(height)) / 2;

        // OK button position (centered at bottom)
        let button_width = 8u16; // " [ OK ] "
        let button_x = x + (width - button_width) / 2;
        let button_y = y + height - 3; // 3 from bottom: border(1) + padding(1) + button(1)

        ErrorDialog {
            error_message,
            width,
            height,
            x,
            y,
            button_x,
            button_y,
            button_width,
        }
    }

    /// Wraps text to fit within a given width
    fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    pub fn render(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let content_width = self.width.saturating_sub(4); // Remove borders (2 on each side)

        // Fill entire dialog with background
        for row in 0..self.height {
            for col in 0..self.width {
                let x = self.x + col;
                let y = self.y + row;
                buffer.set(
                    x,
                    y,
                    Cell::new(' ', theme.prompt_danger_fg, theme.prompt_danger_bg),
                );
            }
        }

        // Draw border
        // Top border
        buffer.set(
            self.x,
            self.y,
            Cell::new(
                charset.border_top_left,
                theme.prompt_danger_fg,
                theme.prompt_danger_bg,
            ),
        );

        for col in 1..self.width - 1 {
            buffer.set(
                self.x + col,
                self.y,
                Cell::new(
                    charset.border_horizontal,
                    theme.prompt_danger_fg,
                    theme.prompt_danger_bg,
                ),
            );
        }

        buffer.set(
            self.x + self.width - 1,
            self.y,
            Cell::new(
                charset.border_top_right,
                theme.prompt_danger_fg,
                theme.prompt_danger_bg,
            ),
        );

        // Side borders
        for row in 1..self.height - 1 {
            buffer.set(
                self.x,
                self.y + row,
                Cell::new(
                    charset.border_vertical,
                    theme.prompt_danger_fg,
                    theme.prompt_danger_bg,
                ),
            );

            buffer.set(
                self.x + self.width - 1,
                self.y + row,
                Cell::new(
                    charset.border_vertical,
                    theme.prompt_danger_fg,
                    theme.prompt_danger_bg,
                ),
            );
        }

        // Bottom border
        buffer.set(
            self.x,
            self.y + self.height - 1,
            Cell::new(
                charset.border_bottom_left,
                theme.prompt_danger_fg,
                theme.prompt_danger_bg,
            ),
        );

        for col in 1..self.width - 1 {
            buffer.set(
                self.x + col,
                self.y + self.height - 1,
                Cell::new(
                    charset.border_horizontal,
                    theme.prompt_danger_fg,
                    theme.prompt_danger_bg,
                ),
            );
        }

        buffer.set(
            self.x + self.width - 1,
            self.y + self.height - 1,
            Cell::new(
                charset.border_bottom_right,
                theme.prompt_danger_fg,
                theme.prompt_danger_bg,
            ),
        );

        // Render title "Error" centered in top row
        let title = "Error";
        let title_start = self.x + 2 + (content_width.saturating_sub(title.len() as u16)) / 2;
        for (i, ch) in title.chars().enumerate() {
            buffer.set(
                title_start + i as u16,
                self.y + 1,
                Cell::new(ch, theme.prompt_danger_fg, theme.prompt_danger_bg),
            );
        }

        // Render error message (word-wrapped, centered)
        let message_lines = Self::wrap_text(&self.error_message, content_width as usize);
        let message_start_y = self.y + 3;

        for (line_idx, line) in message_lines.iter().enumerate() {
            let line_y = message_start_y + line_idx as u16;
            let line_start = self.x + 2 + (content_width.saturating_sub(line.len() as u16)) / 2;

            for (i, ch) in line.chars().enumerate() {
                if i < content_width as usize {
                    buffer.set(
                        line_start + i as u16,
                        line_y,
                        Cell::new(ch, theme.prompt_danger_fg, theme.prompt_danger_bg),
                    );
                }
            }
        }

        // Render [OK] button
        let button_text = " [ OK ] ";
        for (i, ch) in button_text.chars().enumerate() {
            buffer.set(
                self.button_x + i as u16,
                self.button_y,
                Cell::new(
                    ch,
                    theme.dialog_button_primary_danger_fg,
                    theme.dialog_button_primary_danger_bg,
                ),
            );
        }

        // Draw shadow (like windows have)
        self.draw_shadow(buffer, charset, theme);
    }

    fn draw_shadow(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let shadow_char = charset.shadow;
        let shadow_color = theme.window_shadow_color;
        let (buffer_width, buffer_height) = buffer.dimensions();

        // Right shadow (2 cells wide)
        for row in 1..=self.height {
            for offset in 0..2 {
                let x = self.x + self.width + offset;
                let y = self.y + row;
                if x < buffer_width && y < buffer_height {
                    if let Some(existing_cell) = buffer.get(x, y) {
                        let cell = Cell::new(shadow_char, existing_cell.fg_color, shadow_color);
                        buffer.set(x, y, cell);
                    }
                }
            }
        }

        // Bottom shadow (2 cells wide)
        for col in 2..=self.width + 1 {
            let x = self.x + col;
            let y = self.y + self.height;
            if x < buffer_width && y < buffer_height {
                if let Some(existing_cell) = buffer.get(x, y) {
                    let cell = Cell::new(shadow_char, existing_cell.fg_color, shadow_color);
                    buffer.set(x, y, cell);
                }
            }
        }
    }

    /// Check if a click is on the OK button
    pub fn is_ok_button_clicked(&self, mouse_x: u16, mouse_y: u16) -> bool {
        mouse_y == self.button_y
            && mouse_x >= self.button_x
            && mouse_x < self.button_x + self.button_width
    }
}
