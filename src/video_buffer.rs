use crossterm::{
    cursor,
    execute,
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
    pub fn new(character: char, fg_color: Color, bg_color: Color) -> Self {
        Self {
            character,
            fg_color,
            bg_color,
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
    #[allow(dead_code)]
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
