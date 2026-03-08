#![allow(clippy::collapsible_if)]

mod app;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
mod framebuffer;
mod input;
mod lockscreen;
mod persist;
mod rendering;
mod term_emu;
mod ui;
mod utils;
mod window;

use app::{AppConfig, AppState};
use std::io;
use utils::{ClipboardManager, CommandHistory, CommandIndexer};
use window::WindowManager;

/// Persist mode state passed through initialization
#[cfg(unix)]
#[allow(dead_code)]
struct PersistState {
    client: Option<persist::client::PersistClient>,
    windows: Vec<persist::protocol::WindowInfo>,
    is_temporary: bool,
    startup_warning: Option<String>,
}

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

    // Create charset, theme, and keybinding profile
    let mut charset = app::initialization::initialize_charset(&cli_args, &app_config);
    let mut theme = app::initialization::initialize_theme(&cli_args, &app_config);
    let mut keybinding_profile =
        app::initialization::initialize_keybinding_profile(&cli_args, &app_config);

    // Validate shell configuration early (before terminal setup) so warnings are visible
    let shell_config = app::initialization::validate_shell_config(&cli_args);

    // ===== PERSIST MODE =====
    // Fork daemon before any thread creation (setup_terminal, mouse input, PTY readers).
    // This must happen before any threads are spawned or terminal state is modified.
    #[cfg(unix)]
    let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 25));
    #[cfg(unix)]
    let persist_state = {
        let no_persist = cli_args.no_persist;
        let force_attach = cli_args.force_attach;
        if no_persist {
            // Standalone mode (no daemon)
            PersistState {
                client: None,
                windows: Vec::new(),
                is_temporary: false,
                startup_warning: None,
            }
        } else {
            // Persist mode: try to connect or fork daemon
            match persist::client::PersistClient::connect(term_cols, term_rows, force_attach) {
                Ok(persist::client::ConnectResult::Connected(client, windows)) => {
                    // Attached to existing daemon
                    PersistState {
                        client: Some(client),
                        windows,
                        is_temporary: false,
                        startup_warning: None,
                    }
                }
                Ok(persist::client::ConnectResult::Denied(_reason)) => {
                    // Another client is attached - run as temporary
                    PersistState {
                        client: None,
                        windows: Vec::new(),
                        is_temporary: true,
                        startup_warning: Some(
                            "Session already attached. Running in temporary mode.".to_string(),
                        ),
                    }
                }
                Ok(persist::client::ConnectResult::NoDaemon) => {
                    // No daemon running - fork one
                    match persist::forker::fork_daemon() {
                        Ok(persist::forker::ForkResult::Child) => {
                            // We are the daemon - run daemon loop (never returns)
                            persist::daemon::run_daemon(&shell_config);
                        }
                        Ok(persist::forker::ForkResult::Parent(_child_pid)) => {
                            // Retry with exponential backoff (~630ms total max)
                            let backoff_ms = [10, 20, 40, 80, 160, 320];
                            let mut connected_state = None;
                            for delay in &backoff_ms {
                                std::thread::sleep(std::time::Duration::from_millis(*delay));
                                if !persist::socket::socket_exists() {
                                    continue;
                                }
                                match persist::client::PersistClient::connect(
                                    term_cols, term_rows, false,
                                ) {
                                    Ok(persist::client::ConnectResult::Connected(
                                        client,
                                        windows,
                                    )) => {
                                        connected_state = Some(PersistState {
                                            client: Some(client),
                                            windows,
                                            is_temporary: false,
                                            startup_warning: None,
                                        });
                                        break;
                                    }
                                    _ => continue,
                                }
                            }
                            connected_state.unwrap_or_else(|| PersistState {
                                client: None,
                                windows: Vec::new(),
                                is_temporary: false,
                                startup_warning: Some(
                                    "Failed to connect to daemon, running standalone".to_string(),
                                ),
                            })
                        }
                        Err(e) => PersistState {
                            client: None,
                            windows: Vec::new(),
                            is_temporary: false,
                            startup_warning: Some(format!(
                                "Fork failed ({}), running standalone",
                                e
                            )),
                        },
                    }
                }
                Err(e) => PersistState {
                    client: None,
                    windows: Vec::new(),
                    is_temporary: false,
                    startup_warning: Some(format!(
                        "Persist check failed ({}), running standalone",
                        e
                    )),
                },
            }
        }
    };
    // On non-Unix, persist mode is not available
    #[cfg(not(unix))]
    let _persist_state_unused = false;

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

    // Set persist client on window manager and restore any existing windows (Unix only)
    #[cfg(unix)]
    let persist_startup_warning = {
        let PersistState {
            client: persist_client,
            windows: persist_windows,
            startup_warning,
            ..
        } = persist_state;
        if let Some(client) = persist_client {
            window_manager.set_persist_client(client);
            if !persist_windows.is_empty() {
                window_manager.restore_persist_windows(persist_windows);
            }
        }
        startup_warning
    };

    // Initialize application state
    let (cols, rows) = backend.dimensions();
    // If tint_terminal is set via CLI, update config (won't persist to file)
    if cli_args.tint_terminal {
        app_config.tint_terminal = true;
    }
    let mut app_state = AppState::new(cols, rows, &app_config, &charset);

    // Show persist startup warning as toast (if any)
    #[cfg(unix)]
    if let Some(warning) = persist_startup_warning {
        app_state.active_toast = Some(ui::toast::Toast::new(warning));
    }

    // Disable exit button if --no-exit flag is set
    if cli_args.no_exit {
        app_state.exit_button.enabled = false;
    }

    // Initialize autocomplete system (command indexer and history)
    let command_indexer = CommandIndexer::new();
    let mut command_history = CommandHistory::new();

    // Clipboard manager
    let mut clipboard_manager = ClipboardManager::new();

    // Show splash screen for 1 second (skip when reattaching to existing session)
    #[cfg(unix)]
    let has_restored_windows = window_manager.window_count() > 0;
    #[cfg(not(unix))]
    let has_restored_windows = false;

    if !has_restored_windows {
        ui::splash_screen::show_splash_screen(&mut video_buffer, &mut backend, &charset, &theme)?;
    }

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
        &mut keybinding_profile,
        &mut mouse_input_manager,
        &cli_args,
        &command_indexer,
        &mut command_history,
        &mut clipboard_manager,
        &_gpm_disable_connection,
    )?;

    // Detach from daemon on exit (if in persist mode)
    #[cfg(unix)]
    window_manager.detach_persist_client();

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
