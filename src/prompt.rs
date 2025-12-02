use crate::charset::{Charset, CharsetMode};
use crate::theme::Theme;
use crate::video_buffer::{self, Cell, VideoBuffer};
use crossterm::style::Color;

/// Prompt types with different visual styles (similar to Bootstrap alerts)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PromptType {
    #[allow(dead_code)]
    Info, // Blue theme
    #[allow(dead_code)]
    Success, // Green theme
    #[allow(dead_code)]
    Warning, // Yellow theme
    Danger, // Red theme
}

impl PromptType {
    /// Get the background color for this prompt type
    pub fn background_color(&self, theme: &Theme) -> Color {
        match self {
            PromptType::Info => theme.prompt_info_bg,
            PromptType::Success => theme.prompt_success_bg,
            PromptType::Warning => theme.prompt_warning_bg,
            PromptType::Danger => theme.prompt_danger_bg,
        }
    }

    /// Get the foreground color for this prompt type
    pub fn foreground_color(&self, theme: &Theme) -> Color {
        match self {
            PromptType::Info => theme.prompt_info_fg,
            PromptType::Success => theme.prompt_success_fg,
            PromptType::Warning => theme.prompt_warning_fg,
            PromptType::Danger => theme.prompt_danger_fg,
        }
    }
}

/// Button in a prompt
#[derive(Clone, Debug)]
pub struct PromptButton {
    pub text: String,
    pub action: PromptAction,
    pub is_primary: bool, // Primary buttons have attractive colors, secondary are muted
}

/// Action to take when a button is clicked
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PromptAction {
    Confirm,
    Cancel,
    #[allow(dead_code)]
    Custom(u32),
}

impl PromptButton {
    /// Create a new button
    pub fn new(text: String, action: PromptAction, is_primary: bool) -> Self {
        Self {
            text,
            action,
            is_primary,
        }
    }

    /// Get button colors based on whether it's primary
    #[allow(dead_code)]
    pub fn colors(&self, prompt_type: PromptType, theme: &Theme) -> (Color, Color) {
        if self.is_primary {
            // Primary button: attractive colors based on prompt type
            match prompt_type {
                PromptType::Info => (
                    theme.dialog_button_primary_info_fg,
                    theme.dialog_button_primary_info_bg,
                ),
                PromptType::Success => (
                    theme.dialog_button_primary_success_fg,
                    theme.dialog_button_primary_success_bg,
                ),
                PromptType::Warning => (
                    theme.dialog_button_primary_warning_fg,
                    theme.dialog_button_primary_warning_bg,
                ),
                PromptType::Danger => (
                    theme.dialog_button_primary_danger_fg,
                    theme.dialog_button_primary_danger_bg,
                ),
            }
        } else {
            // Secondary button: muted colors
            (
                theme.dialog_button_secondary_fg,
                theme.dialog_button_secondary_bg,
            )
        }
    }

    /// Get the rendered width of the button (includes brackets and spaces)
    pub fn width(&self) -> u16 {
        // Format: [ Text ]
        self.text.len() as u16 + 4
    }
}

/// Text alignment for prompt messages
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextAlign {
    #[allow(dead_code)]
    Left,
    Center,
}

/// Positioning mode for prompts
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PositionMode {
    /// Center on the entire screen (default behavior)
    CenteredOnScreen,
    /// Center within a specific region (e.g., inside a window)
    CenteredInRegion {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    },
}

/// A modal prompt dialog (no title bar, centered on screen)
pub struct Prompt {
    pub prompt_type: PromptType,
    pub message: String,
    pub buttons: Vec<PromptButton>,
    pub width: u16,
    pub height: u16,
    pub x: u16,
    pub y: u16,
    pub selected_button_index: usize, // Index of currently selected button for keyboard navigation
    pub text_align: TextAlign,        // Text alignment for the message
    pub show_selection_indicators: bool, // Whether to show >< around selected button
    pub position_mode: PositionMode,  // How to position the dialog
}

impl Prompt {
    /// Create a new prompt (auto-sized and centered) with center alignment
    pub fn new(
        prompt_type: PromptType,
        message: String,
        buttons: Vec<PromptButton>,
        buffer_width: u16,
        buffer_height: u16,
    ) -> Self {
        Self::new_with_alignment(
            prompt_type,
            message,
            buttons,
            buffer_width,
            buffer_height,
            TextAlign::Center,
        )
    }

    /// Create a new prompt (auto-sized and centered) with custom alignment
    pub fn new_with_alignment(
        prompt_type: PromptType,
        message: String,
        buttons: Vec<PromptButton>,
        buffer_width: u16,
        buffer_height: u16,
        text_align: TextAlign,
    ) -> Self {
        // Calculate dimensions
        let message_lines: Vec<&str> = message.lines().collect();

        // Strip color codes when calculating width
        let max_message_width = message_lines
            .iter()
            .map(|line| Self::strip_color_codes(line).len())
            .max()
            .unwrap_or(0) as u16;

        // Calculate total button width (with spacing)
        let total_button_width: u16 = buttons.iter().map(|b| b.width()).sum::<u16>()
            + (buttons.len().saturating_sub(1)) as u16 * 2; // 2 spaces between buttons

        // Width is max of message width and button width, plus padding
        let content_width = max_message_width.max(total_button_width);
        let width = content_width + 6; // 2 for padding on each side + 2 for borders

        // Height: message lines + padding + button row
        let height = message_lines.len() as u16 + 7; // 1 top padding + message + 1 padding + buttons + button shadow + 1 padding + borders

        // Center on screen
        let x = (buffer_width.saturating_sub(width)) / 2;
        let y = (buffer_height.saturating_sub(height)) / 2;

        // Find the first primary button as default selection, or use first button
        let selected_button_index = buttons.iter().position(|b| b.is_primary).unwrap_or(0);

        Self {
            prompt_type,
            message,
            buttons,
            width,
            height,
            x,
            y,
            selected_button_index,
            text_align,
            show_selection_indicators: false, // Default: no indicators (backward compatible)
            position_mode: PositionMode::CenteredOnScreen,
        }
    }

    /// Override the default selected button index
    pub fn with_selected_button(mut self, index: usize) -> Self {
        if index < self.buttons.len() {
            self.selected_button_index = index;
        }
        self
    }

    /// Enable or disable selection indicators
    /// When enabled: selected button shows [ ] brackets, unselected shows spaces
    /// When disabled: all buttons show [ ] brackets (default behavior)
    pub fn with_selection_indicators(mut self, show: bool) -> Self {
        self.show_selection_indicators = show;
        self
    }

    /// Position the prompt centered within a specific region
    pub fn centered_in_region(
        mut self,
        region_x: u16,
        region_y: u16,
        region_width: u16,
        region_height: u16,
    ) -> Self {
        self.position_mode = PositionMode::CenteredInRegion {
            x: region_x,
            y: region_y,
            width: region_width,
            height: region_height,
        };
        // Recalculate position
        self.x = region_x + (region_width.saturating_sub(self.width)) / 2;
        self.y = region_y + (region_height.saturating_sub(self.height)) / 2;
        self
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

    /// Render the prompt to the video buffer
    pub fn render(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let bg_color = self.prompt_type.background_color(theme);
        let default_fg_color = self.prompt_type.foreground_color(theme);

        // Fill the entire prompt area with the background color
        for y in 0..self.height {
            for x in 0..self.width {
                buffer.set(
                    self.x + x,
                    self.y + y,
                    Cell::new(' ', default_fg_color, bg_color),
                );
            }
        }

        // Draw border using charset
        let (tl, tr, bl, br, h, v) = match charset.mode {
            CharsetMode::Unicode | CharsetMode::UnicodeSingleLine => (
                charset.border_top_left,
                charset.border_top_right,
                charset.border_bottom_left,
                charset.border_bottom_right,
                charset.border_horizontal,
                charset.border_vertical,
            ),
            CharsetMode::Ascii => ('+', '+', '+', '+', '-', '|'),
        };

        // Top border
        buffer.set(self.x, self.y, Cell::new(tl, default_fg_color, bg_color));
        for bx in 1..self.width - 1 {
            buffer.set(
                self.x + bx,
                self.y,
                Cell::new(h, default_fg_color, bg_color),
            );
        }
        buffer.set(
            self.x + self.width - 1,
            self.y,
            Cell::new(tr, default_fg_color, bg_color),
        );

        // Side borders
        for by in 1..self.height - 1 {
            buffer.set(
                self.x,
                self.y + by,
                Cell::new(v, default_fg_color, bg_color),
            );
            buffer.set(
                self.x + self.width - 1,
                self.y + by,
                Cell::new(v, default_fg_color, bg_color),
            );
        }

        // Bottom border
        buffer.set(
            self.x,
            self.y + self.height - 1,
            Cell::new(bl, default_fg_color, bg_color),
        );
        for bx in 1..self.width - 1 {
            buffer.set(
                self.x + bx,
                self.y + self.height - 1,
                Cell::new(h, default_fg_color, bg_color),
            );
        }
        buffer.set(
            self.x + self.width - 1,
            self.y + self.height - 1,
            Cell::new(br, default_fg_color, bg_color),
        );

        // Render message (with alignment and color support)
        let message_lines: Vec<&str> = self.message.lines().collect();
        let message_start_y = self.y + 2; // Leave space at top

        for (i, line) in message_lines.iter().enumerate() {
            let line_y = message_start_y + i as u16;

            // Calculate line_x based on alignment
            let stripped_line = Self::strip_color_codes(line);
            let line_x = match self.text_align {
                TextAlign::Center => {
                    self.x + (self.width.saturating_sub(stripped_line.len() as u16)) / 2
                }
                TextAlign::Left => self.x + 3, // 3 spaces from left edge
            };

            // Render line with color code parsing
            let mut current_x = line_x;
            let mut current_color = default_fg_color;
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
                    buffer.set(current_x, line_y, Cell::new(ch, current_color, bg_color));
                    current_x += 1;
                }
            }
        }

        // Render buttons (centered, at bottom with space for shadows)
        let button_y = self.y + self.height - 3;

        // In UTF8 mode, buttons have shadows (1 cell right, 1 cell down)
        // Account for shadow space in button width calculation
        let has_button_shadow = matches!(
            charset.mode,
            CharsetMode::Unicode | CharsetMode::UnicodeSingleLine
        );
        let button_shadow_extra = if has_button_shadow { 1 } else { 0 };

        let total_button_width: u16 = self
            .buttons
            .iter()
            .map(|b| b.width() + button_shadow_extra)
            .sum::<u16>()
            + (self.buttons.len().saturating_sub(1)) as u16 * 2; // 2 spaces between buttons

        let mut button_x = self.x + (self.width.saturating_sub(total_button_width)) / 2;

        // Shadow color for buttons
        let button_shadow_bg = Color::Black;

        for (index, button) in self.buttons.iter().enumerate() {
            let is_selected = index == self.selected_button_index;

            // Prompt button colors are fixed regardless of theme:
            // - Non-selected: white text on black background
            // - Selected: yellow text on black background
            let (button_fg, button_bg) = if is_selected {
                (Color::Black, theme.prompt_warning_bg)
            } else {
                (Color::Black, Color::White)
            };

            let button_start_x = button_x;
            let button_width = button.width();

            // When selection indicators are enabled:
            // - Selected button: [ Text ] (with brackets)
            // - Not selected button:   Text   (spaces instead of brackets)
            let (left_bracket, right_bracket) = if self.show_selection_indicators {
                if is_selected { ('[', ']') } else { (' ', ' ') }
            } else {
                // Default behavior: always show brackets
                ('[', ']')
            };

            // Render button: [ Text ] or   Text
            buffer.set(
                button_x,
                button_y,
                Cell::new(left_bracket, button_fg, button_bg),
            );
            button_x += 1;
            buffer.set(button_x, button_y, Cell::new(' ', button_fg, button_bg));
            button_x += 1;

            for ch in button.text.chars() {
                buffer.set(button_x, button_y, Cell::new(ch, button_fg, button_bg));
                button_x += 1;
            }

            buffer.set(button_x, button_y, Cell::new(' ', button_fg, button_bg));
            button_x += 1;
            buffer.set(
                button_x,
                button_y,
                Cell::new(right_bracket, button_fg, button_bg),
            );
            button_x += 1;

            // Render button shadow (right side and bottom) in UTF8 mode
            // Uses half-block characters for a subtle shadow effect
            if has_button_shadow {
                // The shadow color is the foreground, background is the prompt bg
                buffer.set(
                    button_x,
                    button_y,
                    Cell::new_unchecked('▄', button_shadow_bg, bg_color),
                );

                // Bottom shadow: use upper half block '▀' (U+2580)
                // Shadow fg on prompt bg to create half-height shadow
                for dx in 0..button_width {
                    buffer.set(
                        button_start_x + dx + 1,
                        button_y + 1,
                        Cell::new_unchecked('▀', button_shadow_bg, bg_color),
                    );
                }

                // Corner shadow (bottom-right): use quadrant upper left '▘' (U+2598)
                buffer.set(
                    button_start_x + button_width,
                    button_y + 1,
                    Cell::new_unchecked('▀', button_shadow_bg, bg_color),
                );

                button_x += 1; // Account for right shadow in spacing
            }

            // Add spacing between buttons
            button_x += 2;
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

    /// Check if a click is on a button, return the action if so
    pub fn handle_click(&self, x: u16, y: u16) -> Option<PromptAction> {
        let button_y = self.y + self.height - 2;

        // Only process clicks on the button row
        if y != button_y {
            return None;
        }

        let total_button_width: u16 = self.buttons.iter().map(|b| b.width()).sum::<u16>()
            + (self.buttons.len().saturating_sub(1)) as u16 * 2;

        let mut button_x = self.x + (self.width.saturating_sub(total_button_width)) / 2;

        for button in &self.buttons {
            let button_width = button.width();
            let button_end = button_x + button_width;

            if x >= button_x && x < button_end {
                return Some(button.action);
            }

            button_x = button_end + 2; // Move to next button (with spacing)
        }

        None
    }

    /// Check if point is within prompt bounds
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Move selection to the next button (right/tab)
    pub fn select_next_button(&mut self) {
        if !self.buttons.is_empty() {
            self.selected_button_index = (self.selected_button_index + 1) % self.buttons.len();
        }
    }

    /// Move selection to the previous button (left/shift+tab)
    pub fn select_previous_button(&mut self) {
        if !self.buttons.is_empty() {
            self.selected_button_index = if self.selected_button_index == 0 {
                self.buttons.len() - 1
            } else {
                self.selected_button_index - 1
            };
        }
    }

    /// Get the action of the currently selected button
    pub fn get_selected_action(&self) -> Option<PromptAction> {
        self.buttons
            .get(self.selected_button_index)
            .map(|b| b.action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_creation() {
        let buttons = vec![
            PromptButton::new("Yes".to_string(), PromptAction::Confirm, true),
            PromptButton::new("No".to_string(), PromptAction::Cancel, false),
        ];

        let prompt = Prompt::new(
            PromptType::Danger,
            "Are you sure?".to_string(),
            buttons,
            80,
            24,
        );

        assert_eq!(prompt.prompt_type, PromptType::Danger);
        assert_eq!(prompt.buttons.len(), 2);
    }

    #[test]
    fn test_button_width() {
        let button = PromptButton::new("OK".to_string(), PromptAction::Confirm, true);
        assert_eq!(button.width(), 6); // "[ OK ]"
    }
}
