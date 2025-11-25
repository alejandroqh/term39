use crate::charset::{Charset, CharsetMode};
use crate::selection::{Position, Selection, SelectionType};
use crate::term_grid::{Color as TermColor, NamedColor, TerminalCell};
use crate::terminal_emulator::TerminalEmulator;
use crate::theme::Theme;
use crate::video_buffer::{self, Cell, VideoBuffer};
use crate::window::Window;
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::time::Instant;

/// Close confirmation dialog for a terminal window
#[derive(Clone, Debug)]
pub(crate) struct CloseConfirmation {
    selected_button: usize, // 0 = Cancel, 1 = Close
}

impl CloseConfirmation {
    fn new() -> Self {
        Self {
            selected_button: 0, // Default to Cancel (safe choice)
        }
    }
}

/// A window containing a terminal emulator
pub struct TerminalWindow {
    pub window: Window,
    emulator: TerminalEmulator,
    scroll_offset: usize,         // For scrollback navigation
    selection: Option<Selection>, // Current text selection
    // Cached foreground process name to avoid spawning ps every frame
    cached_process_name: Option<String>,
    process_name_last_update: Instant,
    // Close confirmation state
    pub(crate) pending_close_confirmation: Option<CloseConfirmation>,
    // Track user input (for dirty state detection)
    created_at: Instant,
    has_user_input: bool,
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
        initial_command: Option<String>,
    ) -> std::io::Result<Self> {
        // Calculate content area (excluding 2-char borders and title bar)
        let content_width = width.saturating_sub(4).max(1); // -2 left, -2 right
        let content_height = height.saturating_sub(2).max(1); // -1 title, -1 bottom

        let window = Window::new(id, x, y, width, height, title);

        // Parse initial_command into program + args for direct execution
        let parsed_command = initial_command.as_ref().map(|cmd| Self::parse_command(cmd));

        let emulator = TerminalEmulator::new(
            content_width as usize,
            content_height as usize,
            1000, // 1000 lines of scrollback
            parsed_command,
        )?;

        Ok(Self {
            window,
            emulator,
            scroll_offset: 0,
            selection: None,
            cached_process_name: None,
            process_name_last_update: Instant::now(),
            pending_close_confirmation: None,
            created_at: Instant::now(),
            has_user_input: false,
        })
    }

    /// Parse a command string into (program, args)
    /// Simple shell-like parsing: splits on whitespace, respects quotes
    fn parse_command(cmd: &str) -> (String, Vec<String>) {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for ch in cmd.chars() {
            match ch {
                '"' | '\'' => {
                    in_quotes = !in_quotes;
                }
                ' ' | '\t' if !in_quotes => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        if parts.is_empty() {
            // Empty command, return a safe default
            ("sh".to_string(), vec![])
        } else {
            let program = parts[0].clone();
            let args = parts.into_iter().skip(1).collect();
            (program, args)
        }
    }

    /// Process terminal output (call this regularly in the event loop)
    pub fn process_output(&mut self) -> std::io::Result<bool> {
        self.emulator.process_output()
    }

    /// Send input to the terminal
    pub fn send_str(&mut self, s: &str) -> std::io::Result<()> {
        // Only track user input after initial shell setup (1 second grace period)
        if self.created_at.elapsed().as_secs() >= 1 {
            self.has_user_input = true;
        }
        self.emulator.send_str(s)
    }

    /// Send a character to the terminal
    pub fn send_char(&mut self, c: char) -> std::io::Result<()> {
        // Only track user input after initial shell setup (1 second grace period)
        if self.created_at.elapsed().as_secs() >= 1 {
            self.has_user_input = true;
        }
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
    /// If keyboard_mode_active is true and window is focused, uses keyboard mode colors
    pub fn render(
        &mut self,
        buffer: &mut VideoBuffer,
        charset: &Charset,
        theme: &Theme,
        tint_terminal: bool,
        keyboard_mode_active: bool,
    ) {
        // Get dynamic title with cached process name
        let dynamic_title = self.get_dynamic_title_cached();

        // Render the window frame and title bar with dynamic title
        self.window.render_with_title(
            buffer,
            charset,
            theme,
            Some(&dynamic_title),
            keyboard_mode_active,
        );

        // Render the terminal content
        self.render_terminal_content(buffer, theme, tint_terminal);

        // Render the scrollbar
        self.render_scrollbar(buffer, charset, theme);

        // Render close confirmation on top of window content (if active)
        self.render_close_confirmation(buffer, charset, theme);
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
                    // Use get_render_cell to respect synchronized output snapshot
                    grid.get_render_cell(grid_col, row_idx)
                };

                // Render the cell
                let mut cell = if let Some(term_cell) = term_cell {
                    convert_terminal_cell(term_cell, theme, tint_terminal)
                } else {
                    // Grid doesn't have data for this cell (window is larger than grid)
                    // Use default terminal background to maintain visual consistency
                    Cell::new_unchecked(' ', theme.window_content_fg, theme.window_content_bg)
                };

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

        // Render cursor if visible and not scrolled
        // Applications like Claude hide the cursor to draw their own
        // Use get_render_cursor to respect synchronized output snapshot
        let render_cursor = grid.get_render_cursor();
        if render_cursor.visible && self.scroll_offset == 0 {
            let cursor_x = content_x + render_cursor.x as u16;
            let cursor_y = content_y + render_cursor.y as u16;

            // Check if cursor is within window bounds
            if cursor_x < content_x + content_width && cursor_y < content_y + content_height {
                // Get the current cell at cursor position
                if let Some(current_cell) = buffer.get(cursor_x, cursor_y) {
                    // Create cursor based on cursor shape
                    let cursor_cell = match render_cursor.shape {
                        crate::term_grid::CursorShape::Block => {
                            // For block cursor, show as inverted colors
                            if current_cell.character == ' ' || current_cell.character == '\0' {
                                // For empty space, show a solid block using the foreground color
                                // This makes the cursor visible as a colored block
                                Cell::new('█', current_cell.fg_color, current_cell.bg_color)
                            } else {
                                // For text, invert the colors (swap fg and bg)
                                Cell::new(
                                    current_cell.character,
                                    current_cell.bg_color, // Use bg as fg (inverted)
                                    current_cell.fg_color, // Use fg as bg (inverted)
                                )
                            }
                        }
                        crate::term_grid::CursorShape::Underline => {
                            // For underline cursor, show underscore in foreground color
                            Cell::new('_', current_cell.fg_color, current_cell.bg_color)
                        }
                        crate::term_grid::CursorShape::Bar => {
                            // For bar cursor, show vertical bar in foreground color
                            Cell::new('│', current_cell.fg_color, current_cell.bg_color)
                        }
                    };
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
            CharsetMode::Unicode | CharsetMode::UnicodeSingleLine => '░', // Light shade for track
            CharsetMode::Ascii => '.',
        };
        let thumb_char = match charset.mode {
            CharsetMode::Unicode | CharsetMode::UnicodeSingleLine => '█',
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

    /// Render close confirmation dialog centered in window
    fn render_close_confirmation(
        &self,
        buffer: &mut VideoBuffer,
        charset: &Charset,
        theme: &Theme,
    ) {
        if self.pending_close_confirmation.is_none() {
            return;
        }

        let confirmation = self.pending_close_confirmation.as_ref().unwrap();

        // Dialog dimensions
        let dialog_width = 40u16;
        let dialog_height = 7u16; // Border + message (3 lines) + buttons + border

        // Center in window's content area
        let content_x = self.window.x + 2;
        let content_y = self.window.y + 1;
        let content_width = self.window.width.saturating_sub(4);
        let content_height = self.window.height.saturating_sub(2);

        let dialog_x = content_x + (content_width.saturating_sub(dialog_width)) / 2;
        let dialog_y = content_y + (content_height.saturating_sub(dialog_height)) / 2;

        // Ensure dialog fits within window
        let dialog_width = dialog_width.min(content_width);
        let dialog_height = dialog_height.min(content_height);

        // Draw shadow first (2 cells right, 1 cell down)
        video_buffer::render_shadow(
            buffer,
            dialog_x,
            dialog_y,
            dialog_width,
            dialog_height,
            charset,
            theme,
        );

        // Draw dialog background
        let bg_color = theme.prompt_danger_bg;
        let fg_color = theme.prompt_danger_fg;

        for y in 0..dialog_height {
            for x in 0..dialog_width {
                let cell = Cell::new(' ', fg_color, bg_color);
                buffer.set(dialog_x + x, dialog_y + y, cell);
            }
        }

        // Draw border
        let (tl, tr, bl, br, h, v) = match charset.mode {
            CharsetMode::Unicode | CharsetMode::UnicodeSingleLine => {
                ('╔', '╗', '╚', '╝', '═', '║')
            }
            CharsetMode::Ascii => ('+', '+', '+', '+', '-', '|'),
        };

        // Top border
        buffer.set(dialog_x, dialog_y, Cell::new(tl, fg_color, bg_color));
        for x in 1..dialog_width - 1 {
            buffer.set(dialog_x + x, dialog_y, Cell::new(h, fg_color, bg_color));
        }
        buffer.set(
            dialog_x + dialog_width - 1,
            dialog_y,
            Cell::new(tr, fg_color, bg_color),
        );

        // Side borders
        for y in 1..dialog_height - 1 {
            buffer.set(dialog_x, dialog_y + y, Cell::new(v, fg_color, bg_color));
            buffer.set(
                dialog_x + dialog_width - 1,
                dialog_y + y,
                Cell::new(v, fg_color, bg_color),
            );
        }

        // Bottom border
        buffer.set(
            dialog_x,
            dialog_y + dialog_height - 1,
            Cell::new(bl, fg_color, bg_color),
        );
        for x in 1..dialog_width - 1 {
            buffer.set(
                dialog_x + x,
                dialog_y + dialog_height - 1,
                Cell::new(h, fg_color, bg_color),
            );
        }
        buffer.set(
            dialog_x + dialog_width - 1,
            dialog_y + dialog_height - 1,
            Cell::new(br, fg_color, bg_color),
        );

        // Render message (3 lines centered)
        let messages = [
            "Close this terminal?",
            "",
            "Active content may be lost.",
        ];

        for (i, msg) in messages.iter().enumerate() {
            let msg_x = dialog_x + (dialog_width.saturating_sub(msg.len() as u16)) / 2;
            let msg_y = dialog_y + 1 + i as u16;
            for (j, ch) in msg.chars().enumerate() {
                buffer.set(msg_x + j as u16, msg_y, Cell::new(ch, fg_color, bg_color));
            }
        }

        // Render buttons on second-to-last row
        let button_y = dialog_y + dialog_height - 2;

        // Button texts
        let cancel_text = "[ Cancel ]";
        let close_text = "[ Close ]";

        // Calculate button positions (centered, with spacing)
        let total_width = cancel_text.len() + 4 + close_text.len(); // 4 = spacing
        let buttons_start_x = dialog_x + (dialog_width.saturating_sub(total_width as u16)) / 2;

        let cancel_x = buttons_start_x;
        let close_x = buttons_start_x + cancel_text.len() as u16 + 4;

        // Render Cancel button
        let (cancel_fg, cancel_bg) = if confirmation.selected_button == 0 {
            (
                theme.dialog_button_primary_success_fg,
                theme.dialog_button_primary_success_bg,
            )
        } else {
            (
                theme.dialog_button_secondary_fg,
                theme.dialog_button_secondary_bg,
            )
        };

        for (i, ch) in cancel_text.chars().enumerate() {
            buffer.set(
                cancel_x + i as u16,
                button_y,
                Cell::new(ch, cancel_fg, cancel_bg),
            );
        }

        // Add selection indicator for Cancel
        if confirmation.selected_button == 0 {
            buffer.set(
                cancel_x.saturating_sub(1),
                button_y,
                Cell::new('>', fg_color, bg_color),
            );
            buffer.set(
                cancel_x + cancel_text.len() as u16,
                button_y,
                Cell::new('<', fg_color, bg_color),
            );
        }

        // Render Close button
        let (close_fg, close_bg) = if confirmation.selected_button == 1 {
            (
                theme.dialog_button_primary_danger_fg,
                theme.dialog_button_primary_danger_bg,
            )
        } else {
            (
                theme.dialog_button_secondary_fg,
                theme.dialog_button_secondary_bg,
            )
        };

        for (i, ch) in close_text.chars().enumerate() {
            buffer.set(
                close_x + i as u16,
                button_y,
                Cell::new(ch, close_fg, close_bg),
            );
        }

        // Add selection indicator for Close
        if confirmation.selected_button == 1 {
            buffer.set(
                close_x.saturating_sub(1),
                button_y,
                Cell::new('>', fg_color, bg_color),
            );
            buffer.set(
                close_x + close_text.len() as u16,
                button_y,
                Cell::new('<', fg_color, bg_color),
            );
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

    /// Check if window has unsaved work (user input or non-shell process running)
    /// Ignores shell processes and common shell helpers
    pub fn is_dirty(&self) -> bool {
        // Check if user has typed anything (after initial 1 second grace period)
        if self.has_user_input {
            return true;
        }

        // Check if there's a non-shell process running
        if let Some(process_name) = self.get_foreground_process_name() {
            // List of shell processes and common shell-related tools to ignore
            let ignore_list = [
                // Shells (regular and login shell variants with - prefix)
                "bash", "-bash", "zsh", "-zsh", "sh", "-sh", "fish", "-fish",
                "dash", "-dash", "ksh", "-ksh", "csh", "-csh", "tcsh", "-tcsh",
                "nu", "-nu", "elvish", "-elvish", "xonsh", "-xonsh",
                // Shell prompt tools
                "starship", "gitstatus", "powerlevel10k",
                // Environment tools
                "direnv", "asdf", "mise", "rtx", "fnm", "nvm",
                // Common shell integrations
                "zsh-autocomplete", "zsh-autosuggestions", "zsh-syntax-highlighting",
            ];
            !ignore_list.contains(&process_name.as_str())
        } else {
            false
        }
    }

    /// Check if close confirmation dialog is currently shown
    pub fn has_close_confirmation(&self) -> bool {
        self.pending_close_confirmation.is_some()
    }

    /// Show the close confirmation dialog
    pub fn show_close_confirmation(&mut self) {
        self.pending_close_confirmation = Some(CloseConfirmation::new());
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

    /// Get the name of the foreground process running in the terminal
    pub fn get_foreground_process_name(&self) -> Option<String> {
        self.emulator.get_foreground_process_name()
    }

    /// Get the cached foreground process name, updating cache every 500ms
    /// This avoids spawning ps processes every frame (60fps = 60 times/second)
    fn get_foreground_process_name_cached(&mut self) -> Option<String> {
        use std::time::Duration;

        // Update cache every 500ms (2 times per second instead of 60)
        let elapsed = self.process_name_last_update.elapsed();
        if elapsed >= Duration::from_millis(500) || self.cached_process_name.is_none() {
            self.cached_process_name = self.emulator.get_foreground_process_name();
            self.process_name_last_update = Instant::now();
        }

        self.cached_process_name.clone()
    }

    /// Get the dynamic title including the running process name (with caching)
    /// Format: "Terminal N [ > process ]" where > is a running indicator
    fn get_dynamic_title_cached(&mut self) -> String {
        if let Some(process_name) = self.get_foreground_process_name_cached() {
            // Use '>' as an ASCII-compatible "running" indicator with spacing
            format!("{} [ > {} ]", self.window.title, process_name)
        } else {
            self.window.title.clone()
        }
    }

    /// Get the dynamic title including the running process name
    /// Format: "Terminal N [ > process ]" where > is a running indicator
    #[allow(dead_code)]
    pub fn get_dynamic_title(&self) -> String {
        if let Some(process_name) = self.get_foreground_process_name() {
            // Use '>' as an ASCII-compatible "running" indicator with spacing
            format!("{} [ > {} ]", self.window.title, process_name)
        } else {
            self.window.title.clone()
        }
    }

    /// Update the window's display title with process info
    #[allow(dead_code)]
    pub fn update_title_with_process(&mut self) {
        // Store the base title if not already stored
        if !self.window.title.contains(" [>") {
            // Title is still the base title, keep it
        }
    }

    /// Get the base title (without process info)
    #[allow(dead_code)]
    pub fn get_base_title(&self) -> &str {
        // If the title contains process info, extract base title
        if let Some(idx) = self.window.title.find(" [>") {
            &self.window.title[..idx]
        } else {
            &self.window.title
        }
    }

    /// Get application cursor keys mode state (DECCKM)
    pub fn get_application_cursor_keys(&self) -> bool {
        let grid = self.emulator.grid();
        let grid = grid.lock().unwrap();
        grid.application_cursor_keys
    }

    /// Handle keyboard input for close confirmation dialog
    /// Returns Some(true) if should close, Some(false) if canceled, None if not handled
    pub fn handle_close_confirmation_key(&mut self, key: KeyEvent) -> Option<bool> {
        let confirmation = self.pending_close_confirmation.as_mut()?;

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                confirmation.selected_button = 0; // Cancel
                Some(false) // Don't close, just update UI
            }
            KeyCode::Right | KeyCode::Char('l') => {
                confirmation.selected_button = 1; // Close
                Some(false) // Don't close, just update UI
            }
            KeyCode::Tab => {
                confirmation.selected_button = 1 - confirmation.selected_button; // Toggle
                Some(false) // Don't close, just update UI
            }
            KeyCode::Enter => {
                let should_close = confirmation.selected_button == 1;
                self.pending_close_confirmation = None;
                Some(should_close)
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.pending_close_confirmation = None;
                Some(false) // Cancel
            }
            _ => None, // Ignore other keys
        }
    }

    /// Handle mouse click for close confirmation dialog
    /// Returns Some(true) if should close, Some(false) if canceled, None if not in dialog
    pub fn handle_close_confirmation_click(&mut self, x: u16, y: u16) -> Option<bool> {
        self.pending_close_confirmation.as_ref()?;

        // Calculate dialog bounds
        let dialog_width = 40u16;
        let dialog_height = 7u16;

        let content_x = self.window.x + 2;
        let content_y = self.window.y + 1;
        let content_width = self.window.width.saturating_sub(4);
        let content_height = self.window.height.saturating_sub(2);

        let dialog_x = content_x + (content_width.saturating_sub(dialog_width)) / 2;
        let dialog_y = content_y + (content_height.saturating_sub(dialog_height)) / 2;
        let dialog_width = dialog_width.min(content_width);
        let dialog_height = dialog_height.min(content_height);

        // Check if click is within dialog
        if x < dialog_x
            || x >= dialog_x + dialog_width
            || y < dialog_y
            || y >= dialog_y + dialog_height
        {
            return None; // Click outside dialog
        }

        // Calculate button positions (same as render)
        let button_y = dialog_y + dialog_height - 2;

        if y != button_y {
            return None; // Not on button row
        }

        let cancel_text = "[ Cancel ]";
        let close_text = "[ Close ]";
        let total_width = cancel_text.len() + 4 + close_text.len();
        let buttons_start_x = dialog_x + (dialog_width.saturating_sub(total_width as u16)) / 2;

        let cancel_x = buttons_start_x;
        let cancel_end = cancel_x + cancel_text.len() as u16;
        let close_x = buttons_start_x + cancel_text.len() as u16 + 4;
        let close_end = close_x + close_text.len() as u16;

        if x >= cancel_x && x < cancel_end {
            // Clicked Cancel
            self.pending_close_confirmation = None;
            Some(false)
        } else if x >= close_x && x < close_end {
            // Clicked Close
            self.pending_close_confirmation = None;
            Some(true)
        } else {
            None
        }
    }
}

/// Convert a terminal cell to a video buffer cell
fn convert_terminal_cell(term_cell: &TerminalCell, theme: &Theme, tint_terminal: bool) -> Cell {
    let mut fg = convert_color(&term_cell.fg);
    let mut bg = convert_color(&term_cell.bg);

    // Handle reverse video attribute - swap fg and bg
    if term_cell.attrs.reverse {
        std::mem::swap(&mut fg, &mut bg);
    }

    // Apply theme-based tinting if enabled
    if tint_terminal {
        fg = apply_theme_tint(fg, theme, true);
        bg = apply_theme_tint(bg, theme, false);
        // Apply contrast checking when tinting is enabled
        Cell::new(term_cell.c, fg, bg)
    } else {
        // Preserve original terminal colors without contrast adjustments
        Cell::new_unchecked(term_cell.c, fg, bg)
    }
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
        Color::Blue | Color::DarkBlue => theme.window_border_unfocused_fg,

        // Cyan colors - map to content foreground for differentiation from blue
        Color::Cyan | Color::DarkCyan => theme.window_content_fg,

        // Magenta colors - map to a magenta-ish variation
        Color::Magenta | Color::DarkMagenta => theme.resize_handle_active_fg,

        // RGB colors - apply a theme-based tint transformation
        Color::Rgb { r, g, b } => {
            if is_foreground {
                // For foreground, blend with theme foreground color
                // Use 0.3 blend factor: 30% original color, 70% theme color for strong tinting
                blend_with_theme_color(Color::Rgb { r, g, b }, theme.window_content_fg, 0.3)
            } else {
                // For background, blend with theme background color
                // Use 0.3 blend factor: 30% original color, 70% theme color for strong tinting
                blend_with_theme_color(Color::Rgb { r, g, b }, theme.window_content_bg, 0.3)
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
                    4 => theme.window_border_unfocused_fg, // Blue - use border for visibility
                    5 => theme.resize_handle_active_fg,    // Magenta
                    6 => theme.window_content_fg,          // Cyan - differentiate from blue
                    7 => theme.window_content_fg,          // White
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

        // Catch-all: tint any unhandled colors to prevent full-brightness rendering
        // This ensures consistent tinting across all color types
        _ => {
            if is_foreground {
                theme.window_content_fg
            } else {
                theme.window_content_bg
            }
        }
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
    // Convert both colors to RGB for blending
    let (r1, g1, b1) = color_to_rgb(original);
    let (r2, g2, b2) = color_to_rgb(theme_color);

    Color::Rgb {
        r: (r1 as f32 * blend_factor + r2 as f32 * (1.0 - blend_factor)) as u8,
        g: (g1 as f32 * blend_factor + g2 as f32 * (1.0 - blend_factor)) as u8,
        b: (b1 as f32 * blend_factor + b2 as f32 * (1.0 - blend_factor)) as u8,
    }
}

/// Convert any Color to RGB values
fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb { r, g, b } => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::DarkGrey => (128, 128, 128),
        Color::Red => (255, 0, 0),
        Color::DarkRed => (128, 0, 0),
        Color::Green => (0, 255, 0),
        Color::DarkGreen => (0, 128, 0),
        Color::Yellow => (255, 255, 0),
        Color::DarkYellow => (128, 128, 0),
        Color::Blue => (0, 0, 255),
        Color::DarkBlue => (0, 0, 128),
        Color::Magenta => (255, 0, 255),
        Color::DarkMagenta => (128, 0, 128),
        Color::Cyan => (0, 255, 255),
        Color::DarkCyan => (0, 128, 128),
        Color::White => (255, 255, 255),
        Color::Grey => (192, 192, 192),
        // For indexed colors, use approximate RGB values
        Color::AnsiValue(idx) => ansi_to_rgb(idx),
        // Default to white for reset and unknown
        _ => (255, 255, 255),
    }
}

/// Convert ANSI 256-color palette index to approximate RGB
fn ansi_to_rgb(idx: u8) -> (u8, u8, u8) {
    match idx {
        // Standard 16 colors (0-15)
        0 => (0, 0, 0),        // Black
        1 => (128, 0, 0),      // Dark Red
        2 => (0, 128, 0),      // Dark Green
        3 => (128, 128, 0),    // Dark Yellow
        4 => (0, 0, 128),      // Dark Blue
        5 => (128, 0, 128),    // Dark Magenta
        6 => (0, 128, 128),    // Dark Cyan
        7 => (192, 192, 192),  // Grey
        8 => (128, 128, 128),  // Dark Grey
        9 => (255, 0, 0),      // Red
        10 => (0, 255, 0),     // Green
        11 => (255, 255, 0),   // Yellow
        12 => (0, 0, 255),     // Blue
        13 => (255, 0, 255),   // Magenta
        14 => (0, 255, 255),   // Cyan
        15 => (255, 255, 255), // White

        // 216-color cube (16-231): 6x6x6 RGB cube
        16..=231 => {
            let idx = idx - 16;
            let r = (idx / 36) % 6;
            let g = (idx / 6) % 6;
            let b = idx % 6;
            (
                if r > 0 { 55 + r * 40 } else { 0 },
                if g > 0 { 55 + g * 40 } else { 0 },
                if b > 0 { 55 + b * 40 } else { 0 },
            )
        }

        // Grayscale ramp (232-255): 24 shades of gray
        232..=255 => {
            let gray = 8 + (idx - 232) * 10;
            (gray, gray, gray)
        }
    }
}
