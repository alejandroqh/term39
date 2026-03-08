use super::protocol::{
    ClientMsg, DaemonMsg, WindowInfo, read_message, try_read_message, write_message,
};
use super::socket;
use crate::term_emu::ShellConfig;
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::io::{self, BufWriter, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread;
use std::time::Duration;

/// Maximum replay buffer size per window (64KB should reconstruct most terminal states)
const REPLAY_BUFFER_CAPACITY: usize = 64 * 1024;

/// A PTY-owning window managed by the daemon
struct DaemonWindow {
    window_id: u32,
    title: String,
    cols: u16,
    rows: u16,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    pty_master: Box<dyn MasterPty + Send>,
    writer: BufWriter<Box<dyn Write + Send>>,
    child: Box<dyn Child + Send>,
    rx: Receiver<Vec<u8>>,
    /// Ring buffer of recent PTY output for replaying on reattach
    replay_buffer: Vec<u8>,
}

impl DaemonWindow {
    fn to_info(&self) -> WindowInfo {
        WindowInfo {
            window_id: self.window_id,
            title: self.title.clone(),
            cols: self.cols,
            rows: self.rows,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        }
    }

    /// Append data to the replay buffer, keeping it under the capacity limit
    fn record_output(&mut self, data: &[u8]) {
        if data.len() >= REPLAY_BUFFER_CAPACITY {
            // Data is larger than buffer - just keep the tail
            self.replay_buffer.clear();
            self.replay_buffer
                .extend_from_slice(&data[data.len() - REPLAY_BUFFER_CAPACITY..]);
        } else if self.replay_buffer.len() + data.len() > REPLAY_BUFFER_CAPACITY {
            // Would exceed capacity - drop oldest data
            let overflow = self.replay_buffer.len() + data.len() - REPLAY_BUFFER_CAPACITY;
            self.replay_buffer.drain(..overflow);
            self.replay_buffer.extend_from_slice(data);
        } else {
            self.replay_buffer.extend_from_slice(data);
        }
    }
}

/// Write a message to the daemon log file (for debugging)
fn daemon_log(msg: &str) {
    if let Ok(dir) = socket::persist_dir() {
        let log_path = dir.join("daemon.log");
        use std::fs::OpenOptions;
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(f, "[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg);
        }
    }
}

/// Run the daemon event loop. This never returns under normal operation.
pub fn run_daemon(shell_config: &ShellConfig) -> ! {
    // Set up signal handlers
    setup_signal_handlers();

    // Clean up any stale socket
    if socket::socket_exists() && !socket::is_daemon_alive() {
        socket::cleanup_files();
    }

    // Write lock file with our PID
    let pid = unsafe { libc::getpid() };
    daemon_log(&format!("daemon starting, pid={}", pid));
    if let Err(e) = socket::write_lock_file(pid) {
        daemon_log(&format!("failed to write lock file: {}", e));
        std::process::exit(1);
    }

    // Create Unix socket listener
    let sock_path = match socket::socket_path() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("daemon: failed to get socket path: {}", e);
            std::process::exit(1);
        }
    };

    // Remove stale socket file if it exists
    let _ = std::fs::remove_file(&sock_path);

    let listener = match UnixListener::bind(&sock_path) {
        Ok(l) => l,
        Err(e) => {
            daemon_log(&format!("failed to bind socket: {}", e));
            socket::cleanup_files();
            std::process::exit(1);
        }
    };

    daemon_log(&format!("listening on {:?}", sock_path));

    // Set listener to non-blocking
    listener
        .set_nonblocking(true)
        .expect("daemon: failed to set listener non-blocking");

    let mut windows: Vec<DaemonWindow> = Vec::new();
    let mut next_id: u32 = 1;
    let mut client: Option<UnixStream> = None;
    let mut client_read_buf: Vec<u8> = Vec::new();
    let shell_config = shell_config.clone();
    let mut had_client = false;

    loop {
        // Check for new connections
        match listener.accept() {
            Ok((stream, _addr)) => {
                handle_new_connection(stream, &mut client, &mut client_read_buf, &windows);
                if client.is_some() {
                    had_client = true;
                    // Replay buffered PTY output to reconstruct terminal state
                    // on the new client (programs like vim/btop don't redraw
                    // unless the content has changed)
                    if let Some(ref mut stream) = client {
                        for window in &windows {
                            if !window.replay_buffer.is_empty() {
                                let msg = DaemonMsg::PtyOutput {
                                    window_id: window.window_id,
                                    data: window.replay_buffer.clone(),
                                };
                                let _ = write_to_client(stream, &msg);
                            }
                        }
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No new connections
            }
            Err(e) => {
                eprintln!("daemon: accept error: {}", e);
            }
        }

        // Read client messages
        if let Some(ref mut stream) = client {
            loop {
                match try_read_message::<_, ClientMsg>(stream, &mut client_read_buf) {
                    Ok(Some(msg)) => {
                        let action = handle_client_msg(
                            msg,
                            &mut windows,
                            &mut next_id,
                            &shell_config,
                            stream,
                        );
                        match action {
                            ClientAction::Continue => {}
                            ClientAction::Detach => {
                                daemon_log(&format!(
                                    "client detached, {} windows alive",
                                    windows.len()
                                ));
                                client = None;
                                client_read_buf.clear();
                                break;
                            }
                            ClientAction::Shutdown => {
                                daemon_log("shutdown requested");
                                socket::cleanup_files();
                                std::process::exit(0);
                            }
                        }
                    }
                    Ok(None) => break, // No more data
                    Err(e) => {
                        // Client disconnected
                        if e.kind() == io::ErrorKind::ConnectionReset
                            || e.kind() == io::ErrorKind::BrokenPipe
                            || e.kind() == io::ErrorKind::UnexpectedEof
                        {
                            daemon_log(&format!(
                                "client disconnected ({}), {} windows alive",
                                e.kind(),
                                windows.len()
                            ));
                            client = None;
                            client_read_buf.clear();
                        }
                        break;
                    }
                }
            }
        }

        // Drain PTY output, record in replay buffer, and forward to client
        let mut closed_windows = Vec::new();
        for window in &mut windows {
            loop {
                match window.rx.try_recv() {
                    Ok(data) => {
                        // Always record in replay buffer (for reattach)
                        window.record_output(&data);
                        if let Some(ref mut stream) = client {
                            let msg = DaemonMsg::PtyOutput {
                                window_id: window.window_id,
                                data,
                            };
                            if write_to_client(stream, &msg).is_err() {
                                // Client write failed, disconnect
                                daemon_log("client write failed, disconnecting");
                                client = None;
                                client_read_buf.clear();
                                break;
                            }
                        }
                        // If no client, data is simply dropped.
                        // PTY kernel buffer provides back-pressure.
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // PTY reader died - shell exited
                        closed_windows.push(window.window_id);
                        break;
                    }
                }
            }
        }

        // Also check child exit status for windows not caught by channel disconnect
        for window in &mut windows {
            if !closed_windows.contains(&window.window_id) {
                if let Ok(Some(_)) = window.child.try_wait() {
                    closed_windows.push(window.window_id);
                }
            }
        }

        // Notify client about closed windows and remove them
        for wid in &closed_windows {
            if let Some(ref mut stream) = client {
                let msg = DaemonMsg::WindowClosed { window_id: *wid };
                let _ = write_to_client(stream, &msg);
            }
            windows.retain(|w| w.window_id != *wid);
        }

        // Auto-shutdown: exit daemon when no windows remain, no client connected,
        // and at least one client has connected before (avoid shutdown before first attach)
        if windows.is_empty() && client.is_none() && had_client {
            daemon_log("no windows and no client, shutting down");
            socket::cleanup_files();
            std::process::exit(0);
        }

        // Sleep briefly to avoid busy-waiting
        thread::sleep(Duration::from_millis(1));
    }
}

fn handle_new_connection(
    stream: UnixStream,
    current_client: &mut Option<UnixStream>,
    client_read_buf: &mut Vec<u8>,
    windows: &[DaemonWindow],
) {
    // Set non-blocking for client reads
    if let Err(e) = stream.set_nonblocking(true) {
        eprintln!("daemon: failed to set client non-blocking: {}", e);
        return;
    }

    // Read the first message (must be Attach)
    // Temporarily set blocking for the initial handshake
    let _ = stream.set_nonblocking(false);
    let mut blocking_stream = stream;
    blocking_stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .ok();

    let msg: ClientMsg = match read_message(&mut blocking_stream) {
        Ok(m) => m,
        Err(_) => return,
    };

    match msg {
        ClientMsg::Attach { .. } => {
            if current_client.is_some() {
                daemon_log("client attach denied: another client attached");
                let deny = DaemonMsg::AttachDenied {
                    reason: "Another client is already attached".to_string(),
                };
                let _ = write_message(&mut blocking_stream, &deny);
                return;
            }

            // Accept this client
            let window_infos: Vec<WindowInfo> = windows.iter().map(|w| w.to_info()).collect();
            daemon_log(&format!("client attached, {} windows", window_infos.len()));
            let ok = DaemonMsg::AttachOk {
                windows: window_infos,
            };
            if write_message(&mut blocking_stream, &ok).is_err() {
                return;
            }

            // Switch to non-blocking for ongoing communication
            let _ = blocking_stream.set_nonblocking(true);
            let _ = blocking_stream.set_read_timeout(None);
            *current_client = Some(blocking_stream);
            client_read_buf.clear();
        }
        _ => {
            // First message must be Attach
            let err = DaemonMsg::Error {
                message: "First message must be Attach".to_string(),
            };
            let _ = write_message(&mut blocking_stream, &err);
        }
    }
}

enum ClientAction {
    Continue,
    Detach,
    Shutdown,
}

fn handle_client_msg(
    msg: ClientMsg,
    windows: &mut Vec<DaemonWindow>,
    next_id: &mut u32,
    shell_config: &ShellConfig,
    client_stream: &mut UnixStream,
) -> ClientAction {
    match msg {
        ClientMsg::Attach { .. } => {
            // Already attached, ignore duplicate
            ClientAction::Continue
        }
        ClientMsg::Detach => ClientAction::Detach,
        ClientMsg::Shutdown => ClientAction::Shutdown,
        ClientMsg::PtyInput { window_id, data } => {
            if let Some(window) = windows.iter_mut().find(|w| w.window_id == window_id) {
                let _ = window.writer.write_all(&data);
                let _ = window.writer.flush();
            }
            ClientAction::Continue
        }
        ClientMsg::CreateWindow {
            x,
            y,
            width,
            height,
            title,
            command,
        } => {
            let window_id = *next_id;
            *next_id += 1;

            // Calculate content dimensions (matching TerminalWindow logic)
            let content_width = width.saturating_sub(4).max(1);
            let content_height = height.saturating_sub(2).max(1);

            match create_daemon_window(
                window_id,
                x,
                y,
                width,
                height,
                content_width,
                content_height,
                title,
                command,
                shell_config,
            ) {
                Ok(daemon_window) => {
                    daemon_log(&format!("window {} created", window_id));
                    windows.push(daemon_window);
                    let msg = DaemonMsg::WindowCreated { window_id };
                    let _ = write_to_client(client_stream, &msg);
                }
                Err(e) => {
                    daemon_log(&format!("window create failed: {}", e));
                    let msg = DaemonMsg::Error {
                        message: format!("Failed to create window: {}", e),
                    };
                    let _ = write_to_client(client_stream, &msg);
                }
            }
            ClientAction::Continue
        }
        ClientMsg::CloseWindow { window_id } => {
            windows.retain(|w| w.window_id != window_id);
            let msg = DaemonMsg::WindowClosed { window_id };
            let _ = write_to_client(client_stream, &msg);
            ClientAction::Continue
        }
        ClientMsg::ResizePty {
            window_id,
            cols,
            rows,
        } => {
            if let Some(window) = windows.iter_mut().find(|w| w.window_id == window_id) {
                window.cols = cols;
                window.rows = rows;
                let _ = window.pty_master.resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                });
            }
            ClientAction::Continue
        }
        ClientMsg::GetState => {
            let window_infos: Vec<WindowInfo> = windows.iter().map(|w| w.to_info()).collect();
            let msg = DaemonMsg::State {
                windows: window_infos,
            };
            let _ = write_to_client(client_stream, &msg);
            ClientAction::Continue
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn create_daemon_window(
    window_id: u32,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    content_width: u16,
    content_height: u16,
    title: String,
    command: Option<String>,
    shell_config: &ShellConfig,
) -> io::Result<DaemonWindow> {
    let pty_system = native_pty_system();

    let pty_pair = pty_system
        .openpty(PtySize {
            rows: content_height,
            cols: content_width,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(io::Error::other)?;

    // Build command
    let mut cmd = if let Some(ref cmd_str) = command {
        let parts = parse_command(cmd_str);
        let mut cmd = CommandBuilder::new(&parts.0);
        for arg in &parts.1 {
            cmd.arg(arg);
        }
        cmd
    } else if let Some(ref shell_path) = shell_config.shell_path {
        if std::path::Path::new(shell_path).exists() {
            CommandBuilder::new(shell_path)
        } else {
            CommandBuilder::new_default_prog()
        }
    } else {
        CommandBuilder::new_default_prog()
    };

    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    cmd.env("PROMPT_EOL_MARK", "");
    cmd.env("PROMPT_SP", "");

    let child = pty_pair
        .slave
        .spawn_command(cmd)
        .map_err(io::Error::other)?;

    let pty_master = pty_pair.master;
    let mut reader = pty_master.try_clone_reader().map_err(io::Error::other)?;
    let writer = BufWriter::new(pty_master.take_writer().map_err(io::Error::other)?);

    let (tx, rx): (SyncSender<Vec<u8>>, Receiver<Vec<u8>>) = sync_channel(64);

    thread::spawn(move || {
        let mut buffer = vec![0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buffer[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok(DaemonWindow {
        window_id,
        title,
        cols: content_width,
        rows: content_height,
        x,
        y,
        width,
        height,
        pty_master,
        writer,
        child,
        rx,
        replay_buffer: Vec::new(),
    })
}

/// Parse a command string into (program, args) - mirrors TerminalWindow::parse_command
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
        ("sh".to_string(), vec![])
    } else {
        let program = parts[0].clone();
        let args = parts.into_iter().skip(1).collect();
        (program, args)
    }
}

/// Set up signal handlers for graceful daemon shutdown
fn setup_signal_handlers() {
    unsafe {
        // SIGHUP - terminal hangup (clean up and exit)
        libc::signal(
            libc::SIGHUP,
            handle_signal as *const () as libc::sighandler_t,
        );
        // SIGTERM - termination request (clean up and exit)
        libc::signal(
            libc::SIGTERM,
            handle_signal as *const () as libc::sighandler_t,
        );
        // SIGPIPE - ignore broken pipe (client disconnect)
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }
}

/// Write a message to the client socket, temporarily switching to blocking mode.
/// Non-blocking sockets fail write_all with WouldBlock when the buffer is full
/// (e.g., heavy PTY output). Blocking mode with a timeout prevents this from
/// being treated as a disconnect while still bounding the wait time.
fn write_to_client<T: serde::Serialize>(stream: &mut UnixStream, msg: &T) -> io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let result = write_message(stream, msg);
    stream.set_write_timeout(None)?;
    stream.set_nonblocking(true)?;
    result
}

extern "C" fn handle_signal(sig: libc::c_int) {
    // Log signal before cleanup (best effort, signal handler context)
    if let Ok(dir) = socket::persist_dir() {
        let log_path = dir.join("daemon.log");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        {
            let _ = writeln!(f, "daemon received signal {}, exiting", sig);
        }
    }
    socket::cleanup_files();
    std::process::exit(0);
}
