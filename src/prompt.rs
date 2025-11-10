use crate::charset::Charset;
use crate::video_buffer::{Cell, VideoBuffer};
use crossterm::style::Color;

/// Prompt types with different visual styles (similar to Bootstrap alerts)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PromptType {
    #[allow(dead_code)]
    Info,     // Blue theme
    #[allow(dead_code)]
    Success,  // Green theme
    #[allow(dead_code)]
    Warning,  // Yellow theme
    Danger,   // Red theme
}

impl PromptType {
    /// Get the background color for this prompt type
    pub fn background_color(&self) -> Color {
        match self {
            PromptType::Info => Color::Blue,
            PromptType::Success => Color::Green,
            PromptType::Warning => Color::Yellow,
            PromptType::Danger => Color::Red,
        }
    }

    /// Get the foreground color for this prompt type
    pub fn foreground_color(&self) -> Color {
        match self {
            PromptType::Info => Color::White,
            PromptType::Success => Color::Black,
            PromptType::Warning => Color::Black,
            PromptType::Danger => Color::White,
        }
    }
}

/// Button in a prompt
#[derive(Clone, Debug)]
pub struct PromptButton {
    pub text: String,
    pub action: PromptAction,
    pub is_primary: bool,  // Primary buttons have attractive colors, secondary are muted
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
    pub fn colors(&self, prompt_type: PromptType) -> (Color, Color) {
        if self.is_primary {
            // Primary button: attractive colors based on prompt type
            match prompt_type {
                PromptType::Info => (Color::White, Color::DarkCyan),
                PromptType::Success => (Color::White, Color::DarkGreen),
                PromptType::Warning => (Color::Black, Color::DarkYellow),
                PromptType::Danger => (Color::White, Color::DarkRed),
            }
        } else {
            // Secondary button: muted colors
            (Color::White, Color::DarkGrey)
        }
    }

    /// Get the rendered width of the button (includes brackets and spaces)
    pub fn width(&self) -> u16 {
        // Format: [ Text ]
        self.text.len() as u16 + 4
    }
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
    pub selected_button_index: usize,  // Index of currently selected button for keyboard navigation
}

impl Prompt {
    /// Create a new prompt (auto-sized and centered)
    pub fn new(
        prompt_type: PromptType,
        message: String,
        buttons: Vec<PromptButton>,
        buffer_width: u16,
        buffer_height: u16,
    ) -> Self {
        // Calculate dimensions
        let message_lines: Vec<&str> = message.lines().collect();
        let max_message_width = message_lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;

        // Calculate total button width (with spacing)
        let total_button_width: u16 = buttons.iter().map(|b| b.width()).sum::<u16>()
            + (buttons.len().saturating_sub(1)) as u16 * 2; // 2 spaces between buttons

        // Width is max of message width and button width, plus padding
        let content_width = max_message_width.max(total_button_width);
        let width = content_width + 6; // 2 for padding on each side + 2 for borders

        // Height: message lines + padding + button row
        let height = message_lines.len() as u16 + 6; // 1 top padding + message + 1 padding + buttons + 1 padding + borders

        // Center on screen
        let x = (buffer_width.saturating_sub(width)) / 2;
        let y = (buffer_height.saturating_sub(height)) / 2;

        // Find the first primary button as default selection, or use first button
        let selected_button_index = buttons.iter()
            .position(|b| b.is_primary)
            .unwrap_or(0);

        Self {
            prompt_type,
            message,
            buttons,
            width,
            height,
            x,
            y,
            selected_button_index,
        }
    }

    /// Render the prompt to the video buffer
    pub fn render(&self, buffer: &mut VideoBuffer, _charset: &Charset) {
        let bg_color = self.prompt_type.background_color();
        let fg_color = self.prompt_type.foreground_color();

        // Fill the entire prompt area with the background color (no borders)
        for y in 0..self.height {
            for x in 0..self.width {
                buffer.set(
                    self.x + x,
                    self.y + y,
                    Cell::new(' ', fg_color, bg_color),
                );
            }
        }

        // Render message (centered)
        let message_lines: Vec<&str> = self.message.lines().collect();
        let message_start_y = self.y + 2; // Leave space at top

        for (i, line) in message_lines.iter().enumerate() {
            let line_x = self.x + (self.width.saturating_sub(line.len() as u16)) / 2;
            let line_y = message_start_y + i as u16;

            for (j, ch) in line.chars().enumerate() {
                buffer.set(
                    line_x + j as u16,
                    line_y,
                    Cell::new(ch, fg_color, bg_color),
                );
            }
        }

        // Render buttons (centered, at bottom)
        let button_y = self.y + self.height - 2;
        let total_button_width: u16 = self.buttons.iter().map(|b| b.width()).sum::<u16>()
            + (self.buttons.len().saturating_sub(1)) as u16 * 2; // 2 spaces between buttons

        let mut button_x = self.x + (self.width.saturating_sub(total_button_width)) / 2;

        for (index, button) in self.buttons.iter().enumerate() {
            let (button_fg, button_bg) = button.colors(self.prompt_type);
            let is_selected = index == self.selected_button_index;

            // Render selection indicator before button
            if is_selected {
                // Add ">" indicator before selected button (1 cell to the left)
                if button_x > self.x {
                    buffer.set(button_x - 1, button_y, Cell::new('>', fg_color, bg_color));
                }
            }

            // Render button: [ Text ]
            buffer.set(button_x, button_y, Cell::new('[', button_fg, button_bg));
            button_x += 1;
            buffer.set(button_x, button_y, Cell::new(' ', button_fg, button_bg));
            button_x += 1;

            for ch in button.text.chars() {
                buffer.set(button_x, button_y, Cell::new(ch, button_fg, button_bg));
                button_x += 1;
            }

            buffer.set(button_x, button_y, Cell::new(' ', button_fg, button_bg));
            button_x += 1;
            buffer.set(button_x, button_y, Cell::new(']', button_fg, button_bg));
            button_x += 1;

            // Render selection indicator after button
            if is_selected {
                buffer.set(button_x, button_y, Cell::new('<', fg_color, bg_color));
            }

            // Add spacing between buttons
            button_x += 2;
        }

        // Render shadow
        self.render_shadow(buffer);
    }

    fn render_shadow(&self, buffer: &mut VideoBuffer) {
        let shadow_char = '▓'; // Unicode shadow
        let shadow_color = Color::DarkGrey;
        let (buffer_width, buffer_height) = buffer.dimensions();

        // Right shadow
        let shadow_x = self.x + self.width;
        if shadow_x < buffer_width {
            for y in 1..=self.height {
                let shadow_y = self.y + y;
                if shadow_y < buffer_height {
                    buffer.set(
                        shadow_x,
                        shadow_y,
                        Cell::new(shadow_char, shadow_color, shadow_color),
                    );
                }
            }
        }

        // Bottom shadow
        let shadow_y = self.y + self.height;
        if shadow_y < buffer_height {
            for x in 1..=self.width {
                let shadow_x = self.x + x;
                if shadow_x < buffer_width {
                    buffer.set(
                        shadow_x,
                        shadow_y,
                        Cell::new(shadow_char, shadow_color, shadow_color),
                    );
                }
            }
        }
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
        self.buttons.get(self.selected_button_index).map(|b| b.action)
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
