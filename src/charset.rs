/// Character set configuration for rendering
#[derive(Clone, Copy, Debug)]
pub enum CharsetMode {
    Unicode,
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

    // Window controls
    pub resize_handle: char,
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
            background: '░',          // U+2591 light shade (DOS CP437 177)
            border_top_left: '╔',     // U+2554
            border_top_right: '╗',    // U+2557
            border_bottom_left: '╚',  // U+255A
            border_bottom_right: '╝', // U+255D
            border_horizontal: '═',   // U+2550
            border_vertical: '║',     // U+2551
            resize_handle: '╬', // U+256C (DOS CP437 206) box drawings double vertical and horizontal
            shadow: '▓',        // U+2593 dark shade
            block: '█',         // U+2588 full block
            shade: '░',         // U+2591 light shade
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
            border_horizontal: '-', // Dash for horizontal
            border_vertical: '|',   // Pipe for vertical
            resize_handle: '+',     // Plus for resize handle (matches corners)
            shadow: '#',            // Hash for shadow
            block: '#',             // Hash for "on" state in ASCII mode
            shade: ' ',             // Space for "off" state in ASCII mode
        }
    }

    /// Create charset from command-line arguments
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();

        // Check for --ascii flag
        if args.iter().any(|arg| arg == "--ascii") {
            Self::ascii()
        } else {
            Self::unicode()
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
