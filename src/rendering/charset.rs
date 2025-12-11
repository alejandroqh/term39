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

    // Pivot for tiled window resizing
    pub pivot: char,

    // Menu item icons
    pub icon_copy: char,
    pub icon_paste: char,
    pub icon_clear: char,
    pub icon_settings: char,
    pub icon_help: char,
    pub icon_about: char,
    pub icon_exit: char,

    // Network widget icons
    pub network_signal_1: char, // Weakest signal bar
    pub network_signal_2: char,
    pub network_signal_3: char,
    pub network_signal_4: char, // Strongest signal bar
    pub network_connected: char,
    pub network_disconnected: char,

    // Battery widget icons
    pub battery_full: char,
    pub battery_high: char,
    pub battery_medium: char,
    pub battery_low: char,
    pub battery_critical: char,
    pub battery_charging: char,
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
            pivot: '✛',                 // U+271B Heavy Greek cross
            // Menu item icons (Unicode)
            icon_copy: '\u{29C9}',     // ⧉ Two Joined Squares
            icon_paste: '\u{29E0}',    // ⧠ Square with Contoured Outline
            icon_clear: '\u{232B}',    // ⌫ Erase to the Left
            icon_settings: '\u{2699}', // ⚙ Gear
            icon_help: '?',            // Question mark
            icon_about: '\u{24D8}',    // ⓘ Circled Latin Small Letter I
            icon_exit: '\u{23FB}',     // ⏻ Power Symbol
            // Network widget icons (Unicode)
            network_signal_1: '\u{2582}',  // ▂ Lower one quarter block
            network_signal_2: '\u{2584}',  // ▄ Lower half block
            network_signal_3: '\u{2586}',  // ▆ Lower three quarters block
            network_signal_4: '\u{2588}',  // █ Full block
            network_connected: '\u{25A3}', // ▣ White square containing black small square
            network_disconnected: '\u{2717}', // ✗ Ballot X
            // Battery widget icons (Unicode)
            battery_full: '\u{2588}',     // █ Full block
            battery_high: '\u{2593}',     // ▓ Dark shade
            battery_medium: '\u{2592}',   // ▒ Medium shade
            battery_low: '\u{2591}',      // ░ Light shade
            battery_critical: '\u{2581}', // ▁ Lower one eighth block
            battery_charging: '\u{21AF}', // ↯ Downwards zigzag arrow
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
            pivot: '✛',                 // U+271B Heavy Greek cross
            // Menu item icons (Unicode - same as double-line)
            icon_copy: '\u{29C9}',     // ⧉ Two Joined Squares
            icon_paste: '\u{29E0}',    // ⧠ Square with Contoured Outline
            icon_clear: '\u{232B}',    // ⌫ Erase to the Left
            icon_settings: '\u{2699}', // ⚙ Gear
            icon_help: '?',            // Question mark
            icon_about: '\u{24D8}',    // ⓘ Circled Latin Small Letter I
            icon_exit: '\u{23FB}',     // ⏻ Power Symbol
            // Network widget icons (Unicode - same as double-line)
            network_signal_1: '\u{2582}',  // ▂ Lower one quarter block
            network_signal_2: '\u{2584}',  // ▄ Lower half block
            network_signal_3: '\u{2586}',  // ▆ Lower three quarters block
            network_signal_4: '\u{2588}',  // █ Full block
            network_connected: '\u{25A3}', // ▣ White square containing black small square
            network_disconnected: '\u{2717}', // ✗ Ballot X
            // Battery widget icons (Unicode - same as double-line)
            battery_full: '\u{2588}',     // █ Full block
            battery_high: '\u{2593}',     // ▓ Dark shade
            battery_medium: '\u{2592}',   // ▒ Medium shade
            battery_low: '\u{2591}',      // ░ Light shade
            battery_critical: '\u{2581}', // ▁ Lower one eighth block
            battery_charging: '\u{21AF}', // ↯ Downwards zigzag arrow
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
            pivot: '+',                 // Plus for ASCII mode
            // Menu item icons (ASCII)
            icon_copy: 'C',     // C for Copy
            icon_paste: 'P',    // P for Paste
            icon_clear: 'X',    // X for Clear/Delete
            icon_settings: '*', // * for Settings
            icon_help: '?',     // ? for Help
            icon_about: 'i',    // i for Info/About
            icon_exit: 'Q',     // Q for Quit/Exit
            // Network widget icons (ASCII)
            network_signal_1: '_',     // _ for weakest
            network_signal_2: '.',     // . for low
            network_signal_3: 'o',     // o for medium
            network_signal_4: 'O',     // O for full
            network_connected: '+',    // + for connected
            network_disconnected: 'x', // x for disconnected
            // Battery widget icons (ASCII)
            battery_full: '#',     // # for full
            battery_high: '=',     // = for high
            battery_medium: '-',   // - for medium
            battery_low: '.',      // . for low
            battery_critical: '_', // _ for critical
            battery_charging: '~', // ~ for charging (lightning-like)
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
