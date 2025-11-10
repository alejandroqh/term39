use crate::charset::Charset;
use crate::term_grid::{Color as TermColor, NamedColor, TerminalCell};
use crate::terminal_emulator::TerminalEmulator;
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

        self.emulator.resize(content_width as usize, content_height as usize)
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
    pub fn render(&self, buffer: &mut VideoBuffer, is_resizing: bool, charset: &Charset) {
        // Render the window frame and title bar
        self.window.render(buffer, is_resizing, charset);

        // Render the terminal content
        self.render_terminal_content(buffer);
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

        // Render terminal grid cells
        for row in 0..content_height {
            for col in 0..content_width {
                let grid_row = row as usize;
                let grid_col = col as usize;

                // Get the cell from the terminal grid
                if let Some(term_cell) = grid.get_cell(grid_col, grid_row) {
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
        self.window.contains_point(x, y)
    }

    /// Check if point is in title bar
    pub fn is_in_title_bar(&self, x: u16, y: u16) -> bool {
        self.window.is_in_title_bar(x, y)
    }

    /// Check if point is in close button
    pub fn is_in_close_button(&self, x: u16, y: u16) -> bool {
        self.window.is_in_close_button(x, y)
    }

    /// Check if point is in resize handle
    pub fn is_in_resize_handle(&self, x: u16, y: u16) -> bool {
        self.window.is_in_resize_handle(x, y)
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
