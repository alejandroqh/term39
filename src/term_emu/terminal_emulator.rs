use super::ansi_handler::AnsiHandler;
use super::term_grid::TerminalGrid;
use crate::app::session::{
    MAX_LINES_PER_TERMINAL, SerializableCell, SerializableCursor, SerializableTerminalLine,
};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::io::{BufWriter, Read, Write};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Mutex};
use std::thread;
use vte::Parser;

#[cfg(not(target_os = "linux"))]
use std::process::Command;

/// Shell configuration for terminal emulator
#[derive(Clone, Debug, Default)]
pub struct ShellConfig {
    /// Path to shell executable, None means use OS default
    pub shell_path: Option<String>,
}

impl ShellConfig {
    /// Create a shell config with a custom shell path
    pub fn custom_shell(path: String) -> Self {
        Self {
            shell_path: Some(path),
        }
    }

    /// Validate the shell configuration
    /// Returns Ok(()) if valid, Err with message if invalid
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref path) = self.shell_path {
            if !std::path::Path::new(path).exists() {
                return Err(format!("Shell '{}' not found", path));
            }
            // Check if file is executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(path) {
                    if metadata.permissions().mode() & 0o111 == 0 {
                        return Err(format!("Shell '{}' is not executable", path));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Terminal emulator that manages PTY, parser, and terminal grid
pub struct TerminalEmulator {
    /// Terminal grid (screen buffer)
    grid: Arc<Mutex<TerminalGrid>>,
    /// VTE parser
    parser: Parser,
    /// PTY master (for reading/writing)
    pty_master: Box<dyn MasterPty + Send>,
    /// PTY writer (buffered for efficiency)
    writer: BufWriter<Box<dyn Write + Send>>,
    /// Child process handle
    child: Box<dyn Child + Send>,
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
    /// * `command` - Optional command to run directly. If None, spawns shell based on shell_config.
    ///   Format: Some(("program", vec!["arg1", "arg2"]))
    /// * `shell_config` - Configuration for which shell to use when command is None
    pub fn new(
        cols: usize,
        rows: usize,
        max_scrollback: usize,
        command: Option<(String, Vec<String>)>,
        shell_config: &ShellConfig,
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

        // Spawn process (either command, custom shell, or default shell)
        let mut cmd = if let Some((program, args)) = command {
            // Launch specific command directly (e.g., from Slight launcher)
            let mut cmd = CommandBuilder::new(program);
            for arg in args {
                cmd.arg(arg);
            }
            cmd
        } else if let Some(ref shell_path) = shell_config.shell_path {
            // Use custom shell if specified and valid
            if std::path::Path::new(shell_path).exists() {
                CommandBuilder::new(shell_path)
            } else {
                // Shell doesn't exist, fall back to default (validation should catch this earlier)
                CommandBuilder::new_default_prog()
            }
        } else {
            // Spawn default shell
            CommandBuilder::new_default_prog()
        };

        // Set environment variables
        cmd.env("TERM", "xterm-256color");
        // Enable true color (24-bit RGB) support for applications like nvim, vim, etc.
        cmd.env("COLORTERM", "truecolor");

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

        let writer = BufWriter::new(pty_master.take_writer().map_err(std::io::Error::other)?);

        // Create bounded channel for reading from PTY in background thread
        // Capacity of 64 provides back-pressure while allowing efficient batching
        let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(64);

        // Spawn reader thread
        thread::spawn(move || {
            let mut buffer = vec![0u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF - child process exited normally
                        break;
                    }
                    Ok(n) => {
                        if tx.send(buffer[..n].to_vec()).is_err() {
                            // Receiver dropped - main thread no longer listening
                            break;
                        }
                    }
                    Err(e) => {
                        // Log the error for diagnostics
                        eprintln!("PTY reader thread error: {}", e);
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
            child,
            rx,
        })
    }

    /// Get a clone of the grid Arc for sharing with renderer
    pub fn grid(&self) -> Arc<Mutex<TerminalGrid>> {
        self.grid.clone()
    }

    /// Read output from PTY and process it through the parser
    pub fn process_output(&mut self) -> std::io::Result<bool> {
        // Collect ALL available data from PTY reader thread (non-blocking)
        // This ensures complete escape sequences are processed before rendering,
        // which is important for TUI applications that use cursor movement for redraws
        let mut chunks = Vec::new();
        let mut process_result = Ok(true);

        // First, drain all available chunks without holding the grid lock
        loop {
            match self.rx.try_recv() {
                Ok(data) => {
                    chunks.push(data);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // No more data available right now
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // Reader thread died - child process exited
                    process_result = Ok(false);
                    break;
                }
            }
        }

        // Now process all chunks with a single grid lock acquisition
        if !chunks.is_empty() {
            let mut grid = self.grid.lock().expect("terminal grid mutex poisoned");
            let mut handler = AnsiHandler::new(&mut grid);

            for data in chunks {
                self.parser.advance(&mut handler, &data);
            }
        }

        // Process any queued responses (e.g., DSR cursor position reports)
        let responses = {
            let mut grid = self.grid.lock().expect("terminal grid mutex poisoned");
            grid.take_responses()
        };

        for response in responses {
            // Send response back to PTY
            if let Err(e) = self.write_input(response.as_bytes()) {
                eprintln!("Failed to write terminal response: {}", e);
            }
        }

        // On Windows, the PTY reader thread may not immediately detect when a child
        // process exits (e.g., cmd.exe). Explicitly check if the child has exited
        // using try_wait() to ensure windows are auto-closed properly.
        if process_result.as_ref().is_ok_and(|&running| running) {
            if let Ok(Some(_exit_status)) = self.child.try_wait() {
                // Child process has exited
                process_result = Ok(false);
            }
        }

        process_result
    }

    /// Write input to the PTY (send to shell)
    /// On Windows: flushes immediately to avoid ConPTY buffering issues
    /// On other platforms: buffered for efficiency, call flush_input() after batch
    pub fn write_input(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data)?;
        // Windows ConPTY can lose buffered data - flush immediately
        #[cfg(target_os = "windows")]
        self.writer.flush()?;
        Ok(())
    }

    /// Flush any buffered PTY input
    /// Call this once after processing a batch of keyboard events
    pub fn flush_input(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    /// Resize the terminal and notify the PTY
    pub fn resize(&mut self, cols: usize, rows: usize) -> std::io::Result<()> {
        // Resize the grid
        {
            let mut grid = self.grid.lock().expect("terminal grid mutex poisoned");
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

    /// Send pasted text to the terminal, respecting bracketed paste mode
    /// When bracketed paste mode is enabled (?2004), wraps the text with
    /// ESC[200~ (start) and ESC[201~ (end) sequences
    pub fn send_paste(&mut self, text: &str) -> std::io::Result<()> {
        let bracketed_paste_mode = {
            let grid = self.grid.lock().expect("terminal grid mutex poisoned");
            grid.bracketed_paste_mode
        };

        if bracketed_paste_mode {
            // Bracketed paste: wrap with ESC[200~ and ESC[201~
            self.write_input(b"\x1b[200~")?;
            self.write_input(text.as_bytes())?;
            self.write_input(b"\x1b[201~")?;
        } else {
            // Normal paste: send text directly
            self.write_input(text.as_bytes())?;
        }

        // Flush to ensure paste is sent immediately
        self.writer.flush()
    }

    /// Extract terminal content (scrollback + visible lines) for session persistence
    /// Returns at most MAX_LINES_PER_TERMINAL lines (most recent lines are kept)
    pub fn get_terminal_content(&self) -> (Vec<SerializableTerminalLine>, SerializableCursor) {
        let grid = self.grid.lock().expect("terminal grid mutex poisoned");

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
        let terminal_lines: Vec<Vec<super::term_grid::TerminalCell>> = lines
            .iter()
            .map(|line| {
                line.cells
                    .iter()
                    .map(super::term_grid::TerminalCell::from)
                    .collect()
            })
            .collect();

        // Restore content to grid
        let mut grid = self.grid.lock().expect("terminal grid mutex poisoned");
        grid.restore_content(terminal_lines);
        grid.set_cursor(cursor.x, cursor.y, cursor.visible);
    }

    /// Get the name of the foreground process running in the terminal (macOS)
    /// Returns the process name (e.g., "zsh", "vim", "cargo")
    #[cfg(target_os = "macos")]
    pub fn get_foreground_process_name(&self) -> Option<String> {
        // Get the child process PID
        let child_pid = self.child.process_id()?;

        // Use ps to find the foreground process in the process group
        // First, get the process group ID of the child
        let output = Command::new("ps")
            .args(["-o", "tpgid=", "-p", &child_pid.to_string()])
            .output()
            .ok()?;

        let tpgid = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u32>()
            .ok()?;

        // Now get the process name for the foreground process group
        let output = Command::new("ps")
            .args(["-o", "comm=", "-p", &tpgid.to_string()])
            .output()
            .ok()?;

        let process_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if process_name.is_empty() {
            // Fall back to child process name
            let output = Command::new("ps")
                .args(["-o", "comm=", "-p", &child_pid.to_string()])
                .output()
                .ok()?;

            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if name.is_empty() {
                None
            } else {
                // Extract just the binary name from path
                Some(name.rsplit('/').next().unwrap_or(&name).to_string())
            }
        } else {
            // Extract just the binary name from path
            Some(
                process_name
                    .rsplit('/')
                    .next()
                    .unwrap_or(&process_name)
                    .to_string(),
            )
        }
    }

    /// Get the name of the foreground process running in the terminal (Linux)
    #[cfg(target_os = "linux")]
    pub fn get_foreground_process_name(&self) -> Option<String> {
        use std::fs;

        let child_pid = self.child.process_id()?;

        // Read the stat file to get the foreground process group
        let stat_path = format!("/proc/{}/stat", child_pid);
        let stat_content = fs::read_to_string(&stat_path).ok()?;

        // Parse the stat file to get tpgid (field 8, 1-indexed)
        // The stat format is: pid (comm) state ppid pgrp session tty_nr tpgid ...
        // We need to handle comm containing spaces/parentheses
        let comm_end = stat_content.rfind(')')?;
        let after_comm = &stat_content[comm_end + 2..]; // Skip ") "
        let parts: Vec<&str> = after_comm.split_whitespace().collect();

        // After comm: state(0) ppid(1) pgrp(2) session(3) tty_nr(4) tpgid(5)
        if parts.len() < 6 {
            return None;
        }

        let tpgid: u32 = parts[5].parse().ok()?;

        // Get the process name from /proc/[tpgid]/comm
        let comm_path = format!("/proc/{}/comm", tpgid);
        let name = fs::read_to_string(&comm_path)
            .ok()
            .or_else(|| {
                // Fall back to child process
                fs::read_to_string(format!("/proc/{}/comm", child_pid)).ok()
            })?
            .trim()
            .to_string();

        if name.is_empty() { None } else { Some(name) }
    }

    /// Get the name of the foreground process running in the terminal (Windows)
    #[cfg(target_os = "windows")]
    pub fn get_foreground_process_name(&self) -> Option<String> {
        // Get the child process PID
        let child_pid = self.child.process_id()?;

        // Use wmic to get the process name
        // Note: On Windows, we can't easily get the "foreground" process like on Unix
        // So we just return the shell process name (usually cmd.exe or powershell.exe)
        let output = Command::new("wmic")
            .args([
                "process",
                "where",
                &format!("ProcessId={}", child_pid),
                "get",
                "Name",
                "/value",
            ])
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Parse "Name=process.exe" format
        for line in output_str.lines() {
            if let Some(name) = line.strip_prefix("Name=") {
                let name = name.trim();
                if !name.is_empty() {
                    // Remove .exe extension for cleaner display
                    let name = name.strip_suffix(".exe").unwrap_or(name);
                    return Some(name.to_string());
                }
            }
        }

        None
    }

    /// Get the name of the foreground process running in the terminal (FreeBSD)
    ///
    /// FreeBSD supports procfs but it's not mounted by default.
    /// Falls back to using the `ps` command similar to macOS.
    #[cfg(target_os = "freebsd")]
    pub fn get_foreground_process_name(&self) -> Option<String> {
        let child_pid = self.child.process_id()?;

        // First try procfs if mounted (may not be available)
        if let Ok(status) = std::fs::read_to_string(format!("/proc/{}/status", child_pid)) {
            // FreeBSD procfs status file has different format than Linux
            // First line is the process name
            if let Some(first_line) = status.lines().next() {
                let name = first_line.split_whitespace().next()?.to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }

        // Fallback to ps command (similar to macOS)
        let output = Command::new("ps")
            .args(["-o", "tpgid=", "-p", &child_pid.to_string()])
            .output()
            .ok()?;

        let tpgid = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u32>()
            .ok()?;

        let output = Command::new("ps")
            .args(["-o", "comm=", "-p", &tpgid.to_string()])
            .output()
            .ok()?;

        let process_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if process_name.is_empty() {
            // Fall back to child process name
            let output = Command::new("ps")
                .args(["-o", "comm=", "-p", &child_pid.to_string()])
                .output()
                .ok()?;

            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name.rsplit('/').next().unwrap_or(&name).to_string())
            }
        } else {
            Some(
                process_name
                    .rsplit('/')
                    .next()
                    .unwrap_or(&process_name)
                    .to_string(),
            )
        }
    }

    /// Get the name of the foreground process running in the terminal (NetBSD)
    ///
    /// NetBSD has procfs similar to FreeBSD. Falls back to `ps` command.
    #[cfg(target_os = "netbsd")]
    pub fn get_foreground_process_name(&self) -> Option<String> {
        let child_pid = self.child.process_id()?;

        // Try procfs first
        if let Ok(status) = std::fs::read_to_string(format!("/proc/{}/status", child_pid)) {
            if let Some(first_line) = status.lines().next() {
                let name = first_line.split_whitespace().next()?.to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }

        // Fallback to ps command
        let output = Command::new("ps")
            .args(["-o", "tpgid=", "-p", &child_pid.to_string()])
            .output()
            .ok()?;

        let tpgid = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u32>()
            .ok()?;

        let output = Command::new("ps")
            .args(["-o", "comm=", "-p", &tpgid.to_string()])
            .output()
            .ok()?;

        let process_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if process_name.is_empty() {
            let output = Command::new("ps")
                .args(["-o", "comm=", "-p", &child_pid.to_string()])
                .output()
                .ok()?;

            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name.rsplit('/').next().unwrap_or(&name).to_string())
            }
        } else {
            Some(
                process_name
                    .rsplit('/')
                    .next()
                    .unwrap_or(&process_name)
                    .to_string(),
            )
        }
    }

    /// Get the name of the foreground process running in the terminal (OpenBSD)
    ///
    /// OpenBSD does not have procfs. Uses `ps` command exclusively.
    #[cfg(target_os = "openbsd")]
    pub fn get_foreground_process_name(&self) -> Option<String> {
        let child_pid = self.child.process_id()?;

        // Use ps command (no procfs on OpenBSD)
        let output = Command::new("ps")
            .args(["-o", "tpgid=", "-p", &child_pid.to_string()])
            .output()
            .ok()?;

        let tpgid = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u32>()
            .ok()?;

        let output = Command::new("ps")
            .args(["-o", "comm=", "-p", &tpgid.to_string()])
            .output()
            .ok()?;

        let process_name = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if process_name.is_empty() {
            let output = Command::new("ps")
                .args(["-o", "comm=", "-p", &child_pid.to_string()])
                .output()
                .ok()?;

            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if name.is_empty() {
                None
            } else {
                Some(name.rsplit('/').next().unwrap_or(&name).to_string())
            }
        } else {
            Some(
                process_name
                    .rsplit('/')
                    .next()
                    .unwrap_or(&process_name)
                    .to_string(),
            )
        }
    }

    /// Get the name of the foreground process (fallback for other platforms)
    #[cfg(not(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    pub fn get_foreground_process_name(&self) -> Option<String> {
        None
    }
}
