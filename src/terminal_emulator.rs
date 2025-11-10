use crate::ansi_handler::AnsiHandler;
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
    /// Create a new terminal emulator with a shell process
    pub fn new(cols: usize, rows: usize, max_scrollback: usize) -> std::io::Result<Self> {
        let pty_system = native_pty_system();

        // Create PTY with specified size
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Spawn shell process
        let mut cmd = CommandBuilder::new_default_prog();

        // Set environment variables
        cmd.env("TERM", "xterm-256color");

        // Disable zsh's PROMPT_SP feature (which shows "%" for unterminated lines)
        // This prevents the "%" character from appearing at startup and after 'clear'
        // Set PROMPT_EOL_MARK to empty string to hide the mark entirely
        cmd.env("PROMPT_EOL_MARK", "");

        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Get master PTY for I/O
        let pty_master = pty_pair.master;

        // Get reader and writer
        let mut reader = pty_master
            .try_clone_reader()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        debug_log!("PTY: Got reader");

        let writer = pty_master
            .take_writer()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        debug_log!("PTY: Got writer");

        // Create channel for reading from PTY in background thread
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();

        // Spawn reader thread
        thread::spawn(move || {
            let mut buffer = vec![0u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        debug_log!("PTY reader thread: EOF");
                        break;
                    }
                    Ok(n) => {
                        debug_log!("PTY reader thread: Read {} bytes", n);
                        if tx.send(buffer[..n].to_vec()).is_err() {
                            debug_log!("PTY reader thread: Send failed, exiting");
                            break;
                        }
                    }
                    Err(e) => {
                        debug_log!("PTY reader thread: Error: {}", e);
                        break;
                    }
                }
            }
        });

        debug_log!("PTY: Spawned reader thread");

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
        match self.rx.try_recv() {
            Ok(data) => {
                debug_log!("PTY: Received {} bytes from thread", data.len());
                // Process the bytes through VTE parser
                let mut grid = self.grid.lock().unwrap();
                let mut handler = AnsiHandler::new(&mut grid);

                for &byte in &data {
                    self.parser.advance(&mut handler, byte);
                }

                Ok(true)
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // No data available right now
                Ok(true)
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // Reader thread died - child process exited
                debug_log!("PTY: Reader thread disconnected");
                Ok(false)
            }
        }
    }

    /// Write input to the PTY (send to shell)
    pub fn write_input(&mut self, data: &[u8]) -> std::io::Result<()> {
        debug_log!(
            "PTY: Writing {} bytes: {:?}",
            data.len(),
            String::from_utf8_lossy(data)
        );
        let result = self.writer.write_all(data);
        debug_log!("PTY: Write result: {:?}", result);
        result?;
        let flush_result = self.writer.flush();
        debug_log!("PTY: Flush result: {:?}", flush_result);
        flush_result?;
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
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

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
}
