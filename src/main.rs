#![allow(clippy::collapsible_if)]

mod ansi_handler;
mod app_state;
mod button;
mod charset;
mod cli;
mod clipboard_manager;
mod color_utils;
mod command_history;
mod command_indexer;
mod config;
mod config_manager;
mod config_window;
mod context_menu;
mod dialog_handlers;
mod error_dialog;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
mod fb_config;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
mod fb_setup_window;
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
mod framebuffer;
mod fuzzy_matcher;
#[cfg(target_os = "linux")]
mod gpm_control;
mod info_window;
mod initialization;
mod keyboard_handlers;
mod keyboard_mode;
mod mouse_input;
mod prompt;
mod render_backend;
mod render_frame;
mod selection;
mod session;
mod slight_input;
mod splash_screen;
mod term_grid;
mod terminal_emulator;
mod terminal_window;
mod theme;
mod ui_render;
mod video_buffer;
mod window;
mod window_manager;
mod window_mode_handlers;

use app_state::AppState;
use button::Button;
use chrono::Local;
use clipboard_manager::ClipboardManager;
use command_history::CommandHistory;
use command_indexer::CommandIndexer;
use config_manager::AppConfig;
use config_window::ConfigAction;
use context_menu::MenuAction;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind},
    terminal::{self, ClearType},
};
use error_dialog::ErrorDialog;
use prompt::{Prompt, PromptAction, PromptButton, PromptType};
use selection::SelectionType;
use slight_input::SlightInput;
use std::io;
use std::panic;
use std::time::{Duration, Instant};
use theme::Theme;
use ui_render::CalendarState;
use window_manager::{FocusState, WindowManager};

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let cli_args = cli::Cli::parse_args();

    // Set up panic hook to restore terminal state on panic
    // This prevents the terminal from being left in raw mode if the application crashes
    let default_panic = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal state
        let mut stdout = io::stdout();

        // Best-effort cleanup - ignore errors since we're already panicking
        let _ = crossterm::execute!(stdout, crossterm::event::DisableMouseCapture);
        // Reset colors FIRST before clearing
        let _ = crossterm::execute!(stdout, crossterm::style::ResetColor);
        let _ = crossterm::execute!(
            stdout,
            crossterm::style::SetAttribute(crossterm::style::Attribute::Reset),
            crossterm::style::SetForegroundColor(crossterm::style::Color::Reset),
            crossterm::style::SetBackgroundColor(crossterm::style::Color::Reset)
        );
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        );
        let _ = crossterm::execute!(
            stdout,
            crossterm::cursor::MoveTo(0, 0),
            crossterm::style::ResetColor
        );
        let _ = crossterm::execute!(
            stdout,
            crossterm::cursor::Show,
            crossterm::terminal::LeaveAlternateScreen
        );
        // Final color reset after leaving alternate screen
        let _ = crossterm::execute!(stdout, crossterm::style::ResetColor);
        let _ = crossterm::terminal::disable_raw_mode();

        // Call the default panic handler to print the panic message
        default_panic(panic_info);
    }));

    // Handle --fb-list-fonts flag (exit after listing)
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    if cli_args.fb_list_fonts {
        use framebuffer::font_manager::FontManager;

        println!("Available console fonts:\n");
        let fonts = FontManager::list_available_fonts();

        if fonts.is_empty() {
            println!("No console fonts found in:");
            println!("  - /usr/share/consolefonts/");
            println!("  - /usr/share/kbd/consolefonts/");
            println!("\nInstall fonts with: sudo apt install kbd unifont");
        } else {
            // Group by dimensions
            let mut current_dim = (0, 0);
            for (name, width, height) in fonts {
                if (width, height) != current_dim {
                    if current_dim != (0, 0) {
                        println!();
                    }
                    println!("{}×{} fonts:", width, height);
                    current_dim = (width, height);
                }
                println!("  {}", name);
            }
            println!("\nUse with: term39 -f --fb-font=FONT_NAME");
        }
        return Ok(());
    }

    // Handle --fb-setup flag (run setup wizard)
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    if cli_args.fb_setup {
        use charset::Charset;
        use crossterm::execute;
        use fb_setup_window::{FbSetupAction, FbSetupWindow};
        use framebuffer::font_manager::FontManager;
        use render_backend::RenderBackend;
        use video_buffer::VideoBuffer;

        // Set up terminal for setup wizard
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;

        // Get terminal size and create video buffer
        let (cols, rows) = terminal::size()?;
        let mut video_buffer = VideoBuffer::new(cols, rows);

        // Create setup window
        let mut setup_window = FbSetupWindow::new(cols, rows);

        // Load available fonts
        let fonts = FontManager::list_available_fonts();
        setup_window.set_fonts(fonts);

        // Create charset and theme for rendering
        let charset = Charset::unicode();
        let theme = Theme::from_name("classic");

        // Create terminal backend for rendering
        let mut term_backend = render_backend::TerminalBackend::new()?;

        // Setup wizard event loop
        let mut should_launch = false;
        loop {
            // Render setup window
            setup_window.render(&mut video_buffer, &charset, &theme);

            // Present to terminal
            term_backend.present(&mut video_buffer)?;

            // Poll for crossterm events
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key_event) => {
                        // Ignore key release events (Windows sends both press and release)
                        if key_event.kind != KeyEventKind::Press {
                            continue;
                        }
                        let action = setup_window.handle_key(key_event);
                        match action {
                            FbSetupAction::Close => break,
                            FbSetupAction::SaveAndLaunch => {
                                if let Err(e) = setup_window.save_config() {
                                    eprintln!("Error saving config: {}", e);
                                }
                                should_launch = true;
                                break;
                            }
                            FbSetupAction::SaveOnly => {
                                if let Err(e) = setup_window.save_config() {
                                    eprintln!("Error saving config: {}", e);
                                }
                                break;
                            }
                            _ => {}
                        }
                    }
                    Event::Mouse(mouse_event) => {
                        // Only handle actual left-button clicks, not moves
                        if let crossterm::event::MouseEventKind::Down(
                            crossterm::event::MouseButton::Left,
                        ) = mouse_event.kind
                        {
                            let action =
                                setup_window.handle_click(mouse_event.column, mouse_event.row);
                            match action {
                                FbSetupAction::Close => break,
                                FbSetupAction::SaveAndLaunch => {
                                    if let Err(e) = setup_window.save_config() {
                                        eprintln!("Error saving config: {}", e);
                                    }
                                    should_launch = true;
                                    break;
                                }
                                FbSetupAction::SaveOnly => {
                                    if let Err(e) = setup_window.save_config() {
                                        eprintln!("Error saving config: {}", e);
                                    }
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Event::Resize(new_cols, new_rows) => {
                        video_buffer = VideoBuffer::new(new_cols, new_rows);
                        setup_window = FbSetupWindow::new(new_cols, new_rows);
                        let fonts = FontManager::list_available_fonts();
                        setup_window.set_fonts(fonts);
                    }
                    _ => {}
                }
            }
        }

        // Cleanup terminal - reset colors properly to avoid color bleeding on TTY
        execute!(stdout, crossterm::event::DisableMouseCapture)?;
        execute!(stdout, crossterm::style::ResetColor)?;
        execute!(
            stdout,
            crossterm::style::SetAttribute(crossterm::style::Attribute::Reset),
            crossterm::style::SetForegroundColor(crossterm::style::Color::Reset),
            crossterm::style::SetBackgroundColor(crossterm::style::Color::Reset)
        )?;
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
        execute!(
            stdout,
            crossterm::cursor::MoveTo(0, 0),
            crossterm::style::ResetColor
        )?;
        execute!(
            stdout,
            crossterm::cursor::Show,
            terminal::LeaveAlternateScreen
        )?;
        execute!(stdout, crossterm::style::ResetColor)?;
        terminal::disable_raw_mode()?;

        // If user chose to launch, actually launch the application
        if should_launch {
            let config = setup_window.get_config();

            // Check device permissions before launching
            let mut permission_errors: Vec<String> = Vec::new();
            let mut fix_hints: Vec<String> = Vec::new();

            // Check framebuffer device access
            let fb_device = "/dev/fb0";
            if std::fs::metadata(fb_device).is_err() {
                permission_errors.push(format!("Framebuffer device '{}' not found", fb_device));
                fix_hints.push(
                    "Ensure you're on a Linux console (TTY), not a terminal emulator".to_string(),
                );
            } else if std::fs::File::open(fb_device).is_err() {
                permission_errors.push(format!("No permission to access '{}'", fb_device));
                fix_hints.push("Add user to video group: sudo usermod -aG video $USER".to_string());
            }

            // Check mouse device access
            let mouse_device = config.get_mouse_device();
            if !mouse_device.is_empty() {
                if std::fs::metadata(&mouse_device).is_err() {
                    permission_errors.push(format!("Mouse device '{}' not found", mouse_device));
                    fix_hints.push("Check if the mouse device path is correct".to_string());
                } else if std::fs::File::open(&mouse_device).is_err() {
                    permission_errors.push(format!("No permission to access '{}'", mouse_device));
                    fix_hints
                        .push("Add user to input group: sudo usermod -aG input $USER".to_string());
                }
            }

            // If there are permission errors, show them and exit
            if !permission_errors.is_empty() {
                println!("Configuration saved to ~/.config/term39/fb.toml\n");
                println!("Cannot launch framebuffer mode due to permission issues:\n");
                for error in &permission_errors {
                    println!("  - {}", error);
                }
                println!("\nTo fix:");
                for hint in &fix_hints {
                    println!("  {}", hint);
                }
                println!("\nAfter adding groups, log out and back in for changes to take effect.");
                println!("\nAlternatively, run with sudo:");
                println!("  sudo term39 -f --fb-mode={}", config.display.mode);
                return Ok(());
            }

            println!("Configuration saved! Launching framebuffer mode...\n");

            // Get the current executable path
            let exe_path =
                std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("./term39"));

            // Build command arguments
            let mut args = vec![
                "-f".to_string(),
                format!("--fb-mode={}", config.display.mode),
                format!("--fb-font={}", config.font.name),
            ];

            if config.display.scale != "auto" {
                args.push(format!("--fb-scale={}", config.display.scale));
            }

            if config.mouse.invert_x {
                args.push("--invert-mouse-x".to_string());
            }

            if config.mouse.invert_y {
                args.push("--invert-mouse-y".to_string());
            }

            if config.mouse.swap_buttons {
                args.push("--swap-mouse-buttons".to_string());
            }

            // Launch directly (user has permissions)
            use std::os::unix::process::CommandExt;
            let mut cmd = std::process::Command::new(&exe_path);
            cmd.args(&args);

            // Use exec to replace current process
            let err = cmd.exec();
            // If we get here, exec failed
            eprintln!("Failed to launch: {}", err);
            return Err(err);
        } else {
            println!("Configuration saved to ~/.config/term39/fb.toml");
        }

        return Ok(());
    }

    // Load application configuration
    let mut app_config = AppConfig::load();

    // Load framebuffer configuration (for swap_buttons, etc.)
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    #[allow(unused_variables)] // Only used on Linux with framebuffer
    let fb_config = fb_config::FramebufferConfig::load();

    // Create charset and theme
    let mut charset = initialization::initialize_charset(&cli_args, &app_config);
    let mut theme = initialization::initialize_theme(&cli_args, &app_config);

    // Initialize rendering backend (framebuffer or terminal)
    let mut backend = initialization::initialize_backend(&cli_args)?;

    let mut stdout = io::stdout();

    // Set up terminal modes and mouse capture
    initialization::setup_terminal(&mut stdout)?;

    // Initialize unified mouse input manager (will try to disable GPM cursor if needed)
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    let is_framebuffer_mode = cli_args.framebuffer;
    #[cfg(not(all(target_os = "linux", feature = "framebuffer-backend")))]
    let is_framebuffer_mode = false;

    let (cols_for_mouse, rows_for_mouse) = backend.dimensions();
    let (mut mouse_input_manager, _gpm_disable_connection) = initialization::initialize_mouse_input(
        &cli_args,
        cols_for_mouse,
        rows_for_mouse,
        is_framebuffer_mode,
    );

    // Initialize video buffer and window manager
    let mut video_buffer = initialization::initialize_video_buffer(backend.as_ref());
    let mut window_manager = initialization::initialize_window_manager(&cli_args, &mut app_config)?;

    // Initialize application state
    let (cols, rows) = backend.dimensions();
    let mut app_state = AppState::new(
        cols,
        rows,
        app_config.auto_tiling_on_startup,
        app_config.tint_terminal || cli_args.tint_terminal,
    );

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
    splash_screen::show_splash_screen(&mut video_buffer, &mut backend, &charset, &theme)?;

    // Start with desktop focused - no windows yet
    // User can press 't' to create windows

    // Main loop
    loop {
        // Check if backend was resized and recreate buffer if needed
        if let Some((new_cols, new_rows)) = backend.check_resize()? {
            // Clear the terminal screen to remove artifacts
            use crossterm::execute;
            execute!(stdout, terminal::Clear(ClearType::All))?;
            video_buffer = initialization::initialize_video_buffer(backend.as_ref());
            app_state.update_auto_tiling_button_position(new_rows);

            // Update mouse input manager bounds for the new size
            mouse_input_manager.set_bounds(new_cols, new_rows);

            // Reposition windows to fit the new screen dimensions
            if app_state.auto_tiling_enabled {
                window_manager.auto_position_windows(new_cols, new_rows);
            } else {
                // Clamp windows to new screen bounds
                window_manager.clamp_windows_to_bounds(new_cols, new_rows);
            }
        }

        // Get current dimensions from backend
        let (cols, _rows) = backend.dimensions();

        // Update clipboard buttons state and position
        let has_clipboard_content = clipboard_manager.has_content();
        let has_selection = window_manager.focused_window_has_meaningful_selection();
        app_state.update_button_states(cols, has_clipboard_content, has_selection);

        // Render the complete frame
        let windows_closed = render_frame::render_frame(
            &mut video_buffer,
            &mut backend,
            &mut stdout,
            &mut window_manager,
            &app_state,
            &charset,
            &theme,
            &app_config,
        )?;

        // Auto-reposition remaining windows if any were closed
        if windows_closed && app_state.auto_tiling_enabled {
            let (cols, rows) = backend.dimensions();
            window_manager.auto_position_windows(cols, rows);
        }

        // Poll unified mouse input manager for raw input events (TTY mode only)
        // Skip this for framebuffer mode - it has its own native mouse input
        let raw_mouse_event =
            if mouse_input_manager.uses_raw_input() && !backend.has_native_mouse_input() {
                if let Ok(Some(event)) = mouse_input_manager.poll_event() {
                    // Update TTY cursor position for display
                    let (cursor_col, cursor_row) = mouse_input_manager.cursor_position();
                    backend.set_tty_cursor(cursor_col, cursor_row);
                    Some(Event::Mouse(event))
                } else {
                    None
                }
            } else if !backend.has_native_mouse_input() {
                // In terminal emulator mode, clear any TTY cursor
                backend.clear_tty_cursor();
                None
            } else {
                // Framebuffer mode handles mouse input natively
                None
            };

        // Process raw mouse event if available (from MouseInputManager)
        #[cfg(target_os = "linux")]
        let mut injected_event: Option<Event> = raw_mouse_event;
        #[cfg(not(target_os = "linux"))]
        let _injected_event: Option<Event> = raw_mouse_event;

        // Process framebuffer mouse event if available
        #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
        if injected_event.is_none() {
            if let Some((event_type, button_id, col, row)) = backend.get_mouse_button_event() {
                use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

                // Map button ID to MouseButton, applying swap if configured
                let button = match button_id {
                    0 => {
                        if fb_config.mouse.swap_buttons {
                            MouseButton::Right
                        } else {
                            MouseButton::Left
                        }
                    }
                    1 => {
                        if fb_config.mouse.swap_buttons {
                            MouseButton::Left
                        } else {
                            MouseButton::Right
                        }
                    }
                    2 => MouseButton::Middle,
                    _ => MouseButton::Left, // Fallback
                };

                // Map event type to MouseEventKind
                // event_type: 0=Down, 1=Up, 2=Drag
                let kind = match event_type {
                    0 => MouseEventKind::Down(button),
                    1 => MouseEventKind::Up(button),
                    2 => MouseEventKind::Drag(button),
                    _ => MouseEventKind::Down(button), // Fallback
                };

                let mouse_event = MouseEvent {
                    kind,
                    column: col,
                    row,
                    modifiers: KeyModifiers::empty(),
                };

                injected_event = Some(Event::Mouse(mouse_event));
            }
        }

        // Process framebuffer scroll event if available (and no button event pending)
        #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
        if injected_event.is_none() {
            if let Some((scroll_direction, col, row)) = backend.get_mouse_scroll_event() {
                use crossterm::event::{MouseEvent, MouseEventKind};

                let kind = match scroll_direction {
                    0 => MouseEventKind::ScrollUp,
                    _ => MouseEventKind::ScrollDown,
                };

                let mouse_event = MouseEvent {
                    kind,
                    column: col,
                    row,
                    modifiers: KeyModifiers::empty(),
                };

                injected_event = Some(Event::Mouse(mouse_event));
            }
        }

        // Process injected event (raw/FB) or poll for crossterm event
        // Don't wait if we have an injected event
        #[cfg(target_os = "linux")]
        let has_event = injected_event.is_some() || event::poll(Duration::from_millis(16))?;
        #[cfg(not(target_os = "linux"))]
        let has_event = event::poll(Duration::from_millis(16))?;

        if has_event {
            // Track whether this event is injected (raw/FB) to avoid double-scaling
            #[cfg(target_os = "linux")]
            let is_injected = injected_event.is_some();
            #[cfg(not(target_os = "linux"))]
            let is_injected = false;

            #[cfg(target_os = "linux")]
            let current_event = if let Some(evt) = injected_event {
                evt
            } else {
                event::read()?
            };
            #[cfg(not(target_os = "linux"))]
            let current_event = event::read()?;

            match current_event {
                Event::Key(key_event) => {
                    // Ignore key release events (Windows sends both press and release)
                    if key_event.kind != KeyEventKind::Press {
                        continue;
                    }

                    let current_focus = window_manager.get_focus();

                    // Handle prompt keyboard navigation
                    if let Some(should_exit) =
                        dialog_handlers::handle_prompt_keyboard(&mut app_state, key_event)
                    {
                        if should_exit {
                            break;
                        }
                        continue;
                    }

                    // Handle close confirmation keyboard events (window-specific modal)
                    if let FocusState::Window(window_id) = current_focus {
                        if let Some(should_close) =
                            window_manager.handle_close_confirmation_key(window_id, key_event)
                        {
                            if should_close {
                                // User confirmed close
                                window_manager.close_window(window_id);
                                if app_state.auto_tiling_enabled {
                                    let (cols, rows) = backend.dimensions();
                                    window_manager.auto_position_windows(cols, rows);
                                }
                            }
                            continue; // Handled
                        }
                    }

                    // Handle error dialog keyboard events
                    if dialog_handlers::handle_error_dialog_keyboard(&mut app_state, key_event) {
                        continue;
                    }

                    // Handle Slight input keyboard events
                    if dialog_handlers::handle_slight_input_keyboard(
                        &mut app_state,
                        key_event,
                        &command_indexer,
                        &mut command_history,
                        &mut window_manager,
                        backend.as_ref(),
                    ) {
                        continue;
                    }

                    // Handle calendar keyboard navigation
                    if dialog_handlers::handle_calendar_keyboard(&mut app_state, key_event) {
                        continue;
                    }

                    // Handle help window keyboard events
                    if dialog_handlers::handle_help_window_keyboard(&mut app_state, key_event) {
                        continue;
                    }

                    // Handle about window keyboard events
                    if dialog_handlers::handle_about_window_keyboard(&mut app_state, key_event) {
                        continue;
                    }

                    // Handle config window keyboard events
                    if dialog_handlers::handle_config_window_keyboard(&mut app_state, key_event) {
                        continue;
                    }

                    // Handle Window Mode help window keyboard events
                    if dialog_handlers::handle_winmode_help_window_keyboard(
                        &mut app_state,
                        key_event,
                    ) {
                        continue;
                    }

                    // Handle Window Mode keyboard events (vim-like window control)
                    if window_mode_handlers::handle_window_mode_keyboard(
                        &mut app_state,
                        &mut app_config,
                        key_event,
                        &mut window_manager,
                        backend.as_ref(),
                    ) {
                        continue;
                    }

                    // Handle CTRL+Space / Option+Space to open Slight input popup (needs inline access to command_indexer/history)
                    // Note: Ctrl+Space produces NUL character ('\0') in most terminals
                    // On macOS, Option+Space produces non-breaking space (U+00A0)
                    let is_launcher_shortcut = (key_event.code == KeyCode::Char(' ')
                        && (key_event.modifiers.contains(KeyModifiers::CONTROL)
                            || key_event.modifiers.contains(KeyModifiers::ALT)))
                        || key_event.code == KeyCode::Char('\0')
                        || key_event.code == KeyCode::Char('\u{00a0}'); // Non-breaking space from Option+Space on macOS
                    if is_launcher_shortcut {
                        let (cols, rows) = backend.dimensions();
                        let mut slight_input = SlightInput::new(cols, rows);
                        slight_input
                            .set_autocomplete(command_indexer.clone(), command_history.clone());
                        app_state.active_slight_input = Some(slight_input);
                        continue;
                    }

                    // Handle desktop keyboard shortcuts (F1-F7, ESC, 'q', 'h', 'l', 'c', 's', 't', 'T', copy/paste)
                    if keyboard_handlers::handle_desktop_keyboard(
                        &mut app_state,
                        key_event,
                        current_focus,
                        &mut window_manager,
                        &mut clipboard_manager,
                        backend.as_ref(),
                        &app_config,
                        &cli_args,
                    ) {
                        // Check if exit was requested
                        if app_state.should_exit {
                            break;
                        }
                        continue;
                    }

                    // Forward input to terminal window if a window is focused
                    if matches!(current_focus, FocusState::Window(_)) {
                        keyboard_handlers::forward_to_terminal(key_event, &mut window_manager);
                    }
                }
                Event::Mouse(mut mouse_event) => {
                    // Scale mouse coordinates from TTY space to backend space
                    // Only scale crossterm mouse events, not injected GPM/FB events
                    // (injected events are already scaled at injection time)
                    if !is_injected {
                        let (scaled_col, scaled_row) =
                            backend.scale_mouse_coords(mouse_event.column, mouse_event.row);
                        mouse_event.column = scaled_col;
                        mouse_event.row = scaled_row;
                    }

                    let (_, rows) = backend.dimensions();
                    let bar_y = rows - 1;

                    let mut handled = false;

                    // Check if there's an active prompt - it takes priority
                    #[allow(clippy::collapsible_if)]
                    if let Some(ref prompt) = app_state.active_prompt {
                        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                            if let Some(action) =
                                prompt.handle_click(mouse_event.column, mouse_event.row)
                            {
                                match action {
                                    PromptAction::Confirm => {
                                        // Exit confirmed
                                        break;
                                    }
                                    PromptAction::Cancel => {
                                        // Dismiss prompt
                                        app_state.active_prompt = None;
                                    }
                                    _ => {}
                                }
                                handled = true;
                            } else if prompt.contains_point(mouse_event.column, mouse_event.row) {
                                // Click inside prompt but not on a button - consume the event
                                handled = true;
                            }
                        }
                    }

                    // Check if there's an active error dialog (after prompt, before other events)
                    #[allow(clippy::collapsible_if)]
                    if !handled {
                        if let Some(ref error_dialog) = app_state.active_error_dialog {
                            if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                                // Check if OK button was clicked
                                if error_dialog
                                    .is_ok_button_clicked(mouse_event.column, mouse_event.row)
                                {
                                    app_state.active_error_dialog = None;
                                    handled = true;
                                }
                            }
                        }
                    }

                    // Check if there's an active config window (after prompt, before other events)
                    #[allow(clippy::collapsible_if)]
                    if !handled {
                        if let Some(ref config_win) = app_state.active_config_window {
                            if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                                let action =
                                    config_win.handle_click(mouse_event.column, mouse_event.row);
                                match action {
                                    ConfigAction::Close => {
                                        app_state.active_config_window = None;
                                        handled = true;
                                    }
                                    ConfigAction::ToggleAutoTiling => {
                                        app_config.toggle_auto_tiling_on_startup();
                                        // Update runtime state to match config
                                        app_state.auto_tiling_enabled =
                                            app_config.auto_tiling_on_startup;
                                        // Update button text
                                        let (_, rows) = backend.dimensions();
                                        let auto_tiling_text = if app_state.auto_tiling_enabled {
                                            "█ on] Auto Tiling"
                                        } else {
                                            "off ░] Auto Tiling"
                                        };
                                        app_state.auto_tiling_button =
                                            Button::new(1, rows - 1, auto_tiling_text.to_string());

                                        // Keep config window open (silent save)
                                        handled = true;
                                    }
                                    ConfigAction::ToggleShowDate => {
                                        app_config.toggle_show_date_in_clock();

                                        // Keep config window open (silent save)
                                        handled = true;
                                    }
                                    ConfigAction::CycleTheme => {
                                        // Cycle through all themes
                                        let next_theme = match app_config.theme.as_str() {
                                            "classic" => "monochrome",
                                            "monochrome" => "dark",
                                            "dark" => "dracu",
                                            "dracu" => "green_phosphor",
                                            "green_phosphor" => "amber",
                                            "amber" => "ndd",
                                            "ndd" => "qbasic",
                                            "qbasic" => "turbo",
                                            "turbo" => "norton_commander",
                                            "norton_commander" => "xtree",
                                            "xtree" => "wordperfect",
                                            "wordperfect" => "dbase",
                                            "dbase" => "classic",
                                            _ => "classic",
                                        };
                                        app_config.theme = next_theme.to_string();
                                        let _ = app_config.save();
                                        // Reload theme
                                        theme = Theme::from_name(&app_config.theme);

                                        // Keep config window open (silent save)
                                        handled = true;
                                    }
                                    ConfigAction::CycleBackgroundChar => {
                                        // Cycle to the next background character
                                        app_config.cycle_background_char();
                                        // Update charset with new background character
                                        charset.set_background(app_config.get_background_char());

                                        // Keep config window open (silent save)
                                        handled = true;
                                    }
                                    ConfigAction::ToggleTintTerminal => {
                                        // Toggle terminal tinting and save
                                        app_config.toggle_tint_terminal();
                                        app_state.tint_terminal = app_config.tint_terminal;

                                        // Keep config window open (silent save)
                                        handled = true;
                                    }
                                    ConfigAction::ToggleAutoSave => {
                                        // Toggle auto-save and save
                                        app_config.toggle_auto_save();

                                        // If auto-save was turned OFF, clear existing session
                                        if !app_config.auto_save {
                                            let _ = WindowManager::clear_session_file();
                                        }

                                        // Keep config window open (silent save)
                                        handled = true;
                                    }
                                    ConfigAction::None => {
                                        // Check if click is inside config window
                                        if config_win
                                            .contains_point(mouse_event.column, mouse_event.row)
                                        {
                                            // Click inside config window but not on an option - consume the event
                                            handled = true;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Update button hover state on mouse movement (always active)
                    if !handled {
                        if app_state
                            .new_terminal_button
                            .contains(mouse_event.column, mouse_event.row)
                        {
                            app_state
                                .new_terminal_button
                                .set_state(button::ButtonState::Hovered);
                        } else {
                            app_state
                                .new_terminal_button
                                .set_state(button::ButtonState::Normal);
                        }

                        // Clipboard buttons hover state
                        if app_state
                            .paste_button
                            .contains(mouse_event.column, mouse_event.row)
                        {
                            app_state
                                .paste_button
                                .set_state(button::ButtonState::Hovered);
                        } else {
                            app_state
                                .paste_button
                                .set_state(button::ButtonState::Normal);
                        }

                        if app_state
                            .clear_clipboard_button
                            .contains(mouse_event.column, mouse_event.row)
                        {
                            app_state
                                .clear_clipboard_button
                                .set_state(button::ButtonState::Hovered);
                        } else {
                            app_state
                                .clear_clipboard_button
                                .set_state(button::ButtonState::Normal);
                        }

                        if app_state
                            .copy_button
                            .contains(mouse_event.column, mouse_event.row)
                        {
                            app_state
                                .copy_button
                                .set_state(button::ButtonState::Hovered);
                        } else {
                            app_state.copy_button.set_state(button::ButtonState::Normal);
                        }

                        if app_state
                            .clear_selection_button
                            .contains(mouse_event.column, mouse_event.row)
                        {
                            app_state
                                .clear_selection_button
                                .set_state(button::ButtonState::Hovered);
                        } else {
                            app_state
                                .clear_selection_button
                                .set_state(button::ButtonState::Normal);
                        }

                        // Exit button hover state
                        if app_state
                            .exit_button
                            .contains(mouse_event.column, mouse_event.row)
                        {
                            app_state
                                .exit_button
                                .set_state(button::ButtonState::Hovered);
                        } else {
                            app_state.exit_button.set_state(button::ButtonState::Normal);
                        }

                        // Battery indicator hover state (top bar, right side before clock)
                        let (cols, _) = backend.dimensions();
                        let battery_width = 10u16; // "| [█████] "
                        let clock_width = if app_config.show_date_in_clock {
                            20u16
                        } else {
                            12u16
                        };
                        let battery_start = cols.saturating_sub(battery_width + clock_width);
                        let battery_end = battery_start + battery_width;

                        app_state.battery_hovered = mouse_event.row == 0
                            && mouse_event.column >= battery_start
                            && mouse_event.column < battery_end;

                        // Calculate position for toggle button hover detection (bottom bar, left side)
                        let (_, rows) = backend.dimensions();
                        let bar_y = rows - 1;
                        let button_start_x = 1u16;
                        let button_text_width = app_state.auto_tiling_button.label.len() as u16 + 3; // +1 for "[", +1 for label, +1 for " "
                        let button_end_x = button_start_x + button_text_width;

                        if mouse_event.row == bar_y
                            && mouse_event.column >= button_start_x
                            && mouse_event.column < button_end_x
                        {
                            app_state
                                .auto_tiling_button
                                .set_state(button::ButtonState::Hovered);
                        } else {
                            app_state
                                .auto_tiling_button
                                .set_state(button::ButtonState::Normal);
                        }
                    }

                    // Check if click is on the New Terminal button in the top bar (only if no prompt)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && app_state
                            .new_terminal_button
                            .contains(mouse_event.column, mouse_event.row)
                    {
                        app_state
                            .new_terminal_button
                            .set_state(button::ButtonState::Pressed);

                        // Create a new terminal window (same as pressing 't')
                        let (cols, rows) = backend.dimensions();
                        let (width, height) = WindowManager::calculate_window_size(cols, rows);

                        // Get position: cascade if auto-tiling is off, center otherwise
                        // Minimum y=1 to avoid overlapping with topbar at y=0
                        let (x, y) = if app_state.auto_tiling_enabled {
                            let x = (cols.saturating_sub(width)) / 2;
                            let y = 1 + (rows.saturating_sub(2).saturating_sub(height)) / 2;
                            (x, y.max(1))
                        } else {
                            window_manager.get_cascade_position(width, height, cols, rows)
                        };

                        match window_manager.create_window(
                            x,
                            y,
                            width,
                            height,
                            format!("Terminal {}", window_manager.window_count() + 1),
                            None,
                        ) {
                            Ok(_) => {
                                // Auto-position all windows based on the snap pattern
                                if app_state.auto_tiling_enabled {
                                    window_manager.auto_position_windows(cols, rows);
                                }
                            }
                            Err(error_msg) => {
                                // Show error dialog
                                app_state.active_error_dialog =
                                    Some(ErrorDialog::new(cols, rows, error_msg));
                            }
                        }

                        handled = true;
                    }

                    // Check if click is on the Copy button in the top bar (only if no prompt)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && app_state
                            .copy_button
                            .contains(mouse_event.column, mouse_event.row)
                    {
                        app_state
                            .copy_button
                            .set_state(button::ButtonState::Pressed);

                        // Copy selected text to clipboard and clear selection
                        if let FocusState::Window(window_id) = window_manager.get_focus() {
                            if let Some(text) = window_manager.get_selected_text(window_id) {
                                let _ = clipboard_manager.copy(text);
                                // Clear selection after copying
                                window_manager.clear_selection(window_id);
                            }
                        }

                        handled = true;
                    }

                    // Check if click is on the Clear Selection (X) button in the top bar
                    if !handled
                        && app_state.active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && app_state
                            .clear_selection_button
                            .contains(mouse_event.column, mouse_event.row)
                    {
                        app_state
                            .clear_selection_button
                            .set_state(button::ButtonState::Pressed);

                        // Clear the selection
                        if let FocusState::Window(window_id) = window_manager.get_focus() {
                            window_manager.clear_selection(window_id);
                        }

                        handled = true;
                    }

                    // Check if click is on the Paste button in the top bar (only if no prompt)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && app_state
                            .paste_button
                            .contains(mouse_event.column, mouse_event.row)
                    {
                        app_state
                            .paste_button
                            .set_state(button::ButtonState::Pressed);

                        // Paste clipboard content to focused window
                        if let FocusState::Window(window_id) = window_manager.get_focus() {
                            if let Ok(text) = clipboard_manager.paste() {
                                let _ = window_manager.paste_to_window(window_id, &text);
                            }
                        }

                        handled = true;
                    }

                    // Check if click is on the Clear (X) button in the top bar (only if no prompt)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && app_state
                            .clear_clipboard_button
                            .contains(mouse_event.column, mouse_event.row)
                    {
                        app_state
                            .clear_clipboard_button
                            .set_state(button::ButtonState::Pressed);

                        // Clear the clipboard
                        clipboard_manager.clear();

                        handled = true;
                    }

                    // Check if click is on the Exit button in the top bar (only if no prompt and exit is allowed)
                    if !handled
                        && !cli_args.no_exit
                        && app_state.active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && app_state
                            .exit_button
                            .contains(mouse_event.column, mouse_event.row)
                    {
                        app_state
                            .exit_button
                            .set_state(button::ButtonState::Pressed);

                        // Determine message based on window count
                        let message = if window_manager.window_count() > 0 {
                            "Exit with open windows?\nAll terminal sessions will be closed."
                                .to_string()
                        } else {
                            "Exit term39?".to_string()
                        };

                        // Get dimensions
                        let (cols, rows) = backend.dimensions();

                        // Create prompt with "Cancel" selected by default (index 1)
                        app_state.active_prompt = Some(
                            Prompt::new(
                                PromptType::Danger,
                                message,
                                vec![
                                    PromptButton::new(
                                        "Exit".to_string(),
                                        PromptAction::Confirm,
                                        true,
                                    ), // Index 0
                                    PromptButton::new(
                                        "Cancel".to_string(),
                                        PromptAction::Cancel,
                                        false,
                                    ), // Index 1
                                ],
                                cols,
                                rows,
                            )
                            .with_selected_button(1),
                        ); // Select "Cancel"

                        handled = true;
                    }

                    // Check if click is on the clock in the top bar (only if no prompt)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && mouse_event.row == 0
                    {
                        // Calculate clock position (same logic as render_top_bar)
                        let (cols, _) = backend.dimensions();
                        let now = Local::now();
                        let time_str = if app_config.show_date_in_clock {
                            now.format("%a %b %d, %H:%M").to_string()
                        } else {
                            now.format("%H:%M:%S").to_string()
                        };
                        let clock_with_separator = format!("| {} ", time_str);
                        let clock_width = clock_with_separator.len() as u16;
                        let time_pos = cols.saturating_sub(clock_width);

                        // Check if click is within clock area
                        if mouse_event.column >= time_pos && mouse_event.column < cols {
                            // Show calendar (same as pressing 'c')
                            app_state.active_calendar = Some(CalendarState::new());
                            handled = true;
                        }
                    }

                    // Check if click is on the Auto-Tiling toggle button (only if no prompt)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                    {
                        // Calculate position for toggle button click detection (bottom bar, left side)
                        let (_, rows) = backend.dimensions();
                        let bar_y = rows - 1;
                        let button_start_x = 1u16;
                        let button_text_width = app_state.auto_tiling_button.label.len() as u16 + 3; // +1 for "[", +1 for label, +1 for " "
                        let button_end_x = button_start_x + button_text_width;

                        if mouse_event.row == bar_y
                            && mouse_event.column >= button_start_x
                            && mouse_event.column < button_end_x
                        {
                            app_state
                                .auto_tiling_button
                                .set_state(button::ButtonState::Pressed);

                            // Toggle the auto-tiling state and save to config
                            app_config.toggle_auto_tiling_on_startup();
                            app_state.auto_tiling_enabled = app_config.auto_tiling_on_startup;

                            // Update button label to reflect new state
                            let new_label = if app_state.auto_tiling_enabled {
                                "█ on] Auto Tiling".to_string()
                            } else {
                                "off ░] Auto Tiling".to_string()
                            };
                            app_state.auto_tiling_button = Button::new(1, bar_y, new_label);

                            handled = true;
                        }
                    }

                    // Check if click is on button bar (only if no prompt)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                    {
                        // Calculate where window buttons start (after auto-tiling button)
                        // Format: "[label ]" + 2 spaces
                        let window_buttons_start =
                            1 + 1 + app_state.auto_tiling_button.label.len() as u16 + 1 + 2;

                        handled = window_manager
                            .button_bar_click(
                                mouse_event.column,
                                bar_y,
                                mouse_event.row,
                                window_buttons_start,
                            )
                            .is_some();
                    }

                    // Handle right-click for context menu
                    if !handled
                        && app_state.active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Right)
                    {
                        if let FocusState::Window(_) = window_manager.get_focus() {
                            app_state
                                .context_menu
                                .show(mouse_event.column, mouse_event.row);
                            handled = true;
                        }
                    }

                    // Handle context menu interactions
                    if !handled && app_state.context_menu.visible {
                        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                            if app_state
                                .context_menu
                                .contains_point(mouse_event.column, mouse_event.row)
                            {
                                // Update selection to clicked item before getting action
                                app_state.context_menu.update_selection_from_mouse(
                                    mouse_event.column,
                                    mouse_event.row,
                                );
                                if let Some(action) = app_state.context_menu.get_selected_action() {
                                    if let FocusState::Window(window_id) =
                                        window_manager.get_focus()
                                    {
                                        match action {
                                            MenuAction::Copy => {
                                                if let Some(text) =
                                                    window_manager.get_selected_text(window_id)
                                                {
                                                    let _ = clipboard_manager.copy(text);
                                                    // Clear selection after copying
                                                    window_manager.clear_selection(window_id);
                                                }
                                            }
                                            MenuAction::Paste => {
                                                if let Ok(text) = clipboard_manager.paste() {
                                                    let _ = window_manager
                                                        .paste_to_window(window_id, &text);
                                                }
                                            }
                                            MenuAction::SelectAll => {
                                                window_manager.select_all(window_id);
                                            }
                                            MenuAction::Close => {}
                                        }
                                    }
                                }
                                app_state.context_menu.hide();
                                handled = true;
                            } else {
                                // Clicked outside menu - hide it
                                app_state.context_menu.hide();
                            }
                        } else if mouse_event.kind == MouseEventKind::Moved {
                            // Update menu selection on hover
                            app_state
                                .context_menu
                                .update_selection_from_mouse(mouse_event.column, mouse_event.row);
                        }
                    }

                    // Handle selection events (left-click, drag)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && !app_state.context_menu.visible
                    {
                        match mouse_event.kind {
                            MouseEventKind::Down(MouseButton::Left) => {
                                // Check if click is in a window content area
                                if let FocusState::Window(window_id) = window_manager.get_focus() {
                                    // Track click timing and position for double/triple-click detection
                                    let now = Instant::now();
                                    let click_x = mouse_event.column;
                                    let click_y = mouse_event.row;

                                    // Check if this click is close enough in time and position
                                    // to be considered a multi-click (within 500ms and 2 chars)
                                    let is_multi_click =
                                        if let (Some(last_time), Some((last_x, last_y))) =
                                            (app_state.last_click_time, app_state.last_click_pos)
                                        {
                                            let time_ok =
                                                now.duration_since(last_time).as_millis() < 500;
                                            let pos_ok = click_x.abs_diff(last_x) <= 2
                                                && click_y.abs_diff(last_y) <= 2;
                                            time_ok && pos_ok
                                        } else {
                                            false
                                        };

                                    if is_multi_click {
                                        app_state.click_count += 1;
                                    } else {
                                        app_state.click_count = 1;
                                    }
                                    app_state.last_click_time = Some(now);
                                    app_state.last_click_pos = Some((click_x, click_y));

                                    // Start or expand selection based on click count
                                    let selection_type = match app_state.click_count {
                                        2 => {
                                            window_manager.start_selection(
                                                window_id,
                                                mouse_event.column,
                                                mouse_event.row,
                                                SelectionType::Character,
                                            );
                                            window_manager.expand_selection_to_word(window_id);
                                            window_manager.complete_selection(window_id);
                                            SelectionType::Word
                                        }
                                        3 => {
                                            window_manager.start_selection(
                                                window_id,
                                                mouse_event.column,
                                                mouse_event.row,
                                                SelectionType::Character,
                                            );
                                            window_manager.expand_selection_to_line(window_id);
                                            window_manager.complete_selection(window_id);
                                            SelectionType::Line
                                        }
                                        _ => {
                                            let sel_type = if mouse_event
                                                .modifiers
                                                .contains(KeyModifiers::ALT)
                                            {
                                                SelectionType::Block
                                            } else {
                                                SelectionType::Character
                                            };
                                            window_manager.start_selection(
                                                window_id,
                                                mouse_event.column,
                                                mouse_event.row,
                                                sel_type,
                                            );
                                            sel_type
                                        }
                                    };

                                    if app_state.click_count <= 1
                                        || selection_type == SelectionType::Block
                                    {
                                        app_state.selection_active = true;
                                    }
                                }
                            }
                            MouseEventKind::Drag(MouseButton::Left) => {
                                if app_state.selection_active {
                                    if let FocusState::Window(window_id) =
                                        window_manager.get_focus()
                                    {
                                        window_manager.update_selection(
                                            window_id,
                                            mouse_event.column,
                                            mouse_event.row,
                                        );
                                    }
                                }
                            }
                            MouseEventKind::Up(MouseButton::Left) => {
                                if app_state.selection_active {
                                    if let FocusState::Window(window_id) =
                                        window_manager.get_focus()
                                    {
                                        window_manager.complete_selection(window_id);
                                    }
                                    app_state.selection_active = false;
                                }
                            }
                            _ => {}
                        }
                    }

                    // If not handled by buttons, let window manager handle it (only if no prompt)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && !app_state.context_menu.visible
                    {
                        let window_closed =
                            window_manager.handle_mouse_event(&mut video_buffer, mouse_event);
                        // Auto-reposition remaining windows if a window was closed
                        if window_closed && app_state.auto_tiling_enabled {
                            let (cols, rows) = backend.dimensions();
                            window_manager.auto_position_windows(cols, rows);
                        }
                    }

                    // Check if exit was requested (from Exit button)
                    if app_state.should_exit {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

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
    initialization::cleanup(&mut stdout)?;

    Ok(())
}
