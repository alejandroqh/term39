//! Platform detection infrastructure
//!
//! Provides centralized platform detection constants and helpers for
//! cross-platform code. Supports Linux, FreeBSD, NetBSD, OpenBSD, macOS, and Windows.

/// Returns true if running on a BSD variant
pub const fn is_bsd() -> bool {
    cfg!(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))
}

/// Returns true if the platform supports raw mouse input in console/TTY mode
pub const fn supports_raw_console_mouse() -> bool {
    cfg!(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))
}

/// Detect if we're running in a console/TTY environment (not a terminal emulator)
/// This is used to determine if raw mouse input should be used
pub fn is_console_environment() -> bool {
    // Check TERM environment variable
    if let Ok(term) = std::env::var("TERM") {
        // Linux console
        if term == "linux" {
            return true;
        }
        // FreeBSD console (syscons/vt)
        if term == "cons25" || term == "xterm" && is_bsd() {
            // On BSD, xterm might be the console - check for tty
            if let Ok(tty) = std::env::var("TTY") {
                if tty.starts_with("/dev/ttyv") || tty.starts_with("/dev/ttyC") {
                    return true;
                }
            }
        }
        // wscons on NetBSD/OpenBSD
        if term == "wsvt25" || term == "vt220" {
            return true;
        }
    }

    // Additional check: if no DISPLAY and no SSH
    let no_display = std::env::var("DISPLAY").is_err();
    let no_ssh = std::env::var("SSH_CONNECTION").is_err();
    let no_wayland = std::env::var("WAYLAND_DISPLAY").is_err();

    // If no graphical environment variables and we're on a supported platform
    if no_display && no_ssh && no_wayland && supports_raw_console_mouse() {
        // Try to detect console by checking tty name
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let stdin = std::io::stdin();
            let fd = stdin.as_raw_fd();

            // Get tty name
            unsafe {
                let name = libc::ttyname(fd);
                if !name.is_null() {
                    let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy();
                    // Linux: /dev/tty1-12
                    if name_str.starts_with("/dev/tty") && !name_str.starts_with("/dev/ttyS") {
                        if let Some(num_str) = name_str.strip_prefix("/dev/tty") {
                            if num_str.chars().all(|c| c.is_ascii_digit()) {
                                return true;
                            }
                        }
                    }
                    // FreeBSD: /dev/ttyv0-15
                    if name_str.starts_with("/dev/ttyv") {
                        return true;
                    }
                    // NetBSD/OpenBSD: /dev/ttyC0-15
                    if name_str.starts_with("/dev/ttyC") {
                        return true;
                    }
                }
            }
        }
    }

    false
}
