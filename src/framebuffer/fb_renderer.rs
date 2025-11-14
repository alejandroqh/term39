//! Framebuffer renderer for direct console rendering
//!
//! This module provides low-level framebuffer access via /dev/fb0
//! for rendering characters with pixel-perfect control.

use super::font_manager::FontManager;
use super::text_modes::TextMode;
use crate::video_buffer::{Cell, VideoBuffer};
use crossterm::style::Color;
use framebuffer::Framebuffer;
use std::io;

/// DOS 16-color palette (VGA colors)
/// Order: Black, Blue, Green, Cyan, Red, Magenta, Brown, LightGray,
///        DarkGray, LightBlue, LightGreen, LightCyan, LightRed, LightMagenta, Yellow, White
const DOS_PALETTE: [(u8, u8, u8); 16] = [
    (0, 0, 0),       // Black
    (0, 0, 170),     // Blue
    (0, 170, 0),     // Green
    (0, 170, 170),   // Cyan
    (170, 0, 0),     // Red
    (170, 0, 170),   // Magenta
    (170, 85, 0),    // Brown
    (170, 170, 170), // LightGray
    (85, 85, 85),    // DarkGray
    (85, 85, 255),   // LightBlue
    (85, 255, 85),   // LightGreen
    (85, 255, 255),  // LightCyan
    (255, 85, 85),   // LightRed
    (255, 85, 255),  // LightMagenta
    (255, 255, 85),  // Yellow
    (255, 255, 255), // White
];

/// Cursor sprite dimensions
const CURSOR_WIDTH: usize = 16;
const CURSOR_HEIGHT: usize = 16;

/// Cursor sprite bitmap (16x16 arrow cursor)
/// 0 = transparent, 1 = black outline, 2 = white fill
const CURSOR_SPRITE: [[u8; CURSOR_WIDTH]; CURSOR_HEIGHT] = [
    [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
    [1, 2, 2, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 1, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
];

/// Framebuffer renderer for Linux console
pub struct FramebufferRenderer {
    framebuffer: Framebuffer,
    font: FontManager,
    mode: TextMode,
    width_pixels: usize,
    height_pixels: usize,
    bytes_per_pixel: usize,
    line_length: usize,
    scale: usize,    // Pixel scale factor (1, 2, 3, 4...)
    offset_x: usize, // X offset to center content
    offset_y: usize, // Y offset to center content
    cursor_visible: bool,
    cursor_saved_pixels: Vec<(usize, usize, u8, u8, u8)>, // (x, y, r, g, b)
}

impl FramebufferRenderer {
    /// Initialize framebuffer renderer with specified text mode, optional scale, and optional font
    /// If scale is None, automatically calculates the best integer scale that fits the screen
    /// If font_name is None, automatically selects a font matching the text mode dimensions
    pub fn new(mode: TextMode, scale: Option<usize>, font_name: Option<&str>) -> io::Result<Self> {
        // Open framebuffer device
        let framebuffer = Framebuffer::new("/dev/fb0").map_err(|e| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "Failed to open /dev/fb0: {}. Run as root or add user to 'video' group.",
                    e
                ),
            )
        })?;

        // Get framebuffer info
        let var_screen_info = framebuffer.var_screen_info.clone();
        let width_pixels = var_screen_info.xres as usize;
        let height_pixels = var_screen_info.yres as usize;
        let bytes_per_pixel = (var_screen_info.bits_per_pixel / 8) as usize;

        let fix_screen_info = framebuffer.fix_screen_info.clone();
        let line_length = fix_screen_info.line_length as usize;

        // Load font: try specified font first, then auto-detect
        let font = if let Some(name) = font_name {
            // Try to load the specified font
            FontManager::load_console_font(name).or_else(|e| {
                eprintln!("Warning: Failed to load font '{}': {}", name, e);
                eprintln!("Falling back to auto-detection...");
                // Fall back to auto-detection
                FontManager::load_for_dimensions(mode.char_width, mode.char_height)
            })
        } else {
            // Auto-detect font for text mode dimensions
            FontManager::load_for_dimensions(mode.char_width, mode.char_height)
        }
        .or_else(|_| {
            // Final fallback: try to load any 8x16 font
            FontManager::load_console_font("Lat2-Terminus16")
                .or_else(|_| FontManager::load_console_font("default8x16"))
        })
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to load console font: {}", e),
            )
        })?;

        // Calculate base content dimensions (without scaling)
        let base_width = mode.cols * font.width;
        let base_height = mode.rows * font.height;

        // Determine scale factor
        let scale = scale.unwrap_or_else(|| {
            // Auto-calculate maximum integer scale that fits the screen
            let max_scale_x = width_pixels / base_width;
            let max_scale_y = height_pixels / base_height;
            let auto_scale = max_scale_x.min(max_scale_y);
            // Ensure at least 1x scale
            auto_scale.max(1)
        });

        // Calculate scaled content dimensions
        let content_width = base_width * scale;
        let content_height = base_height * scale;

        // Calculate offsets to center scaled content on screen
        let offset_x = if width_pixels > content_width {
            (width_pixels - content_width) / 2
        } else {
            0
        };
        let offset_y = if height_pixels > content_height {
            (height_pixels - content_height) / 2
        } else {
            0
        };

        println!(
            "Framebuffer initialized: {}x{} pixels, {} bytes/pixel, mode: {} ({}x{} chars)",
            width_pixels, height_pixels, bytes_per_pixel, mode.kind, mode.cols, mode.rows
        );
        println!(
            "Font loaded: {}x{} pixels per character",
            font.width, font.height
        );
        println!(
            "Pixel scale: {}x (base: {}x{} → scaled: {}x{})",
            scale, base_width, base_height, content_width, content_height
        );
        println!("Content centered at offset ({}, {})", offset_x, offset_y);

        Ok(FramebufferRenderer {
            framebuffer,
            font,
            mode,
            width_pixels,
            height_pixels,
            bytes_per_pixel,
            line_length,
            scale,
            offset_x,
            offset_y,
            cursor_visible: true,
            cursor_saved_pixels: Vec::new(),
        })
    }

    /// Convert Color enum to RGB tuple
    fn color_to_rgb(&self, color: Color) -> (u8, u8, u8) {
        match color {
            Color::Black => DOS_PALETTE[0],
            Color::DarkGrey => DOS_PALETTE[8],
            Color::Grey => DOS_PALETTE[7],
            Color::White => DOS_PALETTE[15],
            Color::DarkRed => DOS_PALETTE[4],
            Color::Red => DOS_PALETTE[12],
            Color::DarkGreen => DOS_PALETTE[2],
            Color::Green => DOS_PALETTE[10],
            Color::DarkYellow => DOS_PALETTE[14],
            Color::Yellow => DOS_PALETTE[14],
            Color::DarkBlue => DOS_PALETTE[1],
            Color::Blue => DOS_PALETTE[9],
            Color::DarkMagenta => DOS_PALETTE[5],
            Color::Magenta => DOS_PALETTE[13],
            Color::DarkCyan => DOS_PALETTE[3],
            Color::Cyan => DOS_PALETTE[11],
            Color::Rgb { r, g, b } => (r, g, b),
            _ => DOS_PALETTE[7], // Default to light gray
        }
    }

    /// Put a pixel at (x, y) with RGB color (relative to content area)
    /// Applies scaling: each logical pixel becomes scale×scale physical pixels
    fn put_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        // Apply scaling and offsets
        for sy in 0..self.scale {
            for sx in 0..self.scale {
                let actual_x = x * self.scale + sx + self.offset_x;
                let actual_y = y * self.scale + sy + self.offset_y;

                if actual_x >= self.width_pixels || actual_y >= self.height_pixels {
                    continue;
                }

                let offset = actual_y * self.line_length + actual_x * self.bytes_per_pixel;
                let frame = self.framebuffer.frame.as_mut();

                // Handle different color depths
                match self.bytes_per_pixel {
                    4 => {
                        // RGBA or BGRA (32-bit)
                        if offset + 3 < frame.len() {
                            frame[offset] = b; // Blue
                            frame[offset + 1] = g; // Green
                            frame[offset + 2] = r; // Red
                            frame[offset + 3] = 255; // Alpha
                        }
                    }
                    3 => {
                        // RGB or BGR (24-bit)
                        if offset + 2 < frame.len() {
                            frame[offset] = b; // Blue
                            frame[offset + 1] = g; // Green
                            frame[offset + 2] = r; // Red
                        }
                    }
                    2 => {
                        // RGB565 (16-bit)
                        if offset + 1 < frame.len() {
                            let r5 = (r >> 3) as u16;
                            let g6 = (g >> 2) as u16;
                            let b5 = (b >> 3) as u16;
                            let color = (r5 << 11) | (g6 << 5) | b5;
                            frame[offset] = (color & 0xFF) as u8;
                            frame[offset + 1] = (color >> 8) as u8;
                        }
                    }
                    _ => {} // Unsupported format
                }
            }
        }
    }

    /// Render a single character at text position (col, row)
    pub fn render_char(&mut self, col: usize, row: usize, cell: &Cell) {
        if !self.mode.is_valid_position(col, row) {
            return;
        }

        let x_offset = col * self.font.width;
        let y_offset = row * self.font.height;
        let font_width = self.font.width;
        let font_height = self.font.height;

        let fg_color = self.color_to_rgb(cell.fg_color);
        let bg_color = self.color_to_rgb(cell.bg_color);

        let glyph = self.font.get_glyph(cell.character);

        // Collect pixel states first to avoid borrowing conflicts
        let mut pixel_data = Vec::with_capacity(font_height * font_width);
        for py in 0..font_height {
            for px in 0..font_width {
                let is_set = self.font.is_pixel_set(glyph, px, py);
                pixel_data.push(is_set);
            }
        }

        // Now render pixels (can mutably borrow self)
        let mut idx = 0;
        for py in 0..font_height {
            for px in 0..font_width {
                let color = if pixel_data[idx] { fg_color } else { bg_color };
                idx += 1;

                self.put_pixel(x_offset + px, y_offset + py, color.0, color.1, color.2);
            }
        }
    }

    /// Clear the entire screen - fills borders with black, content area with specified color
    #[allow(dead_code)]
    pub fn clear(&mut self, color: Color) {
        let rgb = self.color_to_rgb(color);

        // First, fill entire framebuffer with black (for borders)
        let frame = self.framebuffer.frame.as_mut();
        for byte in frame.iter_mut() {
            *byte = 0;
        }

        // Then fill the content area with the specified color
        // Note: put_pixel already handles scaling, so we use logical dimensions here
        let base_width = self.mode.cols * self.font.width;
        let base_height = self.mode.rows * self.font.height;

        for y in 0..base_height {
            for x in 0..base_width {
                self.put_pixel(x, y, rgb.0, rgb.1, rgb.2);
            }
        }
    }

    /// Render entire video buffer to framebuffer
    pub fn render_buffer(&mut self, buffer: &VideoBuffer) {
        let (cols, rows) = buffer.dimensions();

        let max_rows = (rows as usize).min(self.mode.rows);
        let max_cols = (cols as usize).min(self.mode.cols);

        for row in 0..max_rows {
            for col in 0..max_cols {
                if let Some(cell) = buffer.get(col as u16, row as u16) {
                    self.render_char(col, row, cell);
                }
            }
        }
    }

    /// Get current text mode
    #[allow(dead_code)]
    pub fn mode(&self) -> &TextMode {
        &self.mode
    }

    /// Get text dimensions (columns, rows)
    pub fn dimensions(&self) -> (usize, usize) {
        (self.mode.cols, self.mode.rows)
    }

    /// Get pixel dimensions (width, height) of the rendering area
    pub fn pixel_dimensions(&self) -> (usize, usize) {
        let width = self.mode.cols * self.font.width;
        let height = self.mode.rows * self.font.height;
        (width, height)
    }

    /// Get a pixel from the framebuffer at (x, y) - returns (r, g, b)
    fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        // Apply scaling and offsets
        let actual_x = x * self.scale + self.offset_x;
        let actual_y = y * self.scale + self.offset_y;

        if actual_x >= self.width_pixels || actual_y >= self.height_pixels {
            return (0, 0, 0);
        }

        let offset = actual_y * self.line_length + actual_x * self.bytes_per_pixel;
        let frame = self.framebuffer.frame.as_ref();

        // Handle different color depths
        match self.bytes_per_pixel {
            4 | 3 => {
                if offset + 2 < frame.len() {
                    (frame[offset + 2], frame[offset + 1], frame[offset])
                } else {
                    (0, 0, 0)
                }
            }
            2 => {
                if offset + 1 < frame.len() {
                    let color = (frame[offset] as u16) | ((frame[offset + 1] as u16) << 8);
                    let r = ((color >> 11) & 0x1F) as u8;
                    let g = ((color >> 5) & 0x3F) as u8;
                    let b = (color & 0x1F) as u8;
                    (
                        (r << 3) | (r >> 2),
                        (g << 2) | (g >> 4),
                        (b << 3) | (b >> 2),
                    )
                } else {
                    (0, 0, 0)
                }
            }
            _ => (0, 0, 0),
        }
    }

    /// Set cursor visibility
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Draw cursor at specified pixel position (logical coordinates, not scaled)
    /// This should be called AFTER all other content is rendered
    pub fn draw_cursor(&mut self, x: usize, y: usize) {
        if !self.cursor_visible {
            return;
        }

        // Save pixels under cursor before drawing
        self.cursor_saved_pixels.clear();

        for (cy, row) in CURSOR_SPRITE.iter().enumerate() {
            for (cx, &sprite_pixel) in row.iter().enumerate() {
                let pixel_x = x + cx;
                let pixel_y = y + cy;

                // Check bounds
                let base_width = self.mode.cols * self.font.width;
                let base_height = self.mode.rows * self.font.height;
                if pixel_x >= base_width || pixel_y >= base_height {
                    continue;
                }

                if sprite_pixel == 0 {
                    continue; // Transparent pixel
                }

                // Save original pixel
                let original = self.get_pixel(pixel_x, pixel_y);
                self.cursor_saved_pixels
                    .push((pixel_x, pixel_y, original.0, original.1, original.2));

                // Draw cursor pixel
                let (r, g, b) = match sprite_pixel {
                    1 => (0, 0, 0),       // Black outline
                    2 => (255, 255, 255), // White fill
                    _ => continue,
                };

                self.put_pixel(pixel_x, pixel_y, r, g, b);
            }
        }
    }

    /// Restore pixels that were saved before drawing cursor
    pub fn restore_cursor_area(&mut self) {
        // Clone the vector to avoid borrowing issues
        let saved = self.cursor_saved_pixels.clone();
        for (x, y, r, g, b) in saved {
            self.put_pixel(x, y, r, g, b);
        }
        self.cursor_saved_pixels.clear();
    }
}
