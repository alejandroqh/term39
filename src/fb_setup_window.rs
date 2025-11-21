//! Framebuffer setup wizard window
//!
//! Interactive configuration wizard for framebuffer settings.
//! Runs in terminal mode to allow configuration before framebuffer initialization.

use crate::charset::Charset;
use crate::fb_config::FramebufferConfig;
use crate::theme::Theme;
use crate::video_buffer::{self, Cell, VideoBuffer};

/// Action to take based on setup window interaction
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FbSetupAction {
    None,
    Close,
    SelectMode(usize),
    CycleScale,
    SelectFont(usize),
    ScrollFontListUp,
    ScrollFontListDown,
    ToggleInvertX,
    ToggleInvertY,
    ToggleSwapButtons,
    SaveAndLaunch,
    SaveOnly,
}

/// Font information for display
#[derive(Clone, Debug)]
pub struct FontInfo {
    pub name: String,
    pub width: usize,
    pub height: usize,
}

/// Framebuffer setup wizard window (centered, with border and title)
pub struct FbSetupWindow {
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,

    // UI layout rows
    mode_start_row: u16,
    scale_row: u16,
    font_label_row: u16,
    font_list_start_row: u16,
    mouse_row: u16,
    buttons_row: u16,

    // Selection state
    pub selected_mode_index: usize,
    pub selected_font_index: usize,
    pub font_list_scroll: usize,

    // Data
    pub config: FramebufferConfig,
    all_fonts: Vec<FontInfo>,           // All available fonts (unfiltered)
    pub available_fonts: Vec<FontInfo>, // Fonts filtered by mode dimensions

    // Constants
    visible_fonts: usize,
}

impl FbSetupWindow {
    /// Create a new framebuffer setup window (centered on screen)
    pub fn new(buffer_width: u16, buffer_height: u16) -> Self {
        // Fixed dimensions for setup window
        let width = 64;
        let height = 28;

        // Center on screen
        let x = (buffer_width.saturating_sub(width)) / 2;
        let y = (buffer_height.saturating_sub(height)) / 2;

        // Load current config or defaults
        let config = FramebufferConfig::load();

        // Calculate row positions
        let mode_start_row = y + 3; // First mode option
        let scale_row = y + 12; // After 8 modes + spacing
        let font_label_row = y + 14; // Font section label
        let font_list_start_row = y + 15; // Font list starts
        let mouse_row = y + 22; // Mouse options
        let buttons_row = y + 25; // Action buttons

        let visible_fonts = 6;

        Self {
            width,
            height,
            x,
            y,
            mode_start_row,
            scale_row,
            font_label_row,
            font_list_start_row,
            mouse_row,
            buttons_row,
            selected_mode_index: config.mode_index(),
            selected_font_index: 0,
            font_list_scroll: 0,
            config,
            all_fonts: Vec::new(),
            available_fonts: Vec::new(),
            visible_fonts,
        }
    }

    /// Filter fonts based on selected mode dimensions
    fn filter_fonts(&mut self) {
        let (req_width, req_height) =
            FramebufferConfig::get_mode_font_dims(self.selected_mode_index);

        // Filter fonts that match the required dimensions
        self.available_fonts = self
            .all_fonts
            .iter()
            .filter(|f| f.width == req_width && f.height == req_height)
            .cloned()
            .collect();

        // Reset selection if current font is not in filtered list
        let current_font = &self.config.font.name;
        self.selected_font_index = self
            .available_fonts
            .iter()
            .position(|f| &f.name == current_font)
            .unwrap_or(0);

        // Update config with first available font if current is not compatible
        if !self.available_fonts.is_empty() && self.selected_font_index == 0 {
            let first_font = &self.available_fonts[0];
            if &first_font.name != current_font {
                // Only update if current font was not found
                if !self.available_fonts.iter().any(|f| &f.name == current_font) {
                    self.config.font.name = first_font.name.clone();
                }
            }
        }

        // Reset scroll
        self.font_list_scroll = 0;
        if self.selected_font_index >= self.visible_fonts {
            self.font_list_scroll = self
                .selected_font_index
                .saturating_sub(self.visible_fonts - 1);
        }
    }

    /// Set available fonts (call after creation with font list from FontManager)
    pub fn set_fonts(&mut self, fonts: Vec<(String, usize, usize)>) {
        // Store all fonts
        self.all_fonts = fonts
            .into_iter()
            .map(|(name, width, height)| FontInfo {
                name,
                width,
                height,
            })
            .collect();

        // Sort fonts by name
        self.all_fonts.sort_by(|a, b| a.name.cmp(&b.name));

        // Filter fonts based on selected mode
        self.filter_fonts();
    }

    /// Render the setup window to the video buffer
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

        let title = " Framebuffer Setup ";
        let title_start = self.x + (self.width - title.len() as u16) / 2;

        // Fill top border before title
        for dx in 1..title_start - self.x {
            buffer.set(
                self.x + dx,
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
        for dx in (title_start + title.len() as u16 - self.x)..(self.width - 1) {
            buffer.set(
                self.x + dx,
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
        for dy in 1..(self.height - 1) {
            // Left border
            buffer.set(
                self.x,
                self.y + dy,
                Cell::new(vertical, border_color, content_bg),
            );

            // Content area
            for dx in 1..(self.width - 1) {
                buffer.set(
                    self.x + dx,
                    self.y + dy,
                    Cell::new(' ', content_fg, content_bg),
                );
            }

            // Right border
            buffer.set(
                self.x + self.width - 1,
                self.y + dy,
                Cell::new(vertical, border_color, content_bg),
            );
        }

        // Draw bottom border
        buffer.set(
            self.x,
            self.y + self.height - 1,
            Cell::new(bottom_left, border_color, content_bg),
        );

        for dx in 1..(self.width - 1) {
            buffer.set(
                self.x + dx,
                self.y + self.height - 1,
                Cell::new(horizontal, border_color, content_bg),
            );
        }

        buffer.set(
            self.x + self.width - 1,
            self.y + self.height - 1,
            Cell::new(bottom_right, border_color, content_bg),
        );

        // Render sections
        self.render_mode_section(buffer, theme);
        self.render_scale_section(buffer, theme);
        self.render_font_section(buffer, charset, theme);
        self.render_mouse_section(buffer, charset, theme);
        self.render_buttons(buffer, theme);

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

    /// Render mode selection section
    fn render_mode_section(&self, buffer: &mut VideoBuffer, theme: &Theme) {
        let fg = theme.config_content_fg;
        let bg = theme.config_content_bg;
        let option_x = self.x + 3;

        // Section label
        let label = "Display Mode:";
        for (i, ch) in label.chars().enumerate() {
            buffer.set(
                option_x + i as u16,
                self.mode_start_row - 1,
                Cell::new(ch, fg, bg),
            );
        }

        // Render mode options in two columns
        for (i, mode) in FramebufferConfig::TEXT_MODES.iter().enumerate() {
            let row = self.mode_start_row + (i as u16 / 2);
            let col_offset = if i % 2 == 0 { 0 } else { 28 };
            let x = option_x + col_offset;

            let selected = i == self.selected_mode_index;
            let indicator = if selected { "(*)" } else { "( )" };
            let desc = FramebufferConfig::TEXT_MODE_DESCRIPTIONS[i];

            // Render radio button
            for (j, ch) in indicator.chars().enumerate() {
                let color = if selected {
                    theme.config_toggle_on_color
                } else {
                    fg
                };
                buffer.set(x + j as u16, row, Cell::new(ch, color, bg));
            }

            // Render mode name
            let mode_text = format!(" {} {}", mode, desc);
            for (j, ch) in mode_text.chars().enumerate() {
                if (x + 3 + j as u16) < self.x + self.width - 3 {
                    buffer.set(x + 3 + j as u16, row, Cell::new(ch, fg, bg));
                }
            }
        }
    }

    /// Render scale selector section
    fn render_scale_section(&self, buffer: &mut VideoBuffer, theme: &Theme) {
        let fg = theme.config_content_fg;
        let bg = theme.config_content_bg;
        let option_x = self.x + 3;

        // Render label
        let label = "Pixel Scale:";
        for (i, ch) in label.chars().enumerate() {
            buffer.set(option_x + i as u16, self.scale_row, Cell::new(ch, fg, bg));
        }

        // Render scale selector: < auto >
        let scale_display = &self.config.display.scale;
        let selector_x = option_x + label.len() as u16 + 2;
        let selector_text = format!("< {} >", scale_display);

        for (i, ch) in selector_text.chars().enumerate() {
            buffer.set(selector_x + i as u16, self.scale_row, Cell::new(ch, fg, bg));
        }
    }

    /// Render font selection section
    fn render_font_section(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let fg = theme.config_content_fg;
        let bg = theme.config_content_bg;
        let option_x = self.x + 3;

        // Section label
        let label = "Console Font:";
        for (i, ch) in label.chars().enumerate() {
            buffer.set(
                option_x + i as u16,
                self.font_label_row,
                Cell::new(ch, fg, bg),
            );
        }

        // Draw font list box
        let list_width = self.width - 8;
        let list_x = option_x;

        // Top border of list
        buffer.set(
            list_x,
            self.font_list_start_row,
            Cell::new(charset.border_top_left(), fg, bg),
        );
        for dx in 1..list_width - 1 {
            buffer.set(
                list_x + dx,
                self.font_list_start_row,
                Cell::new(charset.border_horizontal(), fg, bg),
            );
        }
        buffer.set(
            list_x + list_width - 1,
            self.font_list_start_row,
            Cell::new(charset.border_top_right(), fg, bg),
        );

        // Render visible fonts
        let end_idx = (self.font_list_scroll + self.visible_fonts).min(self.available_fonts.len());

        for i in 0..self.visible_fonts {
            let row = self.font_list_start_row + 1 + i as u16;
            let font_idx = self.font_list_scroll + i;

            // Left border
            buffer.set(list_x, row, Cell::new(charset.border_vertical(), fg, bg));

            // Font entry or empty
            if font_idx < self.available_fonts.len() {
                let font = &self.available_fonts[font_idx];
                let selected = font_idx == self.selected_font_index;

                // Selection indicator
                let indicator = if selected { ">" } else { " " };
                let entry_bg = if selected {
                    theme.window_title_bg_focused
                } else {
                    bg
                };
                let entry_fg = if selected { theme.window_title_fg } else { fg };

                // Clear the line with proper background
                for dx in 1..list_width - 1 {
                    buffer.set(list_x + dx, row, Cell::new(' ', entry_fg, entry_bg));
                }

                // Render indicator
                buffer.set(
                    list_x + 1,
                    row,
                    Cell::new(indicator.chars().next().unwrap(), entry_fg, entry_bg),
                );

                // Render font name and dimensions
                let font_text = format!("{} ({}x{})", font.name, font.width, font.height);
                for (j, ch) in font_text.chars().enumerate() {
                    if j < (list_width - 4) as usize {
                        buffer.set(
                            list_x + 3 + j as u16,
                            row,
                            Cell::new(ch, entry_fg, entry_bg),
                        );
                    }
                }
            } else {
                // Empty row
                for dx in 1..list_width - 1 {
                    buffer.set(list_x + dx, row, Cell::new(' ', fg, bg));
                }
            }

            // Right border
            buffer.set(
                list_x + list_width - 1,
                row,
                Cell::new(charset.border_vertical(), fg, bg),
            );
        }

        // Bottom border of list
        let bottom_row = self.font_list_start_row + self.visible_fonts as u16 + 1;
        buffer.set(
            list_x,
            bottom_row,
            Cell::new(charset.border_bottom_left(), fg, bg),
        );
        for dx in 1..list_width - 1 {
            buffer.set(
                list_x + dx,
                bottom_row,
                Cell::new(charset.border_horizontal(), fg, bg),
            );
        }
        buffer.set(
            list_x + list_width - 1,
            bottom_row,
            Cell::new(charset.border_bottom_right(), fg, bg),
        );

        // Scroll indicators
        if self.font_list_scroll > 0 {
            // Up arrow indicator
            buffer.set(
                list_x + list_width - 2,
                self.font_list_start_row + 1,
                Cell::new('^', theme.config_toggle_on_color, bg),
            );
        }
        if end_idx < self.available_fonts.len() {
            // Down arrow indicator
            buffer.set(
                list_x + list_width - 2,
                bottom_row - 1,
                Cell::new('v', theme.config_toggle_on_color, bg),
            );
        }
    }

    /// Render mouse configuration section
    fn render_mouse_section(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let fg = theme.config_content_fg;
        let bg = theme.config_content_bg;
        let option_x = self.x + 3;

        // Label
        let label = "Mouse:";
        for (i, ch) in label.chars().enumerate() {
            buffer.set(option_x + i as u16, self.mouse_row, Cell::new(ch, fg, bg));
        }

        // Invert X checkbox
        let invert_x_x = option_x + label.len() as u16 + 2;
        let checkbox_x = if self.config.mouse.invert_x {
            format!("[{}]", charset.block())
        } else {
            format!("[{}]", charset.shade())
        };
        for (i, ch) in checkbox_x.chars().enumerate() {
            let color = if i == 1 {
                if self.config.mouse.invert_x {
                    theme.config_toggle_on_color
                } else {
                    theme.config_toggle_off_color
                }
            } else {
                fg
            };
            buffer.set(
                invert_x_x + i as u16,
                self.mouse_row,
                Cell::new(ch, color, bg),
            );
        }

        let invert_x_label = " Invert X";
        for (i, ch) in invert_x_label.chars().enumerate() {
            buffer.set(
                invert_x_x + 3 + i as u16,
                self.mouse_row,
                Cell::new(ch, fg, bg),
            );
        }

        // Invert Y checkbox
        let invert_y_x = invert_x_x + 16;
        let checkbox_y = if self.config.mouse.invert_y {
            format!("[{}]", charset.block())
        } else {
            format!("[{}]", charset.shade())
        };
        for (i, ch) in checkbox_y.chars().enumerate() {
            let color = if i == 1 {
                if self.config.mouse.invert_y {
                    theme.config_toggle_on_color
                } else {
                    theme.config_toggle_off_color
                }
            } else {
                fg
            };
            buffer.set(
                invert_y_x + i as u16,
                self.mouse_row,
                Cell::new(ch, color, bg),
            );
        }

        let invert_y_label = " Invert Y";
        for (i, ch) in invert_y_label.chars().enumerate() {
            buffer.set(
                invert_y_x + 3 + i as u16,
                self.mouse_row,
                Cell::new(ch, fg, bg),
            );
        }

        // Swap buttons checkbox
        let swap_x = invert_y_x + 14;
        let checkbox_swap = if self.config.mouse.swap_buttons {
            format!("[{}]", charset.block())
        } else {
            format!("[{}]", charset.shade())
        };
        for (i, ch) in checkbox_swap.chars().enumerate() {
            let color = if i == 1 {
                if self.config.mouse.swap_buttons {
                    theme.config_toggle_on_color
                } else {
                    theme.config_toggle_off_color
                }
            } else {
                fg
            };
            buffer.set(swap_x + i as u16, self.mouse_row, Cell::new(ch, color, bg));
        }

        let swap_label = " Swap";
        for (i, ch) in swap_label.chars().enumerate() {
            buffer.set(swap_x + 3 + i as u16, self.mouse_row, Cell::new(ch, fg, bg));
        }
    }

    /// Render action buttons
    fn render_buttons(&self, buffer: &mut VideoBuffer, theme: &Theme) {
        let fg = theme.config_content_fg;
        let bg = theme.config_content_bg;

        // Button style
        let button_fg = theme.window_title_fg;
        let button_bg = theme.window_title_bg_focused;

        // Save & Launch button
        let save_launch = " Save & Launch ";
        let save_launch_x = self.x + 8;
        for (i, ch) in save_launch.chars().enumerate() {
            buffer.set(
                save_launch_x + i as u16,
                self.buttons_row,
                Cell::new(ch, button_fg, button_bg),
            );
        }

        // Save button
        let save = " Save ";
        let save_x = save_launch_x + save_launch.len() as u16 + 3;
        for (i, ch) in save.chars().enumerate() {
            buffer.set(
                save_x + i as u16,
                self.buttons_row,
                Cell::new(ch, button_fg, button_bg),
            );
        }

        // Cancel button
        let cancel = " Cancel ";
        let cancel_x = save_x + save.len() as u16 + 3;
        for (i, ch) in cancel.chars().enumerate() {
            buffer.set(cancel_x + i as u16, self.buttons_row, Cell::new(ch, fg, bg));
        }
    }

    /// Handle mouse click and return appropriate action
    pub fn handle_click(&mut self, x: u16, y: u16) -> FbSetupAction {
        let option_x = self.x + 3;

        // Check mode selection (rows mode_start_row to mode_start_row + 3)
        if y >= self.mode_start_row && y < self.mode_start_row + 4 {
            let row_idx = (y - self.mode_start_row) as usize;

            // Check which column
            if x >= option_x && x < option_x + 28 {
                // Left column
                let mode_idx = row_idx * 2;
                if mode_idx < FramebufferConfig::TEXT_MODES.len() {
                    self.selected_mode_index = mode_idx;
                    self.config.set_mode_by_index(mode_idx);
                    self.filter_fonts(); // Re-filter fonts for new mode
                    return FbSetupAction::SelectMode(mode_idx);
                }
            } else if x >= option_x + 28 && x < self.x + self.width - 3 {
                // Right column
                let mode_idx = row_idx * 2 + 1;
                if mode_idx < FramebufferConfig::TEXT_MODES.len() {
                    self.selected_mode_index = mode_idx;
                    self.config.set_mode_by_index(mode_idx);
                    self.filter_fonts(); // Re-filter fonts for new mode
                    return FbSetupAction::SelectMode(mode_idx);
                }
            }
        }

        // Check scale selector
        if y == self.scale_row && x >= option_x && x < self.x + self.width - 3 {
            self.config.cycle_scale();
            return FbSetupAction::CycleScale;
        }

        // Check font list
        if y > self.font_list_start_row
            && y < self.font_list_start_row + self.visible_fonts as u16 + 1
        {
            let list_row = (y - self.font_list_start_row - 1) as usize;
            let font_idx = self.font_list_scroll + list_row;

            if font_idx < self.available_fonts.len() {
                self.selected_font_index = font_idx;
                self.config.font.name = self.available_fonts[font_idx].name.clone();
                return FbSetupAction::SelectFont(font_idx);
            }
        }

        // Check scroll arrows
        let list_x = option_x;
        let list_width = self.width - 8;
        if x == list_x + list_width - 2 {
            // Check up arrow
            if y == self.font_list_start_row + 1 && self.font_list_scroll > 0 {
                self.font_list_scroll = self.font_list_scroll.saturating_sub(1);
                return FbSetupAction::ScrollFontListUp;
            }
            // Check down arrow
            let bottom_row = self.font_list_start_row + self.visible_fonts as u16;
            if y == bottom_row
                && self.font_list_scroll + self.visible_fonts < self.available_fonts.len()
            {
                self.font_list_scroll += 1;
                return FbSetupAction::ScrollFontListDown;
            }
        }

        // Check mouse toggles
        if y == self.mouse_row {
            let invert_x_x = option_x + 8; // "Mouse: " + 2
            let invert_y_x = invert_x_x + 16;
            let swap_x = invert_y_x + 14;

            // Invert X checkbox (click on [X] or label)
            if x >= invert_x_x && x < invert_x_x + 12 {
                self.config.toggle_invert_x();
                return FbSetupAction::ToggleInvertX;
            }

            // Invert Y checkbox (click on [X] or label)
            if x >= invert_y_x && x < invert_y_x + 12 {
                self.config.toggle_invert_y();
                return FbSetupAction::ToggleInvertY;
            }

            // Swap buttons checkbox (click on [X] or label)
            if x >= swap_x && x < swap_x + 9 {
                self.config.toggle_swap_buttons();
                return FbSetupAction::ToggleSwapButtons;
            }
        }

        // Check buttons
        if y == self.buttons_row {
            let save_launch_x = self.x + 8;
            let save_launch_end = save_launch_x + 15; // " Save & Launch "
            let save_x = save_launch_end + 3;
            let save_end = save_x + 6; // " Save "
            let cancel_x = save_end + 3;
            let cancel_end = cancel_x + 8; // " Cancel "

            if x >= save_launch_x && x < save_launch_end {
                return FbSetupAction::SaveAndLaunch;
            }
            if x >= save_x && x < save_end {
                return FbSetupAction::SaveOnly;
            }
            if x >= cancel_x && x < cancel_end {
                return FbSetupAction::Close;
            }
        }

        FbSetupAction::None
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> FbSetupAction {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc => FbSetupAction::Close,
            KeyCode::Enter => FbSetupAction::SaveAndLaunch,
            KeyCode::Up => {
                // Move font selection up
                if self.selected_font_index > 0 {
                    self.selected_font_index -= 1;
                    self.config.font.name =
                        self.available_fonts[self.selected_font_index].name.clone();

                    // Adjust scroll if needed
                    if self.selected_font_index < self.font_list_scroll {
                        self.font_list_scroll = self.selected_font_index;
                    }
                }
                FbSetupAction::SelectFont(self.selected_font_index)
            }
            KeyCode::Down => {
                // Move font selection down
                if self.selected_font_index + 1 < self.available_fonts.len() {
                    self.selected_font_index += 1;
                    self.config.font.name =
                        self.available_fonts[self.selected_font_index].name.clone();

                    // Adjust scroll if needed
                    if self.selected_font_index >= self.font_list_scroll + self.visible_fonts {
                        self.font_list_scroll = self.selected_font_index - self.visible_fonts + 1;
                    }
                }
                FbSetupAction::SelectFont(self.selected_font_index)
            }
            KeyCode::PageUp => {
                // Scroll font list up
                self.font_list_scroll = self.font_list_scroll.saturating_sub(self.visible_fonts);
                if self.selected_font_index >= self.font_list_scroll + self.visible_fonts {
                    self.selected_font_index = self.font_list_scroll + self.visible_fonts - 1;
                    if !self.available_fonts.is_empty() {
                        self.config.font.name =
                            self.available_fonts[self.selected_font_index].name.clone();
                    }
                }
                FbSetupAction::ScrollFontListUp
            }
            KeyCode::PageDown => {
                // Scroll font list down
                let max_scroll = self
                    .available_fonts
                    .len()
                    .saturating_sub(self.visible_fonts);
                self.font_list_scroll =
                    (self.font_list_scroll + self.visible_fonts).min(max_scroll);
                if self.selected_font_index < self.font_list_scroll {
                    self.selected_font_index = self.font_list_scroll;
                    if !self.available_fonts.is_empty() {
                        self.config.font.name =
                            self.available_fonts[self.selected_font_index].name.clone();
                    }
                }
                FbSetupAction::ScrollFontListDown
            }
            KeyCode::Tab => {
                // Cycle through modes
                self.selected_mode_index =
                    (self.selected_mode_index + 1) % FramebufferConfig::TEXT_MODES.len();
                self.config.set_mode_by_index(self.selected_mode_index);
                self.filter_fonts(); // Re-filter fonts for new mode
                FbSetupAction::SelectMode(self.selected_mode_index)
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Save shortcut
                FbSetupAction::SaveOnly
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                // Toggle invert X
                self.config.toggle_invert_x();
                FbSetupAction::ToggleInvertX
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Toggle invert Y
                self.config.toggle_invert_y();
                FbSetupAction::ToggleInvertY
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                // Toggle swap buttons
                self.config.toggle_swap_buttons();
                FbSetupAction::ToggleSwapButtons
            }
            KeyCode::Char(' ') => {
                // Cycle scale with space
                self.config.cycle_scale();
                FbSetupAction::CycleScale
            }
            _ => FbSetupAction::None,
        }
    }

    /// Check if point is within setup window bounds
    #[allow(dead_code)]
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Save current configuration to file
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.save()
    }

    /// Get the configured values for launching
    pub fn get_config(&self) -> &FramebufferConfig {
        &self.config
    }
}
