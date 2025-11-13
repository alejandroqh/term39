//! Text mode definitions for DOS-like screen modes
//!
//! This module defines the classic DOS/VGA text modes with their
//! corresponding character cell dimensions.

use std::fmt;

/// Text mode kinds available in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextModeKind {
    /// 40 columns x 25 rows (16x16 character cells)
    Mode40x25,
    /// 80 columns x 25 rows (8x16 character cells) - Standard DOS mode
    Mode80x25,
    /// 80 columns x 43 rows (8x11 character cells)
    Mode80x43,
    /// 80 columns x 50 rows (8x8 character cells) - High density mode
    Mode80x50,
    /// 160 columns x 50 rows (8x16 character cells) - Double-wide standard
    Mode160x50,
    /// 160 columns x 100 rows (8x16 character cells) - High resolution
    Mode160x100,
    /// 320 columns x 100 rows (8x16 character cells) - Ultra-wide
    Mode320x100,
    /// 320 columns x 200 rows (8x8 character cells) - Maximum resolution
    Mode320x200,
}

impl TextModeKind {
    /// Parse text mode from string (e.g., "80x25", "80x50")
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "40x25" => Some(TextModeKind::Mode40x25),
            "80x25" => Some(TextModeKind::Mode80x25),
            "80x43" => Some(TextModeKind::Mode80x43),
            "80x50" => Some(TextModeKind::Mode80x50),
            "160x50" => Some(TextModeKind::Mode160x50),
            "160x100" => Some(TextModeKind::Mode160x100),
            "320x100" => Some(TextModeKind::Mode320x100),
            "320x200" => Some(TextModeKind::Mode320x200),
            _ => None,
        }
    }

    /// Get all available modes
    #[allow(dead_code)]
    pub fn all_modes() -> &'static [TextModeKind] {
        &[
            TextModeKind::Mode40x25,
            TextModeKind::Mode80x25,
            TextModeKind::Mode80x43,
            TextModeKind::Mode80x50,
            TextModeKind::Mode160x50,
            TextModeKind::Mode160x100,
            TextModeKind::Mode320x100,
            TextModeKind::Mode320x200,
        ]
    }
}

impl fmt::Display for TextModeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextModeKind::Mode40x25 => write!(f, "40x25"),
            TextModeKind::Mode80x25 => write!(f, "80x25"),
            TextModeKind::Mode80x43 => write!(f, "80x43"),
            TextModeKind::Mode80x50 => write!(f, "80x50"),
            TextModeKind::Mode160x50 => write!(f, "160x50"),
            TextModeKind::Mode160x100 => write!(f, "160x100"),
            TextModeKind::Mode320x100 => write!(f, "320x100"),
            TextModeKind::Mode320x200 => write!(f, "320x200"),
        }
    }
}

/// Text mode configuration
#[derive(Debug, Clone)]
pub struct TextMode {
    /// Mode identifier
    pub kind: TextModeKind,
    /// Number of columns (characters per row)
    pub cols: usize,
    /// Number of rows (character rows)
    pub rows: usize,
    /// Character cell width in pixels
    pub char_width: usize,
    /// Character cell height in pixels
    pub char_height: usize,
}

impl TextMode {
    /// Create a new text mode
    pub fn new(kind: TextModeKind) -> Self {
        let (cols, rows, char_width, char_height) = match kind {
            TextModeKind::Mode40x25 => (40, 25, 16, 16),
            TextModeKind::Mode80x25 => (80, 25, 8, 16),
            TextModeKind::Mode80x43 => (80, 43, 8, 11),
            TextModeKind::Mode80x50 => (80, 50, 8, 8),
            TextModeKind::Mode160x50 => (160, 50, 8, 16),
            TextModeKind::Mode160x100 => (160, 100, 8, 16),
            TextModeKind::Mode320x100 => (320, 100, 8, 16),
            TextModeKind::Mode320x200 => (320, 200, 8, 8),
        };

        TextMode {
            kind,
            cols,
            rows,
            char_width,
            char_height,
        }
    }

    /// Get the total width in pixels
    #[allow(dead_code)]
    pub fn pixel_width(&self) -> usize {
        self.cols * self.char_width
    }

    /// Get the total height in pixels
    #[allow(dead_code)]
    pub fn pixel_height(&self) -> usize {
        self.rows * self.char_height
    }

    /// Check if a screen position is valid
    pub fn is_valid_position(&self, col: usize, row: usize) -> bool {
        col < self.cols && row < self.rows
    }
}

impl Default for TextMode {
    fn default() -> Self {
        TextMode::new(TextModeKind::Mode80x25)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_dimensions() {
        let mode_80x25 = TextMode::new(TextModeKind::Mode80x25);
        assert_eq!(mode_80x25.cols, 80);
        assert_eq!(mode_80x25.rows, 25);
        assert_eq!(mode_80x25.char_width, 8);
        assert_eq!(mode_80x25.char_height, 16);
        assert_eq!(mode_80x25.pixel_width(), 640);
        assert_eq!(mode_80x25.pixel_height(), 400);
    }

    #[test]
    fn test_mode_parsing() {
        assert_eq!(
            TextModeKind::from_str("80x25"),
            Some(TextModeKind::Mode80x25)
        );
        assert_eq!(
            TextModeKind::from_str("80x50"),
            Some(TextModeKind::Mode80x50)
        );
        assert_eq!(TextModeKind::from_str("invalid"), None);
    }

    #[test]
    fn test_mode_display() {
        let mode = TextModeKind::Mode80x25;
        assert_eq!(format!("{}", mode), "80x25");
    }
}
