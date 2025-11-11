use crate::charset::{Charset, CharsetMode};
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
    scroll_offset: usize, // For scrollback navigation
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
        // Calculate content area (excluding borders and title bar)
        let content_width = width.saturating_sub(2).max(1);
        let content_height = height.saturating_sub(2).max(1);

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

        // Calculate new content dimensions
        let content_width = new_width.saturating_sub(2).max(1);
        let content_height = new_height.saturating_sub(2).max(1);

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
        is_resizing: bool,
        charset: &Charset,
        theme: &Theme,
    ) {
        // Render the window frame and title bar
        self.window.render(buffer, is_resizing, charset, theme);

        // Render the terminal content
        self.render_terminal_content(buffer);

        // Render the scrollbar
        self.render_scrollbar(buffer, charset, theme);
    }

    fn render_terminal_content(&self, buffer: &mut VideoBuffer) {
        if self.window.is_minimized {
            return;
        }

        let grid = self.emulator.grid();
        let grid = grid.lock().unwrap();

        // Content area starts at (window.x + 1, window.y + 1)
        let content_x = self.window.x + 1;
        let content_y = self.window.y + 1;
        let content_width = self.window.width.saturating_sub(2);
        let content_height = self.window.height.saturating_sub(2);

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
                    let cell = convert_terminal_cell(term_cell);
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

        let scrollbar_x = self.window.x + self.window.width - 1;
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

    /// Check if point is in resize handle
    pub fn is_in_resize_handle(&self, x: u16, y: u16) -> bool {
        if self.window.is_minimized {
            return false;
        }
        self.window.is_in_resize_handle(x, y)
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
        let scrollbar_x = self.window.x + self.window.width - 1;
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
}

/// Convert a terminal cell to a video buffer cell
fn convert_terminal_cell(term_cell: &TerminalCell) -> Cell {
    let fg = convert_color(&term_cell.fg);
    let bg = convert_color(&term_cell.bg);

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
