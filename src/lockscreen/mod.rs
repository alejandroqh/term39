//! Lockscreen module for term39.
//!
//! Provides a system-authenticated lockscreen that can be triggered by:
//! - Shift+Q keyboard shortcut
//! - SIGUSR1 signal (Unix only) for external triggers (e.g., laptop lid close)

pub mod auth;
#[allow(clippy::module_inception)]
mod lockscreen;
mod pin_setup;

pub use lockscreen::LockScreen;
pub use pin_setup::{PinSetupDialog, PinSetupState};

// Signal handler for external lock trigger (Unix only)
#[cfg(unix)]
pub mod signal_handler {
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Atomic flag set by SIGUSR1 signal handler
    pub static LOCK_REQUESTED: AtomicBool = AtomicBool::new(false);

    /// Set up the SIGUSR1 signal handler for external lock triggering.
    /// Call this once during application initialization.
    pub fn setup() {
        unsafe {
            libc::signal(libc::SIGUSR1, handle_sigusr1 as libc::sighandler_t);
        }
    }

    /// Signal handler function - sets the atomic flag
    extern "C" fn handle_sigusr1(_: libc::c_int) {
        LOCK_REQUESTED.store(true, Ordering::SeqCst);
    }

    /// Check if a lock was requested via signal and clear the flag.
    /// Returns true if SIGUSR1 was received since last check.
    pub fn check_and_clear() -> bool {
        LOCK_REQUESTED.swap(false, Ordering::SeqCst)
    }
}

// Stub signal handler for non-Unix platforms
#[cfg(not(unix))]
pub mod signal_handler {
    /// No-op setup for non-Unix platforms
    pub fn setup() {}

    /// Always returns false on non-Unix platforms
    pub fn check_and_clear() -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_handler_setup() {
        // Should not panic
        signal_handler::setup();
    }

    #[test]
    fn test_signal_handler_check_and_clear() {
        // Should return false initially
        assert!(!signal_handler::check_and_clear());
    }
}
