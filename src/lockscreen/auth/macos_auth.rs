//! macOS authentication backend using dscl command.

#[cfg(target_os = "macos")]
use super::{AuthResult, Authenticator};

#[cfg(target_os = "macos")]
use std::process::Command;

/// macOS authenticator using Directory Services (dscl).
#[cfg(target_os = "macos")]
pub struct MacOsAuthenticator;

#[cfg(target_os = "macos")]
impl MacOsAuthenticator {
    /// Create a new macOS authenticator.
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }
}

#[cfg(target_os = "macos")]
impl Authenticator for MacOsAuthenticator {
    fn is_available(&self) -> bool {
        // dscl is always available on macOS
        std::path::Path::new("/usr/bin/dscl").exists()
    }

    fn authenticate(&self, username: &str, password: &str) -> AuthResult {
        // Use dscl to authenticate against local directory
        // dscl . -authonly <username> <password>
        let output = Command::new("/usr/bin/dscl")
            .args([".", "-authonly", username, password])
            .output();

        match output {
            Ok(out) if out.status.success() => AuthResult::Success,
            Ok(_) => AuthResult::Failure("Invalid username or password".to_string()),
            Err(e) => AuthResult::SystemError(format!("Authentication failed: {}", e)),
        }
    }

    fn get_current_username(&self) -> Option<String> {
        std::env::var("USER").ok()
    }

    fn system_name(&self) -> &'static str {
        "Directory Services"
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn test_macos_new() {
        let result = MacOsAuthenticator::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_macos_available() {
        if let Ok(auth) = MacOsAuthenticator::new() {
            // dscl should be available on macOS
            assert!(auth.is_available());
        }
    }

    #[test]
    fn test_get_username() {
        if let Ok(auth) = MacOsAuthenticator::new() {
            let username = auth.get_current_username();
            assert!(username.is_some());
        }
    }
}
