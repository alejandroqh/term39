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
use std::os::unix::fs::FileTypeExt;

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
    // Pixel format offsets (byte positions for RGB channels)
    r_offset: usize,
    g_offset: usize,
    b_offset: usize,
    // Previous frame buffer for dirty tracking (only render changed cells)
    prev_buffer: Vec<Cell>,
}

impl FramebufferRenderer {
    /// Initialize framebuffer renderer with specified text mode, optional scale, and optional font
    /// If scale is None, automatically calculates the best integer scale that fits the screen
    /// If font_name is None, automatically selects a font matching the text mode dimensions
    pub fn new(mode: TextMode, scale: Option<usize>, font_name: Option<&str>) -> io::Result<Self> {
        // Verify /dev/fb0 is a character device before opening
        // This prevents potential security issues with symlink attacks
        let fb_path = std::path::Path::new("/dev/fb0");
        let metadata = std::fs::metadata(fb_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to access /dev/fb0: {}", e),
            )
        })?;

        if !metadata.file_type().is_char_device() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "/dev/fb0 is not a character device - possible security issue",
            ));
        }

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

        // Get pixel format offsets from VarScreenInfo (handles RGB vs BGR)
        let r_offset = (var_screen_info.red.offset / 8) as usize;
        let g_offset = (var_screen_info.green.offset / 8) as usize;
        let b_offset = (var_screen_info.blue.offset / 8) as usize;

        let fix_screen_info = framebuffer.fix_screen_info.clone();
        let line_length = fix_screen_info.line_length as usize;

        // Load font: try specified font first, then auto-detect
        // Supports both system fonts and embedded fonts (prefixed with "[Embedded] ")
        let font = if let Some(name) = font_name {
            // Try to load the specified font (supports embedded fonts)
            FontManager::load_font_by_name(name).or_else(|e| {
                eprintln!("Warning: Failed to load font '{}': {}", name, e);
                eprintln!("Falling back to auto-detection...");
                // Fall back to auto-detection (which also falls back to embedded fonts)
                FontManager::load_for_dimensions(mode.char_width, mode.char_height)
            })
        } else {
            // Auto-detect font for text mode dimensions (falls back to embedded fonts)
            FontManager::load_for_dimensions(mode.char_width, mode.char_height)
        }
        .or_else(|_| {
            // Final fallback: use embedded font
            FontManager::load_embedded_default()
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
        // Write debug info to file since stdout may not be visible in framebuffer mode
        if let Ok(mut f) = std::fs::File::create("/tmp/term39-font-debug.log") {
            use std::io::Write;
            let _ = writeln!(
                f,
                "Font loaded: {}x{} - {}",
                font.width,
                font.height,
                font.debug_info()
            );
        }
        println!(
            "Font loaded: {}x{} pixels per character - {}",
            font.width,
            font.height,
            font.debug_info()
        );
        println!(
            "Pixel scale: {}x (base: {}x{} → scaled: {}x{})",
            scale, base_width, base_height, content_width, content_height
        );
        println!("Content centered at offset ({}, {})", offset_x, offset_y);

        // Initialize previous buffer for dirty tracking
        let prev_buffer_size = mode.cols * mode.rows;
        let prev_buffer = vec![Cell::default(); prev_buffer_size];

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
            r_offset,
            g_offset,
            b_offset,
            prev_buffer,
        })
    }

    /// Convert Color enum to RGB tuple
    #[inline(always)]
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
            Color::DarkYellow => DOS_PALETTE[6], // Brown (dark yellow)
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
    /// Optimized with fast path for scale=1 (most common on modern displays)
    #[inline(always)]
    fn put_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        // Precalculate base position once
        let base_x = x * self.scale + self.offset_x;
        let base_y = y * self.scale + self.offset_y;

        // Early exit if entire scaled block is out of bounds
        if base_x >= self.width_pixels || base_y >= self.height_pixels {
            return;
        }

        // Hoist frame borrow and constants outside loop
        let frame = self.framebuffer.frame.as_mut();
        let line_length = self.line_length;
        let bytes_per_pixel = self.bytes_per_pixel;
        let r_offset = self.r_offset;
        let g_offset = self.g_offset;
        let b_offset = self.b_offset;
        let frame_len = frame.len();

        // Fast path for scale=1 (eliminates loop overhead entirely)
        if self.scale == 1 {
            let offset = base_y * line_length + base_x * bytes_per_pixel;
            match bytes_per_pixel {
                4 if offset + 3 < frame_len => {
                    frame[offset + r_offset] = r;
                    frame[offset + g_offset] = g;
                    frame[offset + b_offset] = b;
                    frame[offset + 3] = 255;
                }
                3 if offset + 2 < frame_len => {
                    frame[offset + r_offset] = r;
                    frame[offset + g_offset] = g;
                    frame[offset + b_offset] = b;
                }
                2 if offset + 1 < frame_len => {
                    let r5 = (r >> 3) as u16;
                    let g6 = (g >> 2) as u16;
                    let b5 = (b >> 3) as u16;
                    let color = (r5 << 11) | (g6 << 5) | b5;
                    frame[offset] = (color & 0xFF) as u8;
                    frame[offset + 1] = (color >> 8) as u8;
                }
                _ => {}
            }
            return;
        }

        // Scaled rendering path (scale > 1)
        let width_pixels = self.width_pixels;
        let height_pixels = self.height_pixels;
        let scale = self.scale;

        for sy in 0..scale {
            let actual_y = base_y + sy;
            if actual_y >= height_pixels {
                break;
            }

            for sx in 0..scale {
                let actual_x = base_x + sx;
                if actual_x >= width_pixels {
                    break;
                }

                let offset = actual_y * line_length + actual_x * bytes_per_pixel;

                match bytes_per_pixel {
                    4 if offset + 3 < frame_len => {
                        frame[offset + r_offset] = r;
                        frame[offset + g_offset] = g;
                        frame[offset + b_offset] = b;
                        frame[offset + 3] = 255;
                    }
                    3 if offset + 2 < frame_len => {
                        frame[offset + r_offset] = r;
                        frame[offset + g_offset] = g;
                        frame[offset + b_offset] = b;
                    }
                    2 if offset + 1 < frame_len => {
                        let r5 = (r >> 3) as u16;
                        let g6 = (g >> 2) as u16;
                        let b5 = (b >> 3) as u16;
                        let color = (r5 << 11) | (g6 << 5) | b5;
                        frame[offset] = (color & 0xFF) as u8;
                        frame[offset + 1] = (color >> 8) as u8;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Render a single character at text position (col, row)
    /// Uses single-pass rendering for optimal performance
    #[inline]
    pub fn render_char(&mut self, col: usize, row: usize, cell: &Cell) {
        if !self.mode.is_valid_position(col, row) {
            return;
        }

        // Debug: log first few characters rendered
        static DEBUG_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let count = DEBUG_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if count < 20 {
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/term39-font-debug.log")
            {
                use std::io::Write;
                let _ = writeln!(
                    f,
                    "render_char: col={}, row={}, char='{}' (code={}), glyph_count={}",
                    col,
                    row,
                    cell.character,
                    cell.character as usize,
                    self.font.glyph_count()
                );
            }
        }

        let x_offset = col * self.font.width;
        let y_offset = row * self.font.height;
        let font_width = self.font.width;
        let font_height = self.font.height;

        let fg_color = self.color_to_rgb(cell.fg_color);
        let bg_color = self.color_to_rgb(cell.bg_color);

        // Copy glyph data to stack-allocated buffer to avoid borrow conflicts with put_pixel
        // Max glyph size: 24 rows * 3 bytes/row = 72 bytes (covers all standard fonts up to 24px)
        let glyph_src = self.font.get_glyph(cell.character);
        let glyph_len = glyph_src.len().min(72);
        let mut glyph_data = [0u8; 72];
        glyph_data[..glyph_len].copy_from_slice(&glyph_src[..glyph_len]);

        let is_width_8 = self.font.is_width_8;
        let bytes_per_row = self.font.bytes_per_row;

        // Single-pass rendering: check pixel and render immediately
        // This eliminates the intermediate array and reduces memory accesses
        for py in 0..font_height {
            for px in 0..font_width {
                // Inline pixel checking to avoid borrow conflicts with put_pixel
                let is_set = if is_width_8 {
                    // Fast path for 8-pixel-wide fonts
                    py < glyph_len && (glyph_data[py] & (0x80 >> px)) != 0
                } else {
                    // General path for wider fonts
                    let row_start = py * bytes_per_row;
                    let byte_index = row_start + (px >> 3);
                    let bit_index = 7 - (px & 7);
                    byte_index < glyph_len && (glyph_data[byte_index] & (1 << bit_index)) != 0
                };
                let color = if is_set { fg_color } else { bg_color };
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
    /// Uses dirty tracking to only render cells that have changed
    pub fn render_buffer(&mut self, buffer: &VideoBuffer) {
        let (cols, rows) = buffer.dimensions();

        let max_rows = (rows as usize).min(self.mode.rows);
        let max_cols = (cols as usize).min(self.mode.cols);

        for row in 0..max_rows {
            for col in 0..max_cols {
                if let Some(cell) = buffer.get(col as u16, row as u16) {
                    // Calculate index into prev_buffer
                    let idx = row * self.mode.cols + col;

                    // Only render if cell has changed from previous frame
                    if idx < self.prev_buffer.len() {
                        let prev_cell = &self.prev_buffer[idx];
                        if prev_cell != cell {
                            self.render_char(col, row, cell);
                            // Update previous buffer
                            self.prev_buffer[idx] = *cell;
                        }
                    } else {
                        // Index out of bounds, render anyway
                        self.render_char(col, row, cell);
                    }
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

    /// Get pixel dimensions (width, height) of the rendering area (base, unscaled)
    pub fn pixel_dimensions(&self) -> (usize, usize) {
        let width = self.mode.cols * self.font.width;
        let height = self.mode.rows * self.font.height;
        (width, height)
    }

    /// Get scale factor
    #[allow(dead_code)]
    pub fn scale(&self) -> usize {
        self.scale
    }

    /// Get offsets (x, y)
    #[allow(dead_code)]
    pub fn offsets(&self) -> (usize, usize) {
        (self.offset_x, self.offset_y)
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

        // Handle different color depths - use dynamic offsets
        match self.bytes_per_pixel {
            4 | 3 => {
                if offset + 2 < frame.len() {
                    (
                        frame[offset + self.r_offset],
                        frame[offset + self.g_offset],
                        frame[offset + self.b_offset],
                    )
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
    #[allow(dead_code)]
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
        // Use std::mem::take to move ownership without cloning
        let saved = std::mem::take(&mut self.cursor_saved_pixels);
        for (x, y, r, g, b) in saved {
            self.put_pixel(x, y, r, g, b);
        }
    }
}
