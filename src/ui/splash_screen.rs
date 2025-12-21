use crate::app::config;
use crate::rendering::{
    Cell, Charset, CharsetMode, ParsedCell, RenderBackend, Theme, VideoBuffer, parse_ansi_to_cells,
    render_shadow,
};
use crossterm::style::Color;
use std::io;
use std::thread;
use std::time;
use tui_banner::{Banner, Fill, Gradient, Palette};

/// Generate the TERM39 banner using tui-banner
fn generate_banner(charset: &Charset, bg_color: Color) -> Vec<Vec<Cell>> {
    let banner_result = match charset.mode {
        CharsetMode::Unicode | CharsetMode::UnicodeSingleLine => {
            // Unicode mode: Use vertical gradient (light sweep look) with Keep fill
            Banner::new("TERM39").map(|b| {
                b.gradient(Gradient::vertical(Palette::from_hex(&[
                    "#FFFFFF", "#F0F0F0", "#DCDCDC", "#C8C8C8", "#B4B4B4", "#A0A0A0", "#808080",
                ])))
                .fill(Fill::Keep)
                .render()
            })
        }
        CharsetMode::Ascii => {
            // ASCII mode: Use solid fill with simple character
            Banner::new("TERM39").map(|b| b.fill(Fill::Solid('#')).render())
        }
    };

    match banner_result {
        Ok(ansi_string) => {
            // Parse ANSI output to cells
            let parsed = parse_ansi_to_cells(&ansi_string, bg_color);

            // Convert ParsedCell to Cell, overriding background with theme bg
            parsed
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|pc: ParsedCell| {
                            Cell::new_unchecked(pc.character, pc.fg_color, bg_color)
                        })
                        .collect()
                })
                .collect()
        }
        Err(_) => {
            // Fallback to simple text if banner generation fails
            vec![vec![
                Cell::new_unchecked('T', Color::White, bg_color),
                Cell::new_unchecked('E', Color::White, bg_color),
                Cell::new_unchecked('R', Color::White, bg_color),
                Cell::new_unchecked('M', Color::White, bg_color),
                Cell::new_unchecked('3', Color::White, bg_color),
                Cell::new_unchecked('9', Color::White, bg_color),
            ]]
        }
    }
}

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

    let content_bg = theme.splash_bg;

    // Generate banner using tui-banner
    let banner_cells = generate_banner(charset, content_bg);

    // Calculate banner dimensions
    let art_width = banner_cells.iter().map(|r| r.len()).max().unwrap_or(0) as u16;
    let art_height = banner_cells.len() as u16;

    // License information to display below ASCII art
    let license_lines = [
        "",
        &format!("Version {}", config::VERSION),
        "MIT License",
        &format!("Copyright (c) 2025 {}", config::AUTHORS),
    ];

    let license_height = license_lines.len() as u16;
    let total_content_height = art_height + license_height;

    let window_width = art_width + 6; // 2 for borders + 4 for padding
    let window_height = total_content_height + 4; // Top border + padding + content + padding + bottom border

    // Center the window
    let window_x = (cols.saturating_sub(window_width)) / 2;
    let window_y = (rows.saturating_sub(window_height)) / 2;

    let border_color = theme.splash_border;

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

    // Render the banner cells
    let content_start_y = window_y + 2; // Start after top border and padding
    let content_x = window_x + 3; // Left padding

    for (row_idx, row) in banner_cells.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            buffer.set(
                content_x + col_idx as u16,
                content_start_y + row_idx as u16,
                *cell,
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
