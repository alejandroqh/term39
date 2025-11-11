use crate::charset::Charset;
use crate::config_manager::AppConfig;
use crate::video_buffer::{Cell, VideoBuffer};
use crossterm::style::Color;

/// Action to take based on config window interaction
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConfigAction {
    None,
    #[allow(dead_code)]
    Close,
    ToggleAutoTiling,
    ToggleShowDate,
}

/// Configuration modal window (centered, with border and title)
pub struct ConfigWindow {
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
    auto_arrange_row: u16, // Row where auto arrange toggle is rendered
    show_date_row: u16,    // Row where show date toggle is rendered
}

impl ConfigWindow {
    /// Create a new configuration window (centered on screen)
    pub fn new(buffer_width: u16, buffer_height: u16) -> Self {
        // Fixed dimensions for config window
        let width = 60;
        let height = 10;

        // Center on screen
        let x = (buffer_width.saturating_sub(width)) / 2;
        let y = (buffer_height.saturating_sub(height)) / 2;

        // Calculate row positions for toggle options
        let auto_arrange_row = y + 3; // Title at y+1, blank at y+2, first option at y+3
        let show_date_row = y + 5; // Blank at y+4, second option at y+5

        Self {
            width,
            height,
            x,
            y,
            auto_arrange_row,
            show_date_row,
        }
    }

    /// Render the configuration window to the video buffer
    pub fn render(&self, buffer: &mut VideoBuffer, charset: &Charset, config: &AppConfig) {
        let title_bg = Color::Blue;
        let title_fg = Color::White;
        let border_color = Color::Cyan;
        let content_bg = Color::Black;
        let content_fg = Color::White;

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

        let title = " Settings ";
        let title_start = self.x + (self.width - title.len() as u16) / 2;

        // Fill top border before title
        for x in 1..title_start - self.x {
            buffer.set(
                self.x + x,
                self.y,
                Cell::new(horizontal, border_color, content_bg),
            );
        }

        // Render title with special background
        for (i, ch) in title.chars().enumerate() {
            buffer.set(
                title_start + i as u16,
                self.y,
                Cell::new(ch, title_fg, title_bg),
            );
        }

        // Fill top border after title
        for x in (title_start + title.len() as u16 - self.x)..(self.width - 1) {
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

        // Render configuration options
        self.render_option(
            buffer,
            self.auto_arrange_row,
            "On startup Auto Tiling:",
            config.auto_tiling_on_startup,
            charset,
        );

        self.render_option(
            buffer,
            self.show_date_row,
            "Show Date in clock:",
            config.show_date_in_clock,
            charset,
        );

        // Render instruction at bottom
        let instruction = "Press ESC to close";
        let instruction_x = self.x + (self.width - instruction.len() as u16) / 2;
        let instruction_y = self.y + self.height - 2;

        for (i, ch) in instruction.chars().enumerate() {
            buffer.set(
                instruction_x + i as u16,
                instruction_y,
                Cell::new(ch, Color::DarkGrey, content_bg),
            );
        }
    }

    /// Render a single configuration option with toggle
    fn render_option(
        &self,
        buffer: &mut VideoBuffer,
        row: u16,
        label: &str,
        enabled: bool,
        charset: &Charset,
    ) {
        let fg = Color::White;
        let bg = Color::Black;

        let option_x = self.x + 3; // 3 spaces from left border

        // Render label
        for (i, ch) in label.chars().enumerate() {
            buffer.set(option_x + i as u16, row, Cell::new(ch, fg, bg));
        }

        // Render toggle indicator
        let toggle_x = option_x + label.len() as u16 + 1;

        if enabled {
            // [█ on]
            let toggle_on = format!("[{} on]", charset.block());
            for (i, ch) in toggle_on.chars().enumerate() {
                let color = if i == 1 { Color::Green } else { fg }; // Block character in green
                buffer.set(toggle_x + i as u16, row, Cell::new(ch, color, bg));
            }
        } else {
            // [off ░]
            let toggle_off = format!("[off {}]", charset.shade());
            for (i, ch) in toggle_off.chars().enumerate() {
                let color = if i == 4 { Color::DarkGrey } else { fg }; // Shade character in dark grey
                buffer.set(toggle_x + i as u16, row, Cell::new(ch, color, bg));
            }
        }
    }

    /// Handle mouse click and return appropriate action
    pub fn handle_click(&self, x: u16, y: u16) -> ConfigAction {
        // Check if click is on auto tiling row
        if y == self.auto_arrange_row {
            // Click anywhere on the row toggles the option
            if x >= self.x && x < self.x + self.width {
                return ConfigAction::ToggleAutoTiling;
            }
        }

        // Check if click is on show date row
        if y == self.show_date_row {
            // Click anywhere on the row toggles the option
            if x >= self.x && x < self.x + self.width {
                return ConfigAction::ToggleShowDate;
            }
        }

        ConfigAction::None
    }

    /// Check if point is within config window bounds
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}
