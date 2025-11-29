use std::fmt;
use unicode_width::UnicodeWidthChar;

/// Terminal color representation supporting 256-color palette and 24-bit truecolor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Named ANSI colors (0-15)
    Named(NamedColor),
    /// 256-color palette (0-255)
    Indexed(u8),
    /// 24-bit RGB color
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    BrightBlack = 8,
    BrightRed = 9,
    BrightGreen = 10,
    BrightYellow = 11,
    BrightBlue = 12,
    BrightMagenta = 13,
    BrightCyan = 14,
    BrightWhite = 15,
}

impl Default for Color {
    fn default() -> Self {
        Color::Named(NamedColor::White)
    }
}

/// Character cell attributes (bold, italic, underline, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CellAttributes {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
}

/// A single terminal cell containing a character and its display attributes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalCell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
    pub attrs: CellAttributes,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: Color::Named(NamedColor::White),
            bg: Color::Named(NamedColor::Black),
            attrs: CellAttributes::default(),
        }
    }
}

impl TerminalCell {
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.c = ' ';
        self.fg = Color::Named(NamedColor::White);
        self.bg = Color::Named(NamedColor::Black);
        self.attrs = CellAttributes::default();
    }
}

/// Cursor position and visibility state
#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub visible: bool,
    pub shape: CursorShape,
}

/// Saved cursor state (for DECSC/DECRC)
#[derive(Debug, Clone, Copy)]
pub struct SavedCursorState {
    pub cursor: Cursor,
    pub attrs: CellAttributes,
    pub fg: Color,
    pub bg: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    Block,
    Underline,
    Bar,
}

/// VT100 Character Set designation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharacterSet {
    /// Standard ASCII character set
    #[default]
    Ascii,
    /// DEC Special Graphics (line drawing characters)
    /// Maps ASCII 0x5f-0x7e to box-drawing characters
    DecSpecialGraphics,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            visible: true,
            shape: CursorShape::Block,
        }
    }
}

/// Terminal grid with scrollback buffer
pub struct TerminalGrid {
    /// Screen rows (visible portion)
    rows: Vec<Vec<TerminalCell>>,
    /// Scrollback buffer (lines that have scrolled off the top)
    scrollback: Vec<Vec<TerminalCell>>,
    /// Maximum scrollback lines
    max_scrollback: usize,
    /// Terminal dimensions
    cols: usize,
    rows_count: usize,
    /// Cursor state
    pub cursor: Cursor,
    /// Current cell attributes (for new characters)
    pub current_attrs: CellAttributes,
    pub current_fg: Color,
    pub current_bg: Color,
    /// Scroll region (for CSI scrolling)
    scroll_region_top: usize,
    scroll_region_bottom: usize,
    /// Saved cursor state (for DECSC/DECRC - includes colors and attributes)
    saved_cursor: Option<SavedCursorState>,
    /// Alternate screen buffer
    alt_screen: Option<Vec<Vec<TerminalCell>>>,
    /// Tab stops (every 8 columns by default)
    tab_stops: Vec<bool>,
    /// DEC Private Modes
    /// Application cursor keys mode (DECCKM ?1)
    pub application_cursor_keys: bool,
    /// Bracketed paste mode (?2004)
    pub bracketed_paste_mode: bool,
    /// Focus event reporting (?1004)
    pub focus_event_mode: bool,
    /// Synchronized output mode (?2026)
    pub synchronized_output: bool,
    /// Snapshot of rows when synchronized output began (for rendering during sync mode)
    sync_snapshot: Option<Vec<Vec<TerminalCell>>>,
    /// Snapshot of cursor when synchronized output began
    sync_cursor_snapshot: Option<Cursor>,
    /// Mouse tracking modes
    pub mouse_normal_tracking: bool, // ?1000 - Normal mouse tracking (clicks)
    pub mouse_button_tracking: bool, // ?1002 - Button event tracking
    pub mouse_any_event_tracking: bool, // ?1003 - Any event tracking (all motion)
    pub mouse_utf8_mode: bool,       // ?1005 - UTF-8 mouse encoding
    pub mouse_sgr_mode: bool,        // ?1006 - SGR extended mouse mode
    pub mouse_urxvt_mode: bool,      // ?1015 - URXVT mouse mode
    /// Line Feed/New Line Mode (LNM - mode 20)
    /// When set, LF also performs CR (linefeed acts as newline)
    pub lnm_mode: bool,
    /// Auto-wrap mode (DECAWM - ?7)
    /// When set, characters wrap to next line at end of line
    pub auto_wrap_mode: bool,
    /// Insert/Replace mode (IRM - mode 4)
    /// When set, characters are inserted; when reset, characters replace
    pub insert_mode: bool,
    /// Origin mode (DECOM - ?6)
    /// When set, cursor positioning is relative to scroll region
    pub origin_mode: bool,
    /// Response queue for DSR and other queries that need to send data back
    response_queue: Vec<String>,
    /// G0 character set (selected by ESC ( X)
    pub charset_g0: CharacterSet,
    /// G1 character set (selected by ESC ) X)
    pub charset_g1: CharacterSet,
    /// Active character set: true = G0, false = G1 (toggled by SI/SO)
    pub charset_use_g0: bool,
}

impl TerminalGrid {
    pub fn new(cols: usize, rows: usize, max_scrollback: usize) -> Self {
        let mut tab_stops = vec![false; cols];
        for i in (0..cols).step_by(8) {
            tab_stops[i] = true;
        }

        Self {
            rows: vec![vec![TerminalCell::default(); cols]; rows],
            scrollback: Vec::new(),
            max_scrollback,
            cols,
            rows_count: rows,
            cursor: Cursor::default(),
            current_attrs: CellAttributes::default(),
            current_fg: Color::Named(NamedColor::BrightGreen),
            current_bg: Color::Named(NamedColor::Black),
            scroll_region_top: 0,
            scroll_region_bottom: rows.saturating_sub(1),
            saved_cursor: None,
            alt_screen: None,
            tab_stops,
            application_cursor_keys: false,
            bracketed_paste_mode: false,
            focus_event_mode: false,
            synchronized_output: false,
            sync_snapshot: None,
            sync_cursor_snapshot: None,
            mouse_normal_tracking: false,
            mouse_button_tracking: false,
            mouse_any_event_tracking: false,
            mouse_utf8_mode: false,
            mouse_sgr_mode: false,
            mouse_urxvt_mode: false,
            lnm_mode: false,
            auto_wrap_mode: true, // Default: enabled (xterm behavior)
            insert_mode: false,   // Default: replace mode
            origin_mode: false,   // Default: absolute positioning
            response_queue: Vec::new(),
            charset_g0: CharacterSet::Ascii,
            charset_g1: CharacterSet::Ascii,
            charset_use_g0: true, // Default: use G0
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows_count
    }

    pub fn scroll_region_top(&self) -> usize {
        self.scroll_region_top
    }

    pub fn scroll_region_bottom(&self) -> usize {
        self.scroll_region_bottom
    }

    #[allow(dead_code)]
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    /// Get the currently active character set
    pub fn active_charset(&self) -> CharacterSet {
        if self.charset_use_g0 {
            self.charset_g0
        } else {
            self.charset_g1
        }
    }

    /// Map a character through the active character set
    /// DEC Special Graphics maps ASCII characters to box-drawing characters
    pub fn map_char(&self, c: char) -> char {
        match self.active_charset() {
            CharacterSet::Ascii => c,
            CharacterSet::DecSpecialGraphics => {
                // DEC Special Graphics character mapping
                // Maps ASCII 0x5f-0x7e to box-drawing characters
                match c {
                    '_' => ' ', // 0x5f -> blank
                    '`' => '◆', // 0x60 -> diamond
                    'a' => '▒', // 0x61 -> checkerboard
                    'b' => '␉', // 0x62 -> HT symbol
                    'c' => '␌', // 0x63 -> FF symbol
                    'd' => '␍', // 0x64 -> CR symbol
                    'e' => '␊', // 0x65 -> LF symbol
                    'f' => '°', // 0x66 -> degree symbol
                    'g' => '±', // 0x67 -> plus/minus
                    'h' => '␤', // 0x68 -> NL symbol
                    'i' => '␋', // 0x69 -> VT symbol
                    'j' => '┘', // 0x6a -> lower right corner
                    'k' => '┐', // 0x6b -> upper right corner
                    'l' => '┌', // 0x6c -> upper left corner
                    'm' => '└', // 0x6d -> lower left corner
                    'n' => '┼', // 0x6e -> crossing lines
                    'o' => '⎺', // 0x6f -> horizontal line - scan 1
                    'p' => '⎻', // 0x70 -> horizontal line - scan 3
                    'q' => '─', // 0x71 -> horizontal line - scan 5
                    'r' => '⎼', // 0x72 -> horizontal line - scan 7
                    's' => '⎽', // 0x73 -> horizontal line - scan 9
                    't' => '├', // 0x74 -> left tee
                    'u' => '┤', // 0x75 -> right tee
                    'v' => '┴', // 0x76 -> bottom tee
                    'w' => '┬', // 0x77 -> top tee
                    'x' => '│', // 0x78 -> vertical line
                    'y' => '≤', // 0x79 -> less than or equal
                    'z' => '≥', // 0x7a -> greater than or equal
                    '{' => 'π', // 0x7b -> pi
                    '|' => '≠', // 0x7c -> not equal
                    '}' => '£', // 0x7d -> UK pound
                    '~' => '·', // 0x7e -> bullet/centered dot
                    _ => c,     // Other characters pass through unchanged
                }
            }
        }
    }

    /// Set G0 character set (ESC ( X)
    pub fn set_charset_g0(&mut self, charset: CharacterSet) {
        self.charset_g0 = charset;
    }

    /// Set G1 character set (ESC ) X)
    pub fn set_charset_g1(&mut self, charset: CharacterSet) {
        self.charset_g1 = charset;
    }

    /// Shift In (SI / Ctrl+O / 0x0F) - Select G0 character set
    pub fn shift_in(&mut self) {
        self.charset_use_g0 = true;
    }

    /// Shift Out (SO / Ctrl+N / 0x0E) - Select G1 character set
    pub fn shift_out(&mut self) {
        self.charset_use_g0 = false;
    }

    /// Queue a response to be sent back to the PTY (for DSR and other queries)
    pub fn queue_response(&mut self, response: String) {
        self.response_queue.push(response);
    }

    /// Take all queued responses (drains the queue)
    pub fn take_responses(&mut self) -> Vec<String> {
        std::mem::take(&mut self.response_queue)
    }

    /// Queue cursor position report (DSR response to CSI 6 n)
    /// Format: CSI row ; col R (1-based coordinates)
    /// When origin mode (DECOM) is set, reports position relative to scroll region
    pub fn queue_cursor_position_report(&mut self) {
        let (row, col) = if self.origin_mode {
            // In origin mode, report position relative to scroll region
            let row = (self.cursor.y.saturating_sub(self.scroll_region_top)) + 1;
            let col = self.cursor.x + 1;
            (row, col)
        } else {
            // Normal mode, report absolute position
            let row = self.cursor.y + 1;
            let col = self.cursor.x + 1;
            (row, col)
        };
        let response = format!("\x1b[{};{}R", row, col);
        self.queue_response(response);
    }

    /// Resize the terminal grid
    pub fn resize(&mut self, new_cols: usize, new_rows: usize) {
        // Resize existing rows
        for row in &mut self.rows {
            row.resize(new_cols, TerminalCell::default());
        }

        // Add or remove rows
        match new_rows.cmp(&self.rows_count) {
            std::cmp::Ordering::Greater => {
                self.rows
                    .resize(new_rows, vec![TerminalCell::default(); new_cols]);
            }
            std::cmp::Ordering::Less => {
                self.rows.truncate(new_rows);
            }
            std::cmp::Ordering::Equal => {}
        }

        // Update tab stops
        self.tab_stops.resize(new_cols, false);
        for i in (0..new_cols).step_by(8) {
            self.tab_stops[i] = true;
        }

        self.cols = new_cols;
        self.rows_count = new_rows;
        self.scroll_region_bottom = new_rows.saturating_sub(1);

        // Clamp cursor to new bounds
        self.cursor.x = self.cursor.x.min(new_cols.saturating_sub(1));
        self.cursor.y = self.cursor.y.min(new_rows.saturating_sub(1));
    }

    /// Get a cell at the given position (returns None if out of bounds)
    /// This returns the live cell, use get_render_cell for rendering during synchronized output
    pub fn get_cell(&self, x: usize, y: usize) -> Option<&TerminalCell> {
        self.rows.get(y)?.get(x)
    }

    /// Get a cell for rendering - respects synchronized output snapshot
    /// During synchronized output mode, returns the snapshot cell to prevent visual tearing
    pub fn get_render_cell(&self, x: usize, y: usize) -> Option<&TerminalCell> {
        if let Some(snapshot) = &self.sync_snapshot {
            snapshot.get(y)?.get(x)
        } else {
            self.rows.get(y)?.get(x)
        }
    }

    /// Get cursor for rendering - respects synchronized output snapshot
    pub fn get_render_cursor(&self) -> &Cursor {
        if let Some(cursor) = &self.sync_cursor_snapshot {
            cursor
        } else {
            &self.cursor
        }
    }

    /// Get a mutable cell at the given position
    pub fn get_cell_mut(&mut self, x: usize, y: usize) -> Option<&mut TerminalCell> {
        self.rows.get_mut(y)?.get_mut(x)
    }

    /// Get a line from scrollback (0 = oldest)
    #[allow(dead_code)]
    pub fn get_scrollback_line(&self, idx: usize) -> Option<&Vec<TerminalCell>> {
        self.scrollback.get(idx)
    }

    /// Begin synchronized output mode - takes a snapshot of current state for rendering
    pub fn begin_synchronized_output(&mut self) {
        if !self.synchronized_output {
            self.synchronized_output = true;
            // Take snapshot of current state for rendering during sync mode
            self.sync_snapshot = Some(self.rows.clone());
            self.sync_cursor_snapshot = Some(self.cursor);
        }
    }

    /// End synchronized output mode - clears snapshot to allow live rendering
    pub fn end_synchronized_output(&mut self) {
        self.synchronized_output = false;
        self.sync_snapshot = None;
        self.sync_cursor_snapshot = None;
    }

    /// Write a character at the current cursor position
    pub fn put_char(&mut self, c: char) {
        if self.cursor.y >= self.rows_count {
            return;
        }

        match c {
            '\n' => self.linefeed(),
            '\r' => self.carriage_return(),
            '\t' => self.tab(),
            '\x08' => self.backspace(),
            c if c.is_control() => {
                // Ignore other control characters
            }
            c => {
                // Map character through active character set (for line drawing, etc.)
                let c = self.map_char(c);

                // Get character width (0 for combining marks, 1 for normal, 2 for wide/fullwidth)
                let char_width = c.width().unwrap_or(0);

                // Skip zero-width characters (combining marks, etc.) - they don't advance cursor
                // but we should still render them (TODO: proper combining char support)
                if char_width == 0 {
                    return;
                }

                // Write character at cursor position
                if self.cursor.x < self.cols {
                    // Copy values before mutable borrow
                    let fg = self.current_fg;
                    let bg = self.current_bg;
                    let attrs = self.current_attrs;

                    // Handle wide characters (width = 2)
                    if char_width == 2 {
                        // Check if we have room for a wide character
                        if self.cursor.x + 1 >= self.cols {
                            // Not enough room on this line for wide char
                            // Either wrap to next line or stay at end
                            if self.auto_wrap_mode {
                                if self.cursor.y == self.rows_count - 1 {
                                    // At last row, can't wrap, skip the character
                                    return;
                                }
                                // Wrap to next line
                                self.cursor.x = 0;
                                self.linefeed();
                            } else {
                                // No wrap mode, skip the character
                                return;
                            }
                        }

                        // Write the wide character to first cell
                        if let Some(cell) = self.get_cell_mut(self.cursor.x, self.cursor.y) {
                            cell.c = c;
                            cell.fg = fg;
                            cell.bg = bg;
                            cell.attrs = attrs;
                        }

                        // Write a placeholder space to second cell (for wide char continuation)
                        if let Some(cell) = self.get_cell_mut(self.cursor.x + 1, self.cursor.y) {
                            cell.c = ' ';
                            cell.fg = fg;
                            cell.bg = bg;
                            cell.attrs = attrs;
                        }

                        self.cursor.x += 2;
                    } else {
                        // Normal width character
                        if let Some(cell) = self.get_cell_mut(self.cursor.x, self.cursor.y) {
                            cell.c = c;
                            cell.fg = fg;
                            cell.bg = bg;
                            cell.attrs = attrs;
                        }
                        self.cursor.x += 1;
                    }

                    // Auto-wrap at end of line (if DECAWM is enabled)
                    if self.cursor.x >= self.cols {
                        if self.auto_wrap_mode {
                            // Don't auto-wrap at the very last row - just stay at the last column
                            // This prevents unwanted scrolling when drawing the last row
                            if self.cursor.y == self.rows_count - 1 {
                                self.cursor.x = self.cols - 1;
                            } else {
                                self.cursor.x = 0;
                                self.linefeed();
                            }
                        } else {
                            // No auto-wrap: stay at last column
                            self.cursor.x = self.cols - 1;
                        }
                    }
                }
            }
        }
    }

    /// Move cursor to the next line, scrolling if necessary
    /// If LNM (Line Feed/New Line Mode) is set, also performs carriage return
    fn linefeed(&mut self) {
        // If LNM is set, linefeed also performs carriage return
        if self.lnm_mode {
            self.cursor.x = 0;
        }

        if self.cursor.y == self.scroll_region_bottom {
            // At bottom of scroll region
            // Don't scroll during synchronized output
            if self.synchronized_output {
                // Stay at last row without scrolling
            } else {
                self.scroll_up(1);
            }
        } else if self.cursor.y < self.rows_count - 1 {
            self.cursor.y += 1;
        }
    }

    /// Move cursor to start of line
    fn carriage_return(&mut self) {
        self.cursor.x = 0;
    }

    /// Reverse linefeed - move cursor up one line, scrolling down if at top of scroll region
    pub fn reverse_linefeed(&mut self) {
        if self.cursor.y == self.scroll_region_top {
            // At top of scroll region, scroll down
            self.scroll_down(1);
        } else if self.cursor.y > 0 {
            self.cursor.y -= 1;
        }
    }

    /// Next line - carriage return + linefeed
    pub fn next_line(&mut self) {
        self.carriage_return();
        self.linefeed();
    }

    /// Reset terminal to initial state
    pub fn reset(&mut self) {
        // Clear screen
        self.clear_screen();

        // Reset cursor
        self.cursor = Cursor::default();

        // Reset attributes
        self.current_attrs = CellAttributes::default();
        self.current_fg = Color::Named(NamedColor::BrightGreen);
        self.current_bg = Color::Named(NamedColor::Black);

        // Reset scroll region
        self.scroll_region_top = 0;
        self.scroll_region_bottom = self.rows_count.saturating_sub(1);

        // Clear saved cursor
        self.saved_cursor = None;

        // Clear alt screen
        self.alt_screen = None;

        // Reset scrollback
        self.scrollback.clear();

        // Reset DEC private modes
        self.application_cursor_keys = false;
        self.bracketed_paste_mode = false;
        self.focus_event_mode = false;
        self.synchronized_output = false;
        self.sync_snapshot = None;
        self.sync_cursor_snapshot = None;
        self.mouse_normal_tracking = false;
        self.mouse_button_tracking = false;
        self.mouse_any_event_tracking = false;
        self.mouse_utf8_mode = false;
        self.mouse_sgr_mode = false;
        self.mouse_urxvt_mode = false;
        self.lnm_mode = false;
        self.auto_wrap_mode = true;
        self.insert_mode = false;
        self.origin_mode = false;

        // Reset character sets
        self.charset_g0 = CharacterSet::Ascii;
        self.charset_g1 = CharacterSet::Ascii;
        self.charset_use_g0 = true;

        // Clear response queue
        self.response_queue.clear();
    }

    /// Move cursor to next tab stop
    fn tab(&mut self) {
        for x in (self.cursor.x + 1)..self.cols {
            if self.tab_stops[x] {
                self.cursor.x = x;
                return;
            }
        }
        self.cursor.x = self.cols.saturating_sub(1);
    }

    /// Move cursor back one position
    fn backspace(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
        }
    }

    /// Scroll the scroll region up by n lines
    pub fn scroll_up(&mut self, n: usize) {
        for _ in 0..n {
            // Remove top line of scroll region
            if self.scroll_region_top < self.rows_count {
                let line = self.rows.remove(self.scroll_region_top);

                // Only add to scrollback if NOT in alternate screen
                if self.alt_screen.is_none() {
                    self.scrollback.push(line);

                    // Limit scrollback size
                    if self.scrollback.len() > self.max_scrollback {
                        self.scrollback.remove(0);
                    }
                }

                // Insert blank line at bottom of scroll region
                let insert_pos = self.scroll_region_bottom.min(self.rows_count - 1);
                self.rows
                    .insert(insert_pos, vec![TerminalCell::default(); self.cols]);
            }
        }
    }

    /// Scroll the scroll region down by n lines
    pub fn scroll_down(&mut self, n: usize) {
        for _ in 0..n {
            // Remove line at bottom of scroll region
            if self.scroll_region_bottom < self.rows_count {
                self.rows.remove(self.scroll_region_bottom);

                // Insert blank line at top of scroll region
                self.rows.insert(
                    self.scroll_region_top,
                    vec![TerminalCell::default(); self.cols],
                );
            }
        }
    }

    /// Clear the screen
    pub fn clear_screen(&mut self) {
        let bg = self.current_bg;
        for row in &mut self.rows {
            for cell in row {
                cell.c = ' ';
                cell.fg = Color::Named(NamedColor::White);
                cell.bg = bg;
                cell.attrs = CellAttributes::default();
            }
        }
    }

    /// Clear the current line
    pub fn clear_line(&mut self) {
        let bg = self.current_bg;
        if let Some(row) = self.rows.get_mut(self.cursor.y) {
            for cell in row {
                cell.c = ' ';
                cell.fg = Color::Named(NamedColor::White);
                cell.bg = bg;
                cell.attrs = CellAttributes::default();
            }
        }
    }

    /// Erase from cursor to end of line
    pub fn erase_to_eol(&mut self) {
        let bg = self.current_bg;
        if let Some(row) = self.rows.get_mut(self.cursor.y) {
            for x in self.cursor.x..self.cols {
                if let Some(cell) = row.get_mut(x) {
                    cell.c = ' ';
                    cell.fg = Color::Named(NamedColor::White);
                    cell.bg = bg;
                    cell.attrs = CellAttributes::default();
                }
            }
        }
    }

    /// Erase from beginning of line to cursor (inclusive)
    pub fn erase_to_bol(&mut self) {
        let bg = self.current_bg;
        if let Some(row) = self.rows.get_mut(self.cursor.y) {
            for x in 0..=self.cursor.x {
                if let Some(cell) = row.get_mut(x) {
                    cell.c = ' ';
                    cell.fg = Color::Named(NamedColor::White);
                    cell.bg = bg;
                    cell.attrs = CellAttributes::default();
                }
            }
        }
    }

    /// Erase from cursor to end of screen
    pub fn erase_to_eos(&mut self) {
        // Clear rest of current line
        self.erase_to_eol();

        // Clear all lines below
        let bg = self.current_bg;
        for y in (self.cursor.y + 1)..self.rows_count {
            if let Some(row) = self.rows.get_mut(y) {
                for cell in row {
                    cell.c = ' ';
                    cell.fg = Color::Named(NamedColor::White);
                    cell.bg = bg;
                    cell.attrs = CellAttributes::default();
                }
            }
        }
    }

    /// Erase from beginning of screen to cursor (inclusive)
    pub fn erase_from_bos(&mut self) {
        let bg = self.current_bg;

        // Clear all lines above cursor
        for y in 0..self.cursor.y {
            if let Some(row) = self.rows.get_mut(y) {
                for cell in row {
                    cell.c = ' ';
                    cell.fg = Color::Named(NamedColor::White);
                    cell.bg = bg;
                    cell.attrs = CellAttributes::default();
                }
            }
        }

        // Clear from beginning of current line to cursor (inclusive)
        self.erase_to_bol();
    }

    /// Delete n characters at cursor, shifting remaining characters left (DCH)
    pub fn delete_chars(&mut self, n: usize) {
        if let Some(row) = self.rows.get_mut(self.cursor.y) {
            let start = self.cursor.x;
            let end = self.cols;
            let n = n.min(end.saturating_sub(start));

            // Shift characters left
            for x in start..(end - n) {
                if let Some(src_cell) = row.get(x + n).cloned() {
                    if let Some(cell) = row.get_mut(x) {
                        *cell = src_cell;
                    }
                }
            }

            // Fill vacated positions with blanks
            let bg = self.current_bg;
            for x in (end - n)..end {
                if let Some(cell) = row.get_mut(x) {
                    cell.c = ' ';
                    cell.fg = Color::Named(NamedColor::White);
                    cell.bg = bg;
                    cell.attrs = CellAttributes::default();
                }
            }
        }
    }

    /// Insert n blank characters at cursor, shifting existing characters right (ICH)
    pub fn insert_chars(&mut self, n: usize) {
        if let Some(row) = self.rows.get_mut(self.cursor.y) {
            let start = self.cursor.x;
            let end = self.cols;
            let n = n.min(end.saturating_sub(start));

            // Shift characters right (from end to start to avoid overwriting)
            for x in (start + n..end).rev() {
                if let Some(src_cell) = row.get(x - n).cloned() {
                    if let Some(cell) = row.get_mut(x) {
                        *cell = src_cell;
                    }
                }
            }

            // Fill inserted positions with blanks
            let bg = self.current_bg;
            for x in start..(start + n).min(end) {
                if let Some(cell) = row.get_mut(x) {
                    cell.c = ' ';
                    cell.fg = Color::Named(NamedColor::White);
                    cell.bg = bg;
                    cell.attrs = CellAttributes::default();
                }
            }
        }
    }

    /// Erase n characters at cursor without moving cursor (ECH)
    pub fn erase_chars(&mut self, n: usize) {
        let bg = self.current_bg;
        if let Some(row) = self.rows.get_mut(self.cursor.y) {
            for x in self.cursor.x..(self.cursor.x + n).min(self.cols) {
                if let Some(cell) = row.get_mut(x) {
                    cell.c = ' ';
                    cell.fg = Color::Named(NamedColor::White);
                    cell.bg = bg;
                    cell.attrs = CellAttributes::default();
                }
            }
        }
    }

    /// Move cursor to absolute position (0-indexed)
    pub fn goto(&mut self, x: usize, y: usize) {
        self.cursor.x = x.min(self.cols.saturating_sub(1));
        self.cursor.y = y.min(self.rows_count.saturating_sub(1));
    }

    /// Move cursor with origin mode awareness (for CSI H and similar)
    /// When origin mode (DECOM) is set, positions are relative to scroll region
    /// and cursor is constrained to scroll region
    pub fn goto_origin_aware(&mut self, x: usize, y: usize) {
        if self.origin_mode {
            // In origin mode, positions are relative to scroll region
            let actual_y = (self.scroll_region_top + y).min(self.scroll_region_bottom);
            self.cursor.x = x.min(self.cols.saturating_sub(1));
            self.cursor.y = actual_y;
        } else {
            // Normal mode, absolute positioning
            self.goto(x, y);
        }
    }

    /// Move cursor relatively
    pub fn move_cursor(&mut self, dx: isize, dy: isize) {
        let new_x = (self.cursor.x as isize + dx).max(0) as usize;
        let new_y = (self.cursor.y as isize + dy).max(0) as usize;
        self.goto(new_x, new_y);
    }

    /// Set origin mode (DECOM - CSI ?6h)
    /// Per VT100 spec: When origin mode is set, cursor moves to home (top of scroll region)
    pub fn set_origin_mode(&mut self, enabled: bool) {
        self.origin_mode = enabled;
        // Per VT100 spec: cursor moves to home when origin mode changes
        if enabled {
            // In origin mode, home is top-left of scroll region
            self.cursor.x = 0;
            self.cursor.y = self.scroll_region_top;
        } else {
            // In normal mode, home is top-left of screen
            self.cursor.x = 0;
            self.cursor.y = 0;
        }
    }

    /// Save cursor position only (CSI s - SCP)
    pub fn save_cursor_position(&mut self) {
        self.saved_cursor = Some(SavedCursorState {
            cursor: self.cursor,
            attrs: self.current_attrs,
            fg: self.current_fg,
            bg: self.current_bg,
        });
    }

    /// Restore cursor position only (CSI u - RCP)
    pub fn restore_cursor_position(&mut self) {
        if let Some(saved) = self.saved_cursor {
            self.cursor = saved.cursor;
            // Don't restore colors/attrs for CSI u
        }
    }

    /// Save cursor position and attributes (DECSC - ESC 7)
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(SavedCursorState {
            cursor: self.cursor,
            attrs: self.current_attrs,
            fg: self.current_fg,
            bg: self.current_bg,
        });
    }

    /// Restore cursor position and attributes (DECRC - ESC 8)
    pub fn restore_cursor(&mut self) {
        if let Some(saved) = self.saved_cursor {
            self.cursor = saved.cursor;
            self.current_attrs = saved.attrs;
            self.current_fg = saved.fg;
            self.current_bg = saved.bg;
        }
    }

    /// Switch to alternate screen buffer
    pub fn use_alt_screen(&mut self) {
        if self.alt_screen.is_none() {
            // Save cursor position (part of DECSC/DECRC behavior with alt screen)
            self.save_cursor();
            // Save current screen
            self.alt_screen = Some(self.rows.clone());
            // Clear current screen
            self.clear_screen();
            // Reset cursor to home position
            self.cursor = Cursor::default();
        }
    }

    /// Switch back to main screen buffer
    pub fn use_main_screen(&mut self) {
        if let Some(main_screen) = self.alt_screen.take() {
            self.rows = main_screen;
            // Restore cursor position (part of DECSC/DECRC behavior with alt screen)
            self.restore_cursor();
        }
    }

    /// Set scroll region (DECSTBM)
    /// Per VT100/VT510 spec: After setting margins, cursor moves to home position
    /// In origin mode: home is top of scroll region; otherwise: top-left of screen
    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        // Validate: top must be less than bottom, minimum 2 lines
        let top = top.min(self.rows_count.saturating_sub(2));
        let bottom = bottom.min(self.rows_count.saturating_sub(1));

        if top < bottom {
            self.scroll_region_top = top;
            self.scroll_region_bottom = bottom;
        } else {
            // Invalid region, reset to full screen
            self.scroll_region_top = 0;
            self.scroll_region_bottom = self.rows_count.saturating_sub(1);
        }

        // Per VT510 spec: DECSTBM moves cursor to home position
        // In origin mode, home is column 1, line 1 of scroll region
        // In normal mode, home is column 1, line 1 of screen (0,0)
        if self.origin_mode {
            self.cursor.x = 0;
            self.cursor.y = self.scroll_region_top;
        } else {
            self.cursor.x = 0;
            self.cursor.y = 0;
        }
    }

    /// Reset scroll region to full screen
    #[allow(dead_code)]
    pub fn reset_scroll_region(&mut self) {
        self.scroll_region_top = 0;
        self.scroll_region_bottom = self.rows_count.saturating_sub(1);
    }

    /// Restore terminal content from saved lines (for session restoration)
    /// This replaces the scrollback and visible screen with the saved content
    pub fn restore_content(&mut self, lines: Vec<Vec<TerminalCell>>) {
        if lines.is_empty() {
            return;
        }

        // Clear current content
        self.scrollback.clear();
        self.rows.clear();

        // Split lines into scrollback and visible screen
        let total_lines = lines.len();
        if total_lines <= self.rows_count {
            // All lines fit in visible screen, no scrollback
            self.rows = lines;
            // Pad with empty lines if needed
            while self.rows.len() < self.rows_count {
                self.rows.push(vec![TerminalCell::default(); self.cols]);
            }
        } else {
            // Some lines go into scrollback
            let scrollback_count = total_lines - self.rows_count;
            self.scrollback = lines[..scrollback_count].to_vec();
            self.rows = lines[scrollback_count..].to_vec();
        }

        // Ensure rows match the current column count
        for row in &mut self.rows {
            row.resize(self.cols, TerminalCell::default());
        }
        for row in &mut self.scrollback {
            row.resize(self.cols, TerminalCell::default());
        }
    }

    /// Set cursor position (for session restoration)
    pub fn set_cursor(&mut self, x: usize, y: usize, visible: bool) {
        self.cursor.x = x.min(self.cols.saturating_sub(1));
        self.cursor.y = y.min(self.rows_count.saturating_sub(1));
        self.cursor.visible = visible;
    }
}

impl fmt::Debug for TerminalGrid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalGrid")
            .field("cols", &self.cols)
            .field("rows", &self.rows_count)
            .field("scrollback_lines", &self.scrollback.len())
            .field("cursor", &self.cursor)
            .finish()
    }
}
