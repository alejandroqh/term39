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

    /// Scale mouse coordinates from TTY/input space to backend space
    /// For terminal backend: returns coordinates as-is
    /// For framebuffer backend: scales from TTY dimensions to framebuffer dimensions
    fn scale_mouse_coords(&self, col: u16, row: u16) -> (u16, u16) {
        (col, row) // Default: no scaling
    }

    /// Update cursor from raw input (framebuffer mode only)
    /// Returns true if cursor moved
    fn update_cursor(&mut self) -> bool {
        false // Default: no cursor updates
    }

    /// Draw cursor on screen (framebuffer mode only)
    fn draw_cursor(&mut self) {
        // Default: no-op
    }

    /// Restore cursor area (framebuffer mode only)
    fn restore_cursor_area(&mut self) {
        // Default: no-op
    }
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
    tty_cols: u16, // Actual TTY dimensions for mouse coordinate scaling
    tty_rows: u16,
    mouse_input: Option<crate::framebuffer::MouseInput>,
    cursor_tracker: crate::framebuffer::CursorTracker,
}

#[cfg(feature = "framebuffer-backend")]
impl FramebufferBackend {
    /// Create a new framebuffer backend with specified text mode, optional scale, optional font, and optional mouse device
    pub fn new(
        mode: crate::framebuffer::TextMode,
        scale: Option<usize>,
        font_name: Option<&str>,
        mouse_device: Option<&str>,
    ) -> io::Result<Self> {
        use crossterm::terminal;

        let renderer = crate::framebuffer::FramebufferRenderer::new(mode, scale, font_name)?;

        // Get actual TTY dimensions for mouse coordinate scaling
        let (tty_cols, tty_rows) = terminal::size()?;

        // Try to open raw mouse input (uses specified device or auto-detects)
        let mouse_input = match crate::framebuffer::MouseInput::new(mouse_device) {
            Ok(input) => Some(input),
            Err(e) => {
                eprintln!("Warning: Failed to open mouse input device: {}", e);
                eprintln!("Mouse cursor will not be rendered.");
                eprintln!("Try: sudo chmod a+r /dev/input/mice /dev/input/event*");
                None
            }
        };

        // Get pixel dimensions for cursor tracker
        let (pixel_width, pixel_height) = renderer.pixel_dimensions();
        eprintln!(
            "Initializing cursor tracker with bounds: {}x{} pixels",
            pixel_width, pixel_height
        );
        let cursor_tracker = crate::framebuffer::CursorTracker::new(pixel_width, pixel_height);

        Ok(Self {
            renderer,
            tty_cols,
            tty_rows,
            mouse_input,
            cursor_tracker,
        })
    }

    /// Get current cursor position (pixel coordinates)
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.cursor_tracker.x, self.cursor_tracker.y)
    }

    /// Set cursor visibility
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.renderer.set_cursor_visible(visible);
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

    fn scale_mouse_coords(&self, col: u16, row: u16) -> (u16, u16) {
        // Scale mouse coordinates from TTY space to framebuffer space
        let (fb_cols, fb_rows) = self.dimensions();

        // If TTY dimensions match framebuffer dimensions, no scaling needed
        if self.tty_cols == fb_cols && self.tty_rows == fb_rows {
            return (col, row);
        }

        // Calculate scaling factors
        let col_scale = fb_cols as f32 / self.tty_cols as f32;
        let row_scale = fb_rows as f32 / self.tty_rows as f32;

        // Apply scaling
        let scaled_col = (col as f32 * col_scale).round() as u16;
        let scaled_row = (row as f32 * row_scale).round() as u16;

        // Clamp to framebuffer dimensions
        let clamped_col = scaled_col.min(fb_cols.saturating_sub(1));
        let clamped_row = scaled_row.min(fb_rows.saturating_sub(1));

        (clamped_col, clamped_row)
    }

    fn update_cursor(&mut self) -> bool {
        if let Some(ref mut mouse_input) = self.mouse_input {
            let mut moved = false;
            // Process all pending mouse events
            while let Ok(Some(event)) = mouse_input.read_event() {
                eprintln!("Mouse event: dx={}, dy={}", event.dx, event.dy);
                let old_x = self.cursor_tracker.x;
                let old_y = self.cursor_tracker.y;
                self.cursor_tracker.update(event.dx, event.dy);
                eprintln!(
                    "Cursor moved from ({}, {}) to ({}, {})",
                    old_x, old_y, self.cursor_tracker.x, self.cursor_tracker.y
                );
                moved = true;
            }
            moved
        } else {
            false
        }
    }

    fn draw_cursor(&mut self) {
        let (x, y) = self.cursor_position();
        self.renderer.draw_cursor(x, y);
    }

    fn restore_cursor_area(&mut self) {
        self.renderer.restore_cursor_area();
    }
}
