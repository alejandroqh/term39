use crate::term_emu::{CellAttributes, Color, Cursor, CursorShape, NamedColor, TerminalCell};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Maximum number of lines to save per terminal (scrollback + visible)
pub const MAX_LINES_PER_TERMINAL: usize = 2000;

/// Maximum session file size (10 MB) to prevent memory exhaustion attacks
const MAX_SESSION_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Session file version for compatibility checking
const SESSION_VERSION: u8 = 1;

/// Complete session state that can be saved/restored
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionState {
    /// Session format version
    pub version: u8,
    /// Next window ID to use
    pub next_id: u32,
    /// All windows in z-order (last = topmost)
    pub windows: Vec<WindowSnapshot>,
    /// ID of focused window (None = desktop focused)
    pub focused_window_id: Option<u32>,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            version: SESSION_VERSION,
            next_id: 1,
            windows: Vec::new(),
            focused_window_id: None,
        }
    }
}

/// Snapshot of a single window's state
#[derive(Debug, Serialize, Deserialize)]
pub struct WindowSnapshot {
    // Identification
    pub id: u32,
    pub title: String,

    // Geometry
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,

    // State
    pub is_focused: bool,
    pub is_minimized: bool,
    pub is_maximized: bool,

    // Pre-maximize state (for restore)
    pub pre_maximize_x: u16,
    pub pre_maximize_y: u16,
    pub pre_maximize_width: u16,
    pub pre_maximize_height: u16,

    // Terminal state
    pub scroll_offset: usize,
    pub cursor: SerializableCursor,

    // Terminal content (capped at MAX_LINES_PER_TERMINAL)
    pub terminal_lines: Vec<SerializableTerminalLine>,
}

/// Serializable version of a terminal line
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableTerminalLine {
    pub cells: Vec<SerializableCell>,
}

/// Serializable version of a terminal cell
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableCell {
    pub c: char,
    pub fg: SerializableColor,
    pub bg: SerializableColor,
    pub attrs: SerializableCellAttributes,
}

/// Serializable version of Color
#[derive(Debug, Serialize, Deserialize)]
pub enum SerializableColor {
    Named(SerializableNamedColor),
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// Serializable version of NamedColor
#[derive(Debug, Serialize, Deserialize)]
pub enum SerializableNamedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

/// Serializable version of CellAttributes
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableCellAttributes {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
}

/// Serializable version of Cursor
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableCursor {
    pub x: usize,
    pub y: usize,
    pub visible: bool,
    pub shape: SerializableCursorShape,
}

/// Serializable version of CursorShape
#[derive(Debug, Serialize, Deserialize)]
pub enum SerializableCursorShape {
    Block,
    Underline,
    Bar,
}

// Conversion functions from internal types to serializable types

impl From<&TerminalCell> for SerializableCell {
    fn from(cell: &TerminalCell) -> Self {
        Self {
            c: cell.c,
            fg: SerializableColor::from(&cell.fg),
            bg: SerializableColor::from(&cell.bg),
            attrs: SerializableCellAttributes::from(&cell.attrs),
        }
    }
}

impl From<&Color> for SerializableColor {
    fn from(color: &Color) -> Self {
        match color {
            Color::Named(nc) => SerializableColor::Named(SerializableNamedColor::from(nc)),
            Color::Indexed(i) => SerializableColor::Indexed(*i),
            Color::Rgb(r, g, b) => SerializableColor::Rgb(*r, *g, *b),
        }
    }
}

impl From<&NamedColor> for SerializableNamedColor {
    fn from(nc: &NamedColor) -> Self {
        match nc {
            NamedColor::Black => SerializableNamedColor::Black,
            NamedColor::Red => SerializableNamedColor::Red,
            NamedColor::Green => SerializableNamedColor::Green,
            NamedColor::Yellow => SerializableNamedColor::Yellow,
            NamedColor::Blue => SerializableNamedColor::Blue,
            NamedColor::Magenta => SerializableNamedColor::Magenta,
            NamedColor::Cyan => SerializableNamedColor::Cyan,
            NamedColor::White => SerializableNamedColor::White,
            NamedColor::BrightBlack => SerializableNamedColor::BrightBlack,
            NamedColor::BrightRed => SerializableNamedColor::BrightRed,
            NamedColor::BrightGreen => SerializableNamedColor::BrightGreen,
            NamedColor::BrightYellow => SerializableNamedColor::BrightYellow,
            NamedColor::BrightBlue => SerializableNamedColor::BrightBlue,
            NamedColor::BrightMagenta => SerializableNamedColor::BrightMagenta,
            NamedColor::BrightCyan => SerializableNamedColor::BrightCyan,
            NamedColor::BrightWhite => SerializableNamedColor::BrightWhite,
        }
    }
}

impl From<&CellAttributes> for SerializableCellAttributes {
    fn from(attrs: &CellAttributes) -> Self {
        Self {
            bold: attrs.bold,
            dim: attrs.dim,
            italic: attrs.italic,
            underline: attrs.underline,
            blink: attrs.blink,
            reverse: attrs.reverse,
            hidden: attrs.hidden,
            strikethrough: attrs.strikethrough,
        }
    }
}

impl From<&Cursor> for SerializableCursor {
    fn from(cursor: &Cursor) -> Self {
        Self {
            x: cursor.x,
            y: cursor.y,
            visible: cursor.visible,
            shape: SerializableCursorShape::from(&cursor.shape),
        }
    }
}

impl From<&CursorShape> for SerializableCursorShape {
    fn from(shape: &CursorShape) -> Self {
        match shape {
            CursorShape::Block => SerializableCursorShape::Block,
            CursorShape::Underline => SerializableCursorShape::Underline,
            CursorShape::Bar => SerializableCursorShape::Bar,
        }
    }
}

// Conversion functions from serializable types back to internal types

impl From<&SerializableCell> for TerminalCell {
    fn from(cell: &SerializableCell) -> Self {
        Self {
            c: cell.c,
            fg: Color::from(&cell.fg),
            bg: Color::from(&cell.bg),
            attrs: CellAttributes::from(&cell.attrs),
        }
    }
}

impl From<&SerializableColor> for Color {
    fn from(color: &SerializableColor) -> Self {
        match color {
            SerializableColor::Named(nc) => Color::Named(NamedColor::from(nc)),
            SerializableColor::Indexed(i) => Color::Indexed(*i),
            SerializableColor::Rgb(r, g, b) => Color::Rgb(*r, *g, *b),
        }
    }
}

impl From<&SerializableNamedColor> for NamedColor {
    fn from(nc: &SerializableNamedColor) -> Self {
        match nc {
            SerializableNamedColor::Black => NamedColor::Black,
            SerializableNamedColor::Red => NamedColor::Red,
            SerializableNamedColor::Green => NamedColor::Green,
            SerializableNamedColor::Yellow => NamedColor::Yellow,
            SerializableNamedColor::Blue => NamedColor::Blue,
            SerializableNamedColor::Magenta => NamedColor::Magenta,
            SerializableNamedColor::Cyan => NamedColor::Cyan,
            SerializableNamedColor::White => NamedColor::White,
            SerializableNamedColor::BrightBlack => NamedColor::BrightBlack,
            SerializableNamedColor::BrightRed => NamedColor::BrightRed,
            SerializableNamedColor::BrightGreen => NamedColor::BrightGreen,
            SerializableNamedColor::BrightYellow => NamedColor::BrightYellow,
            SerializableNamedColor::BrightBlue => NamedColor::BrightBlue,
            SerializableNamedColor::BrightMagenta => NamedColor::BrightMagenta,
            SerializableNamedColor::BrightCyan => NamedColor::BrightCyan,
            SerializableNamedColor::BrightWhite => NamedColor::BrightWhite,
        }
    }
}

impl From<&SerializableCellAttributes> for CellAttributes {
    fn from(attrs: &SerializableCellAttributes) -> Self {
        Self {
            bold: attrs.bold,
            dim: attrs.dim,
            italic: attrs.italic,
            underline: attrs.underline,
            blink: attrs.blink,
            reverse: attrs.reverse,
            hidden: attrs.hidden,
            strikethrough: attrs.strikethrough,
        }
    }
}

impl From<&SerializableCursor> for Cursor {
    fn from(cursor: &SerializableCursor) -> Self {
        Self {
            x: cursor.x,
            y: cursor.y,
            visible: cursor.visible,
            shape: CursorShape::from(&cursor.shape),
        }
    }
}

impl From<&SerializableCursorShape> for CursorShape {
    fn from(shape: &SerializableCursorShape) -> Self {
        match shape {
            SerializableCursorShape::Block => CursorShape::Block,
            SerializableCursorShape::Underline => CursorShape::Underline,
            SerializableCursorShape::Bar => CursorShape::Bar,
        }
    }
}

/// Get the default session file path (XDG config directory)
pub fn get_session_path() -> io::Result<PathBuf> {
    let config_dir =
        directories::ProjectDirs::from("com", "term39", "term39").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Could not determine config directory",
            )
        })?;

    let config_path = config_dir.config_dir();

    // Create config directory if it doesn't exist
    fs::create_dir_all(config_path)?;

    Ok(config_path.join("session.json"))
}

/// Save session state to a file
pub fn save_session(state: &SessionState, path: &Path) -> io::Result<()> {
    // Serialize to JSON
    let json = serde_json::to_string_pretty(state).map_err(io::Error::other)?;

    // Write to temp file first, then rename (atomic operation)
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, json)?;
    fs::rename(&temp_path, path)?;

    Ok(())
}

/// Load session state from a file
pub fn load_session(path: &Path) -> io::Result<Option<SessionState>> {
    // If file doesn't exist, return None (not an error)
    if !path.exists() {
        return Ok(None);
    }

    // Check file size before loading to prevent memory exhaustion
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_SESSION_FILE_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Session file too large: {} bytes (max {} bytes)",
                metadata.len(),
                MAX_SESSION_FILE_SIZE
            ),
        ));
    }

    // Read file
    let contents = fs::read_to_string(path)?;

    // Deserialize
    let state: SessionState = serde_json::from_str(&contents).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse session file: {}", e),
        )
    })?;

    // Check version compatibility
    if state.version != SESSION_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Incompatible session version: {} (expected {})",
                state.version, SESSION_VERSION
            ),
        ));
    }

    Ok(Some(state))
}

/// Clear/delete session file
pub fn clear_session() -> io::Result<()> {
    let path = get_session_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
