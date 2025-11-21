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
    CycleDevice,
    ToggleInvertX,
    ToggleInvertY,
    ToggleSwapButtons,
    SaveAndLaunch,
    SaveOnly,
}

/// Focus areas for keyboard navigation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FocusArea {
    Modes,
    Scale,
    Fonts,
    Device,
    Options,
    Buttons,
}

impl FocusArea {
    fn next(self) -> Self {
        match self {
            FocusArea::Modes => FocusArea::Scale,
            FocusArea::Scale => FocusArea::Fonts,
            FocusArea::Fonts => FocusArea::Device,
            FocusArea::Device => FocusArea::Options,
            FocusArea::Options => FocusArea::Buttons,
            FocusArea::Buttons => FocusArea::Modes,
        }
    }

    fn prev(self) -> Self {
        match self {
            FocusArea::Modes => FocusArea::Buttons,
            FocusArea::Scale => FocusArea::Modes,
            FocusArea::Fonts => FocusArea::Scale,
            FocusArea::Device => FocusArea::Fonts,
            FocusArea::Options => FocusArea::Device,
            FocusArea::Buttons => FocusArea::Options,
        }
    }
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
    device_row: u16,
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

    // Focus system for keyboard navigation
    pub focus: FocusArea,
    pub option_index: usize, // 0=InvertX, 1=InvertY, 2=Swap
    pub button_index: usize, // 0=Save&Launch, 1=Save, 2=Cancel
}

impl FbSetupWindow {
    /// Create a new framebuffer setup window (centered on screen)
    pub fn new(buffer_width: u16, buffer_height: u16) -> Self {
        // Fixed dimensions for setup window
        let width = 76; // Wide enough for two-column mode display
        let height = 30; // Increased for device selector

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
        let device_row = y + 23; // Mouse device selector
        let mouse_row = y + 24; // Mouse options (invert/swap)
        let buttons_row = y + 27; // Action buttons

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
            device_row,
            mouse_row,
            buttons_row,
            selected_mode_index: config.mode_index(),
            selected_font_index: 0,
            font_list_scroll: 0,
            config,
            all_fonts: Vec::new(),
            available_fonts: Vec::new(),
            visible_fonts,
            focus: FocusArea::Modes,
            option_index: 0,
            button_index: 0,
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
        let content_fg = theme.config_content_fg;

        // Get border characters
        let top_left = charset.border_top_left();
        let top_right = charset.border_top_right();
        let bottom_left = charset.border_bottom_left();
        let bottom_right = charset.border_bottom_right();
        let horizontal = charset.border_horizontal();
        let vertical = charset.border_vertical();

        // Draw top border with title (use title_bg for top border consistency)
        buffer.set(self.x, self.y, Cell::new(top_left, border_color, title_bg));

        let title = " Framebuffer Setup ";
        let title_start = self.x + (self.width - title.len() as u16) / 2;

        // Fill top border before title
        for dx in 1..title_start - self.x {
            buffer.set(
                self.x + dx,
                self.y,
                Cell::new(horizontal, border_color, title_bg),
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
                Cell::new(horizontal, border_color, title_bg),
            );
        }

        buffer.set(
            self.x + self.width - 1,
            self.y,
            Cell::new(top_right, border_color, title_bg),
        );

        // Draw content area with side borders
        for dy in 1..(self.height - 1) {
            // Left border (use title_bg for consistent frame)
            buffer.set(
                self.x,
                self.y + dy,
                Cell::new(vertical, border_color, title_bg),
            );

            // Content area (use title_bg for consistent DOS dialog look)
            for dx in 1..(self.width - 1) {
                buffer.set(
                    self.x + dx,
                    self.y + dy,
                    Cell::new(' ', content_fg, title_bg),
                );
            }

            // Right border (use title_bg for consistent frame)
            buffer.set(
                self.x + self.width - 1,
                self.y + dy,
                Cell::new(vertical, border_color, title_bg),
            );
        }

        // Draw bottom border (use title_bg for consistent frame)
        buffer.set(
            self.x,
            self.y + self.height - 1,
            Cell::new(bottom_left, border_color, title_bg),
        );

        for dx in 1..(self.width - 1) {
            buffer.set(
                self.x + dx,
                self.y + self.height - 1,
                Cell::new(horizontal, border_color, title_bg),
            );
        }

        buffer.set(
            self.x + self.width - 1,
            self.y + self.height - 1,
            Cell::new(bottom_right, border_color, title_bg),
        );

        // Render sections with focus indicators
        self.render_mode_section(buffer, theme);
        self.render_scale_section(buffer, theme);
        self.render_font_section(buffer, charset, theme);
        self.render_mouse_section(buffer, charset, theme);
        self.render_buttons(buffer, theme);
        self.render_help_bar(buffer, theme);

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
        let bg = theme.config_title_bg; // Use title_bg for consistent DOS dialog look
        let option_x = self.x + 3;
        let is_focused = self.focus == FocusArea::Modes;

        // Clear mode section rows to ensure consistent background
        for row_offset in 0..5 {
            // -1 for label, 0-3 for the 4 rows of modes
            let row = if row_offset == 0 {
                self.mode_start_row - 1
            } else {
                self.mode_start_row + row_offset - 1
            };
            for dx in 1..(self.width - 1) {
                buffer.set(self.x + dx, row, Cell::new(' ', fg, bg));
            }
        }

        // Section label with focus indicator
        let label = if is_focused {
            "Display Mode: [1-8]"
        } else {
            "Display Mode:"
        };
        let label_fg = if is_focused {
            theme.config_toggle_on_color
        } else {
            fg
        };
        for (i, ch) in label.chars().enumerate() {
            buffer.set(
                option_x + i as u16,
                self.mode_start_row - 1,
                Cell::new(ch, label_fg, bg),
            );
        }

        // Render mode options in two columns
        for (i, mode) in FramebufferConfig::TEXT_MODES.iter().enumerate() {
            let row = self.mode_start_row + (i as u16 / 2);
            let col_offset = if i % 2 == 0 { 0 } else { 35 };
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
        let bg = theme.config_title_bg; // Use title_bg for consistent DOS dialog look
        let option_x = self.x + 3;
        let is_focused = self.focus == FocusArea::Scale;

        // Clear scale row first (to handle variable-length values)
        for dx in 1..(self.width - 1) {
            buffer.set(self.x + dx, self.scale_row, Cell::new(' ', fg, bg));
        }

        // Render label with focus indicator
        let label = "Pixel Scale:";
        let label_fg = if is_focused {
            theme.config_toggle_on_color
        } else {
            fg
        };
        for (i, ch) in label.chars().enumerate() {
            buffer.set(
                option_x + i as u16,
                self.scale_row,
                Cell::new(ch, label_fg, bg),
            );
        }

        // Render scale selector: < auto >
        let scale_display = &self.config.display.scale;
        let selector_x = option_x + label.len() as u16 + 2;
        let selector_text = format!("< {} >", scale_display);

        let selector_fg = if is_focused {
            theme.window_title_fg
        } else {
            fg
        };
        let selector_bg = if is_focused {
            theme.window_title_bg_focused
        } else {
            bg
        };
        for (i, ch) in selector_text.chars().enumerate() {
            buffer.set(
                selector_x + i as u16,
                self.scale_row,
                Cell::new(ch, selector_fg, selector_bg),
            );
        }
    }

    /// Render font selection section
    fn render_font_section(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let fg = theme.config_content_fg;
        let bg = theme.config_title_bg; // Use title_bg for consistent DOS dialog look
        let option_x = self.x + 3;
        let is_focused = self.focus == FocusArea::Fonts;

        // Clear font label row
        for dx in 1..(self.width - 1) {
            buffer.set(self.x + dx, self.font_label_row, Cell::new(' ', fg, bg));
        }

        // Section label with focus indicator
        let label = "Console Font:";
        let label_fg = if is_focused {
            theme.config_toggle_on_color
        } else {
            fg
        };
        for (i, ch) in label.chars().enumerate() {
            buffer.set(
                option_x + i as u16,
                self.font_label_row,
                Cell::new(ch, label_fg, bg),
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
        let bg = theme.config_title_bg; // Use title_bg for consistent DOS dialog look
        let option_x = self.x + 3;
        let device_focused = self.focus == FocusArea::Device;
        let options_focused = self.focus == FocusArea::Options;

        // Clear device row first (to handle variable-length device names)
        for dx in 1..(self.width - 1) {
            buffer.set(self.x + dx, self.device_row, Cell::new(' ', fg, bg));
        }

        // Clear mouse options row
        for dx in 1..(self.width - 1) {
            buffer.set(self.x + dx, self.mouse_row, Cell::new(' ', fg, bg));
        }

        // Device selector row with focus indicator
        let device_label = "Device:";
        let device_label_fg = if device_focused {
            theme.config_toggle_on_color
        } else {
            fg
        };
        for (i, ch) in device_label.chars().enumerate() {
            buffer.set(
                option_x + i as u16,
                self.device_row,
                Cell::new(ch, device_label_fg, bg),
            );
        }

        // Render device selector: < auto > or < /dev/input/mice >
        let device_display = self.config.device_display_name();
        let selector_x = option_x + device_label.len() as u16 + 2;
        let selector_text = format!("< {} >", device_display);

        let selector_fg = if device_focused {
            theme.window_title_fg
        } else {
            fg
        };
        let selector_bg = if device_focused {
            theme.window_title_bg_focused
        } else {
            bg
        };
        for (i, ch) in selector_text.chars().enumerate() {
            buffer.set(
                selector_x + i as u16,
                self.device_row,
                Cell::new(ch, selector_fg, selector_bg),
            );
        }

        // Mouse options row (checkboxes) with focus indicator
        let label = "Options:";
        let options_label_fg = if options_focused {
            theme.config_toggle_on_color
        } else {
            fg
        };
        for (i, ch) in label.chars().enumerate() {
            buffer.set(
                option_x + i as u16,
                self.mouse_row,
                Cell::new(ch, options_label_fg, bg),
            );
        }

        // Invert X checkbox
        let invert_x_x = option_x + label.len() as u16 + 1;
        let x_focused = options_focused && self.option_index == 0;
        let checkbox_x = if self.config.mouse.invert_x {
            format!("[{}]", charset.block())
        } else {
            format!("[{}]", charset.shade())
        };
        let option_bg = if x_focused {
            theme.window_title_bg_focused
        } else {
            bg
        };
        for (i, ch) in checkbox_x.chars().enumerate() {
            let color = if i == 1 {
                if self.config.mouse.invert_x {
                    theme.config_toggle_on_color
                } else {
                    theme.config_toggle_off_color
                }
            } else if x_focused {
                theme.window_title_fg
            } else {
                fg
            };
            buffer.set(
                invert_x_x + i as u16,
                self.mouse_row,
                Cell::new(ch, color, option_bg),
            );
        }

        let invert_x_label = " Invert X";
        let label_fg = if x_focused { theme.window_title_fg } else { fg };
        for (i, ch) in invert_x_label.chars().enumerate() {
            buffer.set(
                invert_x_x + 3 + i as u16,
                self.mouse_row,
                Cell::new(ch, label_fg, option_bg),
            );
        }

        // Invert Y checkbox
        let invert_y_x = invert_x_x + 16;
        let y_focused = options_focused && self.option_index == 1;
        let checkbox_y = if self.config.mouse.invert_y {
            format!("[{}]", charset.block())
        } else {
            format!("[{}]", charset.shade())
        };
        let option_bg = if y_focused {
            theme.window_title_bg_focused
        } else {
            bg
        };
        for (i, ch) in checkbox_y.chars().enumerate() {
            let color = if i == 1 {
                if self.config.mouse.invert_y {
                    theme.config_toggle_on_color
                } else {
                    theme.config_toggle_off_color
                }
            } else if y_focused {
                theme.window_title_fg
            } else {
                fg
            };
            buffer.set(
                invert_y_x + i as u16,
                self.mouse_row,
                Cell::new(ch, color, option_bg),
            );
        }

        let invert_y_label = " Invert Y";
        let label_fg = if y_focused { theme.window_title_fg } else { fg };
        for (i, ch) in invert_y_label.chars().enumerate() {
            buffer.set(
                invert_y_x + 3 + i as u16,
                self.mouse_row,
                Cell::new(ch, label_fg, option_bg),
            );
        }

        // Swap buttons checkbox
        let swap_x = invert_y_x + 14;
        let swap_focused = options_focused && self.option_index == 2;
        let checkbox_swap = if self.config.mouse.swap_buttons {
            format!("[{}]", charset.block())
        } else {
            format!("[{}]", charset.shade())
        };
        let option_bg = if swap_focused {
            theme.window_title_bg_focused
        } else {
            bg
        };
        for (i, ch) in checkbox_swap.chars().enumerate() {
            let color = if i == 1 {
                if self.config.mouse.swap_buttons {
                    theme.config_toggle_on_color
                } else {
                    theme.config_toggle_off_color
                }
            } else if swap_focused {
                theme.window_title_fg
            } else {
                fg
            };
            buffer.set(
                swap_x + i as u16,
                self.mouse_row,
                Cell::new(ch, color, option_bg),
            );
        }

        let swap_label = " Swap";
        let label_fg = if swap_focused {
            theme.window_title_fg
        } else {
            fg
        };
        for (i, ch) in swap_label.chars().enumerate() {
            buffer.set(
                swap_x + 3 + i as u16,
                self.mouse_row,
                Cell::new(ch, label_fg, option_bg),
            );
        }
    }

    /// Render action buttons
    fn render_buttons(&self, buffer: &mut VideoBuffer, theme: &Theme) {
        let fg = theme.config_content_fg;
        let bg = theme.config_title_bg; // Use title_bg for consistent DOS dialog look
        let is_focused = self.focus == FocusArea::Buttons;

        // Clear buttons row
        for dx in 1..(self.width - 1) {
            buffer.set(self.x + dx, self.buttons_row, Cell::new(' ', fg, bg));
        }

        // Button styles
        let button_fg = theme.window_title_fg;
        let button_bg = theme.window_title_bg_focused;

        // Save & Launch button (F1)
        let sl_focused = is_focused && self.button_index == 0;
        let save_launch = if sl_focused {
            "[ F1 Save & Launch ]"
        } else {
            "  F1 Save & Launch  "
        };
        let save_launch_x = self.x + 4;
        let (sl_fg, sl_bg) = if sl_focused {
            (button_bg, theme.config_toggle_on_color) // Inverted colors for selected
        } else {
            (button_fg, button_bg)
        };
        for (i, ch) in save_launch.chars().enumerate() {
            buffer.set(
                save_launch_x + i as u16,
                self.buttons_row,
                Cell::new(ch, sl_fg, sl_bg),
            );
        }

        // Save button (F2)
        let s_focused = is_focused && self.button_index == 1;
        let save = if s_focused {
            "[ F2 Save ]"
        } else {
            "  F2 Save  "
        };
        let save_x = save_launch_x + 20 + 2;
        let (s_fg, s_bg) = if s_focused {
            (button_bg, theme.config_toggle_on_color) // Inverted colors for selected
        } else {
            (button_fg, button_bg)
        };
        for (i, ch) in save.chars().enumerate() {
            buffer.set(
                save_x + i as u16,
                self.buttons_row,
                Cell::new(ch, s_fg, s_bg),
            );
        }

        // Cancel button (F3/Esc)
        let c_focused = is_focused && self.button_index == 2;
        let cancel = if c_focused {
            "[ F3 Cancel ]"
        } else {
            "  F3 Cancel  "
        };
        let cancel_x = save_x + 11 + 2;
        let (c_fg, c_bg) = if c_focused {
            (button_bg, theme.config_toggle_on_color) // Inverted colors for selected
        } else {
            (fg, bg)
        };
        for (i, ch) in cancel.chars().enumerate() {
            buffer.set(
                cancel_x + i as u16,
                self.buttons_row,
                Cell::new(ch, c_fg, c_bg),
            );
        }
    }

    /// Render help bar with keyboard shortcuts
    fn render_help_bar(&self, buffer: &mut VideoBuffer, theme: &Theme) {
        let fg = theme.config_content_fg;
        let bg = theme.config_title_bg;
        let help_row = self.y + self.height - 2;

        // Clear help row
        for dx in 1..(self.width - 1) {
            buffer.set(self.x + dx, help_row, Cell::new(' ', fg, bg));
        }

        // Show context-sensitive help based on focus
        let help_text = match self.focus {
            FocusArea::Modes => "Tab:Next  Arrows:Navigate  1-8:Select  Enter:Confirm",
            FocusArea::Scale => "Tab:Next  Left/Right:Change  Space/Enter:Cycle",
            FocusArea::Fonts => "Tab:Next  Up/Down:Select  PgUp/PgDn:Scroll  Home/End",
            FocusArea::Device => "Tab:Next  Left/Right:Change  D:Cycle  Space/Enter:Cycle",
            FocusArea::Options => "Tab:Next  Left/Right:Select  Space/Enter:Toggle  X Y B",
            FocusArea::Buttons => "Tab:Next  Left/Right:Select  Enter/Space:Activate",
        };

        let help_x = self.x + 3;
        for (i, ch) in help_text.chars().enumerate() {
            if help_x + (i as u16) < self.x + self.width - 1 {
                buffer.set(help_x + i as u16, help_row, Cell::new(ch, fg, bg));
            }
        }
    }

    /// Handle mouse click and return appropriate action
    pub fn handle_click(&mut self, x: u16, y: u16) -> FbSetupAction {
        let option_x = self.x + 3;

        // Check mode selection (rows mode_start_row to mode_start_row + 3)
        if y >= self.mode_start_row && y < self.mode_start_row + 4 {
            let row_idx = (y - self.mode_start_row) as usize;

            // Check which column
            if x >= option_x && x < option_x + 35 {
                // Left column
                let mode_idx = row_idx * 2;
                if mode_idx < FramebufferConfig::TEXT_MODES.len() {
                    self.selected_mode_index = mode_idx;
                    self.config.set_mode_by_index(mode_idx);
                    self.filter_fonts(); // Re-filter fonts for new mode
                    return FbSetupAction::SelectMode(mode_idx);
                }
            } else if x >= option_x + 35 && x < self.x + self.width - 3 {
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

        // Check device selector
        if y == self.device_row && x >= option_x && x < self.x + self.width - 3 {
            self.config.cycle_device();
            return FbSetupAction::CycleDevice;
        }

        // Check mouse toggles
        if y == self.mouse_row {
            let invert_x_x = option_x + 9; // "Options:" + 1
            let invert_y_x = invert_x_x + 13;
            let swap_x = invert_y_x + 13;

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

    /// Handle keyboard input with focus-based navigation
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> FbSetupAction {
        use crossterm::event::{KeyCode, KeyModifiers};

        match key.code {
            // Global shortcuts
            KeyCode::Esc => FbSetupAction::Close,
            KeyCode::F(10) => FbSetupAction::Close,

            // Tab navigation between focus areas
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.focus = self.focus.prev();
                } else {
                    self.focus = self.focus.next();
                }
                FbSetupAction::None
            }
            KeyCode::BackTab => {
                self.focus = self.focus.prev();
                FbSetupAction::None
            }

            // Number keys for direct mode selection (1-8)
            KeyCode::Char(c @ '1'..='8') => {
                let mode_idx = (c as usize) - ('1' as usize);
                if mode_idx < FramebufferConfig::TEXT_MODES.len() {
                    self.selected_mode_index = mode_idx;
                    self.config.set_mode_by_index(mode_idx);
                    self.filter_fonts();
                    self.focus = FocusArea::Modes;
                    return FbSetupAction::SelectMode(mode_idx);
                }
                FbSetupAction::None
            }

            // Function keys for buttons
            KeyCode::F(1) => {
                self.focus = FocusArea::Buttons;
                self.button_index = 0;
                FbSetupAction::None
            }
            KeyCode::F(2) => {
                self.focus = FocusArea::Buttons;
                self.button_index = 1;
                FbSetupAction::None
            }
            KeyCode::F(3) => {
                self.focus = FocusArea::Buttons;
                self.button_index = 2;
                FbSetupAction::None
            }

            // Enter activates current focus
            KeyCode::Enter => match self.focus {
                FocusArea::Buttons => match self.button_index {
                    0 => FbSetupAction::SaveAndLaunch,
                    1 => FbSetupAction::SaveOnly,
                    2 => FbSetupAction::Close,
                    _ => FbSetupAction::SaveAndLaunch,
                },
                FocusArea::Options => {
                    // Toggle current option
                    match self.option_index {
                        0 => {
                            self.config.toggle_invert_x();
                            FbSetupAction::ToggleInvertX
                        }
                        1 => {
                            self.config.toggle_invert_y();
                            FbSetupAction::ToggleInvertY
                        }
                        2 => {
                            self.config.toggle_swap_buttons();
                            FbSetupAction::ToggleSwapButtons
                        }
                        _ => FbSetupAction::None,
                    }
                }
                FocusArea::Scale => {
                    self.config.cycle_scale();
                    FbSetupAction::CycleScale
                }
                FocusArea::Device => {
                    self.config.cycle_device();
                    FbSetupAction::CycleDevice
                }
                _ => FbSetupAction::None,
            },

            // Space activates/toggles in most focus areas
            KeyCode::Char(' ') => match self.focus {
                FocusArea::Scale => {
                    self.config.cycle_scale();
                    FbSetupAction::CycleScale
                }
                FocusArea::Device => {
                    self.config.cycle_device();
                    FbSetupAction::CycleDevice
                }
                FocusArea::Options => match self.option_index {
                    0 => {
                        self.config.toggle_invert_x();
                        FbSetupAction::ToggleInvertX
                    }
                    1 => {
                        self.config.toggle_invert_y();
                        FbSetupAction::ToggleInvertY
                    }
                    2 => {
                        self.config.toggle_swap_buttons();
                        FbSetupAction::ToggleSwapButtons
                    }
                    _ => FbSetupAction::None,
                },
                FocusArea::Buttons => match self.button_index {
                    0 => FbSetupAction::SaveAndLaunch,
                    1 => FbSetupAction::SaveOnly,
                    2 => FbSetupAction::Close,
                    _ => FbSetupAction::None,
                },
                _ => FbSetupAction::None,
            },

            // Arrow key navigation within focus areas
            KeyCode::Up => match self.focus {
                FocusArea::Modes => {
                    // Move up in two-column mode layout
                    if self.selected_mode_index >= 2 {
                        self.selected_mode_index -= 2;
                        self.config.set_mode_by_index(self.selected_mode_index);
                        self.filter_fonts();
                        return FbSetupAction::SelectMode(self.selected_mode_index);
                    }
                    FbSetupAction::None
                }
                FocusArea::Fonts => {
                    if self.selected_font_index > 0 {
                        self.selected_font_index -= 1;
                        self.config.font.name =
                            self.available_fonts[self.selected_font_index].name.clone();
                        if self.selected_font_index < self.font_list_scroll {
                            self.font_list_scroll = self.selected_font_index;
                        }
                    }
                    FbSetupAction::SelectFont(self.selected_font_index)
                }
                _ => FbSetupAction::None,
            },

            KeyCode::Down => match self.focus {
                FocusArea::Modes => {
                    // Move down in two-column mode layout
                    if self.selected_mode_index + 2 < FramebufferConfig::TEXT_MODES.len() {
                        self.selected_mode_index += 2;
                        self.config.set_mode_by_index(self.selected_mode_index);
                        self.filter_fonts();
                        return FbSetupAction::SelectMode(self.selected_mode_index);
                    }
                    FbSetupAction::None
                }
                FocusArea::Fonts => {
                    if self.selected_font_index + 1 < self.available_fonts.len() {
                        self.selected_font_index += 1;
                        self.config.font.name =
                            self.available_fonts[self.selected_font_index].name.clone();
                        if self.selected_font_index >= self.font_list_scroll + self.visible_fonts {
                            self.font_list_scroll =
                                self.selected_font_index - self.visible_fonts + 1;
                        }
                    }
                    FbSetupAction::SelectFont(self.selected_font_index)
                }
                _ => FbSetupAction::None,
            },

            KeyCode::Left => match self.focus {
                FocusArea::Modes => {
                    // Move to left column
                    if self.selected_mode_index % 2 == 1 {
                        self.selected_mode_index -= 1;
                        self.config.set_mode_by_index(self.selected_mode_index);
                        self.filter_fonts();
                        return FbSetupAction::SelectMode(self.selected_mode_index);
                    }
                    FbSetupAction::None
                }
                FocusArea::Scale => {
                    self.config.cycle_scale_reverse();
                    FbSetupAction::CycleScale
                }
                FocusArea::Device => {
                    self.config.cycle_device_reverse();
                    FbSetupAction::CycleDevice
                }
                FocusArea::Options => {
                    if self.option_index > 0 {
                        self.option_index -= 1;
                    }
                    FbSetupAction::None
                }
                FocusArea::Buttons => {
                    if self.button_index > 0 {
                        self.button_index -= 1;
                    }
                    FbSetupAction::None
                }
                _ => FbSetupAction::None,
            },

            KeyCode::Right => match self.focus {
                FocusArea::Modes => {
                    // Move to right column
                    if self.selected_mode_index.is_multiple_of(2)
                        && self.selected_mode_index + 1 < FramebufferConfig::TEXT_MODES.len()
                    {
                        self.selected_mode_index += 1;
                        self.config.set_mode_by_index(self.selected_mode_index);
                        self.filter_fonts();
                        return FbSetupAction::SelectMode(self.selected_mode_index);
                    }
                    FbSetupAction::None
                }
                FocusArea::Scale => {
                    self.config.cycle_scale();
                    FbSetupAction::CycleScale
                }
                FocusArea::Device => {
                    self.config.cycle_device();
                    FbSetupAction::CycleDevice
                }
                FocusArea::Options => {
                    if self.option_index < 2 {
                        self.option_index += 1;
                    }
                    FbSetupAction::None
                }
                FocusArea::Buttons => {
                    if self.button_index < 2 {
                        self.button_index += 1;
                    }
                    FbSetupAction::None
                }
                _ => FbSetupAction::None,
            },

            // Page Up/Down for font list
            KeyCode::PageUp => {
                if self.focus == FocusArea::Fonts {
                    self.font_list_scroll =
                        self.font_list_scroll.saturating_sub(self.visible_fonts);
                    if self.selected_font_index >= self.font_list_scroll + self.visible_fonts {
                        self.selected_font_index = self.font_list_scroll + self.visible_fonts - 1;
                        if !self.available_fonts.is_empty() {
                            self.config.font.name =
                                self.available_fonts[self.selected_font_index].name.clone();
                        }
                    }
                    return FbSetupAction::ScrollFontListUp;
                }
                FbSetupAction::None
            }
            KeyCode::PageDown => {
                if self.focus == FocusArea::Fonts {
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
                    return FbSetupAction::ScrollFontListDown;
                }
                FbSetupAction::None
            }

            // Home/End for font list
            KeyCode::Home => {
                if self.focus == FocusArea::Fonts && !self.available_fonts.is_empty() {
                    self.selected_font_index = 0;
                    self.font_list_scroll = 0;
                    self.config.font.name = self.available_fonts[0].name.clone();
                    return FbSetupAction::SelectFont(0);
                }
                FbSetupAction::None
            }
            KeyCode::End => {
                if self.focus == FocusArea::Fonts && !self.available_fonts.is_empty() {
                    self.selected_font_index = self.available_fonts.len() - 1;
                    self.font_list_scroll = self
                        .available_fonts
                        .len()
                        .saturating_sub(self.visible_fonts);
                    self.config.font.name =
                        self.available_fonts[self.selected_font_index].name.clone();
                    return FbSetupAction::SelectFont(self.selected_font_index);
                }
                FbSetupAction::None
            }

            // Letter shortcuts (work globally)
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    FbSetupAction::SaveOnly
                } else {
                    FbSetupAction::None
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.config.cycle_device();
                self.focus = FocusArea::Device;
                FbSetupAction::CycleDevice
            }
            KeyCode::Char('x') | KeyCode::Char('X') => {
                self.config.toggle_invert_x();
                self.focus = FocusArea::Options;
                self.option_index = 0;
                FbSetupAction::ToggleInvertX
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.config.toggle_invert_y();
                self.focus = FocusArea::Options;
                self.option_index = 1;
                FbSetupAction::ToggleInvertY
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.config.toggle_swap_buttons();
                self.focus = FocusArea::Options;
                self.option_index = 2;
                FbSetupAction::ToggleSwapButtons
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => FbSetupAction::Close,

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
