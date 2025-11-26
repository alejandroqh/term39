use crate::charset::Charset;
use crate::cli::Cli;
use crate::config_manager::AppConfig;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
use crate::fb_config::FramebufferConfig;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
use crate::framebuffer::text_modes::{TextMode, TextModeKind};
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
use crate::render_backend::FramebufferBackend;
use crate::render_backend::{RenderBackend, TerminalBackend};
use crate::theme::Theme;
use crate::video_buffer::VideoBuffer;
use crate::window_manager::WindowManager;
use crossterm::{cursor, event, execute, style, terminal};
use std::io::{self, Write};

/// Initializes the rendering backend based on CLI arguments
pub fn initialize_backend(
    #[cfg_attr(
        not(all(target_os = "linux", feature = "framebuffer-backend")),
        allow(unused_variables)
    )]
    cli_args: &Cli,
) -> io::Result<Box<dyn RenderBackend>> {
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    if cli_args.framebuffer {
        // Load framebuffer configuration from fb.toml
        let fb_config = FramebufferConfig::load();

        // Resolve mode: CLI arg takes precedence, then config file, then default
        // Note: CLI arg has a default value of "80x25", so we check if it differs from default
        // to determine if user explicitly set it
        let mode_str = if cli_args.fb_mode != "80x25" {
            // User explicitly set a mode via CLI
            cli_args.fb_mode.clone()
        } else if FramebufferConfig::exists() {
            // Use config file value
            fb_config.display.mode.clone()
        } else {
            // Use CLI default (80x25)
            cli_args.fb_mode.clone()
        };

        let mode_kind = TextModeKind::from_str(&mode_str).unwrap_or_else(|| {
            eprintln!(
                "Warning: Invalid framebuffer mode '{}', using default 80x25",
                mode_str
            );
            TextModeKind::Mode80x25
        });

        let mode = TextMode::new(mode_kind);

        // Resolve scale: CLI arg takes precedence, then config file
        let scale_str = cli_args
            .fb_scale
            .clone()
            .unwrap_or_else(|| fb_config.display.scale.clone());

        let scale = if scale_str == "auto" {
            None // Auto-calculate scale
        } else {
            scale_str
                .parse::<usize>()
                .ok()
                .filter(|&n| (1..=8).contains(&n))
        };

        // Resolve font: CLI arg takes precedence, then config file
        let font_name = cli_args.fb_font.clone().or_else(|| {
            if FramebufferConfig::exists() {
                Some(fb_config.font.name.clone())
            } else {
                None
            }
        });

        // Resolve mouse device: CLI arg takes precedence, then config file
        let mouse_device = cli_args
            .mouse_device
            .clone()
            .or_else(|| fb_config.mouse.device.clone());

        // Resolve mouse axis inversion: CLI arg takes precedence, then config file
        let invert_x = cli_args.invert_mouse_x || fb_config.mouse.invert_x;
        let invert_y = cli_args.invert_mouse_y || fb_config.mouse.invert_y;

        // Try to initialize framebuffer backend
        match FramebufferBackend::new(
            mode,
            scale,
            font_name.as_deref(),
            mouse_device.as_deref(),
            invert_x,
            invert_y,
        ) {
            Ok(fb_backend) => {
                println!("Framebuffer backend initialized: {}", mode_kind);
                return Ok(Box::new(fb_backend));
            }
            Err(e) => {
                eprintln!("Failed to initialize framebuffer: {}", e);
                eprintln!("Falling back to terminal backend...");
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
    }

    // Use terminal backend (default or fallback)
    Ok(Box::new(TerminalBackend::new()?))
}

/// Sets up terminal modes and mouse capture
pub fn setup_terminal(stdout: &mut io::Stdout) -> io::Result<()> {
    // Enter raw mode for low-level terminal control
    terminal::enable_raw_mode()?;

    // Disable flow control (CTRL+S/CTRL+Q) by sending escape sequences
    #[cfg(unix)]
    {
        // Send escape sequence to disable software flow control
        let _ = stdout.write_all(b"\x1b[?1036l"); // Disable metaSendsEscape
        let _ = stdout.flush();

        // Use libc to disable IXON (software flow control) on Unix systems
        use std::os::unix::io::AsRawFd;
        unsafe {
            let mut termios: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(stdout.as_raw_fd(), &mut termios) == 0 {
                // Disable IXON (software flow control) to allow CTRL+S/CTRL+Q
                termios.c_iflag &= !libc::IXON;
                let _ = libc::tcsetattr(stdout.as_raw_fd(), libc::TCSANOW, &termios);
            }
        }
    }

    // Enter alternate screen buffer to prevent scrolling the parent terminal
    // Hide cursor and enable mouse capture
    // Skip EnableMouseCapture on Linux console (TERM=linux) since GPM handles mouse there
    #[cfg(target_os = "linux")]
    let is_linux_console = std::env::var("TERM").map(|t| t == "linux").unwrap_or(false);
    #[cfg(not(target_os = "linux"))]
    let is_linux_console = false;

    if is_linux_console {
        // On Linux console, GPM provides mouse support - don't send ANSI mouse escape sequences
        execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    } else {
        // On terminal emulators, enable mouse capture via ANSI escape sequences
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            event::EnableMouseCapture
        )?;
    }

    // Clear the screen
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    Ok(())
}

/// Loads and configures charset based on CLI and config
pub fn initialize_charset(cli_args: &Cli, app_config: &AppConfig) -> Charset {
    let mut charset = if cli_args.ascii {
        Charset::ascii()
    } else if cli_args.single_line {
        Charset::unicode_single_line()
    } else {
        Charset::unicode()
    };

    // Set the background character from config
    charset.set_background(app_config.get_background_char());

    charset
}

/// Loads theme from CLI or config
pub fn initialize_theme(cli_args: &Cli, app_config: &AppConfig) -> Theme {
    // Use theme from CLI args if provided, otherwise use config theme
    let theme_name = cli_args.theme.as_ref().unwrap_or(&app_config.theme);
    Theme::from_name(theme_name)
}

/// Initializes or restores window manager
pub fn initialize_window_manager(
    cli_args: &Cli,
    app_config: &mut AppConfig,
) -> io::Result<WindowManager> {
    let window_manager = if !cli_args.no_restore {
        // Try to restore session, fall back to new if it fails
        let manager =
            WindowManager::restore_session_from_file().unwrap_or_else(|_| WindowManager::new());

        // If auto-save is disabled, clear session after loading (one-time load)
        if !app_config.auto_save {
            let _ = WindowManager::clear_session_file();
        }

        manager
    } else {
        WindowManager::new()
    };

    Ok(window_manager)
}

/// Creates a new video buffer for the given backend dimensions
pub fn initialize_video_buffer(backend: &dyn RenderBackend) -> VideoBuffer {
    let (cols, rows) = backend.dimensions();
    VideoBuffer::new(cols, rows)
}

/// Initializes the unified mouse input manager
///
/// Returns a MouseInputManager and optionally a GpmConnection if GPM needed disabling
#[cfg(target_os = "linux")]
pub fn initialize_mouse_input(
    cli_args: &Cli,
    cols: u16,
    rows: u16,
    is_framebuffer_mode: bool,
) -> (
    crate::mouse_input::MouseInputManager,
    Option<crate::gpm_control::GpmConnection>,
) {
    use crate::mouse_input::{MouseInputManager, MouseInputMode};

    // Detect the mouse input mode
    let mode = MouseInputMode::detect(is_framebuffer_mode);

    // Try to disable GPM if it's running and we're using raw input
    let gpm_connection = if mode.uses_raw_input() {
        crate::gpm_control::try_disable_gpm()
    } else {
        None
    };

    // Get mouse configuration
    let device_path = cli_args.mouse_device.as_deref();
    let invert_x = cli_args.invert_mouse_x;
    let invert_y = cli_args.invert_mouse_y;
    let swap_buttons = cli_args.swap_mouse_buttons;
    let sensitivity = cli_args.mouse_sensitivity;

    // Create the mouse input manager
    let manager = MouseInputManager::new(
        mode,
        cols,
        rows,
        device_path,
        invert_x,
        invert_y,
        swap_buttons,
        sensitivity,
    )
    .unwrap_or_else(|e| {
        eprintln!("Warning: Failed to initialize mouse input: {}", e);
        // Return a fallback manager with no raw input
        MouseInputManager::new(
            MouseInputMode::TerminalEmulator,
            cols,
            rows,
            None,
            false,
            false,
            false,
            None,
        )
        .expect("Terminal emulator mode should always succeed")
    });

    (manager, gpm_connection)
}

/// Initializes the unified mouse input manager (non-Linux version)
#[cfg(not(target_os = "linux"))]
pub fn initialize_mouse_input(
    _cli_args: &Cli,
    cols: u16,
    rows: u16,
    _is_framebuffer_mode: bool,
) -> (crate::mouse_input::MouseInputManager, Option<()>) {
    use crate::mouse_input::{MouseInputManager, MouseInputMode};

    let manager = MouseInputManager::new(
        MouseInputMode::TerminalEmulator,
        cols,
        rows,
        None,
        false,
        false,
        false,
        None,
    )
    .expect("Terminal emulator mode should always succeed");

    (manager, None)
}

/// Cleanup function to restore terminal state
pub fn cleanup(stdout: &mut io::Stdout) -> io::Result<()> {
    // Disable mouse capture (only if not on Linux console)
    #[cfg(target_os = "linux")]
    let is_linux_console = std::env::var("TERM").map(|t| t == "linux").unwrap_or(false);
    #[cfg(not(target_os = "linux"))]
    let is_linux_console = false;

    if !is_linux_console {
        execute!(stdout, event::DisableMouseCapture)?;
    }

    // Reset colors FIRST to ensure default colors are used for subsequent operations
    execute!(stdout, style::ResetColor)?;

    // Set explicit default attributes (important for TTY/Linux console)
    // This ensures the terminal doesn't inherit any residual color state
    execute!(
        stdout,
        style::SetAttribute(style::Attribute::Reset),
        style::SetForegroundColor(style::Color::Reset),
        style::SetBackgroundColor(style::Color::Reset)
    )?;

    // Clear screen with default colors
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    // Move cursor to home position and reset colors again
    // This is important for TTY where color state can persist
    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        style::ResetColor
    )?;

    // Show cursor and leave alternate screen
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;

    // Final color reset after leaving alternate screen (for TTY)
    execute!(stdout, style::ResetColor)?;

    // Disable raw mode
    terminal::disable_raw_mode()?;

    Ok(())
}
