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
}
