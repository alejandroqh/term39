use crate::charset::Charset;
use crate::theme::Theme;
use crate::video_buffer::{self, Cell, VideoBuffer};

/// Which edge is being resized
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResizeEdge {
    Left,
    Right,
    Bottom,
}

/// Represents a window in the UI
#[derive(Clone, Debug)]
pub struct Window {
    pub id: u32,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub title: String,

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
    /// Create a new window
    pub fn new(id: u32, x: u16, y: u16, width: u16, height: u16, title: String) -> Self {
        // Minimum size to accommodate buttons and 2-char borders (24 = 4 for borders + 20 content)
        let width = width.max(24);
        let height = height.max(5);

        Self {
            id,
            x,
            y,
            width,
            height,
            title,
            is_focused: false,
            is_minimized: false,
            is_maximized: false,
            pre_maximize_x: x,
            pre_maximize_y: y,
            pre_maximize_width: width,
            pre_maximize_height: height,
        }
    }

    /// Check if point is in title bar (not including 2-char borders)
    pub fn is_in_title_bar(&self, x: u16, y: u16) -> bool {
        x > self.x + 1 && x < self.x + self.width - 2 && y == self.y
    }

    /// Check if point is in close button [X]
    pub fn is_in_close_button(&self, x: u16, y: u16) -> bool {
        // [X] is at position x+2 (after 2-char left border)
        y == self.y && x > self.x + 1 && x <= self.x + 4
    }

    /// Check if point is in maximize button [+]
    #[allow(dead_code)]
    pub fn is_in_maximize_button(&self, x: u16, y: u16) -> bool {
        // [+] is at position x+5
        y == self.y && x >= self.x + 5 && x <= self.x + 7
    }

    /// Check if point is in minimize button [_]
    #[allow(dead_code)]
    pub fn is_in_minimize_button(&self, x: u16, y: u16) -> bool {
        // [_] is at position x+8
        y == self.y && x >= self.x + 8 && x <= self.x + 10
    }

    /// Check if point is on left border (excluding corners)
    /// Both characters of the 2-char left border are resizable
    pub fn is_on_left_border(&self, x: u16, y: u16) -> bool {
        (x == self.x || x == self.x + 1) && y > self.y && y < self.y + self.height - 1
    }

    /// Check if point is on bottom border (excluding 2-char corners)
    pub fn is_on_bottom_border(&self, x: u16, y: u16) -> bool {
        y == self.y + self.height - 1 && x > self.x + 1 && x < self.x + self.width - 2
    }

    /// Check if point is on right border outer edge (resizable, excluding scrollbar)
    /// Only the outer character (width-1) is resizable, inner char (width-2) has scrollbar
    pub fn is_on_right_border(&self, x: u16, y: u16) -> bool {
        x == self.x + self.width - 1 && y > self.y && y < self.y + self.height - 1
    }

    /// Determine which resize edge (if any) is at the given point
    /// Returns Some(edge) if on a resizable border, None otherwise
    pub fn get_resize_edge(&self, x: u16, y: u16) -> Option<ResizeEdge> {
        if self.is_on_left_border(x, y) {
            Some(ResizeEdge::Left)
        } else if self.is_on_right_border(x, y) {
            Some(ResizeEdge::Right)
        } else if self.is_on_bottom_border(x, y) {
            Some(ResizeEdge::Bottom)
        } else {
            None
        }
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
    pub fn render(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        if self.is_minimized {
            return;
        }

        // Draw the window frame
        self.render_frame(buffer, charset, theme);

        // Draw the title bar with buttons
        self.render_title_bar(buffer, theme);

        // Draw the content area
        self.render_content(buffer, theme);

        // Draw the shadow
        video_buffer::render_shadow(
            buffer,
            self.x,
            self.y,
            self.width,
            self.height,
            charset,
            theme,
        );
    }

    fn render_frame(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        // Use window content background for title bar (both focused and unfocused)
        let title_bg = theme.window_content_bg;

        // Border background uses content bg for consistency
        let border_bg = theme.window_content_bg;

        // Top border (title bar) with corner characters
        // Top-left corner (2 chars wide)
        buffer.set(
            self.x,
            self.y,
            Cell::new(charset.border_top_left, theme.window_border, title_bg),
        );
        buffer.set(
            self.x + 1,
            self.y,
            Cell::new(charset.border_horizontal, theme.window_border, title_bg),
        );

        // Top border middle
        for x in 2..self.width - 3 {
            buffer.set(
                self.x + x,
                self.y,
                Cell::new(' ', theme.window_border, title_bg),
            );
        }

        // T-junction separator before corner (╠ - double line on right)
        buffer.set(
            self.x + self.width - 3,
            self.y,
            Cell::new('╠', theme.window_border, title_bg),
        );

        // Top-right corner (2 chars wide)
        buffer.set(
            self.x + self.width - 2,
            self.y,
            Cell::new(charset.border_horizontal, theme.window_border, title_bg),
        );
        buffer.set(
            self.x + self.width - 1,
            self.y,
            Cell::new(charset.border_top_right, theme.window_border, title_bg),
        );

        // Side borders - 2 characters wide
        for y in 1..self.height - 1 {
            // Left border (2 chars): outer vertical + inner space
            // Outer left border (resizable)
            buffer.set(
                self.x,
                self.y + y,
                Cell::new(charset.border_vertical, theme.window_border, border_bg),
            );
            // Inner left border (resizable)
            buffer.set(
                self.x + 1,
                self.y + y,
                Cell::new(' ', theme.window_border, border_bg),
            );

            // Right border (2 chars): inner space + outer vertical
            // Inner right border (scrollbar area) - use border_bg for fg to avoid white overlay
            buffer.set(
                self.x + self.width - 2,
                self.y + y,
                Cell::new(' ', border_bg, border_bg),
            );
            // Outer right border (resizable)
            buffer.set(
                self.x + self.width - 1,
                self.y + y,
                Cell::new(charset.border_vertical, theme.window_border, border_bg),
            );
        }

        // Bottom border - single char height with 2-char wide corners
        // Bottom-left corner
        buffer.set(
            self.x,
            self.y + self.height - 1,
            Cell::new(charset.border_bottom_left, theme.window_border, border_bg),
        );
        // Extension of bottom-left corner
        buffer.set(
            self.x + 1,
            self.y + self.height - 1,
            Cell::new(charset.border_horizontal, theme.window_border, border_bg),
        );

        // Bottom border middle (resizable)
        for x in 2..self.width - 2 {
            buffer.set(
                self.x + x,
                self.y + self.height - 1,
                Cell::new(charset.border_horizontal, theme.window_border, border_bg),
            );
        }

        // Extension of bottom-right corner
        buffer.set(
            self.x + self.width - 2,
            self.y + self.height - 1,
            Cell::new(charset.border_horizontal, theme.window_border, border_bg),
        );
        // Bottom-right corner
        buffer.set(
            self.x + self.width - 1,
            self.y + self.height - 1,
            Cell::new(charset.border_bottom_right, theme.window_border, border_bg),
        );
    }

    fn render_title_bar(&self, buffer: &mut VideoBuffer, theme: &Theme) {
        // Use window content background for title bar
        let title_bg = theme.window_content_bg;

        // Buttons: [X][+][_] followed by title
        let buttons = "[X][+][_] ";
        let mut x_offset = 2; // Start after 2-char left border

        // Render buttons with colored characters and consistent background
        for (i, ch) in buttons.chars().enumerate() {
            let (fg_color, bg_color) = match ch {
                'X' => (theme.button_close_color, theme.button_bg),
                '+' => (theme.button_maximize_color, theme.button_bg),
                '_' => (theme.button_minimize_color, theme.button_bg),
                '[' | ']' => (theme.window_border, theme.button_bg),
                _ => (theme.window_border, title_bg), // Space between buttons uses title background
            };
            buffer.set(self.x + x_offset, self.y, Cell::new(ch, fg_color, bg_color));
            x_offset += 1;

            // After each button group, there's a space
            if i == 2 || i == 5 || i == 8 {
                // These are the closing brackets
                continue;
            }
        }

        // Render title text with border color
        let title_start = self.x + x_offset;
        let title_space = (self.width as i32 - x_offset as i32 - 2) as u16; // -2 for right border

        for (i, ch) in self.title.chars().take(title_space as usize).enumerate() {
            buffer.set(
                title_start + i as u16,
                self.y,
                Cell::new(ch, theme.window_border, title_bg),
            );
        }
    }

    fn render_content(&self, buffer: &mut VideoBuffer, theme: &Theme) {
        // Fill content area with solid background color (no pattern)
        // Account for 2-char borders on left and right
        let content_char = ' ';

        for y in 1..self.height - 1 {
            for x in 2..self.width - 2 {
                buffer.set(
                    self.x + x,
                    self.y + y,
                    Cell::new(
                        content_char,
                        theme.window_content_fg,
                        theme.window_content_bg,
                    ),
                );
            }
        }
    }

    /// Get pre-maximize geometry (for session persistence)
    pub fn get_pre_maximize_geometry(&self) -> (u16, u16, u16, u16) {
        (
            self.pre_maximize_x,
            self.pre_maximize_y,
            self.pre_maximize_width,
            self.pre_maximize_height,
        )
    }

    /// Set pre-maximize geometry (for session restoration)
    pub fn set_pre_maximize_geometry(&mut self, x: u16, y: u16, width: u16, height: u16) {
        self.pre_maximize_x = x;
        self.pre_maximize_y = y;
        self.pre_maximize_width = width;
        self.pre_maximize_height = height;
    }
}
