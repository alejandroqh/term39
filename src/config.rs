/// Application configuration constants
///
/// This module provides centralized configuration for the application.
/// Values are loaded at compile-time from Cargo.toml and can be extended
/// to support runtime configuration files in the future.
/// Application version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name from Cargo.toml
#[allow(dead_code)]
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

/// Application authors from Cargo.toml
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Application repository URL from Cargo.toml
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Application description from Cargo.toml
#[allow(dead_code)]
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
