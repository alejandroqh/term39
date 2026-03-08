use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

/// Messages sent from client to daemon
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMsg {
    /// Attach to the persistent session
    Attach { cols: u16, rows: u16 },
    /// Graceful disconnect
    Detach,
    /// Keyboard input for a specific window's PTY
    PtyInput { window_id: u32, data: Vec<u8> },
    /// Request creation of a new terminal window
    CreateWindow {
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        title: String,
        command: Option<String>,
    },
    /// Close a window (and its PTY)
    CloseWindow { window_id: u32 },
    /// Resize a window's PTY
    ResizePty {
        window_id: u32,
        cols: u16,
        rows: u16,
    },
    /// Update a window's geometry (position and size) after move/resize
    UpdateWindowGeometry {
        window_id: u32,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    },
    /// Request full state snapshot
    GetState,
    /// Kill the daemon
    Shutdown,
    /// Heartbeat response
    Pong,
    /// Force-attach: kick any existing client and attach
    ForceAttach { cols: u16, rows: u16 },
}

/// Messages sent from daemon to client
#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonMsg {
    /// Attach succeeded with current session state
    AttachOk { windows: Vec<WindowInfo> },
    /// Attach denied (another client is attached)
    AttachDenied { reason: String },
    /// PTY output bytes for a window
    PtyOutput { window_id: u32, data: Vec<u8> },
    /// A window was successfully created
    WindowCreated { window_id: u32 },
    /// A window's shell exited
    WindowClosed { window_id: u32 },
    /// Full state snapshot
    State { windows: Vec<WindowInfo> },
    /// Error message
    Error { message: String },
    /// Heartbeat ping (client must respond with Pong)
    Ping,
}

/// Information about a daemon-managed window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub window_id: u32,
    pub title: String,
    pub cols: u16,
    pub rows: u16,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Write a length-prefixed JSON message to a stream
pub fn write_message<W: Write, T: Serialize>(stream: &mut W, msg: &T) -> io::Result<()> {
    let json = serde_json::to_vec(msg).map_err(io::Error::other)?;
    let len = json.len() as u32;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&json)?;
    stream.flush()
}

/// Read a length-prefixed JSON message from a stream (blocking)
pub fn read_message<R: Read, T: for<'de> Deserialize<'de>>(stream: &mut R) -> io::Result<T> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    // Sanity check: reject absurdly large messages (16 MB limit)
    if len > 16 * 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("message too large: {} bytes", len),
        ));
    }

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    serde_json::from_slice(&buf).map_err(io::Error::other)
}

/// Try to read a length-prefixed JSON message from a non-blocking stream
/// Returns Ok(None) if no data is available (WouldBlock)
pub fn try_read_message<R: Read, T: for<'de> Deserialize<'de>>(
    stream: &mut R,
    partial_buf: &mut Vec<u8>,
) -> io::Result<Option<T>> {
    // Try to read the length prefix if we don't have it yet
    while partial_buf.len() < 4 {
        let mut byte = [0u8; 1];
        match stream.read(&mut byte) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionReset,
                    "connection closed",
                ));
            }
            Ok(_) => {
                partial_buf.push(byte[0]);
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                return Ok(None);
            }
            Err(e) => return Err(e),
        }
    }

    // We have at least 4 bytes - parse the length
    let len = u32::from_be_bytes([
        partial_buf[0],
        partial_buf[1],
        partial_buf[2],
        partial_buf[3],
    ]) as usize;

    if len > 16 * 1024 * 1024 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("message too large: {} bytes", len),
        ));
    }

    let total_needed = 4 + len;

    // Read remaining data
    while partial_buf.len() < total_needed {
        let mut tmp = [0u8; 4096];
        let remaining = total_needed - partial_buf.len();
        let to_read = remaining.min(tmp.len());
        match stream.read(&mut tmp[..to_read]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionReset,
                    "connection closed",
                ));
            }
            Ok(n) => {
                partial_buf.extend_from_slice(&tmp[..n]);
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                return Ok(None);
            }
            Err(e) => return Err(e),
        }
    }

    // We have the complete message
    let json_data = &partial_buf[4..total_needed];
    let msg: T = serde_json::from_slice(json_data).map_err(io::Error::other)?;
    partial_buf.clear();
    Ok(Some(msg))
}
