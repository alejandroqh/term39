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

#[cfg(feature = "framebuffer-backend")]
pub mod fb_renderer;
#[cfg(feature = "framebuffer-backend")]
pub mod font_manager;
#[cfg(feature = "framebuffer-backend")]
pub mod mouse_input;
#[cfg(feature = "framebuffer-backend")]
pub mod text_modes;

#[cfg(feature = "framebuffer-backend")]
pub use fb_renderer::FramebufferRenderer;
#[cfg(feature = "framebuffer-backend")]
pub use mouse_input::{CursorTracker, MouseInput};
#[cfg(feature = "framebuffer-backend")]
pub use text_modes::TextMode;
