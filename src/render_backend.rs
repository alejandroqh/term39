//! Rendering backend abstraction
//!
//! This module provides an abstraction over different rendering backends:
//! - Terminal backend: Uses crossterm for cross-platform terminal rendering
//! - Framebuffer backend: Uses direct Linux framebuffer for DOS-like modes

use crate::video_buffer::VideoBuffer;
use std::io;

/// Render backend trait for abstracting terminal vs framebuffer rendering
pub trait RenderBackend {
    /// Present the video buffer to the screen
    fn present(&mut self, buffer: &mut VideoBuffer) -> io::Result<()>;

    /// Get the dimensions of the rendering surface (cols, rows)
    fn dimensions(&self) -> (u16, u16);

    /// Check if the backend has been resized
    fn check_resize(&mut self) -> io::Result<Option<(u16, u16)>>;
}

/// Terminal-based rendering backend (using crossterm)
pub struct TerminalBackend {
    cols: u16,
    rows: u16,
    stdout: io::Stdout,
}

impl TerminalBackend {
    /// Create a new terminal backend
    pub fn new() -> io::Result<Self> {
        use crossterm::terminal;

        let (cols, rows) = terminal::size()?;
        let stdout = io::stdout();

        Ok(Self { cols, rows, stdout })
    }
}

impl RenderBackend for TerminalBackend {
    fn present(&mut self, buffer: &mut VideoBuffer) -> io::Result<()> {
        buffer.present(&mut self.stdout)
    }

    fn dimensions(&self) -> (u16, u16) {
        (self.cols, self.rows)
    }

    fn check_resize(&mut self) -> io::Result<Option<(u16, u16)>> {
        use crossterm::terminal;

        let (new_cols, new_rows) = terminal::size()?;

        if new_cols != self.cols || new_rows != self.rows {
            self.cols = new_cols;
            self.rows = new_rows;
            Ok(Some((new_cols, new_rows)))
        } else {
            Ok(None)
        }
    }
}

/// Framebuffer-based rendering backend (Linux console only)
#[cfg(feature = "framebuffer-backend")]
pub struct FramebufferBackend {
    renderer: crate::framebuffer::FramebufferRenderer,
}

#[cfg(feature = "framebuffer-backend")]
impl FramebufferBackend {
    /// Create a new framebuffer backend with specified text mode and optional scale
    pub fn new(mode: crate::framebuffer::TextMode, scale: Option<usize>) -> io::Result<Self> {
        let renderer = crate::framebuffer::FramebufferRenderer::new(mode, scale)?;

        Ok(Self { renderer })
    }
}

#[cfg(feature = "framebuffer-backend")]
impl RenderBackend for FramebufferBackend {
    fn present(&mut self, buffer: &mut VideoBuffer) -> io::Result<()> {
        self.renderer.render_buffer(buffer);
        Ok(())
    }

    fn dimensions(&self) -> (u16, u16) {
        let (cols, rows) = self.renderer.dimensions();
        (cols as u16, rows as u16)
    }

    fn check_resize(&mut self) -> io::Result<Option<(u16, u16)>> {
        // Framebuffer doesn't resize - mode is fixed
        Ok(None)
    }
}
