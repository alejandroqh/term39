use crate::charset::Charset;
use crate::cli::Cli;
use crate::config_manager::AppConfig;
#[cfg(feature = "framebuffer-backend")]
use crate::framebuffer::text_modes::{TextMode, TextModeKind};
#[cfg(feature = "framebuffer-backend")]
use crate::render_backend::FramebufferBackend;
use crate::render_backend::{RenderBackend, TerminalBackend};
use crate::theme::Theme;
use crate::video_buffer::VideoBuffer;
use crate::window_manager::WindowManager;
use crossterm::{cursor, event, execute, style, terminal};
use std::io::{self, Write};

/// Initializes the rendering backend based on CLI arguments
pub fn initialize_backend(
    #[cfg_attr(not(feature = "framebuffer-backend"), allow(unused_variables))] cli_args: &Cli,
) -> io::Result<Box<dyn RenderBackend>> {
    #[cfg(feature = "framebuffer-backend")]
    if cli_args.framebuffer {
        // Parse the framebuffer mode from CLI args
        let mode_kind = TextModeKind::from_str(&cli_args.fb_mode).unwrap_or_else(|| {
            eprintln!(
                "Warning: Invalid framebuffer mode '{}', using default 80x25",
                cli_args.fb_mode
            );
            TextModeKind::Mode80x25
        });

        let mode = TextMode::new(mode_kind);

        // Parse the scale factor from CLI args
        let scale = cli_args.fb_scale.as_ref().and_then(|s| {
            if s == "auto" {
                None // Auto-calculate scale
            } else {
                s.parse::<usize>().ok().filter(|&n| (1..=8).contains(&n))
            }
        });

        // Get font name from CLI args
        let font_name = cli_args.fb_font.as_deref();

        // Get mouse device from CLI args
        let mouse_device = cli_args.mouse_device.as_deref();

        // Get mouse axis inversion flags from CLI args
        let invert_x = cli_args.invert_mouse_x;
        let invert_y = cli_args.invert_mouse_y;

        // Try to initialize framebuffer backend
        match FramebufferBackend::new(mode, scale, font_name, mouse_device, invert_x, invert_y) {
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
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        event::EnableMouseCapture
    )?;

    // Clear the screen
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    Ok(())
}

/// Loads and configures charset based on CLI and config
pub fn initialize_charset(cli_args: &Cli, app_config: &AppConfig) -> Charset {
    let mut charset = if cli_args.ascii {
        Charset::ascii()
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

/// Initializes GPM (General Purpose Mouse) connection on Linux
#[cfg(target_os = "linux")]
pub fn initialize_gpm() -> Option<crate::gpm_handler::GpmConnection> {
    crate::gpm_handler::GpmConnection::open()
}

/// Cleanup function to restore terminal state
pub fn cleanup(stdout: &mut io::Stdout) -> io::Result<()> {
    // Disable mouse capture
    execute!(stdout, event::DisableMouseCapture)?;

    // Clear screen
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

    // Reset colors to default
    execute!(stdout, style::ResetColor)?;

    // Show cursor and leave alternate screen
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;

    // Disable raw mode
    terminal::disable_raw_mode()?;

    Ok(())
}
