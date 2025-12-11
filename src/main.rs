#![allow(clippy::collapsible_if)]

mod app;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
mod framebuffer;
mod input;
mod lockscreen;
mod rendering;
mod term_emu;
mod ui;
mod utils;
mod window;

use app::{AppConfig, AppState};
use std::io;
use utils::{ClipboardManager, CommandHistory, CommandIndexer};
use window::WindowManager;

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let cli_args = app::cli::Cli::parse_args();

    // Handle --lock flag: send SIGUSR1 to running term39 instance and exit
    #[cfg(unix)]
    if cli_args.lock {
        return lockscreen::signal_sender::send_lock_signal();
    }

    // Set up panic hook to restore terminal state on panic
    app::panic_handler::setup_panic_hook();

    // Handle --fb-list-fonts flag (exit after listing)
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    if cli_args.fb_list_fonts {
        framebuffer::cli_handlers::list_fonts();
        return Ok(());
    }

    // Handle --fb-setup flag (run setup wizard)
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    if cli_args.fb_setup {
        return framebuffer::setup_wizard::run_setup_wizard();
    }

    // Load application configuration
    let mut app_config = AppConfig::load();

    // Load framebuffer configuration (for swap_buttons, etc.)
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    #[allow(unused_variables)] // Only used on Linux with framebuffer
    let fb_config = framebuffer::fb_config::FramebufferConfig::load();

    // Create charset and theme
    let mut charset = app::initialization::initialize_charset(&cli_args, &app_config);
    let mut theme = app::initialization::initialize_theme(&cli_args, &app_config);

    // Validate shell configuration early (before terminal setup) so warnings are visible
    let shell_config = app::initialization::validate_shell_config(&cli_args);

    // Initialize rendering backend (framebuffer or terminal)
    let mut backend = app::initialization::initialize_backend(&cli_args)?;

    let mut stdout = io::stdout();

    // Set up terminal modes and mouse capture
    app::initialization::setup_terminal(&mut stdout)?;

    // Initialize unified mouse input manager (will try to disable GPM cursor if needed)
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    let is_framebuffer_mode = cli_args.framebuffer;
    #[cfg(not(all(target_os = "linux", feature = "framebuffer-backend")))]
    let is_framebuffer_mode = false;

    let (cols_for_mouse, rows_for_mouse) = backend.dimensions();
    let (mut mouse_input_manager, _gpm_disable_connection) =
        app::initialization::initialize_mouse_input(
            &cli_args,
            cols_for_mouse,
            rows_for_mouse,
            is_framebuffer_mode,
        );

    // Initialize video buffer and window manager
    let mut video_buffer = app::initialization::initialize_video_buffer(backend.as_ref());
    let mut window_manager =
        app::initialization::initialize_window_manager(&cli_args, &mut app_config, shell_config)?;

    // Initialize application state
    let (cols, rows) = backend.dimensions();
    // If tint_terminal is set via CLI, update config (won't persist to file)
    if cli_args.tint_terminal {
        app_config.tint_terminal = true;
    }
    let mut app_state = AppState::new(cols, rows, &app_config, &charset);

    // Disable exit button if --no-exit flag is set
    if cli_args.no_exit {
        app_state.exit_button.enabled = false;
    }

    // Initialize autocomplete system (command indexer and history)
    let command_indexer = CommandIndexer::new();
    let mut command_history = CommandHistory::new();

    // Clipboard manager
    let mut clipboard_manager = ClipboardManager::new();

    // Show splash screen for 1 second
    ui::splash_screen::show_splash_screen(&mut video_buffer, &mut backend, &charset, &theme)?;

    // Set up signal handler for external lockscreen trigger (Unix only)
    lockscreen::signal_handler::setup();

    // Start with desktop focused - no windows yet
    // User can press 't' to create windows

    // Run the main event loop
    app::event_loop::run(
        &mut backend,
        &mut video_buffer,
        &mut stdout,
        &mut window_manager,
        &mut app_state,
        &mut app_config,
        &mut charset,
        &mut theme,
        &mut mouse_input_manager,
        &cli_args,
        &command_indexer,
        &mut command_history,
        &mut clipboard_manager,
        &_gpm_disable_connection,
    )?;

    // Save or clear session before exiting (unless --no-save flag is set)
    if !cli_args.no_save {
        if app_config.auto_save {
            let _ = window_manager.save_session_to_file();
        } else {
            // Clear session when auto-save is disabled
            let _ = WindowManager::clear_session_file();
        }
    }

    // Cleanup: restore terminal
    app::initialization::cleanup(&mut stdout);

    Ok(())
}
