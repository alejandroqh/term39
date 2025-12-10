use crate::app::config;
use crate::rendering::{
    Cell, Charset, CharsetMode, RenderBackend, Theme, VideoBuffer, render_shadow,
};
use crossterm::style::Color;
use std::io;
use std::thread;
use std::time;

/// Shows the splash screen with TERM39 logo and license information
pub fn show_splash_screen(
    buffer: &mut VideoBuffer,
    backend: &mut Box<dyn RenderBackend>,
    charset: &Charset,
    theme: &Theme,
) -> io::Result<()> {
    let (cols, rows) = buffer.dimensions();

    // Clear screen to black (outside the splash box)
    // Use new_unchecked for performance - theme colors are pre-validated
    let black_cell = Cell::new_unchecked(' ', theme.splash_fg, Color::Black);
    for y in 0..rows {
        for x in 0..cols {
            buffer.set(x, y, black_cell);
        }
    }

    // Choose ASCII art based on charset mode
    let ascii_art = match charset.mode {
        CharsetMode::Unicode | CharsetMode::UnicodeSingleLine => vec![
            " ███████████ ██████████ ███████████   ██████   ██████  ████████   ████████ ",
            "░█░░░███░░░█░░███░░░░░█░░███░░░░░███ ░░██████ ██████  ███░░░░███ ███░░░░███",
            "░   ░███  ░  ░███  █ ░  ░███    ░███  ░███░█████░███ ░░░    ░███░███   ░███",
            "    ░███     ░██████    ░██████████   ░███░░███ ░███    ██████░ ░░█████████",
            "    ░███     ░███░░█    ░███░░░░░███  ░███ ░░░  ░███   ░░░░░░███ ░░░░░░░███",
            "    ░███     ░███ ░   █ ░███    ░███  ░███      ░███  ███   ░███ ███   ░███",
            "    █████    ██████████ █████   █████ █████     █████░░████████ ░░████████ ",
            "   ░░░░░    ░░░░░░░░░░ ░░░░░   ░░░░░ ░░░░░     ░░░░░  ░░░░░░░░   ░░░░░░░░  ",
        ],
        CharsetMode::Ascii => vec![
            " TTTTTTTTTTT EEEEEEEEEE RRRRRRRRRRR   MMMMMM   MMMMMM  333333333   99999999 ",
            ".T...TTT...T..EEE.....E..RRR.....RRR ..MMMMMM MMMMMM  333....333 999....999",
            ".   .TTT  .  .EEE  E .  .RRR    .RRR  .MMM.MMMMM.MMM .      .333.999   .999",
            "    .TTT     .EEEEEE    .RRRRRRRRR    .MMM..MMM .MMM    333333. ..999999999",
            "    .TTT     .EEE..E    .RRR...RRR    .MMM ...  .MMM   .......333 .......999",
            "    .TTT     .EEE .   E .RRR    .RRR  .MMM      .MMM  333   .333 999   .999",
            "    TTTTT    EEEEEEEEEE RRRRR   RRRRR MMMMM     MMMMM..333333333 ..99999999 ",
            "   .....    .......... .....   ..... .....     .....  ..........   ........  ",
        ],
    };

    // License information to display below ASCII art
    let license_lines = [
        "",
        &format!("Version {}", config::VERSION),
        "MIT License",
        &format!("Copyright (c) 2025 {}", config::AUTHORS),
    ];

    // Calculate window dimensions
    let art_width = ascii_art[0].chars().count() as u16;
    let art_height = ascii_art.len() as u16;
    let license_height = license_lines.len() as u16;
    let total_content_height = art_height + license_height;

    let window_width = art_width + 6; // 2 for borders + 4 for padding
    let window_height = total_content_height + 4; // Top border + padding + content + padding + bottom border

    // Center the window
    let window_x = (cols.saturating_sub(window_width)) / 2;
    let window_y = (rows.saturating_sub(window_height)) / 2;

    let border_color = theme.splash_border;
    let content_bg = theme.splash_bg;

    // Draw top border using charset
    // Use new_unchecked for performance - theme colors are pre-validated
    buffer.set(
        window_x,
        window_y,
        Cell::new_unchecked(charset.border_top_left, border_color, content_bg),
    );
    for x in 1..window_width - 1 {
        buffer.set(
            window_x + x,
            window_y,
            Cell::new_unchecked(charset.border_horizontal, border_color, content_bg),
        );
    }
    buffer.set(
        window_x + window_width - 1,
        window_y,
        Cell::new_unchecked(charset.border_top_right, border_color, content_bg),
    );

    // Draw middle rows (content area)
    for y in 1..window_height - 1 {
        // Left border
        buffer.set(
            window_x,
            window_y + y,
            Cell::new_unchecked(charset.border_vertical, border_color, content_bg),
        );

        // Content
        for x in 1..window_width - 1 {
            buffer.set(
                window_x + x,
                window_y + y,
                Cell::new_unchecked(' ', theme.splash_fg, content_bg),
            );
        }

        // Right border
        buffer.set(
            window_x + window_width - 1,
            window_y + y,
            Cell::new_unchecked(charset.border_vertical, border_color, content_bg),
        );
    }

    // Draw bottom border using charset
    buffer.set(
        window_x,
        window_y + window_height - 1,
        Cell::new_unchecked(charset.border_bottom_left, border_color, content_bg),
    );
    for x in 1..window_width - 1 {
        buffer.set(
            window_x + x,
            window_y + window_height - 1,
            Cell::new_unchecked(charset.border_horizontal, border_color, content_bg),
        );
    }
    buffer.set(
        window_x + window_width - 1,
        window_y + window_height - 1,
        Cell::new_unchecked(charset.border_bottom_right, border_color, content_bg),
    );

    // Draw shadow (right and bottom) using shared function
    render_shadow(
        buffer,
        window_x,
        window_y,
        window_width,
        window_height,
        charset,
        theme,
    );

    // Define gradient colors for TERM39 logo (white to dark gray gradient)
    // Only 7 colors so row 8 (shadow area) stays dark
    let gradient_colors = [
        Color::Rgb {
            r: 0xFF,
            g: 0xFF,
            b: 0xFF,
        }, // #FFFFFF - White
        Color::Rgb {
            r: 0xF0,
            g: 0xF0,
            b: 0xF0,
        }, // #F0F0F0 - Very light gray
        Color::Rgb {
            r: 0xDC,
            g: 0xDC,
            b: 0xDC,
        }, // #DCDCDC - Light gray
        Color::Rgb {
            r: 0xC8,
            g: 0xC8,
            b: 0xC8,
        }, // #C8C8C8 - Medium-light gray
        Color::Rgb {
            r: 0xB4,
            g: 0xB4,
            b: 0xB4,
        }, // #B4B4B4 - Medium gray
        Color::Rgb {
            r: 0xA0,
            g: 0xA0,
            b: 0xA0,
        }, // #A0A0A0 - Medium-dark gray
        Color::Rgb {
            r: 0x80,
            g: 0x80,
            b: 0x80,
        }, // #808080 - Dark gray
        Color::DarkGrey, // Shadow area
    ];

    // Render ASCII art centered in the window with gradient colors
    let content_start_y = window_y + 2; // Start after top border and padding
    let content_x = window_x + 3; // Left padding

    for (i, line) in ascii_art.iter().enumerate() {
        // Get the color for this row (use gradient color if available, otherwise use first color)
        let row_color = gradient_colors
            .get(i)
            .copied()
            .unwrap_or(gradient_colors[0]);

        for (j, ch) in line.chars().enumerate() {
            // Render the actual character with gradient color
            buffer.set(
                content_x + j as u16,
                content_start_y + i as u16,
                Cell::new_unchecked(ch, row_color, content_bg),
            );
        }
    }

    // Render license information below ASCII art
    let license_start_y = content_start_y + art_height;
    for (i, line) in license_lines.iter().enumerate() {
        // Center each license line
        let line_len = line.chars().count() as u16;
        let line_x = if line_len < art_width {
            content_x + (art_width - line_len) / 2
        } else {
            content_x
        };

        for (j, ch) in line.chars().enumerate() {
            buffer.set(
                line_x + j as u16,
                license_start_y + i as u16,
                Cell::new_unchecked(ch, theme.splash_fg, content_bg),
            );
        }
    }

    // Present to screen
    backend.present(&mut *buffer)?;

    // Wait for 1 second
    thread::sleep(time::Duration::from_secs(1));

    Ok(())
}
