//! GPM (General Purpose Mouse) control module
//!
//! This module handles detection and disabling of the GPM daemon to prevent
//! cursor conflicts when using raw mouse input in TTY or framebuffer mode.
//!
//! GPM (General Purpose Mouse) is Linux-specific. BSD systems (FreeBSD, NetBSD, OpenBSD)
//! do not have GPM - their kernel drivers handle console mouse input directly.
//! This module provides no-op stubs for BSD platforms.

#[cfg(target_os = "linux")]
use std::io::{self, Write};
#[cfg(target_os = "linux")]
use std::os::unix::net::UnixStream;

#[cfg(target_os = "linux")]
const GPM_SOCKET: &str = "/dev/gpmctl";

/// Check if GPM daemon is running by looking for its control socket
#[cfg(target_os = "linux")]
pub fn is_gpm_running() -> bool {
    std::path::Path::new(GPM_SOCKET).exists()
}

/// BSD systems don't have GPM - kernel handles mouse directly
#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
pub fn is_gpm_running() -> bool {
    false
}

/// Fallback for other non-Linux platforms
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
pub fn is_gpm_running() -> bool {
    false
}

/// GPM connection handle - keeps connection alive to maintain cursor disable
#[cfg(target_os = "linux")]
pub struct GpmConnection {
    _stream: UnixStream,
}

/// Empty struct for BSD systems (no GPM functionality)
#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
pub struct GpmConnection;

/// Empty struct for other non-Linux platforms
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
pub struct GpmConnection;

#[cfg(target_os = "linux")]
impl GpmConnection {
    /// Try to disable GPM's cursor by connecting and sending a null configuration
    ///
    /// The GPM protocol requires sending a Gpm_Connect structure:
    /// - vc (2 bytes): virtual console number (0 = current)
    /// - eventMask (2 bytes): events to receive (0 = none)
    /// - defaultMask (2 bytes): default event mask (0 = none)
    /// - minMod (2 bytes): minimum modifiers (0)
    /// - maxMod (2 bytes): maximum modifiers (0)
    /// - pid (2 bytes): process ID
    ///
    /// By connecting with empty masks, we tell GPM we don't want any events,
    /// which effectively disables its cursor drawing for our VT.
    pub fn disable_cursor() -> io::Result<Self> {
        let mut stream = UnixStream::connect(GPM_SOCKET)?;

        // Get current virtual console number
        let vc = get_current_vc().unwrap_or(0);

        // Build Gpm_Connect structure (little-endian)
        let mut connect_msg = [0u8; 16];

        // vc: current virtual console
        connect_msg[0..2].copy_from_slice(&(vc as u16).to_le_bytes());

        // eventMask: 0 (no events)
        connect_msg[2..4].copy_from_slice(&0u16.to_le_bytes());

        // defaultMask: 0 (no default events)
        connect_msg[4..6].copy_from_slice(&0u16.to_le_bytes());

        // minMod: 0
        connect_msg[6..8].copy_from_slice(&0u16.to_le_bytes());

        // maxMod: 0xFFFF (accept all modifiers, but since eventMask is 0, this doesn't matter)
        connect_msg[8..10].copy_from_slice(&0u16.to_le_bytes());

        // pid: our process ID
        let pid = std::process::id();
        connect_msg[10..14].copy_from_slice(&pid.to_le_bytes());

        stream.write_all(&connect_msg)?;

        Ok(GpmConnection { _stream: stream })
    }
}

/// BSD implementation - no-op since BSD doesn't have GPM
#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
impl GpmConnection {
    pub fn disable_cursor() -> std::io::Result<Self> {
        Ok(GpmConnection)
    }
}

/// Fallback for other non-Linux platforms
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
impl GpmConnection {
    pub fn disable_cursor() -> std::io::Result<Self> {
        Ok(GpmConnection)
    }
}

/// Get the current virtual console number
#[cfg(target_os = "linux")]
fn get_current_vc() -> Option<i32> {
    // Try to get VT number from environment or /sys
    if let Ok(vt) = std::env::var("XDG_VTNR") {
        if let Ok(num) = vt.parse::<i32>() {
            return Some(num);
        }
    }

    // Try reading from /sys/class/tty/tty0/active
    if let Ok(active) = std::fs::read_to_string("/sys/class/tty/tty0/active") {
        let active = active.trim();
        if let Some(num_str) = active.strip_prefix("tty") {
            if let Ok(num) = num_str.parse::<i32>() {
                return Some(num);
            }
        }
    }

    // Default to 0 (current)
    Some(0)
}

/// Try to disable GPM cursor if GPM is running
/// Returns Some(GpmConnection) if GPM was disabled, None if GPM is not running
pub fn try_disable_gpm() -> Option<GpmConnection> {
    if !is_gpm_running() {
        return None;
    }

    match GpmConnection::disable_cursor() {
        Ok(conn) => {
            eprintln!("GPM detected and cursor disabled");
            Some(conn)
        }
        Err(e) => {
            eprintln!(
                "Warning: GPM is running but could not disable cursor: {}",
                e
            );
            eprintln!(
                "You may see duplicate cursors. Consider stopping GPM: sudo systemctl stop gpm"
            );
            None
        }
    }
}
