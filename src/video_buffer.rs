use crate::charset::Charset;
use crate::color_utils;
use crate::theme::Theme;
use crossterm::{
    cursor, execute,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};
use std::io;

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
            bg_color: Color::Blue,
        }
    }
}

/// Double-buffered video memory for efficient rendering
pub struct VideoBuffer {
    width: u16,
    height: u16,
    front_buffer: Vec<Cell>,
    back_buffer: Vec<Cell>,
}

impl VideoBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        let default_cell = Cell::default();

        Self {
            width,
            height,
            front_buffer: vec![default_cell; size],
            back_buffer: vec![default_cell; size],
        }
    }

    /// Get index for x, y coordinates
    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.width && y < self.height {
            Some((y as usize) * (self.width as usize) + (x as usize))
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

    /// Present back buffer to screen, only updating changed cells
    pub fn present(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        let mut current_fg = Color::Reset;
        let mut current_bg = Color::Reset;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y as usize) * (self.width as usize) + (x as usize);
                let front_cell = &self.front_buffer[idx];
                let back_cell = &self.back_buffer[idx];

                // Only update if cell changed
                if front_cell != back_cell {
                    // Update colors only if they changed
                    if back_cell.fg_color != current_fg {
                        execute!(stdout, SetForegroundColor(back_cell.fg_color))?;
                        current_fg = back_cell.fg_color;
                    }
                    if back_cell.bg_color != current_bg {
                        execute!(stdout, SetBackgroundColor(back_cell.bg_color))?;
                        current_bg = back_cell.bg_color;
                    }

                    // Move cursor and print character
                    execute!(stdout, cursor::MoveTo(x, y), Print(back_cell.character))?;
                }
            }
        }

        // Swap buffers
        std::mem::swap(&mut self.front_buffer, &mut self.back_buffer);

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
    _theme: &Theme,
) {
    let shadow_fg = Color::DarkGrey;
    let shadow_bg = Color::Black;
    let (buffer_width, buffer_height) = buffer.dimensions();

    // Right shadow (2 cells wide to the right)
    for shadow_offset in 0..2 {
        let shadow_x = x + width + shadow_offset;
        if shadow_x < buffer_width {
            for dy in 1..=height {
                let shadow_y = y + dy;
                if shadow_y < buffer_height {
                    // Get existing cell and preserve its character
                    if let Some(existing_cell) = buffer.get(shadow_x, shadow_y) {
                        let shadowed_cell =
                            Cell::new_unchecked(existing_cell.character, shadow_fg, shadow_bg);
                        buffer.set(shadow_x, shadow_y, shadowed_cell);
                    }
                }
            }
        }
    }

    // Bottom shadow (1 cell down)
    let shadow_y = y + height;
    if shadow_y < buffer_height {
        for dx in 1..=width {
            let shadow_x = x + dx;
            if shadow_x < buffer_width {
                // Get existing cell and preserve its character
                if let Some(existing_cell) = buffer.get(shadow_x, shadow_y) {
                    let shadowed_cell =
                        Cell::new_unchecked(existing_cell.character, shadow_fg, shadow_bg);
                    buffer.set(shadow_x, shadow_y, shadowed_cell);
                }
            }
        }
    }
}

/// Render a full-screen shadow overlay (for modal dialogs)
/// This shadows the entire screen to indicate that only the modal dialog is interactive
/// Preserves existing characters and only modifies colors (black bg, dark grey fg)
pub fn render_fullscreen_shadow(buffer: &mut VideoBuffer) {
    let shadow_fg = Color::DarkGrey;
    let shadow_bg = Color::Black;
    let (width, height) = buffer.dimensions();

    // Shadow every cell in the buffer
    for y in 0..height {
        for x in 0..width {
            // Get existing cell and preserve its character
            if let Some(existing_cell) = buffer.get(x, y) {
                let shadowed_cell =
                    Cell::new_unchecked(existing_cell.character, shadow_fg, shadow_bg);
                buffer.set(x, y, shadowed_cell);
            }
        }
    }
}
