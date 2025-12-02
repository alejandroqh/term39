//! PIN-based authentication backend.
//!
//! Provides local PIN authentication using salted SHA-256 hashing.
//! Works without any system dependencies, suitable for musl builds.

use super::{AuthResult, Authenticator};
use sha2::{Digest, Sha256};

/// Maximum PIN length
pub const MAX_PIN_LENGTH: usize = 30;

/// Minimum PIN length
pub const MIN_PIN_LENGTH: usize = 4;

/// PIN-based authenticator for systems without PAM/native auth
pub struct PinAuthenticator {
    /// The stored password hash (SHA-256 as hex)
    pin_hash: String,
    /// The salt used for hashing
    salt: String,
}

impl PinAuthenticator {
    /// Create a new PIN authenticator with the given hash and salt
    pub fn new(pin_hash: String, salt: String) -> Result<Self, String> {
        if pin_hash.is_empty() {
            return Err("PIN hash is required".to_string());
        }
        if salt.is_empty() {
            return Err("Salt is required".to_string());
        }

        Ok(Self { pin_hash, salt })
    }

    /// Hash a PIN with the given salt
    pub fn hash_pin(pin: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(pin.as_bytes());
        let result = hasher.finalize();

        // Convert to hex string
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Validate a PIN (printable ASCII: letters, numbers, symbols)
    pub fn validate_pin(pin: &str) -> Result<(), String> {
        if pin.len() < MIN_PIN_LENGTH {
            return Err(format!(
                "PIN must be at least {} characters",
                MIN_PIN_LENGTH
            ));
        }
        if pin.len() > MAX_PIN_LENGTH {
            return Err(format!("PIN must be at most {} characters", MAX_PIN_LENGTH));
        }
        // Allow printable ASCII characters (0x21-0x7E: letters, numbers, symbols - no spaces)
        if !pin.chars().all(|c| c.is_ascii_graphic()) {
            return Err("PIN must contain printable characters".to_string());
        }
        Ok(())
    }
}

impl Authenticator for PinAuthenticator {
    fn is_available(&self) -> bool {
        // PIN auth is always available if we have a hash
        !self.pin_hash.is_empty()
    }

    fn authenticate(&self, _username: &str, password: &str) -> AuthResult {
        // Hash the provided password with our salt
        let computed_hash = Self::hash_pin(password, &self.salt);

        // Constant-time comparison to prevent timing attacks
        if constant_time_compare(&computed_hash, &self.pin_hash) {
            AuthResult::Success
        } else {
            AuthResult::Failure("Invalid PIN".to_string())
        }
    }

    fn get_current_username(&self) -> Option<String> {
        // For PIN auth, we don't need a username
        // But return the system user for display purposes
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .or_else(|_| std::env::var("LOGNAME"))
            .ok()
    }

    fn system_name(&self) -> &'static str {
        "PIN"
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

/// Securely clear a string from memory
pub fn secure_clear(s: &mut String) {
    // Overwrite with zeros
    unsafe {
        let bytes = s.as_bytes_mut();
        for byte in bytes.iter_mut() {
            std::ptr::write_volatile(byte, 0);
        }
    }
    s.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_pin() {
        let salt = "test_salt";
        let pin = "1234";
        let hash = PinAuthenticator::hash_pin(pin, salt);

        // Hash should be 64 hex characters (256 bits / 4 bits per hex char)
        assert_eq!(hash.len(), 64);

        // Same input should produce same hash
        let hash2 = PinAuthenticator::hash_pin(pin, salt);
        assert_eq!(hash, hash2);

        // Different PIN should produce different hash
        let hash3 = PinAuthenticator::hash_pin("5678", salt);
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_validate_pin() {
        // Valid PINs - letters and numbers
        assert!(PinAuthenticator::validate_pin("1234").is_ok());
        assert!(PinAuthenticator::validate_pin("abcd1234").is_ok());
        assert!(PinAuthenticator::validate_pin("A1B2C3D4").is_ok());

        // Valid PINs - with symbols
        assert!(PinAuthenticator::validate_pin("12-34").is_ok());
        assert!(PinAuthenticator::validate_pin("12@34").is_ok());
        assert!(PinAuthenticator::validate_pin("P@ss!").is_ok());
        assert!(PinAuthenticator::validate_pin("abc#123$").is_ok());

        // Too short
        assert!(PinAuthenticator::validate_pin("123").is_err());

        // Too long
        let long_pin = "a".repeat(31);
        assert!(PinAuthenticator::validate_pin(&long_pin).is_err());

        // Invalid characters - spaces not allowed
        assert!(PinAuthenticator::validate_pin("12 34").is_err());
    }

    #[test]
    fn test_authenticate() {
        let salt = "test_salt";
        let pin = "secret123";
        let hash = PinAuthenticator::hash_pin(pin, salt);

        let auth = PinAuthenticator::new(hash, salt.to_string()).unwrap();

        // Correct PIN should succeed
        assert!(matches!(auth.authenticate("", pin), AuthResult::Success));

        // Wrong PIN should fail
        assert!(matches!(
            auth.authenticate("", "wrong"),
            AuthResult::Failure(_)
        ));
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("hello", "hello"));
        assert!(!constant_time_compare("hello", "world"));
        assert!(!constant_time_compare("hello", "hello1"));
    }
}
