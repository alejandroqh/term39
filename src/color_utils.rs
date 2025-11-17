use crossterm::style::Color;

/// Convert a crossterm Color to RGB values (0-255).
/// For named colors, uses approximate RGB values that match common terminal palettes.
pub fn color_to_rgb(color: &Color) -> (u8, u8, u8) {
    match color {
        Color::Black => (0, 0, 0),
        Color::DarkGrey => (85, 85, 85),
        Color::Red => (255, 85, 85),
        Color::DarkRed => (170, 0, 0),
        Color::Green => (85, 255, 85),
        Color::DarkGreen => (0, 170, 0),
        Color::Yellow => (255, 255, 85),
        Color::DarkYellow => (170, 85, 0),
        Color::Blue => (85, 85, 255),
        Color::DarkBlue => (0, 0, 170),
        Color::Magenta => (255, 85, 255),
        Color::DarkMagenta => (170, 0, 170),
        Color::Cyan => (85, 255, 255),
        Color::DarkCyan => (0, 170, 170),
        Color::White => (255, 255, 255),
        Color::Grey => (170, 170, 170),
        Color::Rgb { r, g, b } => (*r, *g, *b),
        Color::AnsiValue(value) => {
            // Convert ANSI 256 color value to RGB
            ansi_to_rgb(*value)
        }
        Color::Reset => (170, 170, 170), // Default to grey
    }
}

/// Convert ANSI 256 color value to RGB.
fn ansi_to_rgb(value: u8) -> (u8, u8, u8) {
    match value {
        // 16 basic colors (0-15)
        0 => (0, 0, 0),
        1 => (170, 0, 0),
        2 => (0, 170, 0),
        3 => (170, 85, 0),
        4 => (0, 0, 170),
        5 => (170, 0, 170),
        6 => (0, 170, 170),
        7 => (170, 170, 170),
        8 => (85, 85, 85),
        9 => (255, 85, 85),
        10 => (85, 255, 85),
        11 => (255, 255, 85),
        12 => (85, 85, 255),
        13 => (255, 85, 255),
        14 => (85, 255, 255),
        15 => (255, 255, 255),
        // 216 color cube (16-231)
        16..=231 => {
            let index = value - 16;
            let r = (index / 36) * 51;
            let g = ((index % 36) / 6) * 51;
            let b = (index % 6) * 51;
            (r, g, b)
        }
        // Grayscale (232-255)
        232..=255 => {
            let gray = 8 + (value - 232) * 10;
            (gray, gray, gray)
        }
    }
}

/// Calculate relative luminance of a color according to WCAG 2.1.
/// Returns a value between 0.0 (darkest) and 1.0 (lightest).
pub fn calculate_luminance(color: &Color) -> f32 {
    let (r, g, b) = color_to_rgb(color);

    // Convert to 0.0-1.0 range
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    // Apply gamma correction (sRGB)
    let r = if r <= 0.03928 {
        r / 12.92
    } else {
        ((r + 0.055) / 1.055).powf(2.4)
    };
    let g = if g <= 0.03928 {
        g / 12.92
    } else {
        ((g + 0.055) / 1.055).powf(2.4)
    };
    let b = if b <= 0.03928 {
        b / 12.92
    } else {
        ((b + 0.055) / 1.055).powf(2.4)
    };

    // Calculate luminance using WCAG formula
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Calculate contrast ratio between foreground and background colors.
/// Returns a value between 1.0 (no contrast) and 21.0 (maximum contrast).
/// WCAG 2.1 requires:
/// - 4.5:1 for normal text (AA level)
/// - 7:1 for normal text (AAA level)
/// - 3:1 for large text (AA level)
pub fn calculate_contrast_ratio(fg: &Color, bg: &Color) -> f32 {
    let l1 = calculate_luminance(fg);
    let l2 = calculate_luminance(bg);

    let lighter = l1.max(l2);
    let darker = l1.min(l2);

    (lighter + 0.05) / (darker + 0.05)
}

/// Adjust colors to ensure minimum contrast ratio.
/// If the current contrast is insufficient, adjusts the foreground color
/// to either pure white or pure black (whichever provides better contrast).
/// Returns the adjusted (foreground, background) color pair.
pub fn ensure_contrast(fg: Color, bg: Color, min_ratio: f32) -> (Color, Color) {
    let current_ratio = calculate_contrast_ratio(&fg, &bg);

    if current_ratio >= min_ratio {
        // Contrast is already sufficient
        return (fg, bg);
    }

    // Contrast is insufficient, we need to adjust the foreground color
    let bg_luminance = calculate_luminance(&bg);

    // Try white foreground
    let white_ratio = calculate_contrast_ratio(&Color::White, &bg);
    // Try black foreground
    let black_ratio = calculate_contrast_ratio(&Color::Black, &bg);

    // Choose the foreground that provides better contrast
    let adjusted_fg = if white_ratio > black_ratio {
        Color::White
    } else {
        Color::Black
    };

    // If neither white nor black provides sufficient contrast with the background,
    // we need to adjust the background as well
    let final_ratio = calculate_contrast_ratio(&adjusted_fg, &bg);
    if final_ratio >= min_ratio {
        return (adjusted_fg, bg);
    }

    // Last resort: use high contrast pair
    // If background is dark, use white on black; if light, use black on white
    if bg_luminance < 0.5 {
        (Color::White, Color::Black)
    } else {
        (Color::Black, Color::White)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_rgb() {
        assert_eq!(color_to_rgb(&Color::Black), (0, 0, 0));
        assert_eq!(color_to_rgb(&Color::White), (255, 255, 255));
        assert_eq!(
            color_to_rgb(&Color::Rgb {
                r: 128,
                g: 64,
                b: 32
            }),
            (128, 64, 32)
        );
    }

    #[test]
    fn test_luminance() {
        let black_lum = calculate_luminance(&Color::Black);
        let white_lum = calculate_luminance(&Color::White);

        assert!(black_lum < white_lum);
        assert!(black_lum >= 0.0 && black_lum <= 1.0);
        assert!(white_lum >= 0.0 && white_lum <= 1.0);
    }

    #[test]
    fn test_contrast_ratio() {
        // Black on white should have maximum contrast
        let ratio = calculate_contrast_ratio(&Color::Black, &Color::White);
        assert!(ratio > 20.0);

        // Same color should have minimum contrast
        let ratio = calculate_contrast_ratio(&Color::Blue, &Color::Blue);
        assert!(ratio < 1.1);
    }

    #[test]
    fn test_ensure_contrast() {
        // Blue on blue (poor contrast)
        let (fg, bg) = ensure_contrast(Color::Blue, Color::Blue, 4.5);
        let ratio = calculate_contrast_ratio(&fg, &bg);
        assert!(ratio >= 4.5);

        // White on black (good contrast) should remain unchanged
        let (fg, bg) = ensure_contrast(Color::White, Color::Black, 4.5);
        assert!(matches!(fg, Color::White));
        assert!(matches!(bg, Color::Black));
    }
}
