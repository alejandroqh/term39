use crate::ansi_handler::AnsiHandler;
use crate::session::{
    MAX_LINES_PER_TERMINAL, SerializableCell, SerializableCursor, SerializableTerminalLine,
};
use crate::term_grid::TerminalGrid;
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use vte::Parser;

/// Terminal emulator that manages PTY, parser, and terminal grid
pub struct TerminalEmulator {
    /// Terminal grid (screen buffer)
    grid: Arc<Mutex<TerminalGrid>>,
    /// VTE parser
    parser: Parser,
    /// PTY master (for reading/writing)
    pty_master: Box<dyn MasterPty + Send>,
    /// PTY writer
    writer: Box<dyn Write + Send>,
    /// Child process handle
    _child: Box<dyn Child + Send>,
    /// Channel to receive data from PTY reader thread
    rx: Receiver<Vec<u8>>,
}

impl TerminalEmulator {
    /// Create a new terminal emulator with a shell process or direct command
    ///
    /// # Arguments
    /// * `cols` - Number of columns
    /// * `rows` - Number of rows
    /// * `max_scrollback` - Maximum scrollback lines
    /// * `command` - Optional command to run directly. If None, spawns default shell.
    ///   Format: Some(("program", vec!["arg1", "arg2"]))
    pub fn new(
        cols: usize,
        rows: usize,
        max_scrollback: usize,
        command: Option<(String, Vec<String>)>,
    ) -> std::io::Result<Self> {
        let pty_system = native_pty_system();

        // Create PTY with specified size
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(std::io::Error::other)?;

        // Spawn process (either command or default shell)
        let mut cmd = if let Some((program, args)) = command {
            // Launch specific command directly
            let mut cmd = CommandBuilder::new(program);
            for arg in args {
                cmd.arg(arg);
            }
            cmd
        } else {
            // Spawn default shell
            CommandBuilder::new_default_prog()
        };

        // Set environment variables
        cmd.env("TERM", "xterm-256color");

        // Disable zsh's PROMPT_SP feature (which shows "%" for unterminated lines)
        // This prevents the "%" character from appearing at startup and after 'clear'
        // Set PROMPT_EOL_MARK to empty string to hide the mark entirely
        cmd.env("PROMPT_EOL_MARK", "");

        // Disable PROMPT_SP entirely to prevent any cursor positioning at startup
        cmd.env("PROMPT_SP", "");

        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(std::io::Error::other)?;

        // Get master PTY for I/O
        let pty_master = pty_pair.master;

        // Get reader and writer
        let mut reader = pty_master
            .try_clone_reader()
            .map_err(std::io::Error::other)?;

        let writer = pty_master.take_writer().map_err(std::io::Error::other)?;

        // Create channel for reading from PTY in background thread
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();

        // Spawn reader thread
        thread::spawn(move || {
            let mut buffer = vec![0u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        break;
                    }
                    Ok(n) => {
                        if tx.send(buffer[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        let grid = Arc::new(Mutex::new(TerminalGrid::new(cols, rows, max_scrollback)));
        let parser = Parser::new();

        Ok(Self {
            grid,
            parser,
            pty_master,
            writer,
            _child: child,
            rx,
        })
    }

    /// Get a clone of the grid Arc for sharing with renderer
    pub fn grid(&self) -> Arc<Mutex<TerminalGrid>> {
        self.grid.clone()
    }

    /// Read output from PTY and process it through the parser
    pub fn process_output(&mut self) -> std::io::Result<bool> {
        // Try to receive data from PTY reader thread (non-blocking)
        let result = match self.rx.try_recv() {
            Ok(data) => {
                // Process the bytes through VTE parser
                let mut grid = self.grid.lock().unwrap();
                let mut handler = AnsiHandler::new(&mut grid);

                self.parser.advance(&mut handler, &data);

                Ok(true)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // No data available right now
                Ok(true)
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // Reader thread died - child process exited
                Ok(false)
            }
        };

        // Process any queued responses (e.g., DSR cursor position reports)
        let responses = {
            let mut grid = self.grid.lock().unwrap();
            grid.take_responses()
        };

        for response in responses {
            // Send response back to PTY (ignore errors)
            let _ = self.write_input(response.as_bytes());
        }

        result
    }

    /// Write input to the PTY (send to shell)
    pub fn write_input(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Resize the terminal and notify the PTY
    pub fn resize(&mut self, cols: usize, rows: usize) -> std::io::Result<()> {
        // Resize the grid
        {
            let mut grid = self.grid.lock().unwrap();
            grid.resize(cols, rows);
        }

        // Notify PTY of size change (sends SIGWINCH to child process)
        self.pty_master
            .resize(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(std::io::Error::other)?;

        Ok(())
    }

    /// Send a string to the terminal
    pub fn send_str(&mut self, s: &str) -> std::io::Result<()> {
        self.write_input(s.as_bytes())
    }

    /// Send a character to the terminal
    pub fn send_char(&mut self, c: char) -> std::io::Result<()> {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        self.write_input(s.as_bytes())
    }

    /// Extract terminal content (scrollback + visible lines) for session persistence
    /// Returns at most MAX_LINES_PER_TERMINAL lines (most recent lines are kept)
    pub fn get_terminal_content(&self) -> (Vec<SerializableTerminalLine>, SerializableCursor) {
        let grid = self.grid.lock().unwrap();

        let mut all_lines = Vec::new();

        // Get scrollback lines (oldest first)
        let scrollback_len = grid.scrollback_len();
        for i in 0..scrollback_len {
            if let Some(line) = grid.get_scrollback_line(i) {
                let cells: Vec<SerializableCell> =
                    line.iter().map(SerializableCell::from).collect();
                all_lines.push(SerializableTerminalLine { cells });
            }
        }

        // Get visible screen lines
        let rows = grid.rows();
        for y in 0..rows {
            let mut cells = Vec::new();
            let cols = grid.cols();
            for x in 0..cols {
                if let Some(cell) = grid.get_cell(x, y) {
                    cells.push(SerializableCell::from(cell));
                }
            }
            all_lines.push(SerializableTerminalLine { cells });
        }

        // Limit to MAX_LINES_PER_TERMINAL (keep most recent lines)
        if all_lines.len() > MAX_LINES_PER_TERMINAL {
            let skip = all_lines.len() - MAX_LINES_PER_TERMINAL;
            all_lines = all_lines.into_iter().skip(skip).collect();
        }

        // Get cursor position
        let cursor = SerializableCursor::from(&grid.cursor);

        (all_lines, cursor)
    }

    /// Restore terminal content from saved session data
    /// This is called after creating a new terminal to restore previous session content
    pub fn restore_terminal_content(
        &mut self,
        lines: Vec<SerializableTerminalLine>,
        cursor: &SerializableCursor,
    ) {
        // Convert SerializableTerminalLine back to Vec<TerminalCell>
        let terminal_lines: Vec<Vec<crate::term_grid::TerminalCell>> = lines
            .iter()
            .map(|line| {
                line.cells
                    .iter()
                    .map(crate::term_grid::TerminalCell::from)
                    .collect()
            })
            .collect();

        // Restore content to grid
        let mut grid = self.grid.lock().unwrap();
        grid.restore_content(terminal_lines);
        grid.set_cursor(cursor.x, cursor.y, cursor.visible);
    }
}
