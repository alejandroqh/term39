//! Stub authenticator - fallback when no platform auth is available.

use super::{AuthResult, Authenticator};

/// Stub authenticator that is always unavailable.
/// Used as fallback when PAM/Security Framework/Windows Auth is not available.
pub struct StubAuthenticator;

impl StubAuthenticator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StubAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

impl Authenticator for StubAuthenticator {
    fn is_available(&self) -> bool {
        false
    }

    fn authenticate(&self, _username: &str, _password: &str) -> AuthResult {
        AuthResult::SystemError("Authentication system not available".to_string())
    }

    fn get_current_username(&self) -> Option<String> {
        // Try common environment variables
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .or_else(|_| std::env::var("LOGNAME"))
            .ok()
    }

    fn system_name(&self) -> &'static str {
        "None (Disabled)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_unavailable() {
        let auth = StubAuthenticator::new();
        assert!(!auth.is_available());
    }

    #[test]
    fn test_stub_authenticate_fails() {
        let auth = StubAuthenticator::new();
        let result = auth.authenticate("test", "test");
        assert!(matches!(result, AuthResult::SystemError(_)));
    }

    #[test]
    fn test_stub_system_name() {
        let auth = StubAuthenticator::new();
        assert_eq!(auth.system_name(), "None (Disabled)");
    }
}
