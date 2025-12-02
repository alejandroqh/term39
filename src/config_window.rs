use crate::charset::Charset;
use crate::config_manager::{AppConfig, LockscreenAuthMode};
use crate::lockscreen::auth::is_os_auth_compiled;
use crate::theme::Theme;
use crate::video_buffer::{self, Cell, VideoBuffer};

/// Action to take based on config window interaction
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConfigAction {
    None,
    #[allow(dead_code)]
    Close,
    ToggleAutoTiling,
    ToggleShowDate,
    CycleTheme,
    CycleBackgroundChar,
    ToggleTintTerminal,
    ToggleAutoSave,
    ToggleLockscreen,
    CycleLockscreenAuthMode,
    SetupPin,
}

/// Configuration modal window (centered, with border and title)
pub struct ConfigWindow {
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
    auto_arrange_row: u16,    // Row where auto arrange toggle is rendered
    show_date_row: u16,       // Row where show date toggle is rendered
    theme_row: u16,           // Row where theme selector is rendered
    background_char_row: u16, // Row where background character selector is rendered
    tint_terminal_row: u16,   // Row where tint terminal toggle is rendered
    auto_save_row: u16,       // Row where auto-save toggle is rendered
    lockscreen_row: u16,      // Row where lockscreen toggle is rendered
    lockscreen_auth_row: u16, // Row where lockscreen auth mode is rendered
    pin_setup_row: u16,       // Row where PIN setup button is rendered
}

impl ConfigWindow {
    /// Create a new configuration window (centered on screen)
    pub fn new(buffer_width: u16, buffer_height: u16) -> Self {
        // Fixed dimensions for config window
        let width = 60;
        let height = 24; // Increased to fit lockscreen options

        // Center on screen
        let x = (buffer_width.saturating_sub(width)) / 2;
        let y = (buffer_height.saturating_sub(height)) / 2;

        // Calculate row positions for options
        let auto_arrange_row = y + 3; // Title at y+1, blank at y+2, first option at y+3
        let show_date_row = y + 5; // Blank at y+4, second option at y+5
        let theme_row = y + 7; // Blank at y+6, third option at y+7
        let background_char_row = y + 9; // Blank at y+8, fourth option at y+9
        let tint_terminal_row = y + 11; // Blank at y+10, fifth option at y+11
        let auto_save_row = y + 13; // Blank at y+12, sixth option at y+13
        let lockscreen_row = y + 15; // Blank at y+14, seventh option at y+15
        let lockscreen_auth_row = y + 17; // Blank at y+16, eighth option at y+17
        let pin_setup_row = y + 19; // Blank at y+18, ninth option at y+19

        Self {
            width,
            height,
            x,
            y,
            auto_arrange_row,
            show_date_row,
            theme_row,
            background_char_row,
            tint_terminal_row,
            auto_save_row,
            lockscreen_row,
            lockscreen_auth_row,
            pin_setup_row,
        }
    }

    /// Render the configuration window to the video buffer
    pub fn render(
        &self,
        buffer: &mut VideoBuffer,
        charset: &Charset,
        theme: &Theme,
        config: &AppConfig,
        tint_terminal: bool,
        os_auth_available: bool,
    ) {
        let title_bg = theme.config_title_bg;
        let title_fg = theme.config_title_fg;
        let border_color = theme.config_border;
        let content_bg = theme.config_content_bg;

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
                    Cell::new(' ', theme.config_content_fg, content_bg),
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
        );

        self.render_option(
            buffer,
            self.show_date_row,
            "Show Date in clock:",
            config.show_date_in_clock,
            charset,
            theme,
        );

        // Render theme selector
        self.render_theme_selector(buffer, self.theme_row, &config.theme, theme);

        // Render background character selector
        self.render_background_char_selector(buffer, self.background_char_row, config, theme);

        // Render tint terminal toggle
        self.render_option(
            buffer,
            self.tint_terminal_row,
            "Tint terminal content:",
            tint_terminal,
            charset,
            theme,
        );

        // Render auto-save toggle
        self.render_option(
            buffer,
            self.auto_save_row,
            "Auto-save session on exit:",
            config.auto_save,
            charset,
            theme,
        );

        // Render lockscreen toggle
        self.render_option(
            buffer,
            self.lockscreen_row,
            "Lockscreen (Shift+Q):",
            config.lockscreen_enabled,
            charset,
            theme,
        );

        // Render auth mode selector (only if lockscreen enabled AND OS auth is compiled)
        if config.lockscreen_enabled && is_os_auth_compiled() {
            self.render_auth_mode_selector(
                buffer,
                self.lockscreen_auth_row,
                config.lockscreen_auth_mode,
                os_auth_available,
                theme,
            );
        }

        // Render PIN setup button (only if lockscreen enabled and PIN mode, or OS auth not compiled)
        if config.lockscreen_enabled
            && (config.lockscreen_auth_mode == LockscreenAuthMode::Pin || !is_os_auth_compiled())
        {
            self.render_pin_setup_button(
                buffer,
                self.pin_setup_row,
                config.has_pin_configured(),
                theme,
            );
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

    /// Render a single configuration option with toggle
    fn render_option(
        &self,
        buffer: &mut VideoBuffer,
        row: u16,
        label: &str,
        enabled: bool,
        charset: &Charset,
        theme: &Theme,
    ) {
        let fg = theme.config_content_fg;
        let bg = theme.config_content_bg;

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
        theme: &Theme,
    ) {
        let fg = theme.config_content_fg;
        let bg = theme.config_content_bg;

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
            "dracu" => "Dracu",
            "green_phosphor" => "Green Phosphor",
            "amber" => "Amber",
            "ndd" => "NDD",
            "qbasic" => "QBasic",
            "turbo" => "TurboP",
            "norton_commander" => "NCC",
            "xtree" => "XT",
            "wordperfect" => "WP",
            "dbase" => "dB",
            _ => "Classic",
        };

        let selector_x = option_x + label.len() as u16 + 2;
        let selector_text = format!("< {} >", theme_display);

        for (i, ch) in selector_text.chars().enumerate() {
            buffer.set(selector_x + i as u16, row, Cell::new(ch, fg, bg));
        }
    }

    /// Render background character selector showing current character with arrows to cycle
    fn render_background_char_selector(
        &self,
        buffer: &mut VideoBuffer,
        row: u16,
        config: &AppConfig,
        theme: &Theme,
    ) {
        let fg = theme.config_content_fg;
        let bg = theme.config_content_bg;

        let option_x = self.x + 3; // 3 spaces from left border

        // Render label
        let label = "Background character:";
        for (i, ch) in label.chars().enumerate() {
            buffer.set(option_x + i as u16, row, Cell::new(ch, fg, bg));
        }

        // Render character selector: < Light Shade [░] >
        let char_name = config.get_background_char_name();
        let char_sample = config.get_background_char();

        let selector_x = option_x + label.len() as u16 + 2;
        let selector_text = format!("< {} [{}] >", char_name, char_sample);

        for (i, ch) in selector_text.chars().enumerate() {
            buffer.set(selector_x + i as u16, row, Cell::new(ch, fg, bg));
        }
    }

    /// Render auth mode selector showing current mode with arrows to cycle
    fn render_auth_mode_selector(
        &self,
        buffer: &mut VideoBuffer,
        row: u16,
        mode: LockscreenAuthMode,
        os_auth_available: bool,
        theme: &Theme,
    ) {
        let fg = theme.config_content_fg;
        let bg = theme.config_content_bg;

        let option_x = self.x + 3;

        let label = "Auth mode:";
        for (i, ch) in label.chars().enumerate() {
            buffer.set(option_x + i as u16, row, Cell::new(ch, fg, bg));
        }

        let mode_text = match mode {
            LockscreenAuthMode::OsAuth => {
                if os_auth_available {
                    "< OS Auth >"
                } else {
                    "< OS Auth (unavail) >"
                }
            }
            LockscreenAuthMode::Pin => "< PIN >",
        };

        let selector_x = option_x + label.len() as u16 + 2;
        for (i, ch) in mode_text.chars().enumerate() {
            buffer.set(selector_x + i as u16, row, Cell::new(ch, fg, bg));
        }
    }

    /// Render PIN setup button
    fn render_pin_setup_button(
        &self,
        buffer: &mut VideoBuffer,
        row: u16,
        has_pin: bool,
        theme: &Theme,
    ) {
        let fg = theme.config_content_fg;
        let bg = theme.config_content_bg;

        let option_x = self.x + 3;

        let (button_text, status_text) = if has_pin {
            ("[ Change PIN ]", "   PIN is configured")
        } else {
            ("[ Set PIN ]", "      PIN required to enable")
        };

        // Render button in highlight color
        for (i, ch) in button_text.chars().enumerate() {
            buffer.set(
                option_x + i as u16,
                row,
                Cell::new(ch, theme.config_toggle_on_color, bg),
            );
        }

        // Render status text
        let status_x = option_x + button_text.len() as u16;
        for (i, ch) in status_text.chars().enumerate() {
            buffer.set(status_x + i as u16, row, Cell::new(ch, fg, bg));
        }
    }

    /// Handle mouse click and return appropriate action
    pub fn handle_click(&self, x: u16, y: u16, config: &AppConfig) -> ConfigAction {
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

        // Check if click is on background character row
        if y == self.background_char_row {
            // Click anywhere on the row cycles the background character
            if x >= self.x && x < self.x + self.width {
                return ConfigAction::CycleBackgroundChar;
            }
        }

        // Check if click is on tint terminal row
        if y == self.tint_terminal_row {
            // Click anywhere on the row toggles the option
            if x >= self.x && x < self.x + self.width {
                return ConfigAction::ToggleTintTerminal;
            }
        }

        // Check if click is on auto-save row
        if y == self.auto_save_row {
            // Click anywhere on the row toggles the option
            if x >= self.x && x < self.x + self.width {
                return ConfigAction::ToggleAutoSave;
            }
        }

        // Check if click is on lockscreen row
        if y == self.lockscreen_row {
            if x >= self.x && x < self.x + self.width {
                return ConfigAction::ToggleLockscreen;
            }
        }

        // Check if click is on auth mode row (only if lockscreen enabled AND OS auth compiled)
        if config.lockscreen_enabled && is_os_auth_compiled() && y == self.lockscreen_auth_row {
            if x >= self.x && x < self.x + self.width {
                return ConfigAction::CycleLockscreenAuthMode;
            }
        }

        // Check if click is on PIN setup row (only if lockscreen enabled and PIN mode, or OS auth not compiled)
        if config.lockscreen_enabled
            && (config.lockscreen_auth_mode == LockscreenAuthMode::Pin || !is_os_auth_compiled())
            && y == self.pin_setup_row
        {
            if x >= self.x && x < self.x + self.width {
                return ConfigAction::SetupPin;
            }
        }

        ConfigAction::None
    }

    /// Check if point is within config window bounds
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}
