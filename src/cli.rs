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

    /// Use single-line Unicode box drawing characters instead of double-line
    ///
    /// This mode uses thin single-line box characters (┌─┐│└┘) instead of
    /// double-line characters (╔═╗║╚╝). Use this when the font doesn't have
    /// double-line box drawing glyphs.
    #[arg(long, help = "Use single-line Unicode box drawing characters")]
    pub single_line: bool,

    /// Set the color theme
    ///
    /// Available themes:
    ///   - classic:        DOS-inspired blue and cyan colors (default)
    ///   - monochrome:     Grayscale theme with black and white
    ///   - dark:           Dark theme inspired by modern dark schemes
    ///   - dracu:          IDE-inspired dark theme (aliases: darcula, intellij)
    ///   - green_phosphor: Green monochrome CRT style (aliases: green)
    ///   - amber:          Amber/orange monochrome CRT style (alias: orange)
    ///   - ndd:            Cyan on blue disk utility style
    ///   - qbasic:         Yellow on blue IDE style (aliases: basic, edit)
    ///   - turbo:          TurboP - Yellow on grey/blue (alias: pascal)
    ///   - nc:             NCC - Cyan file manager style (alias: norton_commander)
    ///   - xt:             XT - Yellow on blue file manager style (alias: xtree)
    ///   - wp:             WP - White on blue word processor style (alias: wordperfect)
    ///   - db:             dB - Cyan on blue database style (alias: dbase)
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
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
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
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
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
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
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
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
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
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
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
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
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
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    #[arg(long, help = "Invert mouse X-axis for framebuffer mode")]
    pub invert_mouse_x: bool,

    /// Invert mouse Y-axis (Linux console only, requires --features framebuffer-backend)
    ///
    /// Some hardware configurations have inverted Y-axis behavior for raw mouse input.
    /// Use this flag if moving the mouse up/down produces inverted cursor movement
    /// in framebuffer mode.
    ///
    /// Note: Only takes effect when --framebuffer/-f is specified.
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
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
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    #[arg(long, help = "Launch framebuffer setup wizard")]
    pub fb_setup: bool,

    /// Swap left and right mouse buttons (Linux console only)
    ///
    /// By default, button mapping is corrected for most hardware. Use this flag
    /// to swap left/right buttons if your hardware reports them correctly but
    /// clicks appear reversed on screen.
    ///
    /// Note: Only affects mouse input on Linux console (TTY and framebuffer modes).
    #[cfg(target_os = "linux")]
    #[arg(
        long,
        help = "Swap left/right mouse buttons (use if clicks are reversed)"
    )]
    pub swap_mouse_buttons: bool,

    /// Mouse sensitivity for TTY mode (Linux console only)
    ///
    /// Adjust cursor movement speed for raw mouse input in TTY mode.
    /// Values:
    ///   - 0.1 to 0.3: Very slow (for precision)
    ///   - 0.3 to 0.5: Slow (default auto-calculated range)
    ///   - 0.5 to 1.0: Normal
    ///   - 1.0 to 2.0: Fast
    ///   - 2.0 to 5.0: Very fast
    ///
    /// If not specified, sensitivity is automatically calculated based on screen size.
    ///
    /// Note: Only affects TTY mode (Linux console without framebuffer).
    #[cfg(target_os = "linux")]
    #[arg(
        long,
        value_name = "SENSITIVITY",
        help = "Mouse sensitivity for TTY mode (0.1-5.0, default: auto)"
    )]
    pub mouse_sensitivity: Option<f32>,

    /// Disable exit functionality (for use as a window manager)
    ///
    /// When this flag is set, the application will not exit via:
    ///   - 'q' key (from desktop)
    ///   - ESC key (from desktop)
    ///   - F10 key
    ///   - Exit button in the top bar
    ///
    /// This is useful when using term39 as a persistent window manager that should
    /// never exit. The only way to exit when this flag is set is to kill the process
    /// externally (e.g., via SIGTERM or SIGKILL).
    #[arg(
        long,
        help = "Disable exit functionality (for use as a window manager)"
    )]
    pub no_exit: bool,
}

impl Cli {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
