use super::cli::Cli;
use super::config_manager::AppConfig;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
use crate::framebuffer::fb_config::FramebufferConfig;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
use crate::framebuffer::text_modes::{TextMode, TextModeKind};
use super::platform::is_console_environment;
use crate::rendering::{Charset, RenderBackend, TerminalBackend, Theme, VideoBuffer};
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
use crate::rendering::FramebufferBackend;
use crate::term_emu::ShellConfig;
use crate::window::manager::WindowManager;
use crossterm::{cursor, event, execute, style, terminal};
use std::io;
#[cfg(unix)]
use std::io::Write;

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

        // Resolve mouse sensitivity: CLI arg takes precedence, then config file
        let sensitivity = cli_args.mouse_sensitivity.or(fb_config.mouse.sensitivity);

        // Try to initialize framebuffer backend
        match FramebufferBackend::new(
            mode,
            scale,
            font_name.as_deref(),
            mouse_device.as_deref(),
            invert_x,
            invert_y,
            sensitivity,
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
    // Skip EnableMouseCapture on console/TTY since raw device input handles mouse there
    let is_console = is_console_environment();

    if is_console {
        // On console/TTY, raw input provides mouse support - don't send ANSI mouse escape sequences
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

/// Validate shell configuration early (before terminal setup)
/// This allows the warning to be visible to the user
/// Returns the validated ShellConfig
pub fn validate_shell_config(cli_args: &Cli) -> ShellConfig {
    if let Some(ref shell_path) = cli_args.shell {
        let config = ShellConfig::custom_shell(shell_path.clone());
        // Validate shell path exists and is executable
        if let Err(msg) = config.validate() {
            eprintln!("Warning: {}, using system default shell", msg);
            // Give user time to see the warning
            std::thread::sleep(std::time::Duration::from_secs(2));
            ShellConfig::default()
        } else {
            config
        }
    } else {
        ShellConfig::default()
    }
}

/// Initializes or restores window manager
pub fn initialize_window_manager(
    cli_args: &Cli,
    app_config: &mut AppConfig,
    shell_config: ShellConfig,
) -> io::Result<WindowManager> {
    let window_manager = if !cli_args.no_restore {
        // Try to restore session, fall back to new if it fails
        let manager = WindowManager::restore_session_from_file(shell_config.clone())
            .unwrap_or_else(|_| WindowManager::with_shell_config(shell_config));

        // If auto-save is disabled, clear session after loading (one-time load)
        if !app_config.auto_save {
            let _ = WindowManager::clear_session_file();
        }

        manager
    } else {
        WindowManager::with_shell_config(shell_config)
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
/// On Linux, also handles GPM daemon interaction.
/// On BSD, uses sysmouse/wsmouse for raw console input.
#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub fn initialize_mouse_input(
    cli_args: &Cli,
    cols: u16,
    rows: u16,
    is_framebuffer_mode: bool,
) -> (
    crate::input::mouse::MouseInputManager,
    Option<crate::input::gpm_control::GpmConnection>,
) {
    use crate::input::mouse::{MouseInputManager, MouseInputMode};

    // Detect the mouse input mode
    let mode = MouseInputMode::detect(is_framebuffer_mode);

    // Try to disable GPM if it's running and we're using raw input (Linux only)
    let gpm_connection = if mode.uses_raw_input() {
        crate::input::gpm_control::try_disable_gpm()
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

/// Initializes the unified mouse input manager (macOS/Windows/other platforms)
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
)))]
pub fn initialize_mouse_input(
    _cli_args: &Cli,
    cols: u16,
    rows: u16,
    _is_framebuffer_mode: bool,
) -> (crate::input::mouse::MouseInputManager, Option<()>) {
    use crate::input::mouse::{MouseInputManager, MouseInputMode};

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

/// Cleanup function to restore terminal state (fallible version)
/// Following ratatui's pattern: disable raw mode FIRST as it has the most side effects
pub fn try_cleanup(stdout: &mut io::Stdout) -> io::Result<()> {
    // 1. Disable raw mode FIRST (has the most side effects)
    // This follows ratatui's best practice for reliable terminal restoration
    terminal::disable_raw_mode()?;

    // 2. Leave alternate screen and show cursor
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;

    // 3. Disable mouse capture (only if not on console/TTY)
    let is_console = is_console_environment();

    if !is_console {
        execute!(stdout, event::DisableMouseCapture)?;
    }

    // 4. Final color reset
    execute!(stdout, style::ResetColor)?;

    Ok(())
}

/// Cleanup function to restore terminal state (infallible wrapper)
/// Logs errors to stderr but never fails - safe to call from panic handlers
pub fn cleanup(stdout: &mut io::Stdout) {
    if let Err(err) = try_cleanup(stdout) {
        eprintln!("Failed to restore terminal: {err}");
    }
}
