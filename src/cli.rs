use clap::Parser;

const LONG_ABOUT: &str = "\
A modern, retro-styled terminal multiplexer with a classic MS-DOS aesthetic.
Features a full-screen text-based interface with authentic DOS-style rendering.

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
  CTRL+Space  - Command launcher (Slight)
  CTRL+L      - Clear terminal
  CTRL+S      - Save session manually
  ALT+TAB     - Switch between windows

MOUSE CONTROLS:
  Click title bar     - Drag window
  CTRL+Drag          - Drag without snap
  Click [X]          - Close window
  Drag border         - Resize window
  Click window       - Focus window
";

#[derive(Parser, Debug)]
#[command(
    name = "term39",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = "A modern, retro-styled terminal multiplexer with a classic MS-DOS aesthetic",
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

    /// Don't restore previous session on startup
    ///
    /// By default, term39 automatically restores your previous session (window layouts
    /// and terminal content) when you start it. Use this flag to start with a fresh
    /// session instead.
    #[arg(long, help = "Don't restore previous session on startup")]
    pub no_restore: bool,

    /// Don't save session (disables both auto-save on exit and manual save)
    ///
    /// By default, term39 saves your session when you exit and allows manual saving
    /// with Ctrl+S. Use this flag to disable all session saving functionality.
    #[arg(long, help = "Don't save session (disables auto-save and manual save)")]
    pub no_save: bool,

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
    /// Select text mode for framebuffer rendering:
    ///
    /// Classic DOS modes:
    ///   - 40x25:   40 columns × 25 rows (16×16 character cells)
    ///   - 80x25:   80 columns × 25 rows (8×16 character cells) - Standard DOS mode (default)
    ///   - 80x43:   80 columns × 43 rows (8×11 character cells)
    ///   - 80x50:   80 columns × 50 rows (8×8 character cells) - High density mode
    ///
    /// High-resolution modes:
    ///   - 160x50:  160 columns × 50 rows (8×16 character cells) - Double-wide standard
    ///   - 160x100: 160 columns × 100 rows (8×16 character cells) - High resolution
    ///   - 320x100: 320 columns × 100 rows (8×16 character cells) - Ultra-wide
    ///   - 320x200: 320 columns × 200 rows (8×8 character cells) - Maximum resolution
    ///
    /// Note: Only takes effect when --framebuffer/-f is specified.
    #[cfg(feature = "framebuffer-backend")]
    #[arg(
        long,
        value_name = "MODE",
        default_value = "80x25",
        help = "Framebuffer text mode"
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

    /// Framebuffer font name (Linux console only, requires --features framebuffer-backend)
    ///
    /// Specify a console font to use for framebuffer rendering.
    /// If not specified, automatically selects a font matching the text mode dimensions.
    ///
    /// Examples:
    ///   - Uni3-Terminus16      (Good all-around font, 8×16)
    ///   - Unifont-APL8x16      (Excellent Unicode coverage, 8×16)
    ///   - Uni3-TerminusBold16  (Bold variant, 8×16)
    ///   - Unifont              (16×16 full Unicode support)
    ///
    /// Use --fb-list-fonts to see all available fonts on your system.
    ///
    /// Note: Only takes effect when --framebuffer/-f is specified.
    #[cfg(feature = "framebuffer-backend")]
    #[arg(
        long,
        value_name = "FONT",
        help = "Console font name (e.g., Unifont-APL8x16)"
    )]
    pub fb_font: Option<String>,

    /// List available console fonts and exit (Linux console only, requires --features framebuffer-backend)
    ///
    /// Scans /usr/share/consolefonts/ and /usr/share/kbd/consolefonts/ for available
    /// PSF format fonts and displays them with their dimensions.
    ///
    /// Output format: FONT_NAME (WIDTHxHEIGHT)
    ///
    /// Note: Only available when compiled with framebuffer-backend feature.
    #[cfg(feature = "framebuffer-backend")]
    #[arg(long, help = "List available console fonts and exit")]
    pub fb_list_fonts: bool,

    /// Mouse input device path (Linux console only, requires --features framebuffer-backend)
    ///
    /// Specify the input device to use for mouse events in framebuffer mode:
    ///   - /dev/input/mice       (PS/2 protocol, default)
    ///   - /dev/input/event0     (Event interface)
    ///   - /dev/input/event1     (Event interface)
    ///   - /dev/input/event2     (Event interface)
    ///   - etc.
    ///
    /// If not specified, the system will automatically try /dev/input/mice first,
    /// then scan for event devices (event0-event15).
    ///
    /// Note: Only takes effect when --framebuffer/-f is specified.
    #[cfg(feature = "framebuffer-backend")]
    #[arg(
        long,
        value_name = "DEVICE",
        help = "Mouse input device (e.g., /dev/input/event2)"
    )]
    pub mouse_device: Option<String>,

    /// Invert mouse X-axis (Linux console only, requires --features framebuffer-backend)
    ///
    /// Some hardware configurations have inverted X-axis behavior for raw mouse input.
    /// Use this flag if moving the mouse left/right produces inverted cursor movement
    /// in framebuffer mode.
    ///
    /// Note: Only takes effect when --framebuffer/-f is specified.
    #[cfg(feature = "framebuffer-backend")]
    #[arg(long, help = "Invert mouse X-axis for framebuffer mode")]
    pub invert_mouse_x: bool,

    /// Invert mouse Y-axis (Linux console only, requires --features framebuffer-backend)
    ///
    /// Some hardware configurations have inverted Y-axis behavior for raw mouse input.
    /// Use this flag if moving the mouse up/down produces inverted cursor movement
    /// in framebuffer mode.
    ///
    /// Note: Only takes effect when --framebuffer/-f is specified.
    #[cfg(feature = "framebuffer-backend")]
    #[arg(long, help = "Invert mouse Y-axis for framebuffer mode")]
    pub invert_mouse_y: bool,

    /// Launch framebuffer setup wizard (Linux console only, requires --features framebuffer-backend)
    ///
    /// Opens an interactive configuration wizard to set up framebuffer mode settings.
    /// The wizard allows you to:
    ///   - Select text mode (40x25, 80x25, 80x50, etc.)
    ///   - Choose pixel scale factor
    ///   - Select console font
    ///   - Configure mouse settings
    ///
    /// Settings are saved to ~/.config/term39/fb.toml
    /// Use this flag to reconfigure settings even if fb.toml already exists.
    ///
    /// Note: Only available when compiled with framebuffer-backend feature.
    #[cfg(feature = "framebuffer-backend")]
    #[arg(long, help = "Launch framebuffer setup wizard")]
    pub fb_setup: bool,

    /// Swap left and right mouse buttons for GPM (Linux console only)
    ///
    /// Some GPM configurations have left/right mouse buttons swapped.
    /// Use this flag if left-click registers as right-click and vice versa
    /// when running in Linux console with GPM mouse support.
    ///
    /// Note: Only affects GPM mouse input on Linux console.
    #[cfg(target_os = "linux")]
    #[arg(long, help = "Swap left/right mouse buttons for GPM")]
    pub swap_mouse_buttons: bool,
}

impl Cli {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
