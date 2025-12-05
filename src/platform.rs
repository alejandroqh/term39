//! Platform detection infrastructure
//!
//! Provides centralized platform detection constants and helpers for
//! cross-platform code. Supports Linux, FreeBSD, NetBSD, OpenBSD, macOS, and Windows.

/// Supported platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    FreeBSD,
    NetBSD,
    OpenBSD,
    MacOS,
    Windows,
    Unknown,
}

impl Platform {
    /// Returns true if this is a BSD variant (FreeBSD, NetBSD, or OpenBSD)
    pub const fn is_bsd(&self) -> bool {
        matches!(self, Platform::FreeBSD | Platform::NetBSD | Platform::OpenBSD)
    }

    /// Returns true if this is a Unix-like platform
    pub const fn is_unix(&self) -> bool {
        matches!(
            self,
            Platform::Linux
                | Platform::FreeBSD
                | Platform::NetBSD
                | Platform::OpenBSD
                | Platform::MacOS
        )
    }

    /// Returns the name of the platform
    pub const fn name(&self) -> &'static str {
        match self {
            Platform::Linux => "Linux",
            Platform::FreeBSD => "FreeBSD",
            Platform::NetBSD => "NetBSD",
            Platform::OpenBSD => "OpenBSD",
            Platform::MacOS => "macOS",
            Platform::Windows => "Windows",
            Platform::Unknown => "Unknown",
        }
    }
}

/// Get the current platform at compile time
pub const fn current_platform() -> Platform {
    #[cfg(target_os = "linux")]
    {
        Platform::Linux
    }
    #[cfg(target_os = "freebsd")]
    {
        Platform::FreeBSD
    }
    #[cfg(target_os = "netbsd")]
    {
        Platform::NetBSD
    }
    #[cfg(target_os = "openbsd")]
    {
        Platform::OpenBSD
    }
    #[cfg(target_os = "macos")]
    {
        Platform::MacOS
    }
    #[cfg(target_os = "windows")]
    {
        Platform::Windows
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "macos",
        target_os = "windows"
    )))]
    {
        Platform::Unknown
    }
}

/// Returns true if running on a BSD variant
pub const fn is_bsd() -> bool {
    cfg!(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))
}

/// Returns true if running on FreeBSD
pub const fn is_freebsd() -> bool {
    cfg!(target_os = "freebsd")
}

/// Returns true if running on NetBSD
pub const fn is_netbsd() -> bool {
    cfg!(target_os = "netbsd")
}

/// Returns true if running on OpenBSD
pub const fn is_openbsd() -> bool {
    cfg!(target_os = "openbsd")
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

/// Returns true if the platform supports framebuffer rendering
/// Currently only Linux is supported
pub const fn supports_framebuffer() -> bool {
    cfg!(target_os = "linux")
}

/// Get the default raw mouse device path for the current platform
#[cfg(unix)]
pub fn default_mouse_device() -> Option<&'static str> {
    #[cfg(target_os = "linux")]
    {
        Some("/dev/input/mice")
    }
    #[cfg(target_os = "freebsd")]
    {
        Some("/dev/sysmouse")
    }
    #[cfg(target_os = "netbsd")]
    {
        Some("/dev/wsmouse0")
    }
    #[cfg(target_os = "openbsd")]
    {
        Some("/dev/wsmouse")
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    {
        None
    }
}

#[cfg(not(unix))]
pub fn default_mouse_device() -> Option<&'static str> {
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_platform() {
        let platform = current_platform();
        // Should return a valid platform on any supported system
        assert!(
            matches!(
                platform,
                Platform::Linux
                    | Platform::FreeBSD
                    | Platform::NetBSD
                    | Platform::OpenBSD
                    | Platform::MacOS
                    | Platform::Windows
                    | Platform::Unknown
            ),
            "Expected a valid Platform variant"
        );
    }

    #[test]
    fn test_is_bsd() {
        let platform = current_platform();
        assert_eq!(
            platform.is_bsd(),
            cfg!(any(
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))
        );
    }

    #[test]
    fn test_is_unix() {
        let platform = current_platform();
        assert_eq!(platform.is_unix(), cfg!(unix));
    }

    #[test]
    fn test_platform_name() {
        let platform = current_platform();
        assert!(!platform.name().is_empty());
    }
}
