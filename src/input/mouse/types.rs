//! Shared types for mouse input across all platforms

/// Mouse input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants used on Linux/BSD only
pub enum MouseInputMode {
    /// Terminal emulator - uses crossterm ANSI mouse protocol
    TerminalEmulator,
    /// Linux console TTY - uses raw /dev/input with inverted cell cursor
    Tty,
    /// Framebuffer mode - uses raw /dev/input with sprite cursor
    Framebuffer,
}

impl MouseInputMode {
    /// Detect the appropriate mouse input mode based on environment
    #[allow(dead_code)] // Used on Linux/BSD only
    pub fn detect(framebuffer_requested: bool) -> Self {
        let term = std::env::var("TERM").unwrap_or_default();

        // Linux console
        if term == "linux" {
            if framebuffer_requested {
                return MouseInputMode::Framebuffer;
            } else {
                return MouseInputMode::Tty;
            }
        }

        // FreeBSD console (syscons/vt)
        #[cfg(target_os = "freebsd")]
        {
            if term == "cons25" || term == "xterm" {
                // Check if we're on a virtual console
                if let Ok(tty) = std::env::var("TTY") {
                    if tty.starts_with("/dev/ttyv") {
                        return MouseInputMode::Tty;
                    }
                }
            }
        }

        // NetBSD/OpenBSD wscons console
        #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
        {
            if term == "wsvt25" || term == "vt220" {
                return MouseInputMode::Tty;
            }
            // Check if we're on a wscons console
            if let Ok(tty) = std::env::var("TTY") {
                if tty.starts_with("/dev/ttyC") || tty.starts_with("/dev/ttyE") {
                    return MouseInputMode::Tty;
                }
            }
        }

        MouseInputMode::TerminalEmulator
    }

    /// Returns true if this mode uses raw mouse input (not crossterm)
    #[cfg_attr(
        not(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )),
        allow(dead_code)
    )]
    pub fn uses_raw_input(&self) -> bool {
        matches!(self, MouseInputMode::Tty | MouseInputMode::Framebuffer)
    }
}

/// Mouse input protocol type (Linux-specific)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Used on Linux only
pub enum Protocol {
    /// PS/2 protocol (/dev/input/mice)
    Ps2,
    /// Linux input event protocol (/dev/input/eventX)
    InputEvent,
}

/// Mouse button states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

/// Raw mouse event with deltas
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Fields used on Linux/BSD only
pub struct RawMouseEvent {
    pub dx: i8,
    pub dy: i8,
    pub buttons: MouseButtons,
    pub scroll: i8,
    pub scroll_h: i8,
}
