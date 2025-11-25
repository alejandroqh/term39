/// Character set configuration for rendering
#[derive(Clone, Copy, Debug)]
pub enum CharsetMode {
    Unicode,
    UnicodeSingleLine,
    Ascii,
}

/// Character definitions for UI elements
#[derive(Clone, Copy, Debug)]
pub struct Charset {
    pub mode: CharsetMode,

    // Background
    pub background: char,

    // Window borders
    pub border_top_left: char,
    pub border_top_right: char,
    pub border_bottom_left: char,
    pub border_bottom_right: char,
    pub border_horizontal: char,
    pub border_vertical: char,
    pub border_vertical_right: char, // T-junction (╠ or +)

    // Window controls
    pub shadow: char,

    // Configuration window toggles
    pub block: char, // Full block for "on" state
    pub shade: char, // Light shade for "off" state
}

impl Charset {
    /// Create Unicode charset (default)
    pub fn unicode() -> Self {
        Self {
            mode: CharsetMode::Unicode,
            background: '░',            // U+2591 light shade (DOS CP437 177)
            border_top_left: '╔',       // U+2554
            border_top_right: '╗',      // U+2557
            border_bottom_left: '╚',    // U+255A
            border_bottom_right: '╝',   // U+255D
            border_horizontal: '═',     // U+2550
            border_vertical: '║',       // U+2551
            border_vertical_right: '╠', // U+2560 T-junction
            shadow: '▓',                // U+2593 dark shade
            block: '█',                 // U+2588 full block
            shade: '░',                 // U+2591 light shade
        }
    }

    /// Create Unicode single-line charset (for fonts without double-line box drawing)
    /// Uses single-line box drawing characters (U+250x) instead of double-line (U+255x)
    pub fn unicode_single_line() -> Self {
        Self {
            mode: CharsetMode::UnicodeSingleLine,
            background: '░',            // U+2591 light shade (DOS CP437 177)
            border_top_left: '┌',       // U+250C (single-line corner)
            border_top_right: '┐',      // U+2510
            border_bottom_left: '└',    // U+2514
            border_bottom_right: '┘',   // U+2518
            border_horizontal: '─',     // U+2500 (single-line horizontal)
            border_vertical: '│',       // U+2502 (single-line vertical)
            border_vertical_right: '├', // U+251C T-junction
            shadow: '▓',                // U+2593 dark shade
            block: '█',                 // U+2588 full block
            shade: '░',                 // U+2591 light shade
        }
    }

    /// Create ASCII-compatible charset
    pub fn ascii() -> Self {
        Self {
            mode: CharsetMode::Ascii,
            background: ' ',      // Space for clean background
            border_top_left: '+', // Plus for corners
            border_top_right: '+',
            border_bottom_left: '+',
            border_bottom_right: '+',
            border_horizontal: '-',     // Dash for horizontal
            border_vertical: '|',       // Pipe for vertical
            border_vertical_right: '+', // Plus for T-junction
            shadow: '#',                // Hash for shadow
            block: '#',                 // Hash for "on" state in ASCII mode
            shade: ' ',                 // Space for "off" state in ASCII mode
        }
    }

    // Accessor methods for border characters
    pub fn border_top_left(&self) -> char {
        self.border_top_left
    }

    pub fn border_top_right(&self) -> char {
        self.border_top_right
    }

    pub fn border_bottom_left(&self) -> char {
        self.border_bottom_left
    }

    pub fn border_bottom_right(&self) -> char {
        self.border_bottom_right
    }

    pub fn border_horizontal(&self) -> char {
        self.border_horizontal
    }

    pub fn border_vertical(&self) -> char {
        self.border_vertical
    }

    // Accessor methods for toggle characters
    pub fn block(&self) -> char {
        self.block
    }

    pub fn shade(&self) -> char {
        self.shade
    }

    /// Set a custom background character
    pub fn set_background(&mut self, background_char: char) {
        self.background = background_char;
    }
}
