//! Window number overlay for Alt+1-9 quick selection
//! Shows window numbers when Alt/Cmd is held for 500ms+

use super::manager::WindowManager;
use crate::rendering::{Cell, Theme, VideoBuffer};

/// ASCII art digits using block characters (9 wide x 6 tall)
/// Index 0 is unused (for 1-based indexing convenience)
const ASCII_DIGITS: [&[&str]; 10] = [
    // 0 - placeholder (not used)
    &[
        "         ",
        "         ",
        "         ",
        "         ",
        "         ",
        "         ",
    ],
    // 1
    &[
        " \u{2588}\u{2588}      ",
        "\u{2588}\u{2588}\u{2588}      ",
        " \u{2588}\u{2588}      ",
        " \u{2588}\u{2588}      ",
        " \u{2588}\u{2588}      ",
        "         ",
    ],
    // 2
    &[
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   ",
        "     \u{2588}\u{2588}  ",
        " \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   ",
        "\u{2588}\u{2588}       ",
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}  ",
        "         ",
    ],
    // 3
    &[
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   ",
        "     \u{2588}\u{2588}  ",
        " \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   ",
        "     \u{2588}\u{2588}  ",
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   ",
        "         ",
    ],
    // 4
    &[
        "\u{2588}\u{2588}   \u{2588}\u{2588}  ",
        "\u{2588}\u{2588}   \u{2588}\u{2588}  ",
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}  ",
        "     \u{2588}\u{2588}  ",
        "     \u{2588}\u{2588}  ",
        "         ",
    ],
    // 5
    &[
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}  ",
        "\u{2588}\u{2588}       ",
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}  ",
        "     \u{2588}\u{2588}  ",
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}  ",
        "         ",
    ],
    // 6
    &[
        " \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}  ",
        "\u{2588}\u{2588}       ",
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}  ",
        "\u{2588}\u{2588}    \u{2588}\u{2588} ",
        " \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}  ",
        "         ",
    ],
    // 7
    &[
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}  ",
        "     \u{2588}\u{2588}  ",
        "    \u{2588}\u{2588}   ",
        "   \u{2588}\u{2588}    ",
        "   \u{2588}\u{2588}    ",
        "         ",
    ],
    // 8
    &[
        " \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   ",
        "\u{2588}\u{2588}   \u{2588}\u{2588}  ",
        " \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   ",
        "\u{2588}\u{2588}   \u{2588}\u{2588}  ",
        " \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   ",
        "         ",
    ],
    // 9
    &[
        " \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   ",
        "\u{2588}\u{2588}   \u{2588}\u{2588}  ",
        " \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}  ",
        "     \u{2588}\u{2588}  ",
        " \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   ",
        "         ",
    ],
];

/// Minimum window content area size to show ASCII art
const MIN_WIDTH_FOR_ASCII: u16 = 12;
const MIN_HEIGHT_FOR_ASCII: u16 = 10;

/// Extract number from window title (e.g., "Terminal 3" -> Some(3))
fn extract_number_from_title(title: &str) -> Option<usize> {
    // Look for "Terminal N" pattern - extract the number after "Terminal "
    if let Some(rest) = title.strip_prefix("Terminal ") {
        // Take only digits before any other content (e.g., "3 [ > bash ]" -> "3")
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        num_str.parse().ok()
    } else {
        None
    }
}

/// Render window numbers on all visible windows
pub fn render_window_numbers(
    buffer: &mut VideoBuffer,
    window_manager: &WindowManager,
    theme: &Theme,
) {
    // Get window positions with titles
    let positions = window_manager.get_window_positions();

    // Render overlay for each window using the number from its title
    for (_, x, y, width, height, is_minimized, title) in &positions {
        // Skip minimized windows
        if *is_minimized {
            continue;
        }

        // Extract number from window title (e.g., "Terminal 3" -> 3)
        let number = match extract_number_from_title(title) {
            Some(n) if (1..=9).contains(&n) => n,
            _ => continue, // Skip windows without valid 1-9 number in title
        };

        // Choose rendering style based on window size
        if *width >= MIN_WIDTH_FOR_ASCII && *height >= MIN_HEIGHT_FOR_ASCII {
            render_ascii_number(buffer, *x, *y, *width, *height, number, theme);
        } else {
            render_single_digit(buffer, *x, *y, *width, *height, number, theme);
        }
    }
}

/// Render ASCII art number centered in window content area
fn render_ascii_number(
    buffer: &mut VideoBuffer,
    win_x: u16,
    win_y: u16,
    win_width: u16,
    win_height: u16,
    number: usize,
    theme: &Theme,
) {
    let digit = &ASCII_DIGITS[number];
    let digit_width = 9u16;
    let digit_height = 6u16;

    // Calculate content area (inside borders: +1 for left border, title bar at top)
    let content_x = win_x + 1;
    let content_y = win_y + 1; // +1 for title bar
    let content_width = win_width.saturating_sub(2); // -2 for left/right borders
    let content_height = win_height.saturating_sub(2); // -1 title, -1 bottom border

    // Center the digit in content area
    let start_x = content_x + content_width.saturating_sub(digit_width) / 2;
    let start_y = content_y + content_height.saturating_sub(digit_height) / 2;

    let fg = theme.overlay_number_fg;
    let bg = theme.overlay_number_bg;

    for (row_idx, row) in digit.iter().enumerate() {
        for (col_idx, ch) in row.chars().enumerate() {
            let x = start_x + col_idx as u16;
            let y = start_y + row_idx as u16;

            // Only render if within window bounds
            if x >= content_x
                && x < content_x + content_width
                && y >= content_y
                && y < content_y + content_height
            {
                buffer.set(x, y, Cell::new_unchecked(ch, fg, bg));
            }
        }
    }
}

/// Render single digit centered in window (fallback for small windows)
fn render_single_digit(
    buffer: &mut VideoBuffer,
    win_x: u16,
    win_y: u16,
    win_width: u16,
    win_height: u16,
    number: usize,
    theme: &Theme,
) {
    // Calculate content area center
    let content_x = win_x + 1;
    let content_y = win_y + 1;
    let content_width = win_width.saturating_sub(2);
    let content_height = win_height.saturating_sub(2);

    let center_x = content_x + content_width / 2;
    let center_y = content_y + content_height / 2;

    let fg = theme.overlay_number_fg;
    let bg = theme.overlay_number_bg;

    // Render the digit character
    let digit_char = char::from_digit(number as u32, 10).unwrap_or('?');
    buffer.set(center_x, center_y, Cell::new_unchecked(digit_char, fg, bg));
}
