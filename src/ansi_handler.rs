use crate::term_grid::{Color, CursorShape, NamedColor, TerminalGrid};
use vte::{Params, Perform};

/// ANSI escape sequence handler that implements the VTE Perform trait
pub struct AnsiHandler<'a> {
    pub grid: &'a mut TerminalGrid,
}

impl<'a> AnsiHandler<'a> {
    pub fn new(grid: &'a mut TerminalGrid) -> Self {
        Self { grid }
    }

    /// Parse SGR (Select Graphic Rendition) parameters
    fn handle_sgr(&mut self, params: &Params) {
        if params.is_empty() {
            // Reset all attributes (same as SGR 0)
            self.grid.current_attrs = Default::default();
            self.grid.current_fg = Color::Named(NamedColor::BrightGreen);
            self.grid.current_bg = Color::Named(NamedColor::Black);
            return;
        }

        let mut iter = params.iter();
        while let Some(param) = iter.next() {
            match param[0] {
                0 => {
                    // Reset
                    self.grid.current_attrs = Default::default();
                    self.grid.current_fg = Color::Named(NamedColor::BrightGreen);
                    self.grid.current_bg = Color::Named(NamedColor::Black);
                }
                1 => self.grid.current_attrs.bold = true,
                2 => self.grid.current_attrs.dim = true,
                3 => self.grid.current_attrs.italic = true,
                4 => self.grid.current_attrs.underline = true,
                5 => self.grid.current_attrs.blink = true,
                7 => self.grid.current_attrs.reverse = true,
                8 => self.grid.current_attrs.hidden = true,
                9 => self.grid.current_attrs.strikethrough = true,
                22 => {
                    // Normal intensity (not bold, not dim)
                    self.grid.current_attrs.bold = false;
                    self.grid.current_attrs.dim = false;
                }
                23 => self.grid.current_attrs.italic = false,
                24 => self.grid.current_attrs.underline = false,
                25 => self.grid.current_attrs.blink = false,
                27 => self.grid.current_attrs.reverse = false,
                28 => self.grid.current_attrs.hidden = false,
                29 => self.grid.current_attrs.strikethrough = false,
                // Foreground colors (30-37: normal, 90-97: bright)
                30 => self.grid.current_fg = Color::Named(NamedColor::Black),
                31 => self.grid.current_fg = Color::Named(NamedColor::Red),
                32 => self.grid.current_fg = Color::Named(NamedColor::Green),
                33 => self.grid.current_fg = Color::Named(NamedColor::Yellow),
                34 => self.grid.current_fg = Color::Named(NamedColor::Blue),
                35 => self.grid.current_fg = Color::Named(NamedColor::Magenta),
                36 => self.grid.current_fg = Color::Named(NamedColor::Cyan),
                37 => self.grid.current_fg = Color::Named(NamedColor::White),
                38 => {
                    // Extended foreground color
                    if let Some(next_param) = iter.next() {
                        match next_param[0] {
                            2 => {
                                // RGB color
                                if let (Some(r), Some(g), Some(b)) =
                                    (iter.next(), iter.next(), iter.next())
                                {
                                    self.grid.current_fg =
                                        Color::Rgb(r[0] as u8, g[0] as u8, b[0] as u8);
                                }
                            }
                            5 => {
                                // 256-color palette
                                if let Some(idx) = iter.next() {
                                    self.grid.current_fg = Color::Indexed(idx[0] as u8);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                39 => self.grid.current_fg = Color::Named(NamedColor::BrightGreen), // Default foreground
                // Background colors (40-47: normal, 100-107: bright)
                40 => self.grid.current_bg = Color::Named(NamedColor::Black),
                41 => self.grid.current_bg = Color::Named(NamedColor::Red),
                42 => self.grid.current_bg = Color::Named(NamedColor::Green),
                43 => self.grid.current_bg = Color::Named(NamedColor::Yellow),
                44 => self.grid.current_bg = Color::Named(NamedColor::Blue),
                45 => self.grid.current_bg = Color::Named(NamedColor::Magenta),
                46 => self.grid.current_bg = Color::Named(NamedColor::Cyan),
                47 => self.grid.current_bg = Color::Named(NamedColor::White),
                48 => {
                    // Extended background color
                    if let Some(next_param) = iter.next() {
                        match next_param[0] {
                            2 => {
                                // RGB color
                                if let (Some(r), Some(g), Some(b)) =
                                    (iter.next(), iter.next(), iter.next())
                                {
                                    self.grid.current_bg =
                                        Color::Rgb(r[0] as u8, g[0] as u8, b[0] as u8);
                                }
                            }
                            5 => {
                                // 256-color palette
                                if let Some(idx) = iter.next() {
                                    self.grid.current_bg = Color::Indexed(idx[0] as u8);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                49 => self.grid.current_bg = Color::Named(NamedColor::Black), // Default
                // Bright foreground colors (90-97)
                90 => self.grid.current_fg = Color::Named(NamedColor::BrightBlack),
                91 => self.grid.current_fg = Color::Named(NamedColor::BrightRed),
                92 => self.grid.current_fg = Color::Named(NamedColor::BrightGreen),
                93 => self.grid.current_fg = Color::Named(NamedColor::BrightYellow),
                94 => self.grid.current_fg = Color::Named(NamedColor::BrightBlue),
                95 => self.grid.current_fg = Color::Named(NamedColor::BrightMagenta),
                96 => self.grid.current_fg = Color::Named(NamedColor::BrightCyan),
                97 => self.grid.current_fg = Color::Named(NamedColor::BrightWhite),
                // Bright background colors (100-107)
                100 => self.grid.current_bg = Color::Named(NamedColor::BrightBlack),
                101 => self.grid.current_bg = Color::Named(NamedColor::BrightRed),
                102 => self.grid.current_bg = Color::Named(NamedColor::BrightGreen),
                103 => self.grid.current_bg = Color::Named(NamedColor::BrightYellow),
                104 => self.grid.current_bg = Color::Named(NamedColor::BrightBlue),
                105 => self.grid.current_bg = Color::Named(NamedColor::BrightMagenta),
                106 => self.grid.current_bg = Color::Named(NamedColor::BrightCyan),
                107 => self.grid.current_bg = Color::Named(NamedColor::BrightWhite),
                _ => {}
            }
        }
    }
}

impl Perform for AnsiHandler<'_> {
    fn print(&mut self, c: char) {
        self.grid.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.grid.put_char('\n'),
            b'\r' => self.grid.put_char('\r'),
            b'\t' => self.grid.put_char('\t'),
            b'\x08' => self.grid.put_char('\x08'), // Backspace
            b'\x07' => {}                          // Bell (ignore for now)
            b'\x0b' => self.grid.put_char('\n'),   // Vertical Tab - treat as linefeed
            b'\x0c' => {
                // Form feed (Ctrl+L) - clear screen and move cursor to home
                self.grid.clear_screen();
                self.grid.goto(0, 0);
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {
        // DCS sequences (not commonly used)
    }

    fn put(&mut self, _byte: u8) {
        // Used with hook for DCS sequences
    }

    fn unhook(&mut self) {
        // End of DCS sequence
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // OSC (Operating System Command) sequences
        // Could be used for window title, clipboard, etc.
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
        if ignore {
            return;
        }

        // CSI (Control Sequence Introducer) sequences
        match (c, intermediates) {
            // Cursor movement
            ('A', []) => {
                // Cursor Up
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.move_cursor(0, -(n as isize));
            }
            ('B', []) => {
                // Cursor Down
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.move_cursor(0, n as isize);
            }
            ('C', []) => {
                // Cursor Forward
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.move_cursor(n as isize, 0);
            }
            ('D', []) => {
                // Cursor Back
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.move_cursor(-(n as isize), 0);
            }
            ('c', []) => {
                // Primary Device Attributes (DA1)
                // Respond as VT220 with no options
                // Format: CSI ? 62 ; 0 c (VT220)
                self.grid.queue_response("\x1b[?62;0c".to_string());
            }
            ('c', [b'>']) => {
                // Secondary Device Attributes (DA2)
                // Format: CSI > Pp ; Pv ; Pc c
                // Pp=0 (VT100), Pv=0 (version), Pc=0 (ROM cartridge)
                self.grid.queue_response("\x1b[>0;0;0c".to_string());
            }
            ('E', []) => {
                // Cursor Next Line
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.move_cursor(0, n as isize);
                self.grid.cursor.x = 0;
            }
            ('F', []) => {
                // Cursor Previous Line
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.move_cursor(0, -(n as isize));
                self.grid.cursor.x = 0;
            }
            ('G', []) => {
                // Cursor Horizontal Absolute
                let col = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.cursor.x = col
                    .saturating_sub(1)
                    .min(self.grid.cols().saturating_sub(1));
            }
            ('H', []) | ('f', []) => {
                // Cursor Position
                let mut iter = params.iter();
                let row = iter.next().and_then(|p| p.first()).copied().unwrap_or(1) as usize;
                let col = iter.next().and_then(|p| p.first()).copied().unwrap_or(1) as usize;
                self.grid.goto(col.saturating_sub(1), row.saturating_sub(1));
            }
            ('J', []) => {
                // Erase in Display
                let mode = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(0);
                match mode {
                    0 => self.grid.erase_to_eos(),     // Erase below
                    1 => self.grid.erase_from_bos(),   // Erase above
                    2 | 3 => self.grid.clear_screen(), // Erase all
                    _ => {}
                }
            }
            ('K', []) => {
                // Erase in Line
                let mode = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(0);
                match mode {
                    0 => self.grid.erase_to_eol(), // Erase to right
                    1 => self.grid.erase_to_bol(), // Erase to left
                    2 => self.grid.clear_line(),   // Erase all
                    _ => {}
                }
            }
            ('P', []) => {
                // Delete Characters (DCH)
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.delete_chars(n);
            }
            ('@', []) => {
                // Insert Characters (ICH)
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.insert_chars(n);
            }
            ('X', []) => {
                // Erase Characters (ECH)
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.erase_chars(n);
            }
            ('L', []) => {
                // Insert Lines
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.scroll_down(n);
            }
            ('M', []) => {
                // Delete Lines
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.scroll_up(n);
            }
            ('S', []) => {
                // Scroll Up
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.scroll_up(n);
            }
            ('T', []) => {
                // Scroll Down
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.scroll_down(n);
            }
            ('d', []) => {
                // Vertical Position Absolute
                let row = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1) as usize;
                self.grid.cursor.y = row
                    .saturating_sub(1)
                    .min(self.grid.rows().saturating_sub(1));
            }
            ('h', []) => {
                // Standard Mode Set (SM)
                for param in params.iter() {
                    match param[0] {
                        4 => self.grid.insert_mode = true, // IRM - Insert/Replace Mode
                        20 => self.grid.lnm_mode = true,   // LNM - Line Feed/New Line Mode
                        _ => {}
                    }
                }
            }
            ('h', [b'?']) => {
                // DEC Private Mode Set
                for param in params.iter() {
                    match param[0] {
                        1 => self.grid.application_cursor_keys = true, // DECCKM
                        6 => self.grid.origin_mode = true,             // DECOM
                        7 => self.grid.auto_wrap_mode = true,          // DECAWM
                        25 => self.grid.cursor.visible = true,         // Show cursor
                        1002 => self.grid.mouse_button_tracking = true, // Button event tracking
                        1004 => self.grid.focus_event_mode = true,     // Focus events
                        1006 => self.grid.mouse_sgr_mode = true,       // SGR mouse mode
                        1015 => self.grid.mouse_urxvt_mode = true,     // URXVT mouse mode
                        47 => self.grid.use_alt_screen(),              // Alt screen (xterm)
                        1047 => self.grid.use_alt_screen(),            // Alt screen buffer
                        1048 => self.grid.save_cursor(),               // Save cursor
                        1049 => self.grid.use_alt_screen(),            // Alt screen + save cursor
                        2004 => self.grid.bracketed_paste_mode = true, // Bracketed paste
                        2026 => self.grid.begin_synchronized_output(), // Begin sync update
                        _ => {}
                    }
                }
            }
            ('l', []) => {
                // Standard Mode Reset (RM)
                for param in params.iter() {
                    match param[0] {
                        4 => self.grid.insert_mode = false, // IRM - Insert/Replace Mode
                        20 => self.grid.lnm_mode = false,   // LNM - Line Feed/New Line Mode
                        _ => {}
                    }
                }
            }
            ('l', [b'?']) => {
                // DEC Private Mode Reset
                for param in params.iter() {
                    match param[0] {
                        1 => self.grid.application_cursor_keys = false, // DECCKM
                        6 => self.grid.origin_mode = false,             // DECOM
                        7 => self.grid.auto_wrap_mode = false,          // DECAWM
                        25 => self.grid.cursor.visible = false,         // Hide cursor
                        1002 => self.grid.mouse_button_tracking = false, // Button event tracking
                        1004 => self.grid.focus_event_mode = false,     // Focus events
                        1006 => self.grid.mouse_sgr_mode = false,       // SGR mouse mode
                        1015 => self.grid.mouse_urxvt_mode = false,     // URXVT mouse mode
                        47 => self.grid.use_main_screen(),              // Main screen (xterm)
                        1047 => self.grid.use_main_screen(),            // Main screen buffer
                        1048 => self.grid.restore_cursor(),             // Restore cursor
                        1049 => self.grid.use_main_screen(), // Main screen + restore cursor
                        2004 => self.grid.bracketed_paste_mode = false, // Bracketed paste
                        2026 => self.grid.end_synchronized_output(), // End sync update
                        _ => {}
                    }
                }
            }
            ('n', []) => {
                // Device Status Report (DSR)
                let mode = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(0);
                match mode {
                    5 => {
                        // Status report - respond with "OK"
                        self.grid.queue_response("\x1b[0n".to_string());
                    }
                    6 => {
                        // Cursor position report
                        self.grid.queue_cursor_position_report();
                    }
                    _ => {}
                }
            }
            ('m', []) => {
                // SGR (Select Graphic Rendition)
                self.handle_sgr(params);
            }
            ('r', []) => {
                // Set Scroll Region
                let mut iter = params.iter();
                let top = iter.next().and_then(|p| p.first()).copied().unwrap_or(1) as usize;
                let bottom = iter
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(self.grid.rows() as u16) as usize;
                self.grid
                    .set_scroll_region(top.saturating_sub(1), bottom.saturating_sub(1));
            }
            ('s', []) => {
                // Save Cursor Position (CSI s - position only)
                self.grid.save_cursor_position();
            }
            ('u', []) => {
                // Restore Cursor Position (CSI u - position only)
                self.grid.restore_cursor_position();
            }
            ('q', [b' ']) => {
                // Set Cursor Shape (DECSCUSR)
                let shape = params
                    .iter()
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(0);
                self.grid.cursor.shape = match shape {
                    1 | 2 => CursorShape::Block,
                    3 | 4 => CursorShape::Underline,
                    5 | 6 => CursorShape::Bar,
                    _ => CursorShape::Block,
                };
            }
            _ => {
                // Unknown or unimplemented sequence
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        // ESC sequences
        match (byte, intermediates) {
            // ESC D - Index (IND): Move cursor down one line, scroll if needed
            (b'D', []) => {
                self.grid.put_char('\n');
            }

            // ESC M - Reverse Index (RI): Move cursor up one line, scroll if needed
            (b'M', []) => {
                self.grid.reverse_linefeed();
            }

            // ESC E - Next Line (NEL): Move to start of next line
            (b'E', []) => {
                self.grid.next_line();
            }

            // ESC 7 - Save Cursor (DECSC)
            (b'7', []) => {
                self.grid.save_cursor();
            }

            // ESC 8 - Restore Cursor (DECRC)
            (b'8', []) => {
                self.grid.restore_cursor();
            }

            // ESC c - Full Reset (RIS)
            (b'c', []) => {
                self.grid.reset();
            }

            // ESC H - Horizontal Tab Set (HTS)
            (b'H', []) => {
                // Set a tab stop at current cursor position
                // For now, we use default tab stops every 8 columns
            }

            // ESC = - Application Keypad (DECKPAM)
            (b'=', []) => {
                // Switch keypad to application mode
                // Not implemented - affects input handling
            }

            // ESC > - Normal Keypad (DECKPNM)
            (b'>', []) => {
                // Switch keypad to numeric mode
                // Not implemented - affects input handling
            }

            // ESC \ - String Terminator (ST)
            (b'\\', []) => {
                // Terminates OSC, DCS, APC sequences - nothing to do here
            }

            _ => {
                // Unknown or unimplemented escape sequence
            }
        }
    }
}
