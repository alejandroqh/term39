use clap::Parser;

const LONG_ABOUT: &str = "\
A modern, retro-styled terminal multiplexer inspired by Norton Disk Doctor (MS-DOS).
Features a full-screen text-based interface with a classic DOS aesthetic.

 ███████████ ██████████ ███████████   ██████   ██████  ████████   ████████
░█░░░███░░░█░░███░░░░░█░░███░░░░░███ ░░██████ ██████  ███░░░░███ ███░░░░███
░   ░███  ░  ░███  █ ░  ░███    ░███  ░███░█████░███ ░░░    ░███░███   ░███
    ░███     ░██████    ░██████████   ░███░░███ ░███    ██████░ ░░█████████
    ░███     ░███░░█    ░███░░░░░███  ░███ ░░░  ░███   ░░░░░░███ ░░░░░░░███
    ░███     ░███ ░   █ ░███    ░███  ░███      ░███  ███   ░███ ███   ░███
    █████    ██████████ █████   █████ █████     █████░░████████ ░░████████
   ░░░░░    ░░░░░░░░░░ ░░░░░   ░░░░░ ░░░░░     ░░░░░  ░░░░░░░░   ░░░░░░░░

KEYBOARD SHORTCUTS:
  't'         - Create new terminal window
  'T'         - Create new maximized terminal window
  'q' / ESC   - Exit application (from desktop)
  'h'         - Show help screen
  's'         - Show settings/configuration window
  'c'         - Show calendar
  CTRL+L      - Clear terminal
  ALT+TAB     - Switch between windows

MOUSE CONTROLS:
  Click title bar     - Drag window
  CTRL+Drag          - Drag without snap
  Click [X]          - Close window
  Drag ╬ handle      - Resize window
  Click window       - Focus window
";

#[derive(Parser, Debug)]
#[command(
    name = "term39",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = "A modern, retro-styled terminal multiplexer inspired by Norton Disk Doctor",
    long_about = LONG_ABOUT,
    after_help = "For more information, visit: https://github.com/alejandroqh/term39"
)]
pub struct Cli {
    /// Use ASCII-compatible characters instead of Unicode for better terminal compatibility
    ///
    /// This mode uses simple ASCII characters (+-|#) for borders and UI elements
    /// instead of Unicode box-drawing characters, ensuring compatibility with
    /// terminals that don't support extended character sets.
    #[arg(long, help = "Use ASCII-compatible characters instead of Unicode")]
    pub ascii: bool,

    /// Set the color theme (classic, monochrome, dark, green_phosphor, amber)
    ///
    /// Available themes:
    ///   - classic:        DOS-inspired blue and cyan colors (default)
    ///   - monochrome:     Grayscale theme with black and white
    ///   - dark:           Dark theme inspired by Dracula color scheme
    ///   - green_phosphor: Green monochrome resembling classic CRT terminals (IBM 5151, VT220)
    ///   - amber:          Amber/orange monochrome resembling classic terminals (DEC VT100)
    #[arg(long, value_name = "THEME", help = "Set the color theme")]
    pub theme: Option<String>,

    /// Apply theme-based color tinting to terminal content
    ///
    /// When enabled, terminal output colors will be transformed to match the current
    /// theme's color palette. This provides a cohesive aesthetic but may alter the
    /// appearance of terminal programs. Disabled by default to preserve native ANSI colors.
    #[arg(long, help = "Apply theme-based tinting to terminal content")]
    pub tint_terminal: bool,
}

impl Cli {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
