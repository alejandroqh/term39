use crate::rendering::{Cell, Charset, Theme, VideoBuffer, render_shadow};
use crossterm::style::Color;

/// A modal info window with border and title (for Help, About, etc.)
pub struct InfoWindow {
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
    pub title: String,
    pub content: Vec<String>, // Pre-formatted lines of content
}

impl InfoWindow {
    /// Create a new info window (auto-sized and centered)
    pub fn new(title: String, content_text: &str, buffer_width: u16, buffer_height: u16) -> Self {
        // Split content into lines
        let content: Vec<String> = content_text.lines().map(|s| s.to_string()).collect();

        // Calculate dimensions based on content
        let max_line_width = content
            .iter()
            .map(|line| Self::strip_color_codes(line).len())
            .max()
            .unwrap_or(40) as u16;

        // Width: content + padding + borders
        let width = (max_line_width + 6).min(buffer_width - 4); // 6 = 2 borders + 4 padding

        // Height: title + content + padding + borders + instruction
        let content_lines = content.len() as u16;
        let height = (content_lines + 6).min(buffer_height - 4); // 6 = top border + 2 padding + bottom padding + instruction + bottom border

        // Center on screen
        let x = (buffer_width.saturating_sub(width)) / 2;
        let y = (buffer_height.saturating_sub(height)) / 2;

        Self {
            width,
            height,
            x,
            y,
            title,
            content,
        }
    }

    /// Strip color codes from a string for length calculation
    fn strip_color_codes(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Skip until we find '}'
                for next in chars.by_ref() {
                    if next == '}' {
                        break;
                    }
                }
            } else {
                result.push(ch);
            }
        }
        result
    }

    /// Parse color code and return the corresponding Color
    fn parse_color_code(code: &str) -> Option<Color> {
        match code {
            "Y" | "y" => Some(Color::Yellow),
            "C" | "c" => Some(Color::Cyan),
            "W" | "w" => Some(Color::White),
            "G" | "g" => Some(Color::Green),
            "R" | "r" => Some(Color::Red),
            "M" | "m" => Some(Color::Magenta),
            "B" | "b" => Some(Color::Blue),
            "DG" | "dg" => Some(Color::DarkGrey),
            _ => None,
        }
    }

    /// Render the info window to the video buffer
    pub fn render(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let title_bg = theme.config_title_bg;
        let title_fg = theme.config_title_fg;
        let border_color = theme.config_border;
        let content_bg = theme.config_content_bg;
        let content_fg = theme.config_content_fg;

        // Get border characters
        let top_left = charset.border_top_left();
        let top_right = charset.border_top_right();
        let bottom_left = charset.border_bottom_left();
        let bottom_right = charset.border_bottom_right();
        let horizontal = charset.border_horizontal();
        let vertical = charset.border_vertical();

        // Draw top border with title
        buffer.set(
            self.x,
            self.y,
            Cell::new(top_left, border_color, content_bg),
        );

        let title_text = format!(" {} ", self.title);
        let title_start = self.x + (self.width - title_text.len() as u16) / 2;

        // Fill top border before title
        for x in 1..title_start - self.x {
            buffer.set(
                self.x + x,
                self.y,
                Cell::new(horizontal, border_color, content_bg),
            );
        }

        // Render title with special background
        for (i, ch) in title_text.chars().enumerate() {
            buffer.set(
                title_start + i as u16,
                self.y,
                Cell::new(ch, title_fg, title_bg),
            );
        }

        // Fill top border after title
        for x in (title_start + title_text.len() as u16 - self.x)..(self.width - 1) {
            buffer.set(
                self.x + x,
                self.y,
                Cell::new(horizontal, border_color, content_bg),
            );
        }

        buffer.set(
            self.x + self.width - 1,
            self.y,
            Cell::new(top_right, border_color, content_bg),
        );

        // Draw content area with side borders
        for y in 1..(self.height - 1) {
            // Left border
            buffer.set(
                self.x,
                self.y + y,
                Cell::new(vertical, border_color, content_bg),
            );

            // Content area
            for x in 1..(self.width - 1) {
                buffer.set(
                    self.x + x,
                    self.y + y,
                    Cell::new(' ', content_fg, content_bg),
                );
            }

            // Right border
            buffer.set(
                self.x + self.width - 1,
                self.y + y,
                Cell::new(vertical, border_color, content_bg),
            );
        }

        // Draw bottom border
        buffer.set(
            self.x,
            self.y + self.height - 1,
            Cell::new(bottom_left, border_color, content_bg),
        );

        for x in 1..(self.width - 1) {
            buffer.set(
                self.x + x,
                self.y + self.height - 1,
                Cell::new(horizontal, border_color, content_bg),
            );
        }

        buffer.set(
            self.x + self.width - 1,
            self.y + self.height - 1,
            Cell::new(bottom_right, border_color, content_bg),
        );

        // Render content lines with color code support
        let content_start_y = self.y + 2; // Start after border and padding
        let content_x = self.x + 3; // Left padding

        for (i, line) in self.content.iter().enumerate() {
            let line_y = content_start_y + i as u16;
            if line_y >= self.y + self.height - 2 {
                break; // Don't draw past the instruction area
            }

            // Render line with color code parsing
            let mut current_x = content_x;
            let mut current_color = content_fg;
            let mut chars = line.chars();

            while let Some(ch) = chars.next() {
                if ch == '{' {
                    // Collect color code
                    let mut code = String::new();
                    for next in chars.by_ref() {
                        if next == '}' {
                            // Apply color if valid
                            if let Some(color) = Self::parse_color_code(&code) {
                                current_color = color;
                            }
                            break;
                        }
                        code.push(next);
                    }
                } else {
                    // Render character with current color
                    if current_x < self.x + self.width - 3 {
                        buffer.set(current_x, line_y, Cell::new(ch, current_color, content_bg));
                        current_x += 1;
                    }
                }
            }
        }

        // Render instruction at bottom
        let instruction = "Press ESC to close";
        let instruction_x = self.x + (self.width - instruction.len() as u16) / 2;
        let instruction_y = self.y + self.height - 2;

        for (i, ch) in instruction.chars().enumerate() {
            buffer.set(
                instruction_x + i as u16,
                instruction_y,
                Cell::new(ch, theme.config_instructions_fg, content_bg),
            );
        }

        // Render shadow
        render_shadow(
            buffer,
            self.x,
            self.y,
            self.width,
            self.height,
            charset,
            theme,
        );
    }

    /// Check if a point is inside the window bounds
    #[allow(dead_code)]
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}
