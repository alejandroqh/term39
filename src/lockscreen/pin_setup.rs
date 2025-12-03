//! PIN Setup Dialog for configuring PIN authentication.

use crate::charset::{Charset, CharsetMode};
use crate::lockscreen::auth::{MAX_PIN_LENGTH, MIN_PIN_LENGTH, PinAuthenticator, secure_clear};
use crate::theme::Theme;
use crate::video_buffer::{self, Cell, VideoBuffer};
use crossterm::style::Color;

/// State of the PIN setup dialog
#[derive(Debug, Clone, PartialEq)]
pub enum PinSetupState {
    /// Entering new PIN
    EnterPin,
    /// Confirming new PIN
    ConfirmPin,
    /// Setup complete
    Complete { hash: String, salt: String },
    /// Setup cancelled
    Cancelled,
}

/// Focus state for PIN setup
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PinSetupFocus {
    PinField,
    ConfirmButton,
    CancelButton,
}

/// PIN Setup dialog UI
pub struct PinSetupDialog {
    state: PinSetupState,
    focus: PinSetupFocus,

    // Input fields
    pin_input: String,
    confirm_input: String,
    cursor_position: usize,

    // Which PIN field we're on
    is_confirming: bool,

    // Error message
    error_message: Option<String>,

    // Salt for hashing (obtained from config)
    salt: String,

    // Layout
    dialog_width: u16,
    dialog_height: u16,
}

impl PinSetupDialog {
    pub fn new(salt: String) -> Self {
        Self {
            state: PinSetupState::EnterPin,
            focus: PinSetupFocus::PinField,
            pin_input: String::new(),
            confirm_input: String::new(),
            cursor_position: 0,
            is_confirming: false,
            error_message: None,
            salt,
            dialog_width: 50,
            dialog_height: 12,
        }
    }

    pub fn state(&self) -> &PinSetupState {
        &self.state
    }

    /// Insert a character (printable ASCII: letters, numbers, symbols)
    pub fn insert_char(&mut self, c: char) {
        // Allow printable ASCII characters (0x21-0x7E: letters, numbers, symbols - no spaces)
        if !c.is_ascii_graphic() {
            self.error_message = Some("Only printable characters allowed".to_string());
            return;
        }

        let field = if self.is_confirming {
            &mut self.confirm_input
        } else {
            &mut self.pin_input
        };

        if field.len() < MAX_PIN_LENGTH && self.cursor_position <= field.len() {
            field.insert(self.cursor_position, c);
            self.cursor_position += 1;
            self.error_message = None;
        }
    }

    /// Delete character (backspace)
    pub fn delete_char(&mut self) {
        let field = if self.is_confirming {
            &mut self.confirm_input
        } else {
            &mut self.pin_input
        };

        if self.cursor_position > 0 && !field.is_empty() {
            field.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
            self.error_message = None;
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        let field = if self.is_confirming {
            &self.confirm_input
        } else {
            &self.pin_input
        };
        if self.cursor_position < field.len() {
            self.cursor_position += 1;
        }
    }

    /// Handle Tab key (cycle focus)
    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            PinSetupFocus::PinField => PinSetupFocus::ConfirmButton,
            PinSetupFocus::ConfirmButton => PinSetupFocus::CancelButton,
            PinSetupFocus::CancelButton => PinSetupFocus::PinField,
        };
    }

    /// Handle Enter key
    pub fn handle_enter(&mut self) {
        match self.focus {
            PinSetupFocus::PinField | PinSetupFocus::ConfirmButton => {
                self.try_advance();
            }
            PinSetupFocus::CancelButton => {
                self.cancel();
            }
        }
    }

    fn try_advance(&mut self) {
        if !self.is_confirming {
            // Validate first PIN
            if let Err(e) = PinAuthenticator::validate_pin(&self.pin_input) {
                self.error_message = Some(e);
                return;
            }
            // Move to confirmation
            self.is_confirming = true;
            self.state = PinSetupState::ConfirmPin;
            self.cursor_position = 0;
            self.error_message = None;
            self.focus = PinSetupFocus::PinField;
        } else {
            // Validate confirmation
            if self.pin_input != self.confirm_input {
                self.error_message = Some("PINs do not match".to_string());
                secure_clear(&mut self.confirm_input);
                self.cursor_position = 0;
                return;
            }

            // Generate hash
            let hash = PinAuthenticator::hash_pin(&self.pin_input, &self.salt);

            // Clear sensitive data
            secure_clear(&mut self.pin_input);
            secure_clear(&mut self.confirm_input);

            self.state = PinSetupState::Complete {
                hash,
                salt: self.salt.clone(),
            };
        }
    }

    pub fn cancel(&mut self) {
        secure_clear(&mut self.pin_input);
        secure_clear(&mut self.confirm_input);
        self.state = PinSetupState::Cancelled;
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key_event: crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;

        match key_event.code {
            KeyCode::Char(c) => {
                if matches!(self.focus, PinSetupFocus::PinField) {
                    self.insert_char(c);
                }
            }
            KeyCode::Backspace => {
                if matches!(self.focus, PinSetupFocus::PinField) {
                    self.delete_char();
                }
            }
            KeyCode::Left => {
                if matches!(self.focus, PinSetupFocus::PinField) {
                    self.move_cursor_left();
                }
            }
            KeyCode::Right => {
                if matches!(self.focus, PinSetupFocus::PinField) {
                    self.move_cursor_right();
                }
            }
            KeyCode::Tab => {
                self.cycle_focus();
            }
            KeyCode::Enter => {
                self.handle_enter();
            }
            KeyCode::Esc => {
                self.cancel();
            }
            _ => {}
        }
    }

    /// Check if point is within dialog bounds
    pub fn contains_point(&self, x: u16, y: u16, cols: u16, rows: u16) -> bool {
        let dialog_x = (cols.saturating_sub(self.dialog_width)) / 2;
        let dialog_y = (rows.saturating_sub(self.dialog_height)) / 2;
        x >= dialog_x
            && x < dialog_x + self.dialog_width
            && y >= dialog_y
            && y < dialog_y + self.dialog_height
    }

    /// Handle mouse click, return true if a button was clicked and action was taken
    pub fn handle_click(
        &mut self,
        x: u16,
        y: u16,
        cols: u16,
        rows: u16,
        charset: &Charset,
    ) -> bool {
        let dialog_x = (cols.saturating_sub(self.dialog_width)) / 2;
        let dialog_y = (rows.saturating_sub(self.dialog_height)) / 2;
        let button_y = dialog_y + self.dialog_height - 3;

        // Only process clicks on the button row
        if y != button_y {
            return false;
        }

        // Account for button shadows in Unicode mode
        let has_button_shadow = matches!(
            charset.mode,
            CharsetMode::Unicode | CharsetMode::UnicodeSingleLine
        );
        let shadow_extra = if has_button_shadow { 1 } else { 0 };

        // Calculate confirm button bounds
        let confirm_text = if self.is_confirming {
            "[ Set PIN ]"
        } else {
            "[ Next ]"
        };
        let confirm_width = confirm_text.len() as u16;
        let confirm_x = dialog_x + self.dialog_width / 2 - confirm_width - 2;
        let confirm_end = confirm_x + confirm_width + shadow_extra;

        if x >= confirm_x && x < confirm_end {
            self.try_advance();
            return true;
        }

        // Calculate cancel button bounds
        let cancel_text = "[ Cancel ]";
        let cancel_width = cancel_text.len() as u16;
        let cancel_x = dialog_x + self.dialog_width / 2 + 2;
        let cancel_end = cancel_x + cancel_width + shadow_extra;

        if x >= cancel_x && x < cancel_end {
            self.cancel();
            return true;
        }

        false
    }

    /// Render the dialog
    pub fn render(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let (cols, rows) = buffer.dimensions();

        // Center the dialog
        let x = (cols.saturating_sub(self.dialog_width)) / 2;
        let y = (rows.saturating_sub(self.dialog_height)) / 2;

        // Colors
        let bg = theme.config_content_bg;
        let fg = theme.config_content_fg;
        let border = theme.config_border;
        let title_bg = theme.config_title_bg;
        let title_fg = theme.config_title_fg;

        // Fill background
        for dy in 0..self.dialog_height {
            for dx in 0..self.dialog_width {
                buffer.set(x + dx, y + dy, Cell::new(' ', fg, bg));
            }
        }

        // Draw border
        let tl = charset.border_top_left();
        let tr = charset.border_top_right();
        let bl = charset.border_bottom_left();
        let br = charset.border_bottom_right();
        let h = charset.border_horizontal();
        let v = charset.border_vertical();

        // Top border
        buffer.set(x, y, Cell::new(tl, border, bg));
        for dx in 1..self.dialog_width - 1 {
            buffer.set(x + dx, y, Cell::new(h, border, bg));
        }
        buffer.set(x + self.dialog_width - 1, y, Cell::new(tr, border, bg));

        // Title
        let title = if self.is_confirming {
            " Confirm PIN "
        } else {
            " Set PIN "
        };
        let title_x = x + (self.dialog_width - title.len() as u16) / 2;
        for (i, ch) in title.chars().enumerate() {
            buffer.set(title_x + i as u16, y, Cell::new(ch, title_fg, title_bg));
        }

        // Side borders
        for dy in 1..self.dialog_height - 1 {
            buffer.set(x, y + dy, Cell::new(v, border, bg));
            buffer.set(x + self.dialog_width - 1, y + dy, Cell::new(v, border, bg));
        }

        // Bottom border
        buffer.set(x, y + self.dialog_height - 1, Cell::new(bl, border, bg));
        for dx in 1..self.dialog_width - 1 {
            buffer.set(x + dx, y + self.dialog_height - 1, Cell::new(h, border, bg));
        }
        buffer.set(
            x + self.dialog_width - 1,
            y + self.dialog_height - 1,
            Cell::new(br, border, bg),
        );

        // Instruction text
        let instruction = if self.is_confirming {
            "Re-enter your PIN to confirm:".to_string()
        } else {
            format!(
                "Enter a new PIN ({}-{} characters):",
                MIN_PIN_LENGTH, MAX_PIN_LENGTH
            )
        };
        let inst_x = x + 3;
        let inst_y = y + 2;
        for (i, ch) in instruction.chars().enumerate() {
            if inst_x + (i as u16) < x + self.dialog_width - 1 {
                buffer.set(inst_x + i as u16, inst_y, Cell::new(ch, fg, bg));
            }
        }

        // PIN input field
        let field_x = x + 3;
        let field_y = y + 4;
        let field_width = (self.dialog_width - 6) as usize;
        let current_input = if self.is_confirming {
            &self.confirm_input
        } else {
            &self.pin_input
        };

        let field_bg = if matches!(self.focus, PinSetupFocus::PinField) {
            Color::DarkBlue
        } else {
            Color::DarkGrey
        };
        let field_fg = Color::White;

        // Render masked input
        for i in 0..field_width {
            let ch = if i < current_input.len() { '*' } else { ' ' };
            let is_cursor =
                matches!(self.focus, PinSetupFocus::PinField) && i == self.cursor_position;

            if is_cursor {
                buffer.set(
                    field_x + i as u16,
                    field_y,
                    Cell::new(ch, field_bg, field_fg),
                );
            } else {
                buffer.set(
                    field_x + i as u16,
                    field_y,
                    Cell::new(ch, field_fg, field_bg),
                );
            }
        }

        // Error message
        if let Some(ref error) = self.error_message {
            let err_x = x + (self.dialog_width.saturating_sub(error.len() as u16)) / 2;
            let err_y = y + 6;
            for (i, ch) in error.chars().enumerate() {
                if err_x + (i as u16) < x + self.dialog_width - 1 {
                    buffer.set(err_x + i as u16, err_y, Cell::new(ch, Color::Red, bg));
                }
            }
        }

        // Buttons - styled like standard prompts
        let button_y = y + self.dialog_height - 3;

        // Check if we should render button shadows (Unicode mode only)
        let has_button_shadow = matches!(
            charset.mode,
            CharsetMode::Unicode | CharsetMode::UnicodeSingleLine
        );
        let button_shadow_bg = Color::Black;

        // Confirm button
        let confirm_text = if self.is_confirming {
            "[ Set PIN ]"
        } else {
            "[ Next ]"
        };
        let confirm_width = confirm_text.len() as u16;
        let confirm_x = x + self.dialog_width / 2 - confirm_width - 2;
        let confirm_selected = matches!(self.focus, PinSetupFocus::ConfirmButton);
        // Match prompt button colors: selected = black on yellow, unselected = black on white
        let (confirm_fg, confirm_bg) = if confirm_selected {
            (Color::Black, theme.prompt_warning_bg)
        } else {
            (Color::Black, Color::White)
        };
        for (i, ch) in confirm_text.chars().enumerate() {
            buffer.set(
                confirm_x + i as u16,
                button_y,
                Cell::new(ch, confirm_fg, confirm_bg),
            );
        }

        // Confirm button shadow
        if has_button_shadow {
            // Right shadow (half-block character '▄')
            buffer.set(
                confirm_x + confirm_width,
                button_y,
                Cell::new_unchecked('▄', button_shadow_bg, bg),
            );
            // Bottom shadow (upper half block '▀')
            for dx in 0..confirm_width {
                buffer.set(
                    confirm_x + dx + 1,
                    button_y + 1,
                    Cell::new_unchecked('▀', button_shadow_bg, bg),
                );
            }
        }

        // Cancel button
        let cancel_text = "[ Cancel ]";
        let cancel_width = cancel_text.len() as u16;
        let cancel_x = x + self.dialog_width / 2 + 2;
        let cancel_selected = matches!(self.focus, PinSetupFocus::CancelButton);
        // Match prompt button colors: selected = black on yellow, unselected = black on white
        let (cancel_fg, cancel_bg) = if cancel_selected {
            (Color::Black, theme.prompt_warning_bg)
        } else {
            (Color::Black, Color::White)
        };
        for (i, ch) in cancel_text.chars().enumerate() {
            buffer.set(
                cancel_x + i as u16,
                button_y,
                Cell::new(ch, cancel_fg, cancel_bg),
            );
        }

        // Cancel button shadow
        if has_button_shadow {
            // Right shadow (half-block character '▄')
            buffer.set(
                cancel_x + cancel_width,
                button_y,
                Cell::new_unchecked('▄', button_shadow_bg, bg),
            );
            // Bottom shadow (upper half block '▀')
            for dx in 0..cancel_width {
                buffer.set(
                    cancel_x + dx + 1,
                    button_y + 1,
                    Cell::new_unchecked('▀', button_shadow_bg, bg),
                );
            }
        }

        // Shadow
        video_buffer::render_shadow(
            buffer,
            x,
            y,
            self.dialog_width,
            self.dialog_height,
            charset,
            theme,
        );
    }
}

impl Drop for PinSetupDialog {
    fn drop(&mut self) {
        // Ensure sensitive data is cleared on drop
        secure_clear(&mut self.pin_input);
        secure_clear(&mut self.confirm_input);
    }
}
