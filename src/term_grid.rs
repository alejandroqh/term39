use std::fmt;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    Block,
    Underline,
    Bar,
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
    /// Saved cursor position (for save/restore)
    saved_cursor: Option<Cursor>,
    /// Alternate screen buffer
    alt_screen: Option<Vec<Vec<TerminalCell>>>,
    /// Tab stops (every 8 columns by default)
    tab_stops: Vec<bool>,
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
            current_fg: Color::Named(NamedColor::White),
            current_bg: Color::Named(NamedColor::Black),
            scroll_region_top: 0,
            scroll_region_bottom: rows.saturating_sub(1),
            saved_cursor: None,
            alt_screen: None,
            tab_stops,
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows_count
    }

    #[allow(dead_code)]
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
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
    pub fn get_cell(&self, x: usize, y: usize) -> Option<&TerminalCell> {
        self.rows.get(y)?.get(x)
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
                // Write character at cursor position
                if self.cursor.x < self.cols {
                    // Copy values before mutable borrow
                    let fg = self.current_fg;
                    let bg = self.current_bg;
                    let attrs = self.current_attrs;

                    if let Some(cell) = self.get_cell_mut(self.cursor.x, self.cursor.y) {
                        cell.c = c;
                        cell.fg = fg;
                        cell.bg = bg;
                        cell.attrs = attrs;
                    }
                    self.cursor.x += 1;

                    // Auto-wrap at end of line
                    if self.cursor.x >= self.cols {
                        self.cursor.x = 0;
                        self.linefeed();
                    }
                }
            }
        }
    }

    /// Move cursor to the next line, scrolling if necessary
    fn linefeed(&mut self) {
        // Special case: if cursor is at row 0 and column 0, and row 0 is empty,
        // don't move down. This prevents the shell init newline from creating a blank line.
        if self.cursor.y == 0 && self.cursor.x == 0 {
            // Check if row 0 is empty (all spaces)
            if let Some(row) = self.rows.first() {
                let is_empty = row.iter().all(|cell| cell.c == ' ');
                if is_empty {
                    // Don't move cursor, just ignore this linefeed
                    return;
                }
            }
        }

        if self.cursor.y == self.scroll_region_bottom {
            // At bottom of scroll region, scroll up
            self.scroll_up(1);
        } else if self.cursor.y < self.rows_count - 1 {
            self.cursor.y += 1;
        }
    }

    /// Move cursor to start of line
    fn carriage_return(&mut self) {
        self.cursor.x = 0;
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
            // Remove top line of scroll region and add to scrollback
            if self.scroll_region_top < self.rows_count {
                let line = self.rows.remove(self.scroll_region_top);
                self.scrollback.push(line);

                // Limit scrollback size
                if self.scrollback.len() > self.max_scrollback {
                    self.scrollback.remove(0);
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
        for row in &mut self.rows {
            for cell in row {
                cell.reset();
            }
        }
    }

    /// Clear the current line
    pub fn clear_line(&mut self) {
        if let Some(row) = self.rows.get_mut(self.cursor.y) {
            for cell in row {
                cell.reset();
            }
        }
    }

    /// Erase from cursor to end of line
    pub fn erase_to_eol(&mut self) {
        if let Some(row) = self.rows.get_mut(self.cursor.y) {
            for x in self.cursor.x..self.cols {
                if let Some(cell) = row.get_mut(x) {
                    cell.reset();
                }
            }
        }
    }

    /// Erase from cursor to end of screen
    pub fn erase_to_eos(&mut self) {
        // Clear rest of current line
        self.erase_to_eol();

        // Clear all lines below
        for y in (self.cursor.y + 1)..self.rows_count {
            if let Some(row) = self.rows.get_mut(y) {
                for cell in row {
                    cell.reset();
                }
            }
        }
    }

    /// Move cursor to absolute position (0-indexed)
    pub fn goto(&mut self, x: usize, y: usize) {
        self.cursor.x = x.min(self.cols.saturating_sub(1));
        self.cursor.y = y.min(self.rows_count.saturating_sub(1));
    }

    /// Move cursor relatively
    pub fn move_cursor(&mut self, dx: isize, dy: isize) {
        let new_x = (self.cursor.x as isize + dx).max(0) as usize;
        let new_y = (self.cursor.y as isize + dy).max(0) as usize;
        self.goto(new_x, new_y);
    }

    /// Save cursor position
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor);
    }

    /// Restore cursor position
    pub fn restore_cursor(&mut self) {
        if let Some(saved) = self.saved_cursor {
            self.cursor = saved;
        }
    }

    /// Switch to alternate screen buffer
    pub fn use_alt_screen(&mut self) {
        if self.alt_screen.is_none() {
            // Save current screen
            self.alt_screen = Some(self.rows.clone());
            // Clear current screen
            self.clear_screen();
            self.cursor = Cursor::default();
        }
    }

    /// Switch back to main screen buffer
    pub fn use_main_screen(&mut self) {
        if let Some(main_screen) = self.alt_screen.take() {
            self.rows = main_screen;
        }
    }

    /// Set scroll region
    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        self.scroll_region_top = top.min(self.rows_count.saturating_sub(1));
        self.scroll_region_bottom = bottom.min(self.rows_count.saturating_sub(1));
    }

    /// Reset scroll region to full screen
    #[allow(dead_code)]
    pub fn reset_scroll_region(&mut self) {
        self.scroll_region_top = 0;
        self.scroll_region_bottom = self.rows_count.saturating_sub(1);
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
