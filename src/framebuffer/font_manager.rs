//! PSF2 bitmap font loading and management
//!
//! This module handles loading PSF2 (PC Screen Font 2) format fonts
//! from /usr/share/consolefonts/ for authentic DOS-style rendering.

use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// PSF2 file header magic number
const PSF2_MAGIC: u32 = 0x864ab572;

/// PSF1 file header magic number (16-bit)
const PSF1_MAGIC: u16 = 0x0436;

/// PSF1 mode flags
const PSF1_MODE512: u8 = 0x01; // 512 glyphs instead of 256
#[allow(dead_code)]
const PSF1_MODEHASTAB: u8 = 0x02; // Has unicode table

/// PSF1 font header structure (4 bytes)
#[repr(C)]
#[derive(Debug, Clone)]
struct Psf1Header {
    magic: u16,   // Magic bytes (0x0436)
    mode: u8,     // Mode flags
    charsize: u8, // Character height in pixels (width is always 8)
}

impl Psf1Header {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "PSF1 header too small",
            ));
        }

        let magic = u16::from_le_bytes([bytes[0], bytes[1]]);
        if magic != PSF1_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid PSF1 magic: 0x{:04x}", magic),
            ));
        }

        Ok(Psf1Header {
            magic,
            mode: bytes[2],
            charsize: bytes[3],
        })
    }

    fn glyph_count(&self) -> usize {
        if self.mode & PSF1_MODE512 != 0 {
            512
        } else {
            256
        }
    }
}

/// PSF2 font header structure
#[repr(C)]
#[derive(Debug, Clone)]
struct Psf2Header {
    magic: u32,      // Magic bytes (0x864ab572)
    version: u32,    // Zero
    headersize: u32, // Offset of bitmaps in file
    flags: u32,      // 0 if no unicode table
    length: u32,     // Number of glyphs
    charsize: u32,   // Number of bytes per glyph
    height: u32,     // Height in pixels
    width: u32,      // Width in pixels
}

impl Psf2Header {
    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() < 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "PSF2 header too small",
            ));
        }

        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != PSF2_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid PSF2 magic: 0x{:08x}", magic),
            ));
        }

        Ok(Psf2Header {
            magic,
            version: u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            headersize: u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            flags: u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
            length: u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]),
            charsize: u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]),
            height: u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]),
            width: u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]),
        })
    }
}

/// Font manager for loading and accessing PSF2 bitmap fonts
#[derive(Debug)]
pub struct FontManager {
    /// Font width in pixels
    pub width: usize,
    /// Font height in pixels
    pub height: usize,
    /// Number of glyphs in font
    glyph_count: usize,
    /// Bytes per glyph
    bytes_per_glyph: usize,
    /// Glyph bitmap data (row-major, 1 bit per pixel)
    glyphs: Vec<u8>,
}

impl FontManager {
    /// Load a PSF font from file path (supports both PSF1 and PSF2)
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path.as_ref())?;

        // Read first 4 bytes to detect format
        let mut magic_bytes = [0u8; 4];
        file.read_exact(&mut magic_bytes)?;

        // Check if it's PSF2 (magic is 4 bytes)
        let magic32 = u32::from_le_bytes(magic_bytes);
        if magic32 == PSF2_MAGIC {
            // PSF2 format - read rest of header (32 bytes total)
            let mut header_bytes = [0u8; 32];
            header_bytes[0..4].copy_from_slice(&magic_bytes);
            file.read_exact(&mut header_bytes[4..])?;
            let header = Psf2Header::from_bytes(&header_bytes)?;

            // Skip to bitmap data
            let skip_bytes = header.headersize as usize - 32;
            if skip_bytes > 0 {
                let mut skip_buf = vec![0u8; skip_bytes];
                file.read_exact(&mut skip_buf)?;
            }

            // Read all glyph bitmaps
            let total_bytes = (header.length * header.charsize) as usize;
            let mut glyphs = vec![0u8; total_bytes];
            file.read_exact(&mut glyphs)?;

            return Ok(FontManager {
                width: header.width as usize,
                height: header.height as usize,
                glyph_count: header.length as usize,
                bytes_per_glyph: header.charsize as usize,
                glyphs,
            });
        }

        // Check if it's PSF1 (magic is first 2 bytes)
        let magic16 = u16::from_le_bytes([magic_bytes[0], magic_bytes[1]]);
        if magic16 == PSF1_MAGIC {
            // PSF1 format - header is only 4 bytes
            let header = Psf1Header::from_bytes(&magic_bytes)?;

            let width = 8; // PSF1 is always 8 pixels wide
            let height = header.charsize as usize;
            let glyph_count = header.glyph_count();
            let bytes_per_glyph = height; // Each row is 1 byte (8 pixels)

            // Read all glyph bitmaps
            let total_bytes = glyph_count * bytes_per_glyph;
            let mut glyphs = vec![0u8; total_bytes];
            file.read_exact(&mut glyphs)?;

            return Ok(FontManager {
                width,
                height,
                glyph_count,
                bytes_per_glyph,
                glyphs,
            });
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown font format. Magic: 0x{:08x}", magic32),
        ))
    }

    /// Load a font by name from console font directories
    /// Validates font name to prevent directory traversal attacks
    pub fn load_console_font(name: &str) -> io::Result<Self> {
        // Validate font name to prevent directory traversal
        // Font names should only contain alphanumeric characters, hyphens, and underscores
        if name.is_empty() || name.len() > 128 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid font name: must be 1-128 characters",
            ));
        }

        // Check for path traversal attempts
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid font name: path separators not allowed",
            ));
        }

        // Only allow safe characters in font names
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid font name: only alphanumeric, hyphen, underscore, and dot allowed",
            ));
        }

        // Try multiple common locations for console fonts
        let base_paths = [
            "/usr/share/consolefonts",     // Debian/Ubuntu primary location
            "/usr/share/kbd/consolefonts", // Alternative/older location
        ];

        // Try with various extensions: .psf.gz (compressed), .psfu, .psf
        let extensions = [".psf.gz", ".psfu", ".psf"];

        for base_path in &base_paths {
            let base = Path::new(base_path);
            for ext in &extensions {
                let path = base.join(format!("{}{}", name, ext));

                // Additional safety: verify the canonicalized path is within the base directory
                if let Ok(canonical) = path.canonicalize() {
                    if !canonical.starts_with(base_path) {
                        continue; // Skip paths that escape the base directory
                    }

                    // Handle gzip-compressed fonts
                    if ext.ends_with(".gz") {
                        return Self::load_from_gzip_file(canonical);
                    } else {
                        return Self::load_from_file(canonical);
                    }
                }
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Console font '{}' not found in console font directories",
                name
            ),
        ))
    }

    /// Load a PSF font from a gzip-compressed file (supports both PSF1 and PSF2)
    fn load_from_gzip_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        use std::io::BufReader;

        let file = File::open(path.as_ref())?;
        let buf_reader = BufReader::new(file);

        // Decompress gzip
        let mut gz_decoder = GzDecoder::new(buf_reader);

        // Read first 4 bytes to detect format
        let mut magic_bytes = [0u8; 4];
        gz_decoder.read_exact(&mut magic_bytes)?;

        // Check if it's PSF2 (magic is 4 bytes)
        let magic32 = u32::from_le_bytes(magic_bytes);
        if magic32 == PSF2_MAGIC {
            // PSF2 format - read rest of header (32 bytes total)
            let mut header_bytes = [0u8; 32];
            header_bytes[0..4].copy_from_slice(&magic_bytes);
            gz_decoder.read_exact(&mut header_bytes[4..])?;
            let header = Psf2Header::from_bytes(&header_bytes)?;

            // Skip to bitmap data
            let skip_bytes = header.headersize as usize - 32;
            if skip_bytes > 0 {
                let mut skip_buf = vec![0u8; skip_bytes];
                gz_decoder.read_exact(&mut skip_buf)?;
            }

            // Read all glyph bitmaps
            let total_bytes = (header.length * header.charsize) as usize;
            let mut glyphs = vec![0u8; total_bytes];
            gz_decoder.read_exact(&mut glyphs)?;

            return Ok(FontManager {
                width: header.width as usize,
                height: header.height as usize,
                glyph_count: header.length as usize,
                bytes_per_glyph: header.charsize as usize,
                glyphs,
            });
        }

        // Check if it's PSF1 (magic is first 2 bytes)
        let magic16 = u16::from_le_bytes([magic_bytes[0], magic_bytes[1]]);
        if magic16 == PSF1_MAGIC {
            // PSF1 format - header is only 4 bytes
            let header = Psf1Header::from_bytes(&magic_bytes)?;

            let width = 8; // PSF1 is always 8 pixels wide
            let height = header.charsize as usize;
            let glyph_count = header.glyph_count();
            let bytes_per_glyph = height; // Each row is 1 byte (8 pixels)

            // Read all glyph bitmaps
            let total_bytes = glyph_count * bytes_per_glyph;
            let mut glyphs = vec![0u8; total_bytes];
            gz_decoder.read_exact(&mut glyphs)?;

            return Ok(FontManager {
                width,
                height,
                glyph_count,
                bytes_per_glyph,
                glyphs,
            });
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown font format in gzip. Magic: 0x{:08x}", magic32),
        ))
    }

    /// Try to find a suitable font for the given character dimensions
    pub fn load_for_dimensions(char_width: usize, char_height: usize) -> io::Result<Self> {
        // Common console fonts with their dimensions
        // Debian/Ubuntu font names (Uni3-* series, compressed .psf.gz)
        let font_candidates = [
            // 8x16 fonts (80x25 mode)
            ("Uni3-Terminus16", 8, 16),
            ("Uni3-TerminusBold16", 8, 16),
            ("Unifont-APL8x16", 8, 16), // GNU Unifont - excellent Unicode coverage
            ("Lat2-Terminus16", 8, 16), // Fallback for older systems
            ("ter-116n", 8, 16),
            ("default8x16", 8, 16),
            // 8x8 fonts (80x50 mode - rare, might not exist)
            ("Uni2-VGA8", 8, 8),
            ("Lat2-Terminus8", 8, 8),
            ("ter-108n", 8, 8),
            ("default8x8", 8, 8),
            // 8x14 fonts (80x28 mode)
            ("Uni3-Terminus14", 8, 14),
            ("Uni3-TerminusBold14", 8, 14),
            ("Lat2-Terminus14", 8, 14),
            ("ter-114n", 8, 14),
            // 16x32 fonts (might work for high-res 40x25 mode)
            ("Uni3-Terminus32x16", 16, 32),
            ("Uni3-TerminusBold32x16", 16, 32),
            ("Uni2-VGA28x16", 16, 28),
            ("Unifont", 16, 16), // GNU Unifont 16x16 variant
            // 12x24 fonts (alternative for 40x25)
            ("Uni3-Terminus24x12", 12, 24),
            ("Uni3-TerminusBold24x12", 12, 24),
        ];

        // Find exact match first
        for (name, w, h) in &font_candidates {
            if *w == char_width && *h == char_height {
                if let Ok(font) = Self::load_console_font(name) {
                    return Ok(font);
                }
            }
        }

        // Fallback: try to load any font close to requested size
        for (name, w, h) in &font_candidates {
            if (*w as isize - char_width as isize).abs() <= 2
                && (*h as isize - char_height as isize).abs() <= 4
            {
                if let Ok(font) = Self::load_console_font(name) {
                    return Ok(font);
                }
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "No suitable font found for {}x{} characters",
                char_width, char_height
            ),
        ))
    }

    /// Get glyph bitmap for a character
    /// Returns a slice of bytes where each byte represents one row of pixels
    pub fn get_glyph(&self, ch: char) -> &[u8] {
        let code = ch as usize;
        let glyph_index = if code < self.glyph_count { code } else { 0 };

        let start = glyph_index * self.bytes_per_glyph;
        let end = start + self.bytes_per_glyph;

        &self.glyphs[start..end]
    }

    /// Check if a pixel is set in a glyph at (x, y) position
    pub fn is_pixel_set(&self, glyph_data: &[u8], x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        let bytes_per_row = self.width.div_ceil(8);
        let row_start = y * bytes_per_row;
        let byte_index = row_start + (x / 8);
        let bit_index = 7 - (x % 8);

        if byte_index >= glyph_data.len() {
            return false;
        }

        (glyph_data[byte_index] & (1 << bit_index)) != 0
    }

    /// List all available console fonts in the system
    pub fn list_available_fonts() -> Vec<(String, usize, usize)> {
        use std::fs;

        let base_paths = ["/usr/share/consolefonts", "/usr/share/kbd/consolefonts"];

        let mut fonts = Vec::new();

        for base_path in &base_paths {
            if let Ok(entries) = fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        // Check if it's a PSF font file
                        if file_name.ends_with(".psf")
                            || file_name.ends_with(".psfu")
                            || file_name.ends_with(".psf.gz")
                        {
                            // Try to load the font to get its dimensions
                            let font_name = file_name
                                .trim_end_matches(".psf.gz")
                                .trim_end_matches(".psfu")
                                .trim_end_matches(".psf")
                                .to_string();

                            if let Ok(font) = Self::load_console_font(&font_name) {
                                fonts.push((font_name, font.width, font.height));
                            }
                        }
                    }
                }
            }
        }

        // Sort by dimensions, then by name
        fonts.sort_by(|a, b| {
            let dim_cmp = (a.1, a.2).cmp(&(b.1, b.2));
            if dim_cmp == std::cmp::Ordering::Equal {
                a.0.cmp(&b.0)
            } else {
                dim_cmp
            }
        });

        // Deduplicate
        fonts.dedup_by(|a, b| a.0 == b.0);

        fonts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psf2_header_parsing() {
        let mut bytes = vec![0u8; 32];
        // Set magic number
        bytes[0..4].copy_from_slice(&PSF2_MAGIC.to_le_bytes());
        // Set dimensions
        bytes[24..28].copy_from_slice(&16u32.to_le_bytes()); // height
        bytes[28..32].copy_from_slice(&8u32.to_le_bytes()); // width

        let header = Psf2Header::from_bytes(&bytes).unwrap();
        assert_eq!(header.magic, PSF2_MAGIC);
        assert_eq!(header.height, 16);
        assert_eq!(header.width, 8);
    }

    #[test]
    fn test_invalid_magic() {
        let bytes = vec![0u8; 32];
        let result = Psf2Header::from_bytes(&bytes);
        assert!(result.is_err());
    }
}
