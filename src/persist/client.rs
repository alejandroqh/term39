#![allow(dead_code)]

use super::protocol::{
    ClientMsg, DaemonMsg, WindowInfo, read_message, try_read_message, write_message,
};
use super::socket;
use std::io;
use std::os::unix::net::UnixStream;
use std::time::Duration;

/// Client connection to the persist daemon
pub struct PersistClient {
    stream: UnixStream,
    read_buf: Vec<u8>,
}

/// Result of attempting to connect to or start a daemon
pub enum ConnectResult {
    /// Successfully connected to an existing daemon
    Connected(PersistClient, Vec<WindowInfo>),
    /// Connection was denied (another client is attached)
    Denied(String),
    /// No daemon is running
    NoDaemon,
}

impl PersistClient {
    /// Try to connect to an existing daemon.
    /// If `force` is true, sends ForceAttach to kick any existing client.
    pub fn connect(cols: u16, rows: u16, force: bool) -> io::Result<ConnectResult> {
        let sock_path = socket::socket_path()?;

        if !sock_path.exists() {
            return Ok(ConnectResult::NoDaemon);
        }

        // Try to connect
        let mut stream = match UnixStream::connect(&sock_path) {
            Ok(s) => s,
            Err(e) => {
                if e.kind() == io::ErrorKind::ConnectionRefused
                    || e.kind() == io::ErrorKind::NotFound
                {
                    // Socket exists but no one is listening - stale
                    socket::cleanup_files();
                    return Ok(ConnectResult::NoDaemon);
                }
                return Err(e);
            }
        };

        // Set timeout for handshake
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(5)))?;

        // Send Attach or ForceAttach message
        let msg = if force {
            ClientMsg::ForceAttach { cols, rows }
        } else {
            ClientMsg::Attach { cols, rows }
        };
        write_message(&mut stream, &msg)?;

        // Read response
        let response: DaemonMsg = read_message(&mut stream)?;

        match response {
            DaemonMsg::AttachOk { windows } => {
                // Switch to non-blocking for ongoing communication
                stream.set_nonblocking(true)?;
                stream.set_read_timeout(None)?;
                stream.set_write_timeout(None)?;

                let client = PersistClient {
                    stream,
                    read_buf: Vec::new(),
                };
                Ok(ConnectResult::Connected(client, windows))
            }
            DaemonMsg::AttachDenied { reason } => Ok(ConnectResult::Denied(reason)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected response from daemon",
            )),
        }
    }

    /// Send a message to the daemon
    pub fn send(&mut self, msg: &ClientMsg) -> io::Result<()> {
        // Temporarily set blocking for writes
        self.stream.set_nonblocking(false)?;
        let result = write_message(&mut self.stream, msg);
        // If restoring non-blocking fails, return error so caller knows
        // the socket is in a bad state rather than silently continuing
        self.stream.set_nonblocking(true)?;
        result
    }

    /// Try to receive a message from the daemon (non-blocking)
    pub fn try_recv(&mut self) -> io::Result<Option<DaemonMsg>> {
        try_read_message(&mut self.stream, &mut self.read_buf)
    }

    /// Send PTY input for a specific window
    pub fn send_pty_input(&mut self, window_id: u32, data: &[u8]) -> io::Result<()> {
        self.send(&ClientMsg::PtyInput {
            window_id,
            data: data.to_vec(),
        })
    }

    /// Request creation of a new window
    pub fn request_create_window(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        title: String,
        command: Option<String>,
    ) -> io::Result<()> {
        self.send(&ClientMsg::CreateWindow {
            x,
            y,
            width,
            height,
            title,
            command,
        })
    }

    /// Request closing a window
    pub fn request_close_window(&mut self, window_id: u32) -> io::Result<()> {
        self.send(&ClientMsg::CloseWindow { window_id })
    }

    /// Notify daemon of PTY resize
    pub fn send_resize_pty(&mut self, window_id: u32, cols: u16, rows: u16) -> io::Result<()> {
        self.send(&ClientMsg::ResizePty {
            window_id,
            cols,
            rows,
        })
    }

    /// Send detach message (graceful disconnect)
    pub fn detach(&mut self) -> io::Result<()> {
        self.send(&ClientMsg::Detach)
    }

    /// Send shutdown message (kill daemon)
    pub fn shutdown(&mut self) -> io::Result<()> {
        self.send(&ClientMsg::Shutdown)
    }
}
