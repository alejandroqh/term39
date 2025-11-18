use crate::charset::{Charset, CharsetMode};
use crate::selection::{Position, Selection, SelectionType};
use crate::term_grid::{Color as TermColor, NamedColor, TerminalCell};
use crate::terminal_emulator::TerminalEmulator;
use crate::theme::Theme;
use crate::video_buffer::{Cell, VideoBuffer};
use crate::window::Window;
use crossterm::style::Color;

/// A window containing a terminal emulator
pub struct TerminalWindow {
    pub window: Window,
    emulator: TerminalEmulator,
    scroll_offset: usize,         // For scrollback navigation
    selection: Option<Selection>, // Current text selection
}

impl TerminalWindow {
    /// Create a new terminal window
    pub fn new(
        id: u32,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        title: String,
    ) -> std::io::Result<Self> {
        // Calculate content area (excluding 2-char borders and title bar)
        let content_width = width.saturating_sub(4).max(1); // -2 left, -2 right
        let content_height = height.saturating_sub(2).max(1); // -1 title, -1 bottom

        let window = Window::new(id, x, y, width, height, title);
        let emulator = TerminalEmulator::new(
            content_width as usize,
            content_height as usize,
            1000, // 1000 lines of scrollback
        )?;

        Ok(Self {
            window,
            emulator,
            scroll_offset: 0,
            selection: None,
        })
    }

    /// Process terminal output (call this regularly in the event loop)
    pub fn process_output(&mut self) -> std::io::Result<bool> {
        self.emulator.process_output()
    }

    /// Send input to the terminal
    pub fn send_str(&mut self, s: &str) -> std::io::Result<()> {
        self.emulator.send_str(s)
    }

    /// Send a character to the terminal
    pub fn send_char(&mut self, c: char) -> std::io::Result<()> {
        self.emulator.send_char(c)
    }

    /// Resize the window (also resizes the terminal)
    pub fn resize(&mut self, new_width: u16, new_height: u16) -> std::io::Result<()> {
        self.window.width = new_width;
        self.window.height = new_height;

        // Calculate new content dimensions (accounting for 2-char borders)
        let content_width = new_width.saturating_sub(4).max(1); // -2 left, -2 right
        let content_height = new_height.saturating_sub(2).max(1); // -1 title, -1 bottom

        self.emulator
            .resize(content_width as usize, content_height as usize)
    }

    /// Scroll up in the scrollback buffer
    #[allow(dead_code)]
    pub fn scroll_up(&mut self, lines: usize) {
        let grid = self.emulator.grid();
        let grid = grid.lock().unwrap();
        let max_offset = grid.scrollback_len();

        self.scroll_offset = (self.scroll_offset + lines).min(max_offset);
    }

    /// Scroll down in the scrollback buffer
    #[allow(dead_code)]
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Reset scroll to bottom (showing current output)
    #[allow(dead_code)]
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    /// Render the terminal window
    pub fn render(
        &self,
        buffer: &mut VideoBuffer,
        charset: &Charset,
        theme: &Theme,
        tint_terminal: bool,
    ) {
        // Render the window frame and title bar
        self.window.render(buffer, charset, theme);

        // Render the terminal content
        self.render_terminal_content(buffer, theme, tint_terminal);

        // Render the scrollbar
        self.render_scrollbar(buffer, charset, theme);
    }

    fn render_terminal_content(
        &self,
        buffer: &mut VideoBuffer,
        theme: &Theme,
        tint_terminal: bool,
    ) {
        if self.window.is_minimized {
            return;
        }

        let grid = self.emulator.grid();
        let grid = grid.lock().unwrap();

        // Content area starts after 2-char left border and title bar
        let content_x = self.window.x + 2; // After 2-char left border
        let content_y = self.window.y + 1; // After title bar
        let content_width = self.window.width.saturating_sub(4); // -2 left, -2 right
        let content_height = self.window.height.saturating_sub(2); // -1 title, -1 bottom

        let scrollback_len = grid.scrollback_len();
        let visible_rows = grid.rows();

        // Render terminal grid cells
        for row in 0..content_height {
            for col in 0..content_width {
                let grid_col = col as usize;
                let row_idx = row as usize;

                // Calculate which line to display based on scroll offset
                let term_cell = if self.scroll_offset > 0 {
                    // We're scrolled back, need to fetch from scrollback or visible rows
                    let total_lines = scrollback_len + visible_rows;
                    let line_idx =
                        total_lines.saturating_sub(self.scroll_offset + visible_rows) + row_idx;

                    if line_idx < scrollback_len {
                        // Fetch from scrollback
                        grid.get_scrollback_line(line_idx)
                            .and_then(|line| line.get(grid_col))
                    } else {
                        // Fetch from visible rows
                        let visible_row = line_idx - scrollback_len;
                        grid.get_cell(grid_col, visible_row)
                    }
                } else {
                    // Not scrolled, show current visible rows
                    grid.get_cell(grid_col, row_idx)
                };

                // Render the cell
                if let Some(term_cell) = term_cell {
                    let mut cell = convert_terminal_cell(term_cell, theme, tint_terminal);

                    // Apply selection highlighting if this cell is selected
                    if let Some(selection) = &self.selection {
                        let pos = Position::new(col, row);
                        if selection.contains(pos) {
                            // Invert colors for DOS-style selection
                            cell = cell.inverted();
                        }
                    }

                    buffer.set(content_x + col, content_y + row, cell);
                }
            }
        }

        // Render cursor if visible and not scrolled
        if grid.cursor.visible && self.scroll_offset == 0 {
            let cursor_x = content_x + grid.cursor.x as u16;
            let cursor_y = content_y + grid.cursor.y as u16;

            // Check if cursor is within window bounds
            if cursor_x < content_x + content_width && cursor_y < content_y + content_height {
                // Get the current cell at cursor position
                if let Some(current_cell) = buffer.get(cursor_x, cursor_y) {
                    // Invert colors for cursor
                    let cursor_cell = Cell::new(
                        current_cell.character,
                        current_cell.bg_color,
                        current_cell.fg_color,
                    );
                    buffer.set(cursor_x, cursor_y, cursor_cell);
                }
            }
        }
    }

    fn render_scrollbar(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        if self.window.is_minimized {
            return;
        }

        let grid = self.emulator.grid();
        let grid = grid.lock().unwrap();

        let scrollback_len = grid.scrollback_len();

        // Only show scrollbar if there's scrollback content
        if scrollback_len == 0 {
            return;
        }

        let scrollbar_x = self.window.x + self.window.width - 2; // Inner char of 2-char right border
        let (track_start, track_end) = self.get_scrollbar_bounds();

        // Calculate thumb bounds inline to avoid re-locking the grid
        let visible_rows = grid.rows();
        let total_lines = scrollback_len + visible_rows;
        let (thumb_start, thumb_end) = if total_lines <= visible_rows {
            (0, 0)
        } else {
            let track_height = (track_end - track_start) as usize;
            let thumb_size = ((visible_rows as f64 / total_lines as f64) * track_height as f64)
                .max(1.0) as usize;
            let max_scroll = total_lines.saturating_sub(visible_rows);
            // Invert the scroll ratio so thumb is at bottom when at current output (scroll_offset=0)
            let scroll_ratio = if max_scroll > 0 {
                (max_scroll - self.scroll_offset) as f64 / max_scroll as f64
            } else {
                1.0
            };
            let thumb_offset = (scroll_ratio * (track_height - thumb_size) as f64) as usize;
            let thumb_start = track_start + thumb_offset as u16;
            let thumb_end = thumb_start + thumb_size as u16;
            (thumb_start, thumb_end)
        };

        // Choose characters based on charset mode
        let track_char = match charset.mode {
            CharsetMode::Unicode => '║',
            CharsetMode::Ascii => '|',
        };
        let thumb_char = match charset.mode {
            CharsetMode::Unicode => '█',
            CharsetMode::Ascii => '#',
        };

        // Render the scrollbar track and thumb
        for y in track_start..track_end {
            let (ch, fg_color) = if y >= thumb_start && y < thumb_end {
                // Scrollbar thumb
                (thumb_char, theme.scrollbar_thumb_fg)
            } else {
                // Scrollbar track
                (track_char, theme.scrollbar_track_fg)
            };

            let cell = Cell::new(ch, fg_color, theme.window_content_bg);
            buffer.set(scrollbar_x, y, cell);
        }
    }

    /// Get the window's ID
    pub fn id(&self) -> u32 {
        self.window.id
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.window.is_focused = focused;
    }

    /// Check if a point is within the window
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        if self.window.is_minimized {
            return false;
        }
        self.window.contains_point(x, y)
    }

    /// Check if point is in title bar
    pub fn is_in_title_bar(&self, x: u16, y: u16) -> bool {
        if self.window.is_minimized {
            return false;
        }
        self.window.is_in_title_bar(x, y)
    }

    /// Check if point is in close button
    pub fn is_in_close_button(&self, x: u16, y: u16) -> bool {
        if self.window.is_minimized {
            return false;
        }
        self.window.is_in_close_button(x, y)
    }

    /// Get total number of lines (scrollback + visible)
    #[allow(dead_code)]
    pub fn get_total_lines(&self) -> usize {
        let grid = self.emulator.grid();
        let grid = grid.lock().unwrap();
        grid.scrollback_len() + grid.rows()
    }

    /// Get the bounds of the scrollbar track (y_start, y_end)
    pub fn get_scrollbar_bounds(&self) -> (u16, u16) {
        let y_start = self.window.y + 1; // After title bar
        let y_end = self.window.y + self.window.height - 1; // Before bottom border
        (y_start, y_end)
    }

    /// Get the bounds of the scrollbar thumb (y_start, y_end)
    pub fn get_scrollbar_thumb_bounds(&self) -> (u16, u16) {
        let grid = self.emulator.grid();
        let grid = grid.lock().unwrap();

        let scrollback_len = grid.scrollback_len();
        let visible_rows = grid.rows();
        let total_lines = scrollback_len + visible_rows;

        if total_lines <= visible_rows {
            // No scrollbar needed
            return (0, 0);
        }

        let (track_start, track_end) = self.get_scrollbar_bounds();
        let track_height = (track_end - track_start) as usize;

        // Calculate thumb size (proportional to visible area)
        let thumb_size =
            ((visible_rows as f64 / total_lines as f64) * track_height as f64).max(1.0) as usize;

        // Calculate thumb position based on scroll offset
        // Invert the scroll ratio so thumb is at bottom when at current output (scroll_offset=0)
        let max_scroll = total_lines.saturating_sub(visible_rows);
        let scroll_ratio = if max_scroll > 0 {
            (max_scroll - self.scroll_offset) as f64 / max_scroll as f64
        } else {
            1.0
        };

        let thumb_offset = (scroll_ratio * (track_height - thumb_size) as f64) as usize;
        let thumb_start = track_start + thumb_offset as u16;
        let thumb_end = thumb_start + thumb_size as u16;

        (thumb_start, thumb_end)
    }

    /// Check if a point is on the scrollbar
    pub fn is_point_on_scrollbar(&self, x: u16, y: u16) -> bool {
        if self.window.is_minimized {
            return false;
        }
        let scrollbar_x = self.window.x + self.window.width - 2; // Inner char of 2-char right border
        let (y_start, y_end) = self.get_scrollbar_bounds();

        x == scrollbar_x && y >= y_start && y < y_end
    }

    /// Check if a point is on the scrollbar thumb
    pub fn is_point_on_scrollbar_thumb(&self, x: u16, y: u16) -> bool {
        if self.window.is_minimized {
            return false;
        }
        if !self.is_point_on_scrollbar(x, y) {
            return false;
        }

        let (thumb_start, thumb_end) = self.get_scrollbar_thumb_bounds();
        y >= thumb_start && y < thumb_end
    }

    /// Scroll to a specific offset based on mouse position on scrollbar
    pub fn scroll_to_position(&mut self, y: u16) {
        let (track_start, track_end) = self.get_scrollbar_bounds();
        let track_height = (track_end - track_start) as usize;

        if track_height == 0 {
            return;
        }

        let grid = self.emulator.grid();
        let grid = grid.lock().unwrap();
        let total_lines = grid.scrollback_len() + grid.rows();
        let visible_rows = grid.rows();
        let max_scroll = total_lines.saturating_sub(visible_rows);

        // Calculate position ratio (inverted: top = old content, bottom = current)
        let click_offset = y.saturating_sub(track_start) as usize;
        let ratio = click_offset as f64 / track_height as f64;

        // Invert the ratio so clicking at bottom shows current output (scroll_offset=0)
        self.scroll_offset = ((1.0 - ratio) * max_scroll as f64) as usize;
        self.scroll_offset = self.scroll_offset.min(max_scroll);
    }

    /// Get the current scroll offset
    pub fn get_scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Convert screen coordinates to terminal grid position
    fn screen_to_grid_pos(&self, screen_x: u16, screen_y: u16) -> Option<Position> {
        let content_x = self.window.x + 2; // After 2-char left border
        let content_y = self.window.y + 1; // After title bar
        let content_width = self.window.width.saturating_sub(4); // -2 left, -2 right
        let content_height = self.window.height.saturating_sub(2); // -1 title, -1 bottom

        // Check if coordinates are within content area
        if screen_x < content_x
            || screen_x >= content_x + content_width
            || screen_y < content_y
            || screen_y >= content_y + content_height
        {
            return None;
        }

        let col = screen_x - content_x;
        let row = screen_y - content_y;

        Some(Position::new(col, row))
    }

    /// Start a new selection
    pub fn start_selection(&mut self, screen_x: u16, screen_y: u16, selection_type: SelectionType) {
        if let Some(pos) = self.screen_to_grid_pos(screen_x, screen_y) {
            self.selection = Some(Selection::new(pos, selection_type));
        }
    }

    /// Update selection end position
    pub fn update_selection(&mut self, screen_x: u16, screen_y: u16) {
        if let Some(pos) = self.screen_to_grid_pos(screen_x, screen_y) {
            if let Some(selection) = &mut self.selection {
                selection.update_end(pos);
            }
        }
    }

    /// Complete the selection
    pub fn complete_selection(&mut self) {
        if let Some(selection) = &mut self.selection {
            selection.complete();
        }
    }

    /// Clear the selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Expand selection to word boundaries
    pub fn expand_selection_to_word(&mut self) {
        if let Some(selection) = &mut self.selection {
            let grid = self.emulator.grid();
            let grid = grid.lock().unwrap();

            selection.expand_to_word(|pos| {
                grid.get_cell(pos.col as usize, pos.row as usize)
                    .map(|cell| cell.c)
            });
        }
    }

    /// Expand selection to line
    pub fn expand_selection_to_line(&mut self) {
        if let Some(selection) = &mut self.selection {
            let content_width = self.window.width.saturating_sub(4); // -2 left, -2 right
            selection.expand_to_line(content_width);
        }
    }

    /// Get selected text
    pub fn get_selected_text(&self) -> Option<String> {
        let selection = self.selection.as_ref()?;
        if selection.is_empty() {
            return None;
        }

        let grid = self.emulator.grid();
        let grid = grid.lock().unwrap();

        let (start, end) = selection.normalized_bounds();

        let mut result = String::new();

        match selection.selection_type {
            SelectionType::Block => {
                // Rectangle selection
                let min_col = start.col.min(end.col);
                let max_col = start.col.max(end.col);
                let min_row = start.row.min(end.row);
                let max_row = start.row.max(end.row);

                for row in min_row..=max_row {
                    for col in min_col..=max_col {
                        if let Some(cell) = grid.get_cell(col as usize, row as usize) {
                            result.push(cell.c);
                        }
                    }
                    if row < max_row {
                        result.push('\n');
                    }
                }
            }
            _ => {
                // Linear selection (character, word, line)
                if start.row == end.row {
                    // Single line
                    for col in start.col..=end.col {
                        if let Some(cell) = grid.get_cell(col as usize, start.row as usize) {
                            result.push(cell.c);
                        }
                    }
                } else {
                    // Multiple lines
                    // First line (from start.col to end of line)
                    let content_width = self.window.width.saturating_sub(4); // -2 left, -2 right
                    for col in start.col..content_width {
                        if let Some(cell) = grid.get_cell(col as usize, start.row as usize) {
                            result.push(cell.c);
                        }
                    }
                    result.push('\n');

                    // Middle lines (full lines)
                    for row in (start.row + 1)..end.row {
                        for col in 0..content_width {
                            if let Some(cell) = grid.get_cell(col as usize, row as usize) {
                                result.push(cell.c);
                            }
                        }
                        result.push('\n');
                    }

                    // Last line (from start to end.col)
                    for col in 0..=end.col {
                        if let Some(cell) = grid.get_cell(col as usize, end.row as usize) {
                            result.push(cell.c);
                        }
                    }
                }
            }
        }

        // Clean up trailing spaces and return
        let result = result.trim_end().to_string();
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Paste text to terminal
    pub fn paste_text(&mut self, text: &str) -> std::io::Result<()> {
        self.emulator.send_str(text)
    }

    /// Check if there's an active selection
    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    /// Get the content area bounds (for hit testing)
    #[allow(dead_code)]
    pub fn get_content_bounds(&self) -> (u16, u16, u16, u16) {
        let content_x = self.window.x + 2; // After 2-char left border
        let content_y = self.window.y + 1; // After title bar
        let content_width = self.window.width.saturating_sub(4); // -2 left, -2 right
        let content_height = self.window.height.saturating_sub(2); // -1 title, -1 bottom
        (content_x, content_y, content_width, content_height)
    }

    /// Set scroll offset (for session restoration)
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Extract terminal content for session persistence
    pub fn get_terminal_content(
        &self,
    ) -> (
        Vec<crate::session::SerializableTerminalLine>,
        crate::session::SerializableCursor,
    ) {
        self.emulator.get_terminal_content()
    }

    /// Restore terminal content from session
    pub fn restore_terminal_content(
        &mut self,
        lines: Vec<crate::session::SerializableTerminalLine>,
        cursor: &crate::session::SerializableCursor,
    ) {
        self.emulator.restore_terminal_content(lines, cursor);
    }
}

/// Convert a terminal cell to a video buffer cell
fn convert_terminal_cell(term_cell: &TerminalCell, theme: &Theme, tint_terminal: bool) -> Cell {
    let mut fg = convert_color(&term_cell.fg);
    let mut bg = convert_color(&term_cell.bg);

    // Apply theme-based tinting if enabled
    if tint_terminal {
        fg = apply_theme_tint(fg, theme, true);
        bg = apply_theme_tint(bg, theme, false);
    }

    Cell::new(term_cell.c, fg, bg)
}

/// Convert terminal color to crossterm color
fn convert_color(color: &TermColor) -> Color {
    match color {
        TermColor::Named(named) => match named {
            NamedColor::Black => Color::Black,
            NamedColor::Red => Color::DarkRed,
            NamedColor::Green => Color::DarkGreen,
            NamedColor::Yellow => Color::DarkYellow,
            NamedColor::Blue => Color::DarkBlue,
            NamedColor::Magenta => Color::DarkMagenta,
            NamedColor::Cyan => Color::DarkCyan,
            NamedColor::White => Color::Grey,
            NamedColor::BrightBlack => Color::DarkGrey,
            NamedColor::BrightRed => Color::Red,
            NamedColor::BrightGreen => Color::Green,
            NamedColor::BrightYellow => Color::Yellow,
            NamedColor::BrightBlue => Color::Blue,
            NamedColor::BrightMagenta => Color::Magenta,
            NamedColor::BrightCyan => Color::Cyan,
            NamedColor::BrightWhite => Color::White,
        },
        TermColor::Indexed(idx) => Color::AnsiValue(*idx),
        TermColor::Rgb(r, g, b) => Color::Rgb {
            r: *r,
            g: *g,
            b: *b,
        },
    }
}

/// Apply theme-based color tinting to terminal colors
fn apply_theme_tint(color: Color, theme: &Theme, is_foreground: bool) -> Color {
    // Map terminal colors to theme colors
    match color {
        // Background colors - map to theme background
        Color::Black | Color::DarkGrey if !is_foreground => theme.window_content_bg,

        // Foreground colors - map to theme foreground variations
        Color::Black | Color::DarkGrey if is_foreground => {
            // Dark colors map to a darker version of foreground
            darken_color(theme.window_content_fg, 0.6)
        }

        // Bright colors - keep as foreground
        Color::White | Color::Grey => theme.window_content_fg,

        // Red colors - map to theme button close or a red-ish tint
        Color::Red | Color::DarkRed => theme.button_close_color,

        // Green colors - map to theme button maximize or a green-ish tint
        Color::Green | Color::DarkGreen => theme.button_maximize_color,

        // Yellow colors - improve contrast for monochrome theme
        Color::Yellow | Color::DarkYellow => {
            // Monochrome uses DarkGrey which has poor contrast on Black
            // Use Grey instead for better visibility
            match theme.button_minimize_color {
                Color::DarkGrey => Color::Grey,
                _ => theme.button_minimize_color,
            }
        }

        // Blue colors - use border color for visibility (avoid mapping to Black)
        // This ensures blue text is visible across all themes
        Color::Blue | Color::DarkBlue => theme.window_border,

        // Cyan colors - map to content foreground for differentiation from blue
        Color::Cyan | Color::DarkCyan => theme.window_content_fg,

        // Magenta colors - map to a magenta-ish variation
        Color::Magenta | Color::DarkMagenta => theme.resize_handle_active_fg,

        // RGB colors - apply a theme-based tint transformation
        Color::Rgb { r, g, b } => {
            if is_foreground {
                // For foreground, blend with theme foreground color
                blend_with_theme_color(Color::Rgb { r, g, b }, theme.window_content_fg, 0.7)
            } else {
                // For background, blend with theme background color
                blend_with_theme_color(Color::Rgb { r, g, b }, theme.window_content_bg, 0.7)
            }
        }

        // Indexed colors (256-color palette)
        Color::AnsiValue(idx) => {
            // Map 256-color palette to theme colors based on brightness
            if idx < 8 {
                // Standard colors (0-7): map to theme colors
                match idx {
                    0 => theme.window_content_bg,     // Black
                    1 => theme.button_close_color,    // Red
                    2 => theme.button_maximize_color, // Green
                    3 => {
                        // Yellow - improve contrast for monochrome theme
                        match theme.button_minimize_color {
                            Color::DarkGrey => Color::Grey,
                            _ => theme.button_minimize_color,
                        }
                    }
                    4 => theme.window_border, // Blue - use border for visibility
                    5 => theme.resize_handle_active_fg, // Magenta
                    6 => theme.window_content_fg, // Cyan - differentiate from blue
                    7 => theme.window_content_fg, // White
                    _ => color,
                }
            } else if idx < 16 {
                // Bright colors (8-15): map to brighter theme variations
                theme.window_content_fg
            } else {
                // Extended colors: blend with theme
                if is_foreground {
                    theme.window_content_fg
                } else {
                    theme.window_content_bg
                }
            }
        }

        _ => color,
    }
}

/// Darken a color by a factor (0.0 = black, 1.0 = original)
fn darken_color(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb { r, g, b } => Color::Rgb {
            r: (r as f32 * factor) as u8,
            g: (g as f32 * factor) as u8,
            b: (b as f32 * factor) as u8,
        },
        _ => color,
    }
}

/// Blend a color with a theme color
fn blend_with_theme_color(original: Color, theme_color: Color, blend_factor: f32) -> Color {
    match (original, theme_color) {
        (
            Color::Rgb {
                r: r1,
                g: g1,
                b: b1,
            },
            Color::Rgb {
                r: r2,
                g: g2,
                b: b2,
            },
        ) => Color::Rgb {
            r: (r1 as f32 * blend_factor + r2 as f32 * (1.0 - blend_factor)) as u8,
            g: (g1 as f32 * blend_factor + g2 as f32 * (1.0 - blend_factor)) as u8,
            b: (b1 as f32 * blend_factor + b2 as f32 * (1.0 - blend_factor)) as u8,
        },
        _ => original,
    }
}
