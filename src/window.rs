use crate::charset::Charset;
use crate::video_buffer::{Cell, VideoBuffer};
use crossterm::style::Color;

/// Represents a window in the UI
#[derive(Clone, Debug)]
pub struct Window {
    pub id: u32,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub title: String,

    // Colors
    pub title_bg: Color,
    pub title_fg: Color,
    pub border_color: Color,
    pub content_bg: Color,
    pub content_fg: Color,

    // State
    pub is_focused: bool,
    pub is_minimized: bool,
    pub is_maximized: bool,

    // Pre-maximize state (for restore)
    pre_maximize_x: u16,
    pre_maximize_y: u16,
    pre_maximize_width: u16,
    pre_maximize_height: u16,
}

impl Window {
    /// Create a new window with default colors (dark blue content, classic DOS style)
    pub fn new(id: u32, x: u16, y: u16, width: u16, height: u16, title: String) -> Self {
        // Minimum size to accommodate buttons and resize handle
        let width = width.max(20);
        let height = height.max(5);

        Self {
            id,
            x,
            y,
            width,
            height,
            title,
            title_bg: Color::Black,
            title_fg: Color::White,
            border_color: Color::White,
            content_bg: Color::DarkBlue,  // Use DarkBlue to distinguish from background Blue
            content_fg: Color::White,
            is_focused: false,
            is_minimized: false,
            is_maximized: false,
            pre_maximize_x: x,
            pre_maximize_y: y,
            pre_maximize_width: width,
            pre_maximize_height: height,
        }
    }

    /// Check if point is in title bar (not including border)
    pub fn is_in_title_bar(&self, x: u16, y: u16) -> bool {
        x >= self.x + 1 && x < self.x + self.width - 1 && y == self.y
    }

    /// Check if point is in close button [X]
    pub fn is_in_close_button(&self, x: u16, y: u16) -> bool {
        // [X] is at position x+1
        y == self.y && x >= self.x + 1 && x <= self.x + 3
    }

    /// Check if point is in maximize button [+]
    #[allow(dead_code)]
    pub fn is_in_maximize_button(&self, x: u16, y: u16) -> bool {
        // [+] is at position x+4
        y == self.y && x >= self.x + 4 && x <= self.x + 6
    }

    /// Check if point is in minimize button [_]
    #[allow(dead_code)]
    pub fn is_in_minimize_button(&self, x: u16, y: u16) -> bool {
        // [_] is at position x+7
        y == self.y && x >= self.x + 7 && x <= self.x + 9
    }

    /// Check if point is in resize handle (bottom-right corner)
    pub fn is_in_resize_handle(&self, x: u16, y: u16) -> bool {
        let corner_x = self.x + self.width - 1;
        let corner_y = self.y + self.height - 1;
        x == corner_x && y == corner_y
    }

    /// Check if point is within window bounds (including border)
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Maximize the window to fill the screen (except top bar)
    pub fn maximize(&mut self, buffer_width: u16, buffer_height: u16) {
        if !self.is_maximized {
            // Save current position and size
            self.pre_maximize_x = self.x;
            self.pre_maximize_y = self.y;
            self.pre_maximize_width = self.width;
            self.pre_maximize_height = self.height;

            // Set to full screen (leaving top bar at row 0)
            self.x = 0;
            self.y = 1;
            self.width = buffer_width;
            self.height = buffer_height - 1;

            self.is_maximized = true;
        }
    }

    /// Restore the window to its pre-maximize state
    pub fn restore_from_maximize(&mut self) {
        if self.is_maximized {
            self.x = self.pre_maximize_x;
            self.y = self.pre_maximize_y;
            self.width = self.pre_maximize_width;
            self.height = self.pre_maximize_height;

            self.is_maximized = false;
        }
    }

    /// Toggle maximize state
    pub fn toggle_maximize(&mut self, buffer_width: u16, buffer_height: u16) {
        if self.is_maximized {
            self.restore_from_maximize();
        } else {
            self.maximize(buffer_width, buffer_height);
        }
    }

    /// Minimize the window (hide it from view)
    pub fn minimize(&mut self) {
        self.is_minimized = true;
    }

    /// Restore the window from minimized state
    pub fn restore_from_minimize(&mut self) {
        self.is_minimized = false;
    }

    /// Toggle minimize state
    #[allow(dead_code)]
    pub fn toggle_minimize(&mut self) {
        if self.is_minimized {
            self.restore_from_minimize();
        } else {
            self.minimize();
        }
    }

    /// Render the window to the video buffer
    pub fn render(&self, buffer: &mut VideoBuffer, is_resizing: bool, charset: &Charset) {
        if self.is_minimized {
            return;
        }

        // Draw the window frame
        self.render_frame(buffer, is_resizing, charset);

        // Draw the title bar with buttons
        self.render_title_bar(buffer);

        // Draw the content area
        self.render_content(buffer);

        // Draw the shadow
        self.render_shadow(buffer, charset);
    }

    fn render_frame(&self, buffer: &mut VideoBuffer, is_resizing: bool, charset: &Charset) {
        // Change title bar background color based on focus
        let title_bg = if self.is_focused {
            Color::DarkCyan
        } else {
            self.title_bg
        };

        // Top border (title bar) - no border characters, just background
        for x in 0..self.width {
            buffer.set(
                self.x + x,
                self.y,
                Cell::new(' ', self.border_color, title_bg),
            );
        }

        // Side borders and content
        for y in 1..self.height - 1 {
            // Left border
            buffer.set(
                self.x,
                self.y + y,
                Cell::new(charset.border_vertical, self.border_color, self.content_bg),
            );
            // Right border
            buffer.set(
                self.x + self.width - 1,
                self.y + y,
                Cell::new(charset.border_vertical, self.border_color, self.content_bg),
            );
        }

        // Bottom border
        buffer.set(
            self.x,
            self.y + self.height - 1,
            Cell::new(charset.border_bottom_left, self.border_color, self.content_bg),
        );
        for x in 1..self.width - 1 {
            buffer.set(
                self.x + x,
                self.y + self.height - 1,
                Cell::new(charset.border_horizontal, self.border_color, self.content_bg),
            );
        }

        // Bottom-right corner with resize handle
        // Change colors based on whether we're actively resizing
        let (resize_fg, resize_bg) = if is_resizing {
            (Color::Yellow, Color::Grey)  // Bright colors during interaction
        } else {
            (Color::Grey, self.title_bg)  // Normal state - background matches title bar (black)
        };

        buffer.set(
            self.x + self.width - 1,
            self.y + self.height - 1,
            Cell::new(charset.resize_handle, resize_fg, resize_bg),
        );
    }

    fn render_title_bar(&self, buffer: &mut VideoBuffer) {
        // Change title bar background color based on focus
        let title_bg = if self.is_focused {
            Color::DarkCyan
        } else {
            self.title_bg
        };

        // Buttons: [X][+][_] followed by title
        let buttons = "[X][+][_] ";
        let mut x_offset = 1;

        // Render buttons with colored characters
        for (i, ch) in buttons.chars().enumerate() {
            let color = match ch {
                'X' => Color::Red,
                '+' => Color::Green,
                '_' => Color::Yellow,
                _ => self.title_fg,
            };
            buffer.set(
                self.x + x_offset,
                self.y,
                Cell::new(ch, color, title_bg),
            );
            x_offset += 1;

            // After each button group, there's a space
            if i == 2 || i == 5 || i == 8 {
                // These are the closing brackets
                continue;
            }
        }

        // Render title text
        let title_start = self.x + x_offset;
        let title_space = (self.width as i32 - x_offset as i32 - 1) as u16;

        for (i, ch) in self.title.chars().take(title_space as usize).enumerate() {
            buffer.set(
                title_start + i as u16,
                self.y,
                Cell::new(ch, self.title_fg, title_bg),
            );
        }
    }

    fn render_content(&self, buffer: &mut VideoBuffer) {
        // Fill content area with solid background color (no pattern)
        let content_char = ' ';

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                buffer.set(
                    self.x + x,
                    self.y + y,
                    Cell::new(content_char, self.content_fg, self.content_bg),
                );
            }
        }
    }

    fn render_shadow(&self, buffer: &mut VideoBuffer, charset: &Charset) {
        // Use shadow character from charset configuration
        let shadow_char = charset.shadow;
        let shadow_color_bg = Color::DarkGrey;
        let shadow_color_fg = Color::DarkGrey;
        let (buffer_width, buffer_height) = buffer.dimensions();

        // Right shadow (1 cell to the right)
        let shadow_x = self.x + self.width;
        if shadow_x < buffer_width {
            for y in 1..=self.height {
                let shadow_y = self.y + y;
                if shadow_y < buffer_height {
                    buffer.set(
                        shadow_x,
                        shadow_y,
                        Cell::new(shadow_char, shadow_color_fg, shadow_color_bg),
                    );
                }
            }
        }

        // Bottom shadow (1 cell down)
        let shadow_y = self.y + self.height;
        if shadow_y < buffer_height {
            for x in 1..=self.width {
                let shadow_x = self.x + x;
                if shadow_x < buffer_width {
                    buffer.set(
                        shadow_x,
                        shadow_y,
                        Cell::new(shadow_char, shadow_color_fg, shadow_color_bg),
                    );
                }
            }
        }
    }
}
