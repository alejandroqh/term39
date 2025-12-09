use crate::rendering::{Cell, Charset, Theme, VideoBuffer, render_shadow};

/// Which edge is being resized
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResizeEdge {
    Left,
    Right,
    Bottom,
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
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
        // Button layout: "[X] [+] [_] " - Close is at positions 2-4
        y == self.y && x >= self.x + 2 && x <= self.x + 4
    }

    /// Check if point is in maximize button [+]
    pub fn is_in_maximize_button(&self, x: u16, y: u16) -> bool {
        // [+] is at position x+6 (after "[X] ")
        // Button layout: "[X] [+] [_] " - Maximize is chars 4-6 (positions 6-8)
        y == self.y && x >= self.x + 6 && x <= self.x + 8
    }

    /// Check if point is in minimize button [_]
    pub fn is_in_minimize_button(&self, x: u16, y: u16) -> bool {
        // [_] is at position x+10 (after "[X] [+] ")
        // Button layout: "[X] [+] [_] " - Minimize is chars 8-10 (positions 10-12)
        y == self.y && x >= self.x + 10 && x <= self.x + 12
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

    /// Check if point is in bottom-left corner (2-char wide corner area)
    pub fn is_in_bottom_left_corner(&self, x: u16, y: u16) -> bool {
        y == self.y + self.height - 1 && (x == self.x || x == self.x + 1)
    }

    /// Check if point is in bottom-right corner (2-char wide corner area)
    pub fn is_in_bottom_right_corner(&self, x: u16, y: u16) -> bool {
        y == self.y + self.height - 1
            && (x == self.x + self.width - 2 || x == self.x + self.width - 1)
    }

    /// Check if point is in top-left corner (2-char wide corner area)
    pub fn is_in_top_left_corner(&self, x: u16, y: u16) -> bool {
        y == self.y && (x == self.x || x == self.x + 1)
    }

    /// Check if point is in top-right corner (2-char wide corner area)
    pub fn is_in_top_right_corner(&self, x: u16, y: u16) -> bool {
        y == self.y && (x == self.x + self.width - 2 || x == self.x + self.width - 1)
    }

    /// Determine which resize edge (if any) is at the given point
    /// Returns Some(edge) if on a resizable border, None otherwise
    /// Checks corners first, then edges
    pub fn get_resize_edge(&self, x: u16, y: u16) -> Option<ResizeEdge> {
        // Check corners first (they take priority over edges)
        if self.is_in_bottom_left_corner(x, y) {
            Some(ResizeEdge::BottomLeft)
        } else if self.is_in_bottom_right_corner(x, y) {
            Some(ResizeEdge::BottomRight)
        } else if self.is_in_top_left_corner(x, y) {
            Some(ResizeEdge::TopLeft)
        } else if self.is_in_top_right_corner(x, y) {
            Some(ResizeEdge::TopRight)
        } else if self.is_on_left_border(x, y) {
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
    /// If `gaps` is true, leaves 1 char gap on all edges and accounts for shadow
    pub fn maximize(&mut self, buffer_width: u16, buffer_height: u16, gaps: bool) {
        if !self.is_maximized {
            // Save current position and size
            self.pre_maximize_x = self.x;
            self.pre_maximize_y = self.y;
            self.pre_maximize_width = self.width;
            self.pre_maximize_height = self.height;

            if gaps {
                // With gaps: 1 char edge gap + 2 char shadow on right/bottom
                const EDGE_GAP: u16 = 1;
                const SHADOW_SIZE: u16 = 2;

                self.x = EDGE_GAP;
                self.y = 1 + EDGE_GAP; // 1 for top bar + gap
                // Width: buffer_width - left_gap - shadow - right_gap
                self.width = buffer_width.saturating_sub(2 * EDGE_GAP + SHADOW_SIZE);
                // Height: buffer_height - top_bar(1) - top_gap - shadow - bottom_gap
                self.height = buffer_height.saturating_sub(1 + 2 * EDGE_GAP + SHADOW_SIZE);
            } else {
                // No gaps: full screen (leaving top bar at row 0)
                self.x = 0;
                self.y = 1;
                self.width = buffer_width;
                self.height = buffer_height - 1;
            }

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
    /// If `gaps` is true, maximized window will have gaps around edges
    pub fn toggle_maximize(&mut self, buffer_width: u16, buffer_height: u16, gaps: bool) {
        if self.is_maximized {
            self.restore_from_maximize();
        } else {
            self.maximize(buffer_width, buffer_height, gaps);
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
    #[allow(dead_code)]
    pub fn render(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        self.render_with_title(buffer, charset, theme, None, false);
    }

    /// Render the window with an optional dynamic title override
    /// If keyboard_mode_active is true and window is focused, uses keyboard mode colors
    pub fn render_with_title(
        &self,
        buffer: &mut VideoBuffer,
        charset: &Charset,
        theme: &Theme,
        dynamic_title: Option<&str>,
        keyboard_mode_active: bool,
    ) {
        if self.is_minimized {
            return;
        }

        // Draw the window frame
        self.render_frame(buffer, charset, theme, keyboard_mode_active);

        // Draw the title bar with buttons
        self.render_title_bar(buffer, theme, dynamic_title, keyboard_mode_active);

        // Draw the content area
        self.render_content(buffer, theme);

        // Draw the shadow
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

    fn render_frame(
        &self,
        buffer: &mut VideoBuffer,
        charset: &Charset,
        theme: &Theme,
        keyboard_mode_active: bool,
    ) {
        // Use keyboard mode colors when active and focused, otherwise normal focus state colors
        let use_keyboard_colors = keyboard_mode_active && self.is_focused;

        // Use different backgrounds based on focus state
        let title_bg = if use_keyboard_colors {
            theme.keyboard_mode_title_bg
        } else if self.is_focused {
            theme.window_title_focused_bg
        } else {
            theme.window_title_unfocused_bg
        };

        // Border colors based on focus state
        let border_bg = if use_keyboard_colors {
            theme.keyboard_mode_border_bg
        } else if self.is_focused {
            theme.window_border_focused_bg
        } else {
            theme.window_border_unfocused_bg
        };

        // Border foreground color based on focus state
        let border_fg = if use_keyboard_colors {
            theme.keyboard_mode_border_fg
        } else if self.is_focused {
            theme.window_border_focused_fg
        } else {
            theme.window_border_unfocused_fg
        };

        // Top border (title bar) with corner characters
        // Top-left corner (2 chars wide)
        // Use new_unchecked for performance - theme colors are pre-validated
        buffer.set(
            self.x,
            self.y,
            Cell::new_unchecked(charset.border_top_left, border_fg, title_bg),
        );
        buffer.set(
            self.x + 1,
            self.y,
            Cell::new_unchecked(charset.border_horizontal, border_fg, title_bg),
        );

        // Top border middle
        for x in 2..self.width - 3 {
            buffer.set(
                self.x + x,
                self.y,
                Cell::new_unchecked(' ', border_fg, title_bg),
            );
        }

        // T-junction separator before corner
        buffer.set(
            self.x + self.width - 3,
            self.y,
            Cell::new_unchecked(charset.border_vertical_right, border_fg, title_bg),
        );

        // Top-right corner (2 chars wide)
        buffer.set(
            self.x + self.width - 2,
            self.y,
            Cell::new_unchecked(charset.border_horizontal, border_fg, title_bg),
        );
        buffer.set(
            self.x + self.width - 1,
            self.y,
            Cell::new_unchecked(charset.border_top_right, border_fg, title_bg),
        );

        // Side borders - 2 characters wide
        for y in 1..self.height - 1 {
            // Left border (2 chars): outer vertical + inner space
            // Outer left border (resizable)
            buffer.set(
                self.x,
                self.y + y,
                Cell::new_unchecked(charset.border_vertical, border_fg, border_bg),
            );
            // Inner left border (resizable)
            buffer.set(
                self.x + 1,
                self.y + y,
                Cell::new_unchecked(' ', border_fg, border_bg),
            );

            // Right border (2 chars): inner space + outer vertical
            // Inner right border (scrollbar area) - use border_bg for fg to avoid white overlay
            buffer.set(
                self.x + self.width - 2,
                self.y + y,
                Cell::new_unchecked(' ', border_bg, border_bg),
            );
            // Outer right border (resizable)
            buffer.set(
                self.x + self.width - 1,
                self.y + y,
                Cell::new_unchecked(charset.border_vertical, border_fg, border_bg),
            );
        }

        // Bottom border - single char height with 2-char wide corners
        // Bottom-left corner
        buffer.set(
            self.x,
            self.y + self.height - 1,
            Cell::new_unchecked(charset.border_bottom_left, border_fg, border_bg),
        );
        // Extension of bottom-left corner
        buffer.set(
            self.x + 1,
            self.y + self.height - 1,
            Cell::new_unchecked(charset.border_horizontal, border_fg, border_bg),
        );

        // Bottom border middle (resizable)
        for x in 2..self.width - 2 {
            buffer.set(
                self.x + x,
                self.y + self.height - 1,
                Cell::new_unchecked(charset.border_horizontal, border_fg, border_bg),
            );
        }

        // Extension of bottom-right corner
        buffer.set(
            self.x + self.width - 2,
            self.y + self.height - 1,
            Cell::new_unchecked(charset.border_horizontal, border_fg, border_bg),
        );
        // Bottom-right corner
        buffer.set(
            self.x + self.width - 1,
            self.y + self.height - 1,
            Cell::new_unchecked(charset.border_bottom_right, border_fg, border_bg),
        );
    }

    fn render_title_bar(
        &self,
        buffer: &mut VideoBuffer,
        theme: &Theme,
        dynamic_title: Option<&str>,
        keyboard_mode_active: bool,
    ) {
        // Use keyboard mode colors when active and focused
        let use_keyboard_colors = keyboard_mode_active && self.is_focused;

        // Use different colors based on focus state
        let title_bg = if use_keyboard_colors {
            theme.keyboard_mode_title_bg
        } else if self.is_focused {
            theme.window_title_focused_bg
        } else {
            theme.window_title_unfocused_bg
        };

        // Border foreground color based on focus state
        let border_fg = if use_keyboard_colors {
            theme.keyboard_mode_border_fg
        } else if self.is_focused {
            theme.window_border_focused_fg
        } else {
            theme.window_border_unfocused_fg
        };

        // Title text color based on focus state
        let title_fg = if use_keyboard_colors {
            theme.keyboard_mode_title_fg
        } else if self.is_focused {
            theme.window_title_focused_fg
        } else {
            theme.window_title_unfocused_fg
        };

        // Buttons: [X] [+] [_] followed by title (with spacing for better visual parsing)
        let buttons = "[X] [+] [_] ";
        let mut x_offset = 2; // Start after 2-char left border

        // Render buttons with colored characters and consistent background
        // Use new_unchecked for performance - theme colors are pre-validated
        for (i, ch) in buttons.chars().enumerate() {
            let (fg_color, bg_color) = match ch {
                'X' => (theme.button_close_color, theme.button_bg),
                '+' => (theme.button_maximize_color, theme.button_bg),
                '_' => (theme.button_minimize_color, theme.button_bg),
                '[' | ']' => (border_fg, theme.button_bg),
                _ => (border_fg, title_bg), // Space between buttons uses title background
            };
            buffer.set(
                self.x + x_offset,
                self.y,
                Cell::new_unchecked(ch, fg_color, bg_color),
            );
            x_offset += 1;

            // After each button group, there's a space
            if i == 2 || i == 5 || i == 8 {
                // These are the closing brackets
                continue;
            }
        }

        // Use dynamic title if provided, otherwise use stored title
        let title_to_render = dynamic_title.unwrap_or(&self.title);

        // Render title text with title foreground color
        let title_start = self.x + x_offset;
        let title_space = (self.width as i32 - x_offset as i32 - 2) as u16; // -2 for right border

        for (i, ch) in title_to_render
            .chars()
            .take(title_space as usize)
            .enumerate()
        {
            buffer.set(
                title_start + i as u16,
                self.y,
                Cell::new_unchecked(ch, title_fg, title_bg),
            );
        }
    }

    fn render_content(&self, buffer: &mut VideoBuffer, theme: &Theme) {
        // Fill content area with solid background color (no pattern)
        // Account for 2-char borders on left and right
        // Use new_unchecked for performance - theme colors are pre-validated
        let content_cell =
            Cell::new_unchecked(' ', theme.window_content_fg, theme.window_content_bg);

        // Pre-compute base positions to avoid repeated additions
        let base_x = self.x + 2;
        let base_y = self.y + 1;
        let content_width = self.width.saturating_sub(4); // -2 left, -2 right
        let content_height = self.height.saturating_sub(2); // -1 top, -1 bottom

        for dy in 0..content_height {
            let y = base_y + dy;
            for dx in 0..content_width {
                buffer.set(base_x + dx, y, content_cell);
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
