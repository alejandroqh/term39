//! Authentication backend abstraction for the lockscreen feature.
//!
//! Provides a cross-platform interface for system authentication using:
//! - PAM on Linux
//! - dscl command on macOS
//! - LogonUser API on Windows
//! - PIN-based local authentication (fallback for musl builds)

#[cfg(target_os = "macos")]
mod macos_auth;
#[cfg(target_os = "linux")]
mod pam_auth;
mod pin_auth;
mod stub_auth;
#[cfg(target_os = "windows")]
mod windows_auth;

pub use pin_auth::{MAX_PIN_LENGTH, MIN_PIN_LENGTH, PinAuthenticator, secure_clear};

use crate::config_manager::LockscreenAuthMode;

/// Result of an authentication attempt
#[derive(Debug, Clone, PartialEq)]
pub enum AuthResult {
    /// Authentication succeeded
    Success,
    /// Authentication failed with a user-facing message
    Failure(String),
    /// System error (auth system unavailable)
    SystemError(String),
}

/// Cross-platform authentication interface
pub trait Authenticator: Send + Sync {
    /// Check if the authentication system is available
    fn is_available(&self) -> bool;

    /// Attempt authentication with username and password
    fn authenticate(&self, username: &str, password: &str) -> AuthResult;

    /// Get the current system username (for auto-fill)
    fn get_current_username(&self) -> Option<String>;

    /// Get display name for the authentication system (for UI/debugging)
    #[allow(dead_code)]
    fn system_name(&self) -> &'static str;
}

/// Check if OS-level authentication is available
pub fn is_os_auth_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        if let Ok(auth) = pam_auth::PamAuthenticator::new() {
            return auth.is_available();
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(auth) = macos_auth::MacOsAuthenticator::new() {
            return auth.is_available();
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(auth) = windows_auth::WindowsAuthenticator::new() {
            return auth.is_available();
        }
    }

    false
}

/// Factory function to create the appropriate authenticator based on mode
pub fn create_authenticator_with_mode(
    mode: LockscreenAuthMode,
    pin_hash: Option<&str>,
    salt: Option<&str>,
) -> Box<dyn Authenticator> {
    match mode {
        LockscreenAuthMode::Pin => {
            if let (Some(hash), Some(s)) = (pin_hash, salt) {
                if let Ok(auth) = PinAuthenticator::new(hash.to_string(), s.to_string()) {
                    return Box::new(auth);
                }
            }
            // Fallback to stub if PIN not configured
            Box::new(stub_auth::StubAuthenticator::new())
        }
        LockscreenAuthMode::OsAuth => create_authenticator(),
    }
}

/// Factory function to create the platform-specific authenticator
pub fn create_authenticator() -> Box<dyn Authenticator> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(auth) = pam_auth::PamAuthenticator::new() {
            if auth.is_available() {
                return Box::new(auth);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(auth) = macos_auth::MacOsAuthenticator::new() {
            if auth.is_available() {
                return Box::new(auth);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(auth) = windows_auth::WindowsAuthenticator::new() {
            if auth.is_available() {
                return Box::new(auth);
            }
        }
    }

    // Fallback to stub (always unavailable)
    Box::new(stub_auth::StubAuthenticator::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_authenticator() {
        let auth = create_authenticator();
        // Should return something (either real auth or stub)
        let _ = auth.system_name();
    }

    #[test]
    fn test_get_username() {
        let auth = create_authenticator();
        // On most systems, should return Some username
        let username = auth.get_current_username();
        // Just ensure it doesn't panic
        let _ = username;
    }
}
