//! Linux PAM authentication backend.

#[cfg(target_os = "linux")]
use super::{AuthResult, Authenticator};

#[cfg(target_os = "linux")]
use pam::Authenticator as PamAuth;

/// PAM-based authenticator for Linux systems.
#[cfg(target_os = "linux")]
pub struct PamAuthenticator {
    service_name: String,
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
impl Authenticator for PamAuthenticator {
    fn is_available(&self) -> bool {
        // Check if PAM library is available
        std::path::Path::new("/lib/x86_64-linux-gnu/libpam.so.0").exists()
            || std::path::Path::new("/lib64/libpam.so.0").exists()
            || std::path::Path::new("/usr/lib/libpam.so").exists()
            || std::path::Path::new("/usr/lib/x86_64-linux-gnu/libpam.so.0").exists()
            || std::path::Path::new("/lib/libpam.so.0").exists()
    }

    fn authenticate(&self, username: &str, password: &str) -> AuthResult {
        // Create PAM authenticator
        let mut auth = match PamAuth::with_password(&self.service_name) {
            Ok(auth) => auth,
            Err(e) => {
                return AuthResult::SystemError(format!("PAM initialization failed: {}", e));
            }
        };

        // Set credentials
        auth.get_handler().set_credentials(username, password);

        // Attempt authentication
        match auth.authenticate() {
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

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn test_pam_new() {
        // Should succeed on most Linux systems
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
