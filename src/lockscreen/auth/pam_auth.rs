//! PAM authentication backend.
//!
//! Supports Linux, FreeBSD, and NetBSD which all have PAM implementations.
//! OpenBSD uses BSD Auth instead (not PAM).

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
use super::{AuthResult, Authenticator};

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
use pam::client::Client as PamClient;

/// PAM-based authenticator for Linux and BSD systems.
#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
pub struct PamAuthenticator {
    service_name: String,
}

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
impl PamAuthenticator {
    /// Create a new PAM authenticator.
    /// Uses "login" as the PAM service name for compatibility.
    pub fn new() -> Result<Self, String> {
        // Verify PAM configuration exists
        if !std::path::Path::new("/etc/pam.d").exists() {
            return Err("PAM configuration directory not found".to_string());
        }

        Ok(Self {
            service_name: "login".to_string(),
        })
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "netbsd"))]
impl Authenticator for PamAuthenticator {
    fn is_available(&self) -> bool {
        // Check if PAM library is available (platform-specific paths)
        #[cfg(target_os = "linux")]
        {
            std::path::Path::new("/lib/x86_64-linux-gnu/libpam.so.0").exists()
                || std::path::Path::new("/lib64/libpam.so.0").exists()
                || std::path::Path::new("/usr/lib/libpam.so").exists()
                || std::path::Path::new("/usr/lib/x86_64-linux-gnu/libpam.so.0").exists()
                || std::path::Path::new("/lib/libpam.so.0").exists()
        }

        #[cfg(target_os = "freebsd")]
        {
            std::path::Path::new("/usr/lib/libpam.so").exists()
                || std::path::Path::new("/usr/lib/libpam.so.6").exists()
        }

        #[cfg(target_os = "netbsd")]
        {
            std::path::Path::new("/usr/lib/libpam.so").exists()
                || std::path::Path::new("/usr/lib/libpam.so.3").exists()
        }
    }

    fn authenticate(&self, username: &str, password: &str) -> AuthResult {
        // Create PAM client
        let mut client = match PamClient::with_password(&self.service_name) {
            Ok(client) => client,
            Err(e) => {
                return AuthResult::SystemError(format!("PAM initialization failed: {}", e));
            }
        };

        // Set credentials
        client
            .conversation_mut()
            .set_credentials(username, password);

        // Attempt authentication
        match client.authenticate() {
            Ok(()) => AuthResult::Success,
            Err(_) => AuthResult::Failure("Invalid username or password".to_string()),
        }
    }

    fn get_current_username(&self) -> Option<String> {
        // Try environment variables first
        std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .ok()
            .or_else(|| {
                // Fallback: use libc getuid + getpwuid
                unsafe {
                    let uid = libc::getuid();
                    let pw = libc::getpwuid(uid);
                    if !pw.is_null() {
                        let name = std::ffi::CStr::from_ptr((*pw).pw_name);
                        return name.to_str().ok().map(|s| s.to_string());
                    }
                }
                None
            })
    }

    fn system_name(&self) -> &'static str {
        "PAM"
    }
}

#[cfg(all(
    test,
    any(target_os = "linux", target_os = "freebsd", target_os = "netbsd")
))]
mod tests {
    use super::*;

    #[test]
    fn test_pam_new() {
        // Should succeed on most systems with PAM
        let result = PamAuthenticator::new();
        // Don't assert success as CI might not have PAM
        let _ = result;
    }

    #[test]
    fn test_get_username() {
        if let Ok(auth) = PamAuthenticator::new() {
            let username = auth.get_current_username();
            // Should return Some on most systems
            assert!(username.is_some() || std::env::var("USER").is_err());
        }
    }
}
