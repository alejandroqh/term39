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
mod error_dialog;
#[cfg(feature = "framebuffer-backend")]
mod framebuffer;
mod fuzzy_matcher;
#[cfg(target_os = "linux")]
mod gpm_handler;
mod info_window;
mod initialization;
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

use app_state::AppState;
use button::Button;
use chrono::Local;
use clipboard_manager::ClipboardManager;
use command_history::CommandHistory;
use command_indexer::CommandIndexer;
use config_manager::AppConfig;
use config_window::{ConfigAction, ConfigWindow};
use context_menu::MenuAction;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    terminal::{self, ClearType},
};
use error_dialog::ErrorDialog;
use info_window::InfoWindow;
use prompt::{Prompt, PromptAction, PromptButton, PromptType};
use selection::SelectionType;
use slight_input::SlightInput;
use std::io;
use std::time::{Duration, Instant};
use theme::Theme;
use ui_render::CalendarState;
use window_manager::{FocusState, WindowManager};

// Platform detection helper - returns true if running on macOS
fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

fn main() -> io::Result<()> {
    // Parse command-line arguments
    let cli_args = cli::Cli::parse_args();

    // Handle --fb-list-fonts flag (exit after listing)
    #[cfg(feature = "framebuffer-backend")]
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

    // Load application configuration
    let mut app_config = AppConfig::load();

    // Create charset and theme
    let mut charset = initialization::initialize_charset(&cli_args, &app_config);
    let mut theme = initialization::initialize_theme(&cli_args, &app_config);

    // Initialize rendering backend (framebuffer or terminal)
    let mut backend = initialization::initialize_backend(&cli_args)?;

    let mut stdout = io::stdout();

    // Set up terminal modes and mouse capture
    initialization::setup_terminal(&mut stdout)?;

    // Initialize GPM (General Purpose Mouse) for Linux console if available
    #[cfg(target_os = "linux")]
    let gpm_connection = initialization::initialize_gpm();

    // Initialize video buffer and window manager
    let mut video_buffer = initialization::initialize_video_buffer(&backend);
    let mut window_manager = initialization::initialize_window_manager(&cli_args, &mut app_config)?;

    // Initialize application state
    let (cols, rows) = backend.dimensions();
    let mut app_state = AppState::new(
        cols,
        rows,
        app_config.auto_tiling_on_startup,
        app_config.tint_terminal || cli_args.tint_terminal,
    );

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
        if let Some((_new_cols, new_rows)) = backend.check_resize()? {
            // Clear the terminal screen to remove artifacts
            use crossterm::execute;
            execute!(stdout, terminal::Clear(ClearType::All))?;
            video_buffer = initialization::initialize_video_buffer(&backend);
            app_state.update_auto_tiling_button_position(new_rows);
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

        // Check for GPM events first (Linux console mouse support)
        #[cfg(target_os = "linux")]
        let gpm_event = if let Some(ref gpm) = gpm_connection {
            if gpm.has_event() {
                gpm.get_event()
            } else {
                None
            }
        } else {
            None
        };

        // Process GPM event if available - convert to Event::Mouse and fall through
        #[cfg(target_os = "linux")]
        #[cfg_attr(not(feature = "framebuffer-backend"), allow(unused_mut))]
        let mut injected_event = if let Some(gpm_evt) = gpm_event {
            // Convert GPM event to crossterm MouseEvent format
            use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

            // Scale mouse coordinates from TTY space to backend space
            let (scaled_col, scaled_row) = backend.scale_mouse_coords(gpm_evt.x, gpm_evt.y);

            let mouse_event = match gpm_evt.event_type {
                gpm_handler::GpmEventType::Down => {
                    let button = match gpm_evt.button {
                        Some(gpm_handler::GpmButton::Left) => MouseButton::Left,
                        Some(gpm_handler::GpmButton::Middle) => MouseButton::Middle,
                        Some(gpm_handler::GpmButton::Right) => MouseButton::Right,
                        None => MouseButton::Left,
                    };
                    MouseEvent {
                        kind: MouseEventKind::Down(button),
                        column: scaled_col,
                        row: scaled_row,
                        modifiers: KeyModifiers::empty(),
                    }
                }
                gpm_handler::GpmEventType::Up => {
                    let button = match gpm_evt.button {
                        Some(gpm_handler::GpmButton::Left) => MouseButton::Left,
                        Some(gpm_handler::GpmButton::Middle) => MouseButton::Middle,
                        Some(gpm_handler::GpmButton::Right) => MouseButton::Right,
                        None => MouseButton::Left,
                    };
                    MouseEvent {
                        kind: MouseEventKind::Up(button),
                        column: scaled_col,
                        row: scaled_row,
                        modifiers: KeyModifiers::empty(),
                    }
                }
                gpm_handler::GpmEventType::Drag => {
                    let button = match gpm_evt.button {
                        Some(gpm_handler::GpmButton::Left) => MouseButton::Left,
                        Some(gpm_handler::GpmButton::Middle) => MouseButton::Middle,
                        Some(gpm_handler::GpmButton::Right) => MouseButton::Right,
                        None => MouseButton::Left,
                    };
                    MouseEvent {
                        kind: MouseEventKind::Drag(button),
                        column: scaled_col,
                        row: scaled_row,
                        modifiers: KeyModifiers::empty(),
                    }
                }
                gpm_handler::GpmEventType::Move => MouseEvent {
                    kind: MouseEventKind::Moved,
                    column: scaled_col,
                    row: scaled_row,
                    modifiers: KeyModifiers::empty(),
                },
            };

            Some(Event::Mouse(mouse_event))
        } else {
            None
        };

        // Process framebuffer mouse event if available (when GPM is not active)
        #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
        if injected_event.is_none() {
            if let Some((event_type, button_id, col, row)) = backend.get_mouse_button_event() {
                use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

                // Map button ID to MouseButton
                let button = match button_id {
                    0 => MouseButton::Left,
                    1 => MouseButton::Right,
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

        // Process injected GPM/FB event or poll for crossterm event
        // Don't wait if we have an injected event
        #[cfg(target_os = "linux")]
        let has_event = injected_event.is_some() || event::poll(Duration::from_millis(16))?;
        #[cfg(not(target_os = "linux"))]
        let has_event = event::poll(Duration::from_millis(16))?;

        if has_event {
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
                    let current_focus = window_manager.get_focus();

                    // Handle prompt keyboard navigation if a prompt is active
                    if let Some(ref mut prompt) = app_state.active_prompt {
                        match key_event.code {
                            KeyCode::Tab => {
                                // Tab or Shift+Tab to navigate buttons
                                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                                    prompt.select_previous_button();
                                } else {
                                    prompt.select_next_button();
                                }
                                continue;
                            }
                            KeyCode::Left => {
                                // Left arrow - previous button
                                prompt.select_previous_button();
                                continue;
                            }
                            KeyCode::Right => {
                                // Right arrow - next button
                                prompt.select_next_button();
                                continue;
                            }
                            KeyCode::Enter => {
                                // Enter - activate selected button
                                if let Some(action) = prompt.get_selected_action() {
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
                                }
                                continue;
                            }
                            KeyCode::Esc => {
                                // ESC dismisses the prompt
                                app_state.active_prompt = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when prompt is active
                                continue;
                            }
                        }
                    }

                    // Handle error dialog keyboard events if active
                    if app_state.active_error_dialog.is_some() {
                        match key_event.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                // Dismiss error dialog
                                app_state.active_error_dialog = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when error dialog is active
                                continue;
                            }
                        }
                    }

                    // Handle Slight input keyboard events if active
                    if let Some(ref mut slight_input) = app_state.active_slight_input {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                slight_input.insert_char(c);
                                continue;
                            }
                            KeyCode::Backspace => {
                                slight_input.delete_char();
                                continue;
                            }
                            KeyCode::Left => {
                                slight_input.move_cursor_left();
                                continue;
                            }
                            KeyCode::Right => {
                                // If at end of input, accept inline suggestion
                                // Otherwise, move cursor right
                                if slight_input.cursor_position == slight_input.input_text.len() {
                                    slight_input.accept_inline_suggestion();
                                } else {
                                    slight_input.move_cursor_right();
                                }
                                continue;
                            }
                            KeyCode::Up => {
                                slight_input.previous_suggestion();
                                continue;
                            }
                            KeyCode::Down => {
                                slight_input.next_suggestion();
                                continue;
                            }
                            KeyCode::Tab => {
                                slight_input.accept_selected_suggestion();
                                continue;
                            }
                            KeyCode::Home => {
                                slight_input.move_cursor_home();
                                continue;
                            }
                            KeyCode::End => {
                                slight_input.move_cursor_end();
                                continue;
                            }
                            KeyCode::Enter => {
                                // Get the command and create a new terminal window with it
                                let command = slight_input.get_input();

                                // Record command in history before closing
                                if !command.is_empty() {
                                    command_history.record_command(&command);
                                }

                                app_state.active_slight_input = None;

                                if !command.is_empty() {
                                    // Create a new terminal window and run the command
                                    let (cols, rows) = backend.dimensions();

                                    // Window size: 2.5x larger (60*2.5=150, 20*2.5=50)
                                    let width = 150;
                                    let height = 50;

                                    // Get position: cascade if auto-tiling is off, center otherwise
                                    let (x, y) = if app_state.auto_tiling_enabled {
                                        let x = (cols.saturating_sub(width)) / 2;
                                        let y = ((rows.saturating_sub(height)) / 2).max(1);
                                        (x, y)
                                    } else {
                                        window_manager
                                            .get_cascade_position(width, height, cols, rows)
                                    };

                                    match window_manager.create_window(
                                        x,
                                        y,
                                        width,
                                        height,
                                        format!("Terminal {}", window_manager.window_count() + 1),
                                        Some(command),
                                    ) {
                                        Ok(_terminal_id) => {
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
                                }
                                continue;
                            }
                            KeyCode::Esc => {
                                // ESC dismisses the Slight input
                                app_state.active_slight_input = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when Slight input is active
                                continue;
                            }
                        }
                    }

                    // Handle calendar keyboard navigation if calendar is active
                    if let Some(ref mut calendar) = app_state.active_calendar {
                        match key_event.code {
                            KeyCode::Char('<') | KeyCode::Char(',') | KeyCode::Left => {
                                // Previous month
                                calendar.previous_month();
                                continue;
                            }
                            KeyCode::Char('>') | KeyCode::Char('.') | KeyCode::Right => {
                                // Next month
                                calendar.next_month();
                                continue;
                            }
                            KeyCode::Up => {
                                // Previous year
                                calendar.previous_year();
                                continue;
                            }
                            KeyCode::Down => {
                                // Next year
                                calendar.next_year();
                                continue;
                            }
                            KeyCode::Char('t') | KeyCode::Home => {
                                // Reset to today
                                calendar.reset_to_today();
                                continue;
                            }
                            KeyCode::Esc => {
                                // ESC dismisses the calendar
                                app_state.active_calendar = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when calendar is active
                                continue;
                            }
                        }
                    }

                    // Handle help window keyboard events if help window is active
                    if app_state.active_help_window.is_some() {
                        match key_event.code {
                            KeyCode::Esc => {
                                // ESC dismisses the help window
                                app_state.active_help_window = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when help window is active
                                continue;
                            }
                        }
                    }

                    // Handle about window keyboard events if about window is active
                    if app_state.active_about_window.is_some() {
                        match key_event.code {
                            KeyCode::Esc => {
                                // ESC dismisses the about window
                                app_state.active_about_window = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when about window is active
                                continue;
                            }
                        }
                    }

                    // Handle config window keyboard events if config window is active
                    if app_state.active_config_window.is_some() {
                        match key_event.code {
                            KeyCode::Esc => {
                                // ESC dismisses the config window
                                app_state.active_config_window = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when config window is active
                                continue;
                            }
                        }
                    }

                    // Handle F1 to show help (universal help key)
                    if key_event.code == KeyCode::F(1) {
                        if current_focus == FocusState::Desktop {
                            let (cols, rows) = backend.dimensions();

                            // Platform-specific modifier key text
                            let (copy_key, paste_key) = if is_macos() {
                                ("CMD+C", "CMD+V")
                            } else {
                                ("CTRL+SHIFT+C", "CTRL+SHIFT+V")
                            };

                            let help_message = format!(
                                "{{C}}KEYBOARD SHORTCUTS{{W}}\n\
                                \n\
                                {{Y}}'t'{{W}}       - Create new terminal window\n\
                                {{Y}}'T'{{W}}       - Create new maximized terminal window\n\
                                {{Y}}'q'/ESC{{W}}   - Exit application (from desktop)\n\
                                {{Y}}F1{{W}} or {{Y}}'h'{{W}} - Show this help screen\n\
                                {{Y}}'l'{{W}}       - Show license and about information\n\
                                {{Y}}'s'{{W}}       - Show settings/configuration window\n\
                                {{Y}}'c'{{W}}       - Show calendar ({{Y}}\u{2190}\u{2192}{{W}} months, {{Y}}\u{2191}\u{2193}{{W}} years, {{Y}}t{{W}} today)\n\
                                {{Y}}CTRL+Space{{W}} - Command launcher (Slight)\n\
                                \n\
                                {{C}}WINDOW & SESSION{{W}}\n\
                                \n\
                                {{Y}}F2{{W}} or {{Y}}ALT+TAB{{W}} - Switch between windows\n\
                                {{Y}}F3{{W}}              - Save session manually\n\
                                {{Y}}F4{{W}} or {{Y}}CTRL+L{{W}}  - Clear terminal\n\
                                \n\
                                {{C}}COPY & PASTE{{W}}\n\
                                \n\
                                {{Y}}{}{{W}} or {{Y}}F6{{W}} - Copy selected text\n\
                                {{Y}}{}{{W}} or {{Y}}F7{{W}} - Paste from clipboard\n\
                                \n\
                                {{C}}POPUP DIALOG CONTROLS{{W}}\n\
                                \n\
                                {{Y}}TAB/Arrow keys{{W}} - Navigate between buttons\n\
                                {{Y}}ENTER{{W}}          - Activate selected button\n\
                                {{Y}}ESC{{W}}            - Close dialog\n\
                                \n\
                                {{C}}MOUSE CONTROLS{{W}}\n\
                                \n\
                                {{Y}}Click title bar{{W}}     - Drag window\n\
                                {{Y}}CTRL+Drag{{W}}          - Drag without snap\n\
                                {{Y}}Click [X]{{W}}           - Close window\n\
                                {{Y}}Drag border{{W}}         - Resize window\n\
                                {{Y}}Click window{{W}}        - Focus window\n\
                                {{Y}}Click bottom bar{{W}}    - Switch windows",
                                copy_key, paste_key
                            );

                            app_state.active_help_window = Some(InfoWindow::new(
                                "Help".to_string(),
                                &help_message,
                                cols,
                                rows,
                            ));
                        }
                        continue;
                    }

                    // Handle F2 for window cycling (more compatible than ALT+TAB)
                    if key_event.code == KeyCode::F(2) {
                        window_manager.cycle_to_next_window();
                        continue;
                    }

                    // Handle ALT+TAB for window cycling (fallback, may be intercepted by OS)
                    if key_event.code == KeyCode::Tab
                        && key_event.modifiers.contains(KeyModifiers::ALT)
                    {
                        window_manager.cycle_to_next_window();
                        continue;
                    }

                    // Handle F3 to save session (more compatible than CTRL+S)
                    if key_event.code == KeyCode::F(3) {
                        // Save session to file (unless --no-save flag is set OR auto-save is disabled)
                        if cli_args.no_save {
                            // Show info that saving is disabled by --no-save flag
                            let (cols, rows) = backend.dimensions();
                            app_state.active_prompt = Some(Prompt::new(
                                PromptType::Warning,
                                "Session saving is disabled (--no-save flag)".to_string(),
                                vec![PromptButton::new(
                                    "OK".to_string(),
                                    PromptAction::Cancel,
                                    true,
                                )],
                                cols,
                                rows,
                            ));
                        } else if !app_config.auto_save {
                            // Show info that auto-save is disabled in settings
                            let (cols, rows) = backend.dimensions();
                            app_state.active_prompt = Some(Prompt::new(
                                PromptType::Warning,
                                "Session auto-save is disabled in Settings".to_string(),
                                vec![PromptButton::new(
                                    "OK".to_string(),
                                    PromptAction::Cancel,
                                    true,
                                )],
                                cols,
                                rows,
                            ));
                        } else if window_manager.save_session_to_file().is_ok() {
                            // Show success prompt
                            let (cols, rows) = backend.dimensions();
                            app_state.active_prompt = Some(Prompt::new(
                                PromptType::Success,
                                "Session saved successfully!".to_string(),
                                vec![PromptButton::new(
                                    "OK".to_string(),
                                    PromptAction::Cancel,
                                    true,
                                )],
                                cols,
                                rows,
                            ));
                        } else {
                            // Show error prompt if save failed
                            let (cols, rows) = backend.dimensions();
                            app_state.active_prompt = Some(Prompt::new(
                                PromptType::Danger,
                                "Failed to save session!".to_string(),
                                vec![PromptButton::new(
                                    "OK".to_string(),
                                    PromptAction::Cancel,
                                    true,
                                )],
                                cols,
                                rows,
                            ));
                        }
                        continue;
                    }

                    // Handle F4 to clear the terminal (alternative to CTRL+L)
                    if key_event.code == KeyCode::F(4) {
                        if current_focus != FocusState::Desktop {
                            // Send Ctrl+L (form feed, 0x0c) to the shell
                            let _ = window_manager.send_to_focused("\x0c");
                        }
                        continue;
                    }

                    // Handle CTRL+L to clear the terminal (like 'clear' command)
                    if key_event.code == KeyCode::Char('l')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        if current_focus != FocusState::Desktop {
                            // Send Ctrl+L (form feed, 0x0c) to the shell
                            let _ = window_manager.send_to_focused("\x0c");
                        }
                        continue;
                    }

                    // Handle CTRL+Space to open Slight input popup
                    if key_event.code == KeyCode::Char(' ')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        // Create Slight input popup with autocomplete
                        let (cols, rows) = backend.dimensions();
                        let mut slight_input = SlightInput::new(cols, rows);
                        slight_input
                            .set_autocomplete(command_indexer.clone(), command_history.clone());
                        app_state.active_slight_input = Some(slight_input);
                        continue;
                    }

                    // Handle F6 to copy selection (universal alternative)
                    if key_event.code == KeyCode::F(6) {
                        if let FocusState::Window(window_id) = current_focus {
                            if let Some(text) = window_manager.get_selected_text(window_id) {
                                if clipboard_manager.copy(text).is_ok() {
                                    // Clear selection after copying
                                    window_manager.clear_selection(window_id);
                                }
                            }
                        }
                        continue;
                    }

                    // Handle F7 to paste (universal alternative)
                    if key_event.code == KeyCode::F(7) {
                        if let FocusState::Window(window_id) = current_focus {
                            if let Ok(text) = clipboard_manager.paste() {
                                let _ = window_manager.paste_to_window(window_id, &text);
                                // Clear selection after paste
                                window_manager.clear_selection(window_id);
                            }
                        }
                        continue;
                    }

                    // Platform-aware copy shortcut
                    // macOS: CMD+C (SUPER modifier)
                    // Linux/Windows: CTRL+SHIFT+C
                    let is_copy_shortcut = if is_macos() {
                        // On macOS: CMD+C
                        key_event.code == KeyCode::Char('c')
                            && key_event.modifiers.contains(KeyModifiers::SUPER)
                    } else {
                        // On Linux/Windows: CTRL+SHIFT+C
                        key_event.code == KeyCode::Char('C')
                            && key_event.modifiers.contains(KeyModifiers::CONTROL)
                            && key_event.modifiers.contains(KeyModifiers::SHIFT)
                    };

                    if is_copy_shortcut {
                        if let FocusState::Window(window_id) = current_focus {
                            if let Some(text) = window_manager.get_selected_text(window_id) {
                                if clipboard_manager.copy(text).is_ok() {
                                    // Clear selection after copying
                                    window_manager.clear_selection(window_id);
                                }
                            }
                        }
                        continue;
                    }

                    // Platform-aware paste shortcut
                    // macOS: CMD+V (SUPER modifier)
                    // Linux/Windows: CTRL+SHIFT+V
                    let is_paste_shortcut = if is_macos() {
                        // On macOS: CMD+V
                        key_event.code == KeyCode::Char('v')
                            && key_event.modifiers.contains(KeyModifiers::SUPER)
                    } else {
                        // On Linux/Windows: CTRL+SHIFT+V
                        key_event.code == KeyCode::Char('V')
                            && key_event.modifiers.contains(KeyModifiers::CONTROL)
                            && key_event.modifiers.contains(KeyModifiers::SHIFT)
                    };

                    if is_paste_shortcut {
                        if let FocusState::Window(window_id) = current_focus {
                            if let Ok(text) = clipboard_manager.paste() {
                                let _ = window_manager.paste_to_window(window_id, &text);
                                // Clear selection after paste
                                window_manager.clear_selection(window_id);
                            }
                        }
                        continue;
                    }

                    match key_event.code {
                        KeyCode::Esc => {
                            // ESC exits only from desktop (prompts are handled above)
                            if current_focus == FocusState::Desktop {
                                // If windows are open, show confirmation
                                if window_manager.window_count() > 0 {
                                    let (cols, rows) = backend.dimensions();
                                    app_state.active_prompt = Some(Prompt::new(
                                        PromptType::Danger,
                                        "Exit with open windows?\nAll terminal sessions will be closed.".to_string(),
                                        vec![
                                            PromptButton::new("Exit".to_string(), PromptAction::Confirm, true),
                                            PromptButton::new("Cancel".to_string(), PromptAction::Cancel, false),
                                        ],
                                        cols,
                                        rows,
                                    ));
                                } else {
                                    // No windows open, just exit
                                    break;
                                }
                            } else {
                                // Send ESC to terminal
                                let _ = window_manager.send_to_focused("\x1b");
                            }
                        }
                        KeyCode::Char('q') => {
                            // Only exit if desktop is focused (prompts are handled above)
                            if current_focus == FocusState::Desktop {
                                // If windows are open, show confirmation
                                if window_manager.window_count() > 0 {
                                    let (cols, rows) = backend.dimensions();
                                    app_state.active_prompt = Some(Prompt::new(
                                        PromptType::Danger,
                                        "Exit with open windows?\nAll terminal sessions will be closed.".to_string(),
                                        vec![
                                            PromptButton::new("Exit".to_string(), PromptAction::Confirm, true),
                                            PromptButton::new("Cancel".to_string(), PromptAction::Cancel, false),
                                        ],
                                        cols,
                                        rows,
                                    ));
                                } else {
                                    // No windows open, just exit
                                    break;
                                }
                            } else {
                                // Send 'q' to terminal
                                let _ = window_manager.send_char_to_focused('q');
                            }
                        }
                        KeyCode::Char('h') => {
                            // Show help if desktop is focused (prompts are handled above)
                            if current_focus == FocusState::Desktop {
                                let (cols, rows) = backend.dimensions();

                                // Platform-specific modifier key text
                                let (copy_key, paste_key) = if is_macos() {
                                    ("CMD+C", "CMD+V")
                                } else {
                                    ("CTRL+SHIFT+C", "CTRL+SHIFT+V")
                                };

                                let help_message = format!(
                                    "{{C}}KEYBOARD SHORTCUTS{{W}}\n\
                                    \n\
                                    {{Y}}'t'{{W}}       - Create new terminal window\n\
                                    {{Y}}'T'{{W}}       - Create new maximized terminal window\n\
                                    {{Y}}'q'/ESC{{W}}   - Exit application (from desktop)\n\
                                    {{Y}}F1{{W}} or {{Y}}'h'{{W}} - Show this help screen\n\
                                    {{Y}}'l'{{W}}       - Show license and about information\n\
                                    {{Y}}'s'{{W}}       - Show settings/configuration window\n\
                                    {{Y}}'c'{{W}}       - Show calendar ({{Y}}\u{2190}\u{2192}{{W}} months, {{Y}}\u{2191}\u{2193}{{W}} years, {{Y}}t{{W}} today)\n\
                                    {{Y}}CTRL+Space{{W}} - Command launcher (Slight)\n\
                                    \n\
                                    {{C}}WINDOW & SESSION{{W}}\n\
                                    \n\
                                    {{Y}}F2{{W}} or {{Y}}ALT+TAB{{W}} - Switch between windows\n\
                                    {{Y}}F3{{W}}              - Save session manually\n\
                                    {{Y}}F4{{W}} or {{Y}}CTRL+L{{W}}  - Clear terminal\n\
                                    \n\
                                    {{C}}COPY & PASTE{{W}}\n\
                                    \n\
                                    {{Y}}{}{{W}} or {{Y}}F6{{W}} - Copy selected text\n\
                                    {{Y}}{}{{W}} or {{Y}}F7{{W}} - Paste from clipboard\n\
                                    \n\
                                    {{C}}POPUP DIALOG CONTROLS{{W}}\n\
                                    \n\
                                    {{Y}}TAB/Arrow keys{{W}} - Navigate between buttons\n\
                                    {{Y}}ENTER{{W}}          - Activate selected button\n\
                                    {{Y}}ESC{{W}}            - Close dialog\n\
                                    \n\
                                    {{C}}MOUSE CONTROLS{{W}}\n\
                                    \n\
                                    {{Y}}Click title bar{{W}}     - Drag window\n\
                                    {{Y}}CTRL+Drag{{W}}          - Drag without snap\n\
                                    {{Y}}Click [X]{{W}}           - Close window\n\
                                    {{Y}}Drag border{{W}}         - Resize window\n\
                                    {{Y}}Click window{{W}}        - Focus window\n\
                                    {{Y}}Click bottom bar{{W}}    - Switch windows",
                                    copy_key, paste_key
                                );

                                app_state.active_help_window = Some(InfoWindow::new(
                                    "Help".to_string(),
                                    &help_message,
                                    cols,
                                    rows,
                                ));
                            } else if current_focus != FocusState::Desktop {
                                // Send 'h' to terminal
                                let _ = window_manager.send_char_to_focused('h');
                            }
                        }
                        KeyCode::Char('l') => {
                            // Show license and about if desktop is focused
                            if current_focus == FocusState::Desktop {
                                let (cols, rows) = backend.dimensions();
                                let license_message = format!(
                                    "TERM39 - Terminal UI Windows Manager\n\
                                    \n\
                                    A low-level terminal UI windows manager built with Rust.\n\
                                    \n\
                                    Version: {}\n\
                                    Author: {}\n\
                                    Repository: {}\n\
                                    \n\
                                    LICENSE\n\
                                    \n\
                                    This software is licensed under the MIT License.\n\
                                    See LICENSE file or visit the repository for details.\n\
                                    \n\
                                    BUILT WITH\n\
                                    \n\
                                    This project uses the following open source packages:\n\
                                    \n\
                                    - crossterm - Cross-platform terminal manipulation\n\
                                    - portable-pty - Portable pseudo-terminal support\n\
                                    - vte - Virtual terminal emulator parser\n\
                                    - chrono - Date and time library\n\
                                    \n\
                                    All dependencies are used under their respective licenses.",
                                    config::VERSION,
                                    config::AUTHORS,
                                    config::REPOSITORY
                                );

                                app_state.active_about_window = Some(InfoWindow::new(
                                    "About".to_string(),
                                    &license_message,
                                    cols,
                                    rows,
                                ));
                            } else if current_focus != FocusState::Desktop {
                                // Send 'l' to terminal
                                let _ = window_manager.send_char_to_focused('l');
                            }
                        }
                        KeyCode::Char('c') => {
                            // Show calendar if desktop is focused
                            if current_focus == FocusState::Desktop {
                                app_state.active_calendar = Some(CalendarState::new());
                            } else if current_focus != FocusState::Desktop {
                                // Send 'c' to terminal
                                let _ = window_manager.send_char_to_focused('c');
                            }
                        }
                        KeyCode::Char('s') => {
                            // Show settings/config window if desktop is focused
                            if current_focus == FocusState::Desktop {
                                let (cols, rows) = backend.dimensions();
                                app_state.active_config_window =
                                    Some(ConfigWindow::new(cols, rows));
                            } else if current_focus != FocusState::Desktop {
                                // Send 's' to terminal
                                let _ = window_manager.send_char_to_focused('s');
                            }
                        }
                        KeyCode::Char('t') => {
                            // Only create new window if desktop is focused
                            if current_focus == FocusState::Desktop {
                                // Create a new terminal window
                                let (cols, rows) = backend.dimensions();

                                // Window size: 2.5x larger (60*2.5=150, 20*2.5=50)
                                let width = 150;
                                let height = 50;

                                // Get position: cascade if auto-tiling is off, center otherwise
                                let (x, y) = if app_state.auto_tiling_enabled {
                                    let x = (cols.saturating_sub(width)) / 2;
                                    let y = ((rows.saturating_sub(height)) / 2).max(1);
                                    (x, y)
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
                            } else {
                                // Send 't' to terminal
                                let _ = window_manager.send_char_to_focused('t');
                            }
                        }
                        KeyCode::Char('T') => {
                            // Only create maximized window if desktop is focused
                            if current_focus == FocusState::Desktop {
                                // Create a new terminal window
                                let (cols, rows) = backend.dimensions();

                                // Window size: 2.5x larger (60*2.5=150, 20*2.5=50)
                                let width = 150;
                                let height = 50;

                                // Get position: cascade if auto-tiling is off, center otherwise
                                // (will be maximized immediately, but still track for cascading)
                                let (x, y) = if app_state.auto_tiling_enabled {
                                    let x = (cols.saturating_sub(width)) / 2;
                                    let y = ((rows.saturating_sub(height)) / 2).max(1);
                                    (x, y)
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
                                    Ok(window_id) => {
                                        // Maximize the newly created window
                                        window_manager.maximize_window(window_id, cols, rows);
                                    }
                                    Err(error_msg) => {
                                        // Show error dialog
                                        app_state.active_error_dialog =
                                            Some(ErrorDialog::new(cols, rows, error_msg));
                                    }
                                }
                            } else {
                                // Send 'T' to terminal
                                let _ = window_manager.send_char_to_focused('T');
                            }
                        }
                        KeyCode::Char(c) => {
                            // Send character to focused terminal
                            if current_focus != FocusState::Desktop {
                                // Check if CTRL is pressed (but not handled by specific shortcuts above)
                                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                                    // Convert to control character (Ctrl+A = 0x01, Ctrl+B = 0x02, etc.)
                                    // Ctrl+letter maps to ASCII 1-26 for a-z (case insensitive)
                                    if c.is_ascii_alphabetic() {
                                        let control_char = match c.to_ascii_lowercase() {
                                            'a'..='z' => {
                                                // Ctrl+A = 1, Ctrl+B = 2, ..., Ctrl+Z = 26
                                                (c.to_ascii_lowercase() as u8 - b'a' + 1) as char
                                            }
                                            _ => c,
                                        };
                                        let _ = window_manager
                                            .send_to_focused(&control_char.to_string());
                                    } else {
                                        // For non-alphabetic characters with Ctrl, send as-is
                                        // This handles cases like Ctrl+[ which is ESC
                                        let _ = window_manager.send_char_to_focused(c);
                                    }
                                } else {
                                    // Normal character without Ctrl
                                    let _ = window_manager.send_char_to_focused(c);
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\r");
                            }
                        }
                        KeyCode::Backspace => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x7f");
                            }
                        }
                        KeyCode::Tab => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\t");
                            }
                        }
                        KeyCode::Up => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x1b[A");
                            }
                        }
                        KeyCode::Down => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x1b[B");
                            }
                        }
                        KeyCode::Right => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x1b[C");
                            }
                        }
                        KeyCode::Left => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x1b[D");
                            }
                        }
                        KeyCode::Home => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x1b[H");
                            }
                        }
                        KeyCode::End => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x1b[F");
                            }
                        }
                        KeyCode::PageUp => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x1b[5~");
                            }
                        }
                        KeyCode::PageDown => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x1b[6~");
                            }
                        }
                        KeyCode::Delete => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x1b[3~");
                            }
                        }
                        KeyCode::Insert => {
                            if current_focus != FocusState::Desktop {
                                let _ = window_manager.send_to_focused("\x1b[2~");
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mut mouse_event) => {
                    // Scale mouse coordinates from TTY space to backend space
                    let (scaled_col, scaled_row) =
                        backend.scale_mouse_coords(mouse_event.column, mouse_event.row);
                    mouse_event.column = scaled_col;
                    mouse_event.row = scaled_row;

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
                                        // Cycle through themes: classic -> monochrome -> dark -> green_phosphor -> amber -> classic
                                        let next_theme = match app_config.theme.as_str() {
                                            "classic" => "monochrome",
                                            "monochrome" => "dark",
                                            "dark" => "green_phosphor",
                                            "green_phosphor" => "amber",
                                            "amber" => "classic",
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
                        let width = 150;
                        let height = 50;

                        // Get position: cascade if auto-tiling is off, center otherwise
                        let (x, y) = if app_state.auto_tiling_enabled {
                            let x = (cols.saturating_sub(width)) / 2;
                            let y = ((rows.saturating_sub(height)) / 2).max(1);
                            (x, y)
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

                            // Toggle the auto-tiling state
                            app_state.auto_tiling_enabled = !app_state.auto_tiling_enabled;

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
                                                // TODO: Implement select all
                                            }
                                            MenuAction::CopyWindow => {
                                                // TODO: Implement copy window
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
                            if app_state
                                .context_menu
                                .contains_point(mouse_event.column, mouse_event.row)
                            {
                                // TODO: Update hover state if needed
                            }
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
                                    // Track click timing for double/triple-click detection
                                    let now = Instant::now();
                                    let is_multi_click =
                                        if let Some(last_time) = app_state.last_click_time {
                                            now.duration_since(last_time).as_millis() < 500
                                        } else {
                                            false
                                        };

                                    if is_multi_click {
                                        app_state.click_count += 1;
                                    } else {
                                        app_state.click_count = 1;
                                    }
                                    app_state.last_click_time = Some(now);

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
