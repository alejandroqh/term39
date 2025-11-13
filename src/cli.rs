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

    /// Enable framebuffer mode (Linux console only, requires --features framebuffer-backend)
    ///
    /// Use direct framebuffer rendering on Linux console (TTY) for pixel-perfect DOS-like display.
    /// Requires /dev/fb0 access and running on physical console (not SSH or terminal emulators).
    ///
    /// Note: Only available when compiled with framebuffer-backend feature.
    #[cfg(feature = "framebuffer-backend")]
    #[arg(
        long = "framebuffer",
        short = 'f',
        help = "Enable framebuffer mode (Linux console only)"
    )]
    pub framebuffer: bool,

    /// Framebuffer text mode (Linux console only, requires --features framebuffer-backend)
    ///
    /// Select DOS-like text mode for framebuffer rendering:
    ///   - 40x25:  40 columns × 25 rows (16×16 character cells)
    ///   - 80x25:  80 columns × 25 rows (8×16 character cells) - Standard DOS mode (default)
    ///   - 80x43:  80 columns × 43 rows (8×11 character cells)
    ///   - 80x50:  80 columns × 50 rows (8×8 character cells) - High density mode
    ///
    /// Note: Only takes effect when --framebuffer/-f is specified.
    #[cfg(feature = "framebuffer-backend")]
    #[arg(
        long,
        value_name = "MODE",
        default_value = "80x25",
        help = "Framebuffer text mode (40x25, 80x25, 80x43, 80x50)"
    )]
    pub fb_mode: String,

    /// Framebuffer pixel scale factor (Linux console only, requires --features framebuffer-backend)
    ///
    /// Integer scaling factor for framebuffer rendering:
    ///   - 1:  Native resolution (640×400 for 80x25 mode)
    ///   - 2:  2x scaling (1280×800 for 80x25 mode)
    ///   - 3:  3x scaling (1920×1200 for 80x25 mode)
    ///   - 4:  4x scaling (2560×1600 for 80x25 mode)
    ///
    /// Higher scale values make the display larger. Auto-calculated if not specified
    /// to use the maximum scale that fits your screen.
    ///
    /// Note: Only takes effect when --framebuffer/-f is specified.
    #[cfg(feature = "framebuffer-backend")]
    #[arg(
        long,
        value_name = "SCALE",
        help = "Pixel scale factor (1, 2, 3, 4, or auto)"
    )]
    pub fb_scale: Option<String>,
}

impl Cli {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
