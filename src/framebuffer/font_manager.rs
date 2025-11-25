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
const PSF1_MODEHASTAB: u8 = 0x02; // Has unicode table

/// Embedded Terminus font ter-v16n (8x16) - gzip compressed
/// Copyright (c) 2010-2020 Dimitar Toshkov Zhekov
/// Licensed under SIL Open Font License 1.1 - see TERMINUS-FONT-LICENSE.txt
const EMBEDDED_FONT_TER_V16N: &[u8] = include_bytes!("ter-v16n.psf.gz");

/// Embedded Terminus font ter-v32n (16x32) - gzip compressed
/// Copyright (c) 2010-2020 Dimitar Toshkov Zhekov
/// Licensed under SIL Open Font License 1.1 - see TERMINUS-FONT-LICENSE.txt
const EMBEDDED_FONT_TER_V32N: &[u8] = include_bytes!("ter-v32n.psf.gz");

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

/// Parse PSF1 Unicode table from raw bytes
/// PSF1 tables use 16-bit little-endian values with 0xFFFF as separator
/// Returns a HashMap mapping Unicode codepoints to glyph indices
fn parse_psf1_unicode_table(
    data: &[u8],
    glyph_count: usize,
) -> std::collections::HashMap<u32, usize> {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let mut pos = 0;
    let mut glyph_idx = 0;

    while pos + 1 < data.len() && glyph_idx < glyph_count {
        let value = u16::from_le_bytes([data[pos], data[pos + 1]]);
        pos += 2;

        if value == 0xFFFF {
            // End of this glyph's unicode list
            glyph_idx += 1;
        } else if value == 0xFFFE {
            // Start of combining sequence - skip until 0xFFFF
            while pos + 1 < data.len() {
                let v = u16::from_le_bytes([data[pos], data[pos + 1]]);
                pos += 2;
                if v == 0xFFFF {
                    glyph_idx += 1;
                    break;
                }
            }
        } else {
            // Regular Unicode codepoint (16-bit, covers BMP which includes box-drawing)
            map.insert(value as u32, glyph_idx);
        }
    }

    map
}

/// Parse PSF2 Unicode table from raw bytes
/// Returns a HashMap mapping Unicode codepoints to glyph indices
fn parse_psf2_unicode_table(
    data: &[u8],
    glyph_count: usize,
) -> std::collections::HashMap<u32, usize> {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    let mut pos = 0;
    let mut glyph_idx = 0;

    while pos < data.len() && glyph_idx < glyph_count {
        // Read UTF-8 sequences until we hit 0xFF (end of glyph) or 0xFE (combining)
        if data[pos] == 0xFF {
            // End of this glyph's unicode list
            glyph_idx += 1;
            pos += 1;
            continue;
        }

        if data[pos] == 0xFE {
            // Start of combining sequence - skip until 0xFF
            while pos < data.len() && data[pos] != 0xFF {
                pos += 1;
            }
            continue;
        }

        // Decode UTF-8 sequence
        let (codepoint, bytes_read) = decode_utf8(&data[pos..]);
        if bytes_read > 0 {
            map.insert(codepoint, glyph_idx);
            pos += bytes_read;
        } else {
            // Invalid UTF-8, skip byte
            pos += 1;
        }
    }

    map
}

/// Decode a single UTF-8 character from bytes
/// Returns (codepoint, bytes_consumed) or (0, 0) on error
fn decode_utf8(data: &[u8]) -> (u32, usize) {
    if data.is_empty() {
        return (0, 0);
    }

    let b0 = data[0];

    // Single byte (ASCII)
    if b0 < 0x80 {
        return (b0 as u32, 1);
    }

    // Multi-byte sequence
    if b0 < 0xC0 {
        // Invalid start byte
        return (0, 0);
    }

    if b0 < 0xE0 {
        // 2-byte sequence
        if data.len() < 2 {
            return (0, 0);
        }
        let cp = ((b0 as u32 & 0x1F) << 6) | (data[1] as u32 & 0x3F);
        return (cp, 2);
    }

    if b0 < 0xF0 {
        // 3-byte sequence
        if data.len() < 3 {
            return (0, 0);
        }
        let cp =
            ((b0 as u32 & 0x0F) << 12) | ((data[1] as u32 & 0x3F) << 6) | (data[2] as u32 & 0x3F);
        return (cp, 3);
    }

    if b0 < 0xF8 {
        // 4-byte sequence
        if data.len() < 4 {
            return (0, 0);
        }
        let cp = ((b0 as u32 & 0x07) << 18)
            | ((data[1] as u32 & 0x3F) << 12)
            | ((data[2] as u32 & 0x3F) << 6)
            | (data[3] as u32 & 0x3F);
        return (cp, 4);
    }

    (0, 0)
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
    /// Precomputed bytes per row (avoids division in hot path)
    pub bytes_per_row: usize,
    /// Fast path flag: true if font width is exactly 8 pixels (most common)
    pub is_width_8: bool,
    /// Unicode to glyph index mapping (for characters > 127)
    unicode_map: std::collections::HashMap<u32, usize>,
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

            // Read unicode table if present (flags bit 0)
            let unicode_map = if header.flags & 1 != 0 {
                let mut unicode_data = Vec::new();
                file.read_to_end(&mut unicode_data)?;
                parse_psf2_unicode_table(&unicode_data, header.length as usize)
            } else {
                std::collections::HashMap::new()
            };

            let width = header.width as usize;
            let bytes_per_row = width.div_ceil(8);
            let is_width_8 = width == 8;

            return Ok(FontManager {
                width,
                height: header.height as usize,
                glyph_count: header.length as usize,
                bytes_per_glyph: header.charsize as usize,
                glyphs,
                bytes_per_row,
                is_width_8,
                unicode_map,
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

            // Read unicode table if present (PSF1_MODEHASTAB flag)
            let unicode_map = if header.mode & PSF1_MODEHASTAB != 0 {
                let mut unicode_data = Vec::new();
                file.read_to_end(&mut unicode_data)?;
                parse_psf1_unicode_table(&unicode_data, glyph_count)
            } else {
                std::collections::HashMap::new()
            };

            return Ok(FontManager {
                width,
                height,
                glyph_count,
                bytes_per_glyph,
                glyphs,
                bytes_per_row: 1, // PSF1 is always 8 pixels wide = 1 byte per row
                is_width_8: true, // PSF1 is always 8 pixels wide
                unicode_map,
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

        // Try multiple common locations for console fonts across distros
        let base_paths = [
            "/usr/share/consolefonts",           // Debian/Ubuntu primary location
            "/usr/share/kbd/consolefonts",       // Debian/Ubuntu alternative, some older systems
            "/usr/lib/kbd/consolefonts",         // Fedora/RHEL/CentOS
            "/lib/kbd/consolefonts",             // Some Fedora configurations
            "/usr/share/console/consolefonts",   // Some BSDs
            "/usr/local/share/consolefonts",     // Local installations
            "/usr/local/share/kbd/consolefonts", // Local kbd installations
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

            // Read unicode table if present (flags bit 0)
            let unicode_map = if header.flags & 1 != 0 {
                let mut unicode_data = Vec::new();
                gz_decoder.read_to_end(&mut unicode_data)?;
                parse_psf2_unicode_table(&unicode_data, header.length as usize)
            } else {
                std::collections::HashMap::new()
            };

            let width = header.width as usize;
            let bytes_per_row = width.div_ceil(8);
            let is_width_8 = width == 8;

            return Ok(FontManager {
                width,
                height: header.height as usize,
                glyph_count: header.length as usize,
                bytes_per_glyph: header.charsize as usize,
                glyphs,
                bytes_per_row,
                is_width_8,
                unicode_map,
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

            // Read unicode table if present (PSF1_MODEHASTAB flag)
            let unicode_map = if header.mode & PSF1_MODEHASTAB != 0 {
                let mut unicode_data = Vec::new();
                gz_decoder.read_to_end(&mut unicode_data)?;
                parse_psf1_unicode_table(&unicode_data, glyph_count)
            } else {
                std::collections::HashMap::new()
            };

            return Ok(FontManager {
                width,
                height,
                glyph_count,
                bytes_per_glyph,
                glyphs,
                bytes_per_row: 1, // PSF1 is always 8 pixels wide = 1 byte per row
                is_width_8: true, // PSF1 is always 8 pixels wide
                unicode_map,
            });
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown font format in gzip. Magic: 0x{:08x}", magic32),
        ))
    }

    /// Load a PSF font from gzip-compressed bytes (supports both PSF1 and PSF2)
    /// Used for embedded fonts compiled into the binary
    pub fn load_from_gzip_bytes(data: &[u8]) -> io::Result<Self> {
        use std::io::Cursor;

        let cursor = Cursor::new(data);
        let mut gz_decoder = GzDecoder::new(cursor);

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

            // Read unicode table if present (flags bit 0)
            let unicode_map = if header.flags & 1 != 0 {
                let mut unicode_data = Vec::new();
                gz_decoder.read_to_end(&mut unicode_data)?;
                parse_psf2_unicode_table(&unicode_data, header.length as usize)
            } else {
                std::collections::HashMap::new()
            };

            let width = header.width as usize;
            let bytes_per_row = width.div_ceil(8);
            let is_width_8 = width == 8;

            return Ok(FontManager {
                width,
                height: header.height as usize,
                glyph_count: header.length as usize,
                bytes_per_glyph: header.charsize as usize,
                glyphs,
                bytes_per_row,
                is_width_8,
                unicode_map,
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

            // Read unicode table if present (PSF1_MODEHASTAB flag)
            let unicode_map = if header.mode & PSF1_MODEHASTAB != 0 {
                let mut unicode_data = Vec::new();
                gz_decoder.read_to_end(&mut unicode_data)?;
                parse_psf1_unicode_table(&unicode_data, glyph_count)
            } else {
                std::collections::HashMap::new()
            };

            return Ok(FontManager {
                width,
                height,
                glyph_count,
                bytes_per_glyph,
                glyphs,
                bytes_per_row: 1,
                is_width_8: true,
                unicode_map,
            });
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Unknown font format in embedded gzip data. Magic: 0x{:08x}",
                magic32
            ),
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

        // Final fallback: use embedded fonts
        Self::load_embedded_font(char_width, char_height)
    }

    /// Load an embedded font that best matches the requested dimensions
    /// This is the fallback when no system fonts are available
    pub fn load_embedded_font(char_width: usize, char_height: usize) -> io::Result<Self> {
        // Choose embedded font based on requested dimensions
        // ter-v16n is 8x16, ter-v32n is 16x32
        let use_large_font = char_width > 8 || char_height > 16;

        if use_large_font {
            Self::load_from_gzip_bytes(EMBEDDED_FONT_TER_V32N)
        } else {
            Self::load_from_gzip_bytes(EMBEDDED_FONT_TER_V16N)
        }
    }

    /// Load the default embedded font (ter-v16n, 8x16)
    pub fn load_embedded_default() -> io::Result<Self> {
        Self::load_from_gzip_bytes(EMBEDDED_FONT_TER_V16N)
    }

    /// Get glyph bitmap for a character
    /// Returns a slice of bytes where each byte represents one row of pixels
    pub fn get_glyph(&self, ch: char) -> &[u8] {
        let code = ch as u32;

        // Determine glyph index:
        // 1. For ASCII range (0-127), use direct mapping if within glyph count
        // 2. For higher Unicode, look up in unicode_map
        // 3. Fall back to glyph 0 (usually a placeholder/missing glyph)
        let glyph_index = if code < 128 && (code as usize) < self.glyph_count {
            code as usize
        } else if let Some(&idx) = self.unicode_map.get(&code) {
            idx
        } else if (code as usize) < self.glyph_count {
            // Direct mapping for codes within glyph range
            code as usize
        } else {
            0
        };

        let start = glyph_index * self.bytes_per_glyph;
        let end = start + self.bytes_per_glyph;

        &self.glyphs[start..end]
    }

    /// Debug: Print font info
    pub fn debug_info(&self) -> String {
        format!(
            "Font: {}x{}, {} glyphs, {} bytes/glyph, {} total bytes",
            self.width,
            self.height,
            self.glyph_count,
            self.bytes_per_glyph,
            self.glyphs.len()
        )
    }

    /// Get the number of glyphs in the font
    pub fn glyph_count(&self) -> usize {
        self.glyph_count
    }

    /// Check if a pixel is set in a glyph at (x, y) position
    /// Optimized with fast path for common 8-pixel-wide fonts
    #[inline(always)]
    pub fn is_pixel_set(&self, glyph_data: &[u8], x: usize, y: usize) -> bool {
        // Fast path for 8-pixel-wide fonts (most common case: 8x16, 8x8)
        // Eliminates bounds check on x (caller guarantees x < 8) and division
        if self.is_width_8 {
            // Each row is exactly 1 byte when width is 8
            if y >= glyph_data.len() {
                return false;
            }
            // Use shift instead of calculating bit_index = 7 - (x % 8)
            // Since x < 8, we can use it directly: bit 7 is leftmost (x=0), bit 0 is rightmost (x=7)
            return (glyph_data[y] & (0x80 >> x)) != 0;
        }

        // General path for wider fonts (16x16, etc.)
        if x >= self.width || y >= self.height {
            return false;
        }

        // Use precomputed bytes_per_row (avoids division)
        let row_start = y * self.bytes_per_row;
        // Use bit shift instead of division by 8
        let byte_index = row_start + (x >> 3);
        // Use AND instead of modulo 8
        let bit_index = 7 - (x & 7);

        if byte_index >= glyph_data.len() {
            return false;
        }

        (glyph_data[byte_index] & (1 << bit_index)) != 0
    }

    /// List all available console fonts in the system
    /// Includes embedded fonts (ter-v16n, ter-v32n) which are always available
    pub fn list_available_fonts() -> Vec<(String, usize, usize)> {
        use std::fs;

        // Same paths as load_console_font for consistency across distros
        let base_paths = [
            "/usr/share/consolefonts",           // Debian/Ubuntu
            "/usr/share/kbd/consolefonts",       // Debian/Ubuntu alt
            "/usr/lib/kbd/consolefonts",         // Fedora/RHEL/CentOS
            "/lib/kbd/consolefonts",             // Some Fedora configurations
            "/usr/share/console/consolefonts",   // Some BSDs
            "/usr/local/share/consolefonts",     // Local installations
            "/usr/local/share/kbd/consolefonts", // Local kbd installations
        ];

        let mut fonts = Vec::new();

        // Add embedded fonts first (always available)
        fonts.push(("[Embedded] ter-v16n".to_string(), 8, 16));
        fonts.push(("[Embedded] ter-v32n".to_string(), 16, 32));

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

        // Sort by dimensions, then by name (but keep embedded fonts at top)
        fonts.sort_by(|a, b| {
            // Embedded fonts come first
            let a_embedded = a.0.starts_with("[Embedded]");
            let b_embedded = b.0.starts_with("[Embedded]");
            if a_embedded && !b_embedded {
                return std::cmp::Ordering::Less;
            }
            if !a_embedded && b_embedded {
                return std::cmp::Ordering::Greater;
            }

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

    /// Load a font by name, supporting both system fonts and embedded fonts
    /// Embedded font names start with "[Embedded] "
    pub fn load_font_by_name(name: &str) -> io::Result<Self> {
        if name == "[Embedded] ter-v16n" {
            Self::load_from_gzip_bytes(EMBEDDED_FONT_TER_V16N)
        } else if name == "[Embedded] ter-v32n" {
            Self::load_from_gzip_bytes(EMBEDDED_FONT_TER_V32N)
        } else {
            Self::load_console_font(name)
        }
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
