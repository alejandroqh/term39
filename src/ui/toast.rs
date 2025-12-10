//! Toast notification component - auto-dismissing temporary messages

use crate::rendering::{Cell, Charset, CharsetMode, Theme, VideoBuffer};
use std::time::{Duration, Instant};

/// A toast notification that auto-dismisses after a specified duration
pub struct Toast {
    /// The message to display
    pub message: String,
    /// When the toast was created
    pub created_at: Instant,
    /// How long the toast should be visible
    pub duration: Duration,
}

impl Toast {
    /// Create a new toast with a 5-second duration
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            created_at: Instant::now(),
            duration: Duration::from_secs(5),
        }
    }

    /// Check if the toast has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    /// Render the toast at the bottom-center of the screen
    pub fn render(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        let (cols, rows) = buffer.dimensions();

        // Calculate toast dimensions
        let message_len = self.message.chars().count() as u16;
        let padding = 4u16; // 2 on each side
        let width = message_len + padding;

        // Position at bottom-center, above any status bar
        let x = (cols.saturating_sub(width)) / 2;
        let y = rows.saturating_sub(5); // 5 rows from bottom (3 for toast + 1 spacing + 1 for bar)

        // Toast colors - use info prompt colors
        let fg = theme.prompt_info_fg;
        let bg = theme.prompt_info_bg;

        // Get border characters based on charset mode
        let (top_left, top_right, bottom_left, bottom_right, horizontal, vertical) = match charset
            .mode
        {
            CharsetMode::Unicode | CharsetMode::UnicodeSingleLine => ('╔', '╗', '╚', '╝', '═', '║'),
            CharsetMode::Ascii => ('+', '+', '+', '+', '-', '|'),
        };

        // Draw top border
        // Use new_unchecked for performance - theme colors are pre-validated
        buffer.set(x, y, Cell::new_unchecked(top_left, fg, bg));
        for dx in 1..width - 1 {
            buffer.set(x + dx, y, Cell::new_unchecked(horizontal, fg, bg));
        }
        buffer.set(x + width - 1, y, Cell::new_unchecked(top_right, fg, bg));

        // Draw middle row with message
        let msg_y = y + 1;
        buffer.set(x, msg_y, Cell::new_unchecked(vertical, fg, bg));
        buffer.set(x + 1, msg_y, Cell::new_unchecked(' ', fg, bg));
        for (i, ch) in self.message.chars().enumerate() {
            buffer.set(x + 2 + i as u16, msg_y, Cell::new_unchecked(ch, fg, bg));
        }
        buffer.set(x + 2 + message_len, msg_y, Cell::new_unchecked(' ', fg, bg));
        buffer.set(x + width - 1, msg_y, Cell::new_unchecked(vertical, fg, bg));

        // Draw bottom border
        let bottom_y = y + 2;
        buffer.set(x, bottom_y, Cell::new_unchecked(bottom_left, fg, bg));
        for dx in 1..width - 1 {
            buffer.set(x + dx, bottom_y, Cell::new_unchecked(horizontal, fg, bg));
        }
        buffer.set(
            x + width - 1,
            bottom_y,
            Cell::new_unchecked(bottom_right, fg, bg),
        );
    }
}
