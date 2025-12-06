//! LockScreen struct - manages the lockscreen UI and state.

use crate::app::config_manager::LockscreenAuthMode;
use crate::rendering::{Cell, Charset, Theme, VideoBuffer, render_shadow};
use crossterm::style::Color;
use std::time::{Duration, Instant};

use super::auth::{
    AuthResult, Authenticator, create_authenticator, create_authenticator_with_mode, secure_clear,
};

/// State of the lockscreen
#[derive(Debug, Clone, PartialEq)]
pub enum LockScreenState {
    /// Lockscreen is not active
    Inactive,
    /// Lockscreen is active, waiting for input
    Active,
    /// Locked out due to failed attempts (with unlock time)
    LockedOut { until: Instant },
}

/// Focus state for the login form
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputFocus {
    Username,
    Password,
}

/// The lockscreen UI component
pub struct LockScreen {
    // State
    state: LockScreenState,

    // Form fields
    username: String,
    password: String,
    cursor_position: usize,
    focus: InputFocus,

    // Error handling
    error_message: Option<String>,
    failed_attempts: u32,

    // Layout (calculated on render)
    dialog_width: u16,
    dialog_height: u16,

    // Authentication backend
    authenticator: Box<dyn Authenticator>,

    // Authentication mode
    auth_mode: LockscreenAuthMode,

    // Cached PIN hash and salt for PIN mode
    pin_hash: Option<String>,
    pin_salt: Option<String>,
}

impl LockScreen {
    /// Create a new lockscreen instance.
    /// Auto-fills username from the system.
    pub fn new() -> Self {
        let authenticator = create_authenticator();
        let username = authenticator.get_current_username().unwrap_or_default();

        Self {
            state: LockScreenState::Inactive,
            username,
            password: String::new(),
            cursor_position: 0,
            focus: InputFocus::Password, // Start on password since username is pre-filled
            error_message: None,
            failed_attempts: 0,
            dialog_width: 50,
            dialog_height: 14,
            authenticator,
            auth_mode: LockscreenAuthMode::OsAuth,
            pin_hash: None,
            pin_salt: None,
        }
    }

    /// Create a new lockscreen with the specified auth mode
    pub fn new_with_mode(
        auth_mode: LockscreenAuthMode,
        pin_hash: Option<String>,
        pin_salt: Option<String>,
    ) -> Self {
        let authenticator =
            create_authenticator_with_mode(auth_mode, pin_hash.as_deref(), pin_salt.as_deref());
        let username = authenticator.get_current_username().unwrap_or_default();

        Self {
            state: LockScreenState::Inactive,
            username,
            password: String::new(),
            cursor_position: 0,
            focus: InputFocus::Password,
            error_message: None,
            failed_attempts: 0,
            dialog_width: 50,
            dialog_height: if auth_mode == LockscreenAuthMode::Pin {
                12
            } else {
                14
            },
            authenticator,
            auth_mode,
            pin_hash,
            pin_salt,
        }
    }

    /// Update the authenticator (call when config changes)
    pub fn update_auth_mode(
        &mut self,
        auth_mode: LockscreenAuthMode,
        pin_hash: Option<String>,
        pin_salt: Option<String>,
    ) {
        self.auth_mode = auth_mode;
        self.pin_hash = pin_hash.clone();
        self.pin_salt = pin_salt.clone();
        self.authenticator =
            create_authenticator_with_mode(auth_mode, pin_hash.as_deref(), pin_salt.as_deref());
        self.dialog_height = if auth_mode == LockscreenAuthMode::Pin {
            12
        } else {
            14
        };
    }

    /// Check if authentication system is available
    /// For OS Auth: requires system auth available
    /// For PIN: requires PIN configured
    pub fn is_available(&self) -> bool {
        match self.auth_mode {
            LockscreenAuthMode::OsAuth => self.authenticator.is_available(),
            LockscreenAuthMode::Pin => self.pin_hash.is_some() && self.pin_salt.is_some(),
        }
    }

    /// Get the authentication system name (for debugging/display)
    #[allow(dead_code)]
    pub fn auth_system_name(&self) -> &'static str {
        self.authenticator.system_name()
    }

    /// Get current auth mode
    #[allow(dead_code)]
    pub fn auth_mode(&self) -> LockscreenAuthMode {
        self.auth_mode
    }

    /// Activate the lockscreen
    pub fn lock(&mut self) {
        self.state = LockScreenState::Active;
        self.password.clear();
        self.cursor_position = 0;
        self.focus = InputFocus::Password;
        self.error_message = None;
        // Don't reset failed_attempts - they persist until successful unlock
    }

    /// Check if lockscreen is active
    pub fn is_active(&self) -> bool {
        !matches!(self.state, LockScreenState::Inactive)
    }

    /// Get remaining lockout time in seconds
    pub fn lockout_remaining(&self) -> Option<u64> {
        if let LockScreenState::LockedOut { until } = self.state {
            let now = Instant::now();
            if now < until {
                return Some((until - now).as_secs() + 1); // +1 to avoid showing 0
            }
        }
        None
    }

    /// Calculate lockout duration based on failed attempts (progressive backoff)
    fn calculate_lockout_duration(&self) -> Duration {
        // Progressive backoff: 0, 0, 0, 5s, 15s, 30s, 60s, 120s
        let base_seconds = match self.failed_attempts {
            0..=2 => 0, // No lockout for first 3 attempts
            3 => 5,     // 5 seconds after 3rd failure
            4 => 15,    // 15 seconds after 4th failure
            5 => 30,    // 30 seconds after 5th failure
            6 => 60,    // 60 seconds after 6th failure
            _ => 120,   // 2 minutes for 7+ failures
        };
        Duration::from_secs(base_seconds)
    }

    /// Attempt authentication
    pub fn attempt_login(&mut self) -> bool {
        // Check if we're locked out
        if self.lockout_remaining().is_some() {
            return false;
        }

        let result = self
            .authenticator
            .authenticate(&self.username, &self.password);

        match result {
            AuthResult::Success => {
                self.state = LockScreenState::Inactive;
                self.failed_attempts = 0;
                self.password.clear();
                self.error_message = None;
                true
            }
            AuthResult::Failure(msg) => {
                self.failed_attempts += 1;
                self.error_message = Some(msg);
                self.password.clear();
                self.cursor_position = 0;

                // Apply lockout if needed
                let lockout = self.calculate_lockout_duration();
                if !lockout.is_zero() {
                    self.state = LockScreenState::LockedOut {
                        until: Instant::now() + lockout,
                    };
                } else {
                    self.state = LockScreenState::Active;
                }
                false
            }
            AuthResult::SystemError(msg) => {
                self.error_message = Some(msg);
                self.state = LockScreenState::Active;
                false
            }
        }
    }

    /// Update lockout state (call each frame)
    pub fn update(&mut self) {
        if let LockScreenState::LockedOut { until } = self.state {
            if Instant::now() >= until {
                self.state = LockScreenState::Active;
                self.error_message = None;
            }
        }
    }

    // Input handling methods

    /// Insert a character at the cursor position
    pub fn insert_char(&mut self, c: char) {
        // Don't accept input during lockout
        if self.lockout_remaining().is_some() {
            return;
        }

        match self.focus {
            InputFocus::Username => {
                if self.cursor_position <= self.username.len() {
                    self.username.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                }
            }
            InputFocus::Password => {
                if self.cursor_position <= self.password.len() {
                    self.password.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                }
            }
        }
        self.error_message = None; // Clear error on input
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char(&mut self) {
        if self.lockout_remaining().is_some() {
            return;
        }

        if self.cursor_position > 0 {
            match self.focus {
                InputFocus::Username => {
                    self.username.remove(self.cursor_position - 1);
                }
                InputFocus::Password => {
                    self.password.remove(self.cursor_position - 1);
                }
            }
            self.cursor_position -= 1;
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
        let max = match self.focus {
            InputFocus::Username => self.username.len(),
            InputFocus::Password => self.password.len(),
        };
        if self.cursor_position < max {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start of field
    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end of field
    pub fn move_cursor_end(&mut self) {
        self.cursor_position = match self.focus {
            InputFocus::Username => self.username.len(),
            InputFocus::Password => self.password.len(),
        };
    }

    /// Toggle focus between username and password fields
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            InputFocus::Username => {
                self.cursor_position = self.password.len();
                InputFocus::Password
            }
            InputFocus::Password => {
                self.cursor_position = self.username.len();
                InputFocus::Username
            }
        };
    }

    /// Render the lockscreen to the video buffer
    pub fn render(&mut self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let (cols, rows) = buffer.dimensions();

        // Fill entire screen with solid opaque background (security: hide all content)
        let lock_bg = Color::Black;
        let lock_fg = Color::DarkGrey;
        for y in 0..rows {
            for x in 0..cols {
                buffer.set(x, y, Cell::new(' ', lock_fg, lock_bg));
            }
        }

        // Calculate dialog position (centered)
        let dialog_x = (cols.saturating_sub(self.dialog_width)) / 2;
        let dialog_y = (rows.saturating_sub(self.dialog_height)) / 2;

        // Render dialog box
        self.render_dialog(buffer, charset, theme, dialog_x, dialog_y);
    }

    fn render_dialog(
        &self,
        buffer: &mut VideoBuffer,
        charset: &Charset,
        theme: &Theme,
        x: u16,
        y: u16,
    ) {
        // Use theme colors
        let bg_color = theme.config_content_bg;
        let fg_color = theme.config_content_fg;
        let border_color = theme.config_border;
        let title_bg = theme.config_title_bg;
        let title_fg = theme.config_title_fg;

        // Get border characters
        let top_left = charset.border_top_left();
        let top_right = charset.border_top_right();
        let bottom_left = charset.border_bottom_left();
        let bottom_right = charset.border_bottom_right();
        let horizontal = charset.border_horizontal();
        let vertical = charset.border_vertical();

        // Fill dialog background
        for dy in 0..self.dialog_height {
            for dx in 0..self.dialog_width {
                buffer.set(x + dx, y + dy, Cell::new(' ', fg_color, bg_color));
            }
        }

        // Draw top border
        buffer.set(x, y, Cell::new(top_left, border_color, bg_color));
        for dx in 1..self.dialog_width - 1 {
            buffer.set(x + dx, y, Cell::new(horizontal, border_color, bg_color));
        }
        buffer.set(
            x + self.dialog_width - 1,
            y,
            Cell::new(top_right, border_color, bg_color),
        );

        // Draw title centered in top border - different for PIN vs OS Auth
        let title = if self.auth_mode == LockscreenAuthMode::Pin {
            " Enter PIN "
        } else {
            " System Locked "
        };
        let title_start = x + (self.dialog_width - title.len() as u16) / 2;
        for (i, ch) in title.chars().enumerate() {
            buffer.set(title_start + i as u16, y, Cell::new(ch, title_fg, title_bg));
        }

        // Draw side borders and content area
        for dy in 1..self.dialog_height - 1 {
            buffer.set(x, y + dy, Cell::new(vertical, border_color, bg_color));
            buffer.set(
                x + self.dialog_width - 1,
                y + dy,
                Cell::new(vertical, border_color, bg_color),
            );
        }

        // Draw bottom border
        buffer.set(
            x,
            y + self.dialog_height - 1,
            Cell::new(bottom_left, border_color, bg_color),
        );
        for dx in 1..self.dialog_width - 1 {
            buffer.set(
                x + dx,
                y + self.dialog_height - 1,
                Cell::new(horizontal, border_color, bg_color),
            );
        }
        buffer.set(
            x + self.dialog_width - 1,
            y + self.dialog_height - 1,
            Cell::new(bottom_right, border_color, bg_color),
        );

        // Content padding
        let content_x = x + 3;
        let field_width = self.dialog_width - 16; // Width for input fields

        // Different layout for PIN mode vs OS Auth mode
        if self.auth_mode == LockscreenAuthMode::Pin {
            // PIN mode: only show PIN input field
            let pin_y = y + 3;
            let pin_label = "PIN:";
            for (i, ch) in pin_label.chars().enumerate() {
                buffer.set(
                    content_x + i as u16,
                    pin_y,
                    Cell::new(ch, fg_color, bg_color),
                );
            }

            // PIN input field (masked)
            let field_x = content_x + 11;
            self.render_input_field(
                buffer,
                field_x,
                pin_y,
                field_width,
                &self.password,
                true, // Masked with *
                true, // Always focused in PIN mode
                self.cursor_position,
                theme,
            );

            // Error message or lockout message
            let message_y = y + 5;
            if let Some(remaining) = self.lockout_remaining() {
                let lockout_msg = format!("Locked for {} seconds...", remaining);
                let msg_x = x + (self.dialog_width - lockout_msg.len() as u16) / 2;
                for (i, ch) in lockout_msg.chars().enumerate() {
                    buffer.set(
                        msg_x + i as u16,
                        message_y,
                        Cell::new(ch, Color::Yellow, bg_color),
                    );
                }
            } else if let Some(ref error) = self.error_message {
                let msg_x = x + (self.dialog_width - error.len() as u16) / 2;
                for (i, ch) in error.chars().enumerate() {
                    buffer.set(
                        msg_x + i as u16,
                        message_y,
                        Cell::new(ch, Color::Red, bg_color),
                    );
                }
            }

            // Failed attempts counter (if any)
            if self.failed_attempts > 0 {
                let attempts_msg = format!("Failed attempts: {}", self.failed_attempts);
                let attempts_x = x + (self.dialog_width - attempts_msg.len() as u16) / 2;
                for (i, ch) in attempts_msg.chars().enumerate() {
                    buffer.set(
                        attempts_x + i as u16,
                        y + 7,
                        Cell::new(ch, Color::DarkGrey, bg_color),
                    );
                }
            }

            // Instructions at bottom
            let instructions_y = y + 9;
            let instructions = "Enter: Unlock";
            let instructions_x = x + (self.dialog_width - instructions.len() as u16) / 2;
            for (i, ch) in instructions.chars().enumerate() {
                buffer.set(
                    instructions_x + i as u16,
                    instructions_y,
                    Cell::new(ch, Color::DarkGrey, bg_color),
                );
            }
        } else {
            // OS Auth mode: show username and password fields
            // Username label and field (y + 3)
            let username_y = y + 3;
            let username_label = "Username:";
            for (i, ch) in username_label.chars().enumerate() {
                buffer.set(
                    content_x + i as u16,
                    username_y,
                    Cell::new(ch, fg_color, bg_color),
                );
            }

            // Username input field
            let field_x = content_x + 11;
            self.render_input_field(
                buffer,
                field_x,
                username_y,
                field_width,
                &self.username,
                false, // Not masked
                self.focus == InputFocus::Username,
                if self.focus == InputFocus::Username {
                    self.cursor_position
                } else {
                    usize::MAX
                },
                theme,
            );

            // Password label and field (y + 5)
            let password_y = y + 5;
            let password_label = "Password:";
            for (i, ch) in password_label.chars().enumerate() {
                buffer.set(
                    content_x + i as u16,
                    password_y,
                    Cell::new(ch, fg_color, bg_color),
                );
            }

            // Password input field (masked)
            self.render_input_field(
                buffer,
                field_x,
                password_y,
                field_width,
                &self.password,
                true, // Masked with *
                self.focus == InputFocus::Password,
                if self.focus == InputFocus::Password {
                    self.cursor_position
                } else {
                    usize::MAX
                },
                theme,
            );

            // Error message or lockout message (y + 7)
            let message_y = y + 8;
            if let Some(remaining) = self.lockout_remaining() {
                let lockout_msg = format!("Locked for {} seconds...", remaining);
                let msg_x = x + (self.dialog_width - lockout_msg.len() as u16) / 2;
                for (i, ch) in lockout_msg.chars().enumerate() {
                    buffer.set(
                        msg_x + i as u16,
                        message_y,
                        Cell::new(ch, Color::Yellow, bg_color),
                    );
                }
            } else if let Some(ref error) = self.error_message {
                let msg_x = x + (self.dialog_width - error.len() as u16) / 2;
                for (i, ch) in error.chars().enumerate() {
                    buffer.set(
                        msg_x + i as u16,
                        message_y,
                        Cell::new(ch, Color::Red, bg_color),
                    );
                }
            }

            // Failed attempts counter (if any)
            if self.failed_attempts > 0 {
                let attempts_msg = format!("Failed attempts: {}", self.failed_attempts);
                let attempts_x = x + (self.dialog_width - attempts_msg.len() as u16) / 2;
                for (i, ch) in attempts_msg.chars().enumerate() {
                    buffer.set(
                        attempts_x + i as u16,
                        y + 9,
                        Cell::new(ch, Color::DarkGrey, bg_color),
                    );
                }
            }

            // Instructions at bottom (y + 11)
            let instructions_y = y + 11;
            let instructions = "Enter: Unlock | Tab: Switch field";
            let instructions_x = x + (self.dialog_width - instructions.len() as u16) / 2;
            for (i, ch) in instructions.chars().enumerate() {
                buffer.set(
                    instructions_x + i as u16,
                    instructions_y,
                    Cell::new(ch, Color::DarkGrey, bg_color),
                );
            }
        }

        // Render shadow (like other dialogs)
        render_shadow(
            buffer,
            x,
            y,
            self.dialog_width,
            self.dialog_height,
            charset,
            theme,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn render_input_field(
        &self,
        buffer: &mut VideoBuffer,
        x: u16,
        y: u16,
        width: u16,
        text: &str,
        masked: bool,
        focused: bool,
        cursor_pos: usize,
        _theme: &Theme,
    ) {
        let field_bg = if focused {
            Color::DarkBlue
        } else {
            Color::DarkGrey
        };
        let field_fg = Color::White;

        // Draw field content (no brackets)
        let display_text: String = if masked {
            "*".repeat(text.len())
        } else {
            text.to_string()
        };

        for i in 0..width as usize {
            let ch = display_text.chars().nth(i).unwrap_or(' ');
            let is_cursor = focused && i == cursor_pos;

            if is_cursor {
                // Cursor: inverted colors
                buffer.set(x + i as u16, y, Cell::new(ch, field_bg, field_fg));
            } else {
                buffer.set(x + i as u16, y, Cell::new(ch, field_fg, field_bg));
            }
        }
    }
}

impl Default for LockScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LockScreen {
    fn drop(&mut self) {
        // Clear sensitive data from memory
        secure_clear(&mut self.password);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockscreen_new() {
        let ls = LockScreen::new();
        assert!(!ls.is_active());
        assert!(matches!(ls.state, LockScreenState::Inactive));
    }

    #[test]
    fn test_lockscreen_lock_unlock_cycle() {
        let mut ls = LockScreen::new();
        assert!(!ls.is_active());

        ls.lock();
        assert!(ls.is_active());
        assert!(matches!(ls.state, LockScreenState::Active));
        assert!(ls.password.is_empty());
    }

    #[test]
    fn test_progressive_lockout() {
        let mut ls = LockScreen::new();

        ls.failed_attempts = 0;
        assert_eq!(ls.calculate_lockout_duration(), Duration::from_secs(0));

        ls.failed_attempts = 3;
        assert_eq!(ls.calculate_lockout_duration(), Duration::from_secs(5));

        ls.failed_attempts = 6;
        assert_eq!(ls.calculate_lockout_duration(), Duration::from_secs(60));

        ls.failed_attempts = 10;
        assert_eq!(ls.calculate_lockout_duration(), Duration::from_secs(120));
    }

    #[test]
    fn test_input_handling() {
        let mut ls = LockScreen::new();
        ls.lock();

        // Password is the default focus
        ls.insert_char('t');
        ls.insert_char('e');
        ls.insert_char('s');
        ls.insert_char('t');

        assert_eq!(ls.password, "test");
        assert_eq!(ls.cursor_position, 4);
    }

    #[test]
    fn test_toggle_focus() {
        let mut ls = LockScreen::new();
        ls.lock();

        assert!(matches!(ls.focus, InputFocus::Password));

        ls.toggle_focus();
        assert!(matches!(ls.focus, InputFocus::Username));

        ls.toggle_focus();
        assert!(matches!(ls.focus, InputFocus::Password));
    }

    #[test]
    fn test_delete_char() {
        let mut ls = LockScreen::new();
        ls.lock();

        ls.insert_char('a');
        ls.insert_char('b');
        ls.insert_char('c');
        assert_eq!(ls.password, "abc");

        ls.delete_char();
        assert_eq!(ls.password, "ab");
        assert_eq!(ls.cursor_position, 2);
    }
}
