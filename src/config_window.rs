use crate::charset::Charset;
use crate::config_manager::AppConfig;
use crate::theme::Theme;
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
    CycleTheme,
}

/// Configuration modal window (centered, with border and title)
pub struct ConfigWindow {
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
    auto_arrange_row: u16, // Row where auto arrange toggle is rendered
    show_date_row: u16,    // Row where show date toggle is rendered
    theme_row: u16,        // Row where theme selector is rendered
}

impl ConfigWindow {
    /// Create a new configuration window (centered on screen)
    pub fn new(buffer_width: u16, buffer_height: u16) -> Self {
        // Fixed dimensions for config window
        let width = 60;
        let height = 12; // Increased to fit theme selector

        // Center on screen
        let x = (buffer_width.saturating_sub(width)) / 2;
        let y = (buffer_height.saturating_sub(height)) / 2;

        // Calculate row positions for options
        let auto_arrange_row = y + 3; // Title at y+1, blank at y+2, first option at y+3
        let show_date_row = y + 5; // Blank at y+4, second option at y+5
        let theme_row = y + 7; // Blank at y+6, third option at y+7

        Self {
            width,
            height,
            x,
            y,
            auto_arrange_row,
            show_date_row,
            theme_row,
        }
    }

    /// Render the configuration window to the video buffer
    pub fn render(
        &self,
        buffer: &mut VideoBuffer,
        charset: &Charset,
        theme: &Theme,
        config: &AppConfig,
    ) {
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
            theme,
            content_fg,
            content_bg,
        );

        self.render_option(
            buffer,
            self.show_date_row,
            "Show Date in clock:",
            config.show_date_in_clock,
            charset,
            theme,
            content_fg,
            content_bg,
        );

        // Render theme selector
        self.render_theme_selector(
            buffer,
            self.theme_row,
            &config.theme,
            content_fg,
            content_bg,
        );

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
    }

    /// Render a single configuration option with toggle
    fn render_option(
        &self,
        buffer: &mut VideoBuffer,
        row: u16,
        label: &str,
        enabled: bool,
        charset: &Charset,
        theme: &Theme,
        content_fg: Color,
        content_bg: Color,
    ) {
        let fg = content_fg;
        let bg = content_bg;

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
                let color = if i == 1 {
                    theme.config_toggle_on_color
                } else {
                    fg
                }; // Block character in theme color
                buffer.set(toggle_x + i as u16, row, Cell::new(ch, color, bg));
            }
        } else {
            // [off ░]
            let toggle_off = format!("[off {}]", charset.shade());
            for (i, ch) in toggle_off.chars().enumerate() {
                let color = if i == 4 {
                    theme.config_toggle_off_color
                } else {
                    fg
                }; // Shade character in theme color
                buffer.set(toggle_x + i as u16, row, Cell::new(ch, color, bg));
            }
        }
    }

    /// Render theme selector showing current theme with arrows to cycle
    fn render_theme_selector(
        &self,
        buffer: &mut VideoBuffer,
        row: u16,
        current_theme: &str,
        content_fg: Color,
        content_bg: Color,
    ) {
        let fg = content_fg;
        let bg = content_bg;

        let option_x = self.x + 3; // 3 spaces from left border

        // Render label
        let label = "Theme:";
        for (i, ch) in label.chars().enumerate() {
            buffer.set(option_x + i as u16, row, Cell::new(ch, fg, bg));
        }

        // Render theme selector: < classic >
        let theme_display = match current_theme {
            "classic" => "Classic",
            "monochrome" => "Monochrome",
            "dark" => "Dark",
            _ => "Classic",
        };

        let selector_x = option_x + label.len() as u16 + 2;
        let selector_text = format!("< {} >", theme_display);

        for (i, ch) in selector_text.chars().enumerate() {
            buffer.set(selector_x + i as u16, row, Cell::new(ch, fg, bg));
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

        // Check if click is on theme row
        if y == self.theme_row {
            // Click anywhere on the row cycles the theme
            if x >= self.x && x < self.x + self.width {
                return ConfigAction::CycleTheme;
            }
        }

        ConfigAction::None
    }

    /// Check if point is within config window bounds
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}
