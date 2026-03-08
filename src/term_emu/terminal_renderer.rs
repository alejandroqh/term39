#![allow(dead_code)]

use super::ansi_handler::AnsiHandler;
use super::term_grid::TerminalGrid;
use std::sync::{Arc, Mutex};
use vte::Parser;

/// Terminal renderer: grid + VTE parser without PTY ownership.
///
/// Used in two contexts:
/// 1. As the rendering core inside TerminalEmulator (Local mode)
/// 2. Standalone in client mode (Remote mode) where PTY is owned by the daemon
pub struct TerminalRenderer {
    /// Terminal grid (screen buffer)
    grid: Arc<Mutex<TerminalGrid>>,
    /// VTE parser
    parser: Parser,
}

impl TerminalRenderer {
    /// Create a new terminal renderer
    pub fn new(cols: usize, rows: usize, max_scrollback: usize) -> Self {
        Self {
            grid: Arc::new(Mutex::new(TerminalGrid::new(cols, rows, max_scrollback))),
            parser: Parser::new(),
        }
    }

    /// Create from an existing grid Arc (for wrapping in TerminalEmulator)
    pub fn from_grid(grid: Arc<Mutex<TerminalGrid>>) -> Self {
        Self {
            grid,
            parser: Parser::new(),
        }
    }

    /// Get a clone of the grid Arc
    pub fn grid(&self) -> Arc<Mutex<TerminalGrid>> {
        self.grid.clone()
    }

    /// Feed raw PTY output bytes through the VTE parser into the grid
    pub fn feed_output(&mut self, data: &[u8]) {
        let mut grid = self.grid.lock().expect("terminal grid mutex poisoned");
        let mut handler = AnsiHandler::new(&mut grid);
        self.parser.advance(&mut handler, data);
    }

    /// Feed multiple chunks of output data
    pub fn feed_output_chunks(&mut self, chunks: &[Vec<u8>]) {
        if chunks.is_empty() {
            return;
        }
        let mut grid = self.grid.lock().expect("terminal grid mutex poisoned");
        let mut handler = AnsiHandler::new(&mut grid);
        for chunk in chunks {
            self.parser.advance(&mut handler, chunk);
        }
    }

    /// Take queued responses (e.g., DSR cursor position reports)
    pub fn take_responses(&self) -> Vec<String> {
        let mut grid = self.grid.lock().expect("terminal grid mutex poisoned");
        grid.take_responses()
    }

    /// Resize the grid (no PTY notification - that's handled externally)
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let mut grid = self.grid.lock().expect("terminal grid mutex poisoned");
        grid.resize(cols, rows);
    }
}
