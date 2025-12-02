//! Windows authentication backend using LogonUser API.

#[cfg(target_os = "windows")]
use super::{AuthResult, Authenticator};

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Security::{
    LOGON32_LOGON_INTERACTIVE, LOGON32_PROVIDER_DEFAULT, LogonUserW,
};

/// Windows authenticator using LogonUser API.
#[cfg(target_os = "windows")]
pub struct WindowsAuthenticator;

#[cfg(target_os = "windows")]
impl WindowsAuthenticator {
    /// Create a new Windows authenticator.
    pub fn new() -> Result<Self, String> {
        Ok(Self)
    }

    /// Convert a Rust string to a null-terminated wide string.
    fn to_wide_string(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }
}

#[cfg(target_os = "windows")]
impl Authenticator for WindowsAuthenticator {
    fn is_available(&self) -> bool {
        // LogonUser is always available on Windows
        true
    }

    fn authenticate(&self, username: &str, password: &str) -> AuthResult {
        let username_wide = Self::to_wide_string(username);
        let password_wide = Self::to_wide_string(password);
        // Use "." for local machine domain
        let domain_wide = Self::to_wide_string(".");

        let mut token: HANDLE = std::ptr::null_mut();

        let result = unsafe {
            LogonUserW(
                username_wide.as_ptr(),
                domain_wide.as_ptr(),
                password_wide.as_ptr(),
                LOGON32_LOGON_INTERACTIVE,
                LOGON32_PROVIDER_DEFAULT,
                &mut token,
            )
        };

        if result != 0 {
            // Success - close the token handle
            if !token.is_null() {
                unsafe {
                    CloseHandle(token);
                }
            }
            AuthResult::Success
        } else {
            AuthResult::Failure("Invalid username or password".to_string())
        }
    }

    fn get_current_username(&self) -> Option<String> {
        std::env::var("USERNAME").ok()
    }

    fn system_name(&self) -> &'static str {
        "Windows Security"
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn test_windows_new() {
        let result = WindowsAuthenticator::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_windows_available() {
        if let Ok(auth) = WindowsAuthenticator::new() {
            assert!(auth.is_available());
        }
    }

    #[test]
    fn test_get_username() {
        if let Ok(auth) = WindowsAuthenticator::new() {
            let username = auth.get_current_username();
            assert!(username.is_some());
        }
    }
}
