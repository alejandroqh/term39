//! ANSI string parser for converting tui-banner output to VideoBuffer cells

use crossterm::style::Color;

/// Parsed cell from ANSI string
#[derive(Clone, Copy)]
pub struct ParsedCell {
    pub character: char,
    pub fg_color: Color,
    #[allow(dead_code)]
    pub bg_color: Color,
}

/// Parse an ANSI-escaped string into a 2D grid of cells
/// Returns Vec<Vec<ParsedCell>> where outer Vec is rows, inner Vec is columns
pub fn parse_ansi_to_cells(ansi_str: &str, default_bg: Color) -> Vec<Vec<ParsedCell>> {
    let mut result: Vec<Vec<ParsedCell>> = Vec::new();
    let mut current_row: Vec<ParsedCell> = Vec::new();

    let mut current_fg = Color::White;
    let mut current_bg = default_bg;

    let mut chars = ansi_str.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Parse escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                let (new_fg, new_bg) = parse_sgr(&mut chars, current_fg, current_bg, default_bg);
                current_fg = new_fg;
                current_bg = new_bg;
            }
        } else if ch == '\n' {
            result.push(current_row);
            current_row = Vec::new();
        } else {
            current_row.push(ParsedCell {
                character: ch,
                fg_color: current_fg,
                bg_color: current_bg,
            });
        }
    }

    // Push last row if not empty
    if !current_row.is_empty() {
        result.push(current_row);
    }

    result
}

/// Parse SGR (Select Graphic Rendition) sequence
/// Returns (new_fg, new_bg) colors
fn parse_sgr<I: Iterator<Item = char>>(
    chars: &mut std::iter::Peekable<I>,
    current_fg: Color,
    current_bg: Color,
    default_bg: Color,
) -> (Color, Color) {
    let mut params: Vec<u8> = Vec::new();
    let mut current_num = String::new();

    // Collect all parameters until 'm'
    loop {
        match chars.next() {
            Some('m') => break,
            Some(';') | Some(':') => {
                if let Ok(n) = current_num.parse() {
                    params.push(n);
                }
                current_num.clear();
            }
            Some(c) if c.is_ascii_digit() => {
                current_num.push(c);
            }
            Some(_) | None => break,
        }
    }
    // Don't forget last parameter
    if let Ok(n) = current_num.parse() {
        params.push(n);
    }

    parse_sgr_params(&params, current_fg, current_bg, default_bg)
}

/// Parse SGR parameters and return updated colors
fn parse_sgr_params(
    params: &[u8],
    mut fg: Color,
    mut bg: Color,
    default_bg: Color,
) -> (Color, Color) {
    let mut i = 0;

    while i < params.len() {
        let param = params[i];
        match param {
            0 => {
                // Reset
                fg = Color::White;
                bg = default_bg;
            }
            // Standard foreground colors (30-37)
            30 => fg = Color::Black,
            31 => fg = Color::Red,
            32 => fg = Color::Green,
            33 => fg = Color::Yellow,
            34 => fg = Color::Blue,
            35 => fg = Color::Magenta,
            36 => fg = Color::Cyan,
            37 => fg = Color::White,
            38 => {
                // Extended foreground
                if i + 1 < params.len() && params[i + 1] == 2 {
                    // RGB: 38;2;R;G;B
                    if i + 4 < params.len() {
                        fg = Color::Rgb {
                            r: params[i + 2],
                            g: params[i + 3],
                            b: params[i + 4],
                        };
                        i += 4;
                    }
                } else if i + 1 < params.len() && params[i + 1] == 5 {
                    // 256 color: 38;5;N
                    if i + 2 < params.len() {
                        fg = Color::AnsiValue(params[i + 2]);
                        i += 2;
                    }
                }
            }
            39 => fg = Color::White, // Default fg
            // Standard background colors (40-47)
            40 => bg = Color::Black,
            41 => bg = Color::Red,
            42 => bg = Color::Green,
            43 => bg = Color::Yellow,
            44 => bg = Color::Blue,
            45 => bg = Color::Magenta,
            46 => bg = Color::Cyan,
            47 => bg = Color::White,
            48 => {
                // Extended background
                if i + 1 < params.len() && params[i + 1] == 2 {
                    // RGB: 48;2;R;G;B
                    if i + 4 < params.len() {
                        bg = Color::Rgb {
                            r: params[i + 2],
                            g: params[i + 3],
                            b: params[i + 4],
                        };
                        i += 4;
                    }
                } else if i + 1 < params.len() && params[i + 1] == 5 {
                    // 256 color: 48;5;N
                    if i + 2 < params.len() {
                        bg = Color::AnsiValue(params[i + 2]);
                        i += 2;
                    }
                }
            }
            49 => bg = default_bg, // Default bg
            // Bright foreground colors (90-97)
            90 => fg = Color::DarkGrey,
            91 => fg = Color::Red,
            92 => fg = Color::Green,
            93 => fg = Color::Yellow,
            94 => fg = Color::Blue,
            95 => fg = Color::Magenta,
            96 => fg = Color::Cyan,
            97 => fg = Color::White,
            // Bright background colors (100-107)
            100 => bg = Color::DarkGrey,
            101 => bg = Color::Red,
            102 => bg = Color::Green,
            103 => bg = Color::Yellow,
            104 => bg = Color::Blue,
            105 => bg = Color::Magenta,
            106 => bg = Color::Cyan,
            107 => bg = Color::White,
            _ => {}
        }
        i += 1;
    }

    (fg, bg)
}
