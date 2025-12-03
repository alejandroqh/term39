use crate::charset::Charset;
use crate::color_utils;
use crate::theme::Theme;
use crossterm::{
    QueueableCommand, cursor,
    style::{Color, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Represents a single cell in the terminal buffer
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub character: char,
    pub fg_color: Color,
    pub bg_color: Color,
}

impl Cell {
    /// Create a new cell with automatic contrast checking.
    /// If the contrast ratio between foreground and background is too low,
    /// the colors will be automatically adjusted to ensure readability.
    /// Uses WCAG 2.1 AA standard (4.5:1 contrast ratio for normal text).
    pub fn new(character: char, fg_color: Color, bg_color: Color) -> Self {
        // Ensure minimum contrast ratio of 4.5:1 (WCAG AA level)
        let (adjusted_fg, adjusted_bg) = color_utils::ensure_contrast(fg_color, bg_color, 4.5);

        Self {
            character,
            fg_color: adjusted_fg,
            bg_color: adjusted_bg,
        }
    }

    /// Create a new cell without contrast checking.
    /// Use this for special effects like shadows where low contrast is intentional.
    pub fn new_unchecked(character: char, fg_color: Color, bg_color: Color) -> Self {
        Self {
            character,
            fg_color,
            bg_color,
        }
    }

    /// Create a new cell with inverted colors (for selection highlighting)
    pub fn inverted(&self) -> Self {
        Self {
            character: self.character,
            fg_color: self.bg_color,
            bg_color: self.fg_color,
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            character: ' ',
            fg_color: Color::White,
            bg_color: Color::Black, // Neutral default that works across all themes
        }
    }
}

/// Double-buffered video memory for efficient rendering
pub struct VideoBuffer {
    width: u16,
    height: u16,
    front_buffer: Vec<Cell>,
    back_buffer: Vec<Cell>,
    /// TTY cursor position (for raw mouse input mode)
    /// When set, the cell at this position will be rendered with inverted colors
    tty_cursor: Option<(u16, u16)>,
}

impl VideoBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        // Use checked arithmetic to prevent overflow, with a reasonable fallback
        let size = (width as usize).checked_mul(height as usize).unwrap_or(0);
        let default_cell = Cell::default();

        Self {
            width,
            height,
            front_buffer: vec![default_cell; size],
            back_buffer: vec![default_cell; size],
            tty_cursor: None,
        }
    }

    /// Get index for x, y coordinates
    /// Uses checked arithmetic to prevent integer overflow
    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.width && y < self.height {
            // Use checked arithmetic to prevent overflow
            let row_offset = (y as usize).checked_mul(self.width as usize)?;
            row_offset.checked_add(x as usize)
        } else {
            None
        }
    }

    /// Get cell at position from back buffer
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        self.index(x, y).map(|i| &self.back_buffer[i])
    }

    /// Get cell at position from front buffer (what's currently displayed)
    pub fn get_front(&self, x: u16, y: u16) -> Option<&Cell> {
        self.index(x, y).map(|i| &self.front_buffer[i])
    }

    /// Set cell at position in back buffer
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if let Some(i) = self.index(x, y) {
            self.back_buffer[i] = cell;
        }
    }

    /// Clear back buffer with a specific cell
    #[allow(dead_code)]
    pub fn clear(&mut self, cell: Cell) {
        for c in &mut self.back_buffer {
            *c = cell;
        }
    }

    /// Get buffer dimensions
    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Set TTY cursor position for raw mouse input mode
    /// The cell at this position will be rendered with inverted colors
    pub fn set_tty_cursor(&mut self, col: u16, row: u16) {
        self.tty_cursor = Some((col, row));
    }

    /// Clear TTY cursor (hide it)
    pub fn clear_tty_cursor(&mut self) {
        self.tty_cursor = None;
    }

    /// Get current TTY cursor position
    #[allow(dead_code)]
    pub fn get_tty_cursor(&self) -> Option<(u16, u16)> {
        self.tty_cursor
    }

    /// Apply shadow overlay to all cells in the back buffer
    /// This is an optimized version that directly modifies the buffer
    /// without the overhead of get/set methods
    pub fn apply_fullscreen_shadow(&mut self, shadow_fg: Color, shadow_bg: Color) {
        for cell in &mut self.back_buffer {
            cell.fg_color = shadow_fg;
            cell.bg_color = shadow_bg;
        }
    }

    /// Present back buffer to screen, only updating changed cells
    /// Uses queued commands for batched I/O - significantly reduces syscalls
    /// Optimized with run-length encoding for consecutive cells
    pub fn present(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        // Hide cursor at the START of rendering to prevent any cursor flicker
        // This ensures the cursor stays hidden even if PTY output or other
        // operations between frames affected cursor state
        stdout.queue(cursor::Hide)?;

        let mut current_fg = Color::Reset;
        let mut current_bg = Color::Reset;

        // Buffer for accumulating consecutive characters with same colors
        // Pre-allocate with reasonable capacity to avoid reallocations
        let mut run_buffer = String::with_capacity(256);
        let mut run_start_x: u16 = 0;
        let mut run_y: u16 = 0;
        let mut run_char_count: u16 = 0; // Track character count separately for O(1) access
        let mut in_run = false;

        // Extract cursor position once to avoid is_some_and() call per cell
        let cursor_pos = self.tty_cursor;

        for y in 0..self.height {
            // Calculate row start index once per row
            let row_start = (y as usize) * (self.width as usize);

            for x in 0..self.width {
                let idx = row_start + (x as usize);

                // Safety: we know idx is valid because we control the loop bounds
                if idx >= self.front_buffer.len() {
                    continue;
                }

                let front_cell = &self.front_buffer[idx];
                let back_cell = &self.back_buffer[idx];

                // Check if this cell is under the TTY cursor - if so, invert colors
                let is_cursor = cursor_pos.is_some_and(|(cx, cy)| cx == x && cy == y);
                let display_cell = if is_cursor {
                    back_cell.inverted()
                } else {
                    *back_cell
                };

                // Only update if cell changed (compare with inverted if cursor)
                if front_cell != &display_cell {
                    // Check if we can extend the current run
                    // Cell must be immediately adjacent (same row, next column) with same colors
                    let can_extend = in_run
                        && y == run_y
                        && x == run_start_x + run_char_count
                        && display_cell.fg_color == current_fg
                        && display_cell.bg_color == current_bg;

                    if can_extend {
                        // Extend the current run
                        run_buffer.push(display_cell.character);
                        run_char_count += 1;
                    } else {
                        // Flush previous run if any
                        if in_run && !run_buffer.is_empty() {
                            stdout.queue(cursor::MoveTo(run_start_x, run_y))?;
                            stdout.write_all(run_buffer.as_bytes())?;
                            run_buffer.clear();
                        }

                        // Update colors if needed
                        if display_cell.fg_color != current_fg {
                            stdout.queue(SetForegroundColor(display_cell.fg_color))?;
                            current_fg = display_cell.fg_color;
                        }
                        if display_cell.bg_color != current_bg {
                            stdout.queue(SetBackgroundColor(display_cell.bg_color))?;
                            current_bg = display_cell.bg_color;
                        }

                        // Start new run
                        run_start_x = x;
                        run_y = y;
                        run_buffer.push(display_cell.character);
                        run_char_count = 1;
                        in_run = true;
                    }
                }
            }

            // Flush run at end of each row (can't span rows)
            if in_run && !run_buffer.is_empty() {
                stdout.queue(cursor::MoveTo(run_start_x, run_y))?;
                stdout.write_all(run_buffer.as_bytes())?;
                run_buffer.clear();
                run_char_count = 0;
                in_run = false;
            }
        }

        // Update front buffer to reflect what's actually displayed
        // We need to handle cursor separately since it's rendered with inverted colors
        // Use the pre-extracted cursor_pos to avoid repeated is_some_and() calls
        for (idx, back_cell) in self.back_buffer.iter().enumerate() {
            let x = (idx % self.width as usize) as u16;
            let y = (idx / self.width as usize) as u16;

            let is_cursor = cursor_pos.is_some_and(|(cx, cy)| cx == x && cy == y);
            self.front_buffer[idx] = if is_cursor {
                back_cell.inverted()
            } else {
                *back_cell
            };
        }

        // Hide cursor after rendering to prevent it from being visible or affecting PTY output
        // Even hidden cursors have a position, so we also move it to (0, 0)
        stdout.queue(cursor::MoveTo(0, 0))?;
        stdout.queue(cursor::Hide)?;

        // Flush all queued commands at once - single syscall
        stdout.flush()?;

        Ok(())
    }

    /// Save a rectangular region from the front buffer
    #[allow(dead_code)]
    pub fn save_region(&self, x: u16, y: u16, width: u16, height: u16) -> Vec<Cell> {
        let mut saved = Vec::with_capacity((width as usize) * (height as usize));

        for dy in 0..height {
            for dx in 0..width {
                let cell_x = x + dx;
                let cell_y = y + dy;

                if let Some(cell) = self.get_front(cell_x, cell_y) {
                    saved.push(*cell);
                } else {
                    saved.push(Cell::default());
                }
            }
        }

        saved
    }

    /// Restore a rectangular region to the back buffer
    #[allow(dead_code)]
    pub fn restore_region(&mut self, x: u16, y: u16, width: u16, height: u16, saved: &[Cell]) {
        let mut idx = 0;

        for dy in 0..height {
            for dx in 0..width {
                if idx < saved.len() {
                    let cell_x = x + dx;
                    let cell_y = y + dy;
                    self.set(cell_x, cell_y, saved[idx]);
                    idx += 1;
                }
            }
        }
    }
}

/// Render a shadow for a rectangular region
/// Draws a 2-cell shadow on the right side and 1-cell shadow on the bottom of the given region
/// Instead of drawing with a shadow character, this preserves the existing character
/// and modifies its colors to create a "shadowed" effect (black bg, dark grey fg)
pub fn render_shadow(
    buffer: &mut VideoBuffer,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    _charset: &Charset,
    theme: &Theme,
) {
    let shadow_fg = theme.window_shadow_color;
    let shadow_bg = Color::Black;
    let (buffer_width, buffer_height) = buffer.dimensions();

    // Pre-compute shadow boundaries
    let right_shadow_x1 = x + width;
    let right_shadow_x2 = x + width + 1;
    let bottom_shadow_y = y + height;

    // Right shadow (2 cells wide to the right)
    // Process both columns together for better cache locality
    for dy in 1..=height {
        let shadow_y = y + dy;
        if shadow_y >= buffer_height {
            continue;
        }

        // First column of right shadow
        if right_shadow_x1 < buffer_width {
            if let Some(existing_cell) = buffer.get(right_shadow_x1, shadow_y) {
                let shadowed_cell =
                    Cell::new_unchecked(existing_cell.character, shadow_fg, shadow_bg);
                buffer.set(right_shadow_x1, shadow_y, shadowed_cell);
            }
        }

        // Second column of right shadow
        if right_shadow_x2 < buffer_width {
            if let Some(existing_cell) = buffer.get(right_shadow_x2, shadow_y) {
                let shadowed_cell =
                    Cell::new_unchecked(existing_cell.character, shadow_fg, shadow_bg);
                buffer.set(right_shadow_x2, shadow_y, shadowed_cell);
            }
        }
    }

    // Bottom shadow (1 cell down)
    if bottom_shadow_y < buffer_height {
        // Calculate the shadow end position, clamped to buffer width
        let shadow_end = (x + width + 1).min(buffer_width);

        for shadow_x in (x + 1)..shadow_end {
            // Get existing cell and preserve its character
            if let Some(existing_cell) = buffer.get(shadow_x, bottom_shadow_y) {
                let shadowed_cell =
                    Cell::new_unchecked(existing_cell.character, shadow_fg, shadow_bg);
                buffer.set(shadow_x, bottom_shadow_y, shadowed_cell);
            }
        }
    }
}

/// Render a full-screen shadow overlay (for modal dialogs)
/// This shadows the entire screen to indicate that only the modal dialog is interactive
/// Preserves existing characters and only modifies colors (black bg, dark grey fg)
pub fn render_fullscreen_shadow(buffer: &mut VideoBuffer, theme: &Theme) {
    let shadow_fg = theme.window_shadow_color;
    let shadow_bg = Color::Black;

    // Use the optimized method that directly modifies the buffer
    buffer.apply_fullscreen_shadow(shadow_fg, shadow_bg);
}
