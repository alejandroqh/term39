//! Framebuffer backend for Linux console rendering
//!
//! This module provides direct framebuffer (/dev/fb0) access for rendering
//! to the Linux console with DOS-like text mode switching capabilities.
//!
//! Supported text modes:
//! - 40x25 (16x16 character cells)
//! - 80x25 (8x16 character cells) - default DOS mode
//! - 80x43 (8x11 character cells)
//! - 80x50 (8x8 character cells) - high density mode

#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub mod cli_handlers;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub mod fb_config;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub mod fb_renderer;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub mod fb_setup_window;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub mod font_manager;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub mod setup_wizard;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub mod text_modes;

#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub use fb_renderer::FramebufferRenderer;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub use text_modes::TextMode;

// Re-export mouse input types from input module
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub use crate::input::mouse::{CursorTracker, MouseInputManager as MouseInput};
