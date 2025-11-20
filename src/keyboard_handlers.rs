use crate::app_state::AppState;
use crate::cli::Cli;
use crate::clipboard_manager::ClipboardManager;
use crate::config;
use crate::config_manager::AppConfig;
use crate::config_window::ConfigWindow;
use crate::error_dialog::ErrorDialog;
use crate::info_window::InfoWindow;
use crate::prompt::{Prompt, PromptAction, PromptButton, PromptType};
use crate::render_backend::RenderBackend;
use crate::ui_render::CalendarState;
use crate::window_manager::{FocusState, WindowManager};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Platform detection helper - returns true if running on macOS
fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// Handles desktop keyboard shortcuts (when not in a dialog)
/// Returns true if event was handled
#[allow(clippy::too_many_arguments)]
pub fn handle_desktop_keyboard(
    app_state: &mut AppState,
    key_event: KeyEvent,
    current_focus: FocusState,
    window_manager: &mut WindowManager,
    clipboard_manager: &mut ClipboardManager,
    backend: &Box<dyn RenderBackend>,
    app_config: &AppConfig,
    cli_args: &Cli,
) -> bool {
    // Handle F1 to show help (universal help key)
    if key_event.code == KeyCode::F(1) && current_focus == FocusState::Desktop {
        show_help_window(app_state, backend);
        return true;
    }

    // Handle F2 for window cycling (more compatible than ALT+TAB)
    if key_event.code == KeyCode::F(2) {
        window_manager.cycle_to_next_window();
        return true;
    }

    // Handle ALT+TAB for window cycling (fallback, may be intercepted by OS)
    if key_event.code == KeyCode::Tab && key_event.modifiers.contains(KeyModifiers::ALT) {
        window_manager.cycle_to_next_window();
        return true;
    }

    // Handle F3 to save session (more compatible than CTRL+S)
    if key_event.code == KeyCode::F(3) {
        handle_save_session(app_state, window_manager, backend, cli_args, app_config);
        return true;
    }

    // Handle F4 to clear the terminal (alternative to CTRL+L)
    if key_event.code == KeyCode::F(4) && current_focus != FocusState::Desktop {
        let _ = window_manager.send_to_focused("\x0c");
        return true;
    }

    // Handle CTRL+L to clear the terminal (like 'clear' command)
    if key_event.code == KeyCode::Char('l')
        && key_event.modifiers.contains(KeyModifiers::CONTROL)
        && current_focus != FocusState::Desktop
    {
        let _ = window_manager.send_to_focused("\x0c");
        return true;
    }

    // Handle CTRL+Space / Option+Space to open Slight input popup
    // Note: Ctrl+Space produces NUL character ('\0') in most terminals
    // On macOS, Option+Space produces non-breaking space (U+00A0)
    let is_launcher_shortcut = (key_event.code == KeyCode::Char(' ')
        && (key_event.modifiers.contains(KeyModifiers::CONTROL)
            || key_event.modifiers.contains(KeyModifiers::ALT)))
        || key_event.code == KeyCode::Char('\0')
        || key_event.code == KeyCode::Char('\u{00a0}'); // Non-breaking space from Option+Space on macOS
    if is_launcher_shortcut {
        return true; // Signal to open Slight input (handled in main)
    }

    // Handle F6 to copy selection (universal alternative)
    if key_event.code == KeyCode::F(6) {
        if let FocusState::Window(window_id) = current_focus {
            if let Some(text) = window_manager.get_selected_text(window_id) {
                if clipboard_manager.copy(text).is_ok() {
                    window_manager.clear_selection(window_id);
                }
            }
        }
        return true;
    }

    // Handle F7 to paste (universal alternative)
    if key_event.code == KeyCode::F(7) {
        if let FocusState::Window(window_id) = current_focus {
            if let Ok(text) = clipboard_manager.paste() {
                let _ = window_manager.paste_to_window(window_id, &text);
                window_manager.clear_selection(window_id);
            }
        }
        return true;
    }

    // Platform-aware copy shortcut
    let is_copy_shortcut = if is_macos() {
        key_event.code == KeyCode::Char('c') && key_event.modifiers.contains(KeyModifiers::SUPER)
    } else {
        key_event.code == KeyCode::Char('C')
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
            && key_event.modifiers.contains(KeyModifiers::SHIFT)
    };

    if is_copy_shortcut {
        if let FocusState::Window(window_id) = current_focus {
            if let Some(text) = window_manager.get_selected_text(window_id) {
                if clipboard_manager.copy(text).is_ok() {
                    window_manager.clear_selection(window_id);
                }
            }
        }
        return true;
    }

    // Platform-aware paste shortcut
    let is_paste_shortcut = if is_macos() {
        key_event.code == KeyCode::Char('v') && key_event.modifiers.contains(KeyModifiers::SUPER)
    } else {
        key_event.code == KeyCode::Char('V')
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
            && key_event.modifiers.contains(KeyModifiers::SHIFT)
    };

    if is_paste_shortcut {
        if let FocusState::Window(window_id) = current_focus {
            if let Ok(text) = clipboard_manager.paste() {
                let _ = window_manager.paste_to_window(window_id, &text);
                window_manager.clear_selection(window_id);
            }
        }
        return true;
    }

    // Handle character keys based on focus
    match key_event.code {
        KeyCode::Esc => {
            handle_esc_key(app_state, current_focus, window_manager, backend);
            return true;
        }
        KeyCode::Char('q') => {
            handle_q_key(app_state, current_focus, window_manager, backend);
            return true;
        }
        KeyCode::Char('h') if current_focus == FocusState::Desktop => {
            show_help_window(app_state, backend);
            return true;
        }
        KeyCode::Char('l') if current_focus == FocusState::Desktop => {
            show_about_window(app_state, backend);
            return true;
        }
        KeyCode::Char('c') if current_focus == FocusState::Desktop => {
            app_state.active_calendar = Some(CalendarState::new());
            return true;
        }
        KeyCode::Char('s') if current_focus == FocusState::Desktop => {
            let (cols, rows) = backend.dimensions();
            app_state.active_config_window = Some(ConfigWindow::new(cols, rows));
            return true;
        }
        KeyCode::Char('t') if current_focus == FocusState::Desktop => {
            create_terminal_window(app_state, window_manager, backend, false);
            return true;
        }
        KeyCode::Char('T') if current_focus == FocusState::Desktop => {
            create_terminal_window(app_state, window_manager, backend, true);
            return true;
        }
        _ => {}
    }

    false
}

/// Forwards keyboard input to the focused terminal window
pub fn forward_to_terminal(key_event: KeyEvent, window_manager: &mut WindowManager) {
    match key_event.code {
        KeyCode::Char(c) => {
            // Check if CTRL is pressed (but not handled by specific shortcuts above)
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                // Convert to control character (Ctrl+A = 0x01, Ctrl+B = 0x02, etc.)
                if c.is_ascii_alphabetic() {
                    let control_char = (c.to_ascii_lowercase() as u8 - b'a' + 1) as char;
                    let _ = window_manager.send_to_focused(&control_char.to_string());
                } else {
                    // For non-alphabetic characters with Ctrl, send as-is
                    let _ = window_manager.send_char_to_focused(c);
                }
            } else {
                // Normal character without Ctrl
                let _ = window_manager.send_char_to_focused(c);
            }
        }
        KeyCode::Enter => {
            let _ = window_manager.send_to_focused("\r");
        }
        KeyCode::Backspace => {
            let _ = window_manager.send_to_focused("\x7f");
        }
        KeyCode::Tab => {
            let _ = window_manager.send_to_focused("\t");
        }
        KeyCode::Up => {
            let _ = window_manager.send_to_focused("\x1b[A");
        }
        KeyCode::Down => {
            let _ = window_manager.send_to_focused("\x1b[B");
        }
        KeyCode::Right => {
            let _ = window_manager.send_to_focused("\x1b[C");
        }
        KeyCode::Left => {
            let _ = window_manager.send_to_focused("\x1b[D");
        }
        KeyCode::Home => {
            let _ = window_manager.send_to_focused("\x1b[H");
        }
        KeyCode::End => {
            let _ = window_manager.send_to_focused("\x1b[F");
        }
        KeyCode::PageUp => {
            let _ = window_manager.send_to_focused("\x1b[5~");
        }
        KeyCode::PageDown => {
            let _ = window_manager.send_to_focused("\x1b[6~");
        }
        KeyCode::Delete => {
            let _ = window_manager.send_to_focused("\x1b[3~");
        }
        KeyCode::Insert => {
            let _ = window_manager.send_to_focused("\x1b[2~");
        }
        _ => {}
    }
}

// Helper functions

fn show_help_window(app_state: &mut AppState, backend: &Box<dyn RenderBackend>) {
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

fn show_about_window(app_state: &mut AppState, backend: &Box<dyn RenderBackend>) {
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
}

fn handle_esc_key(
    app_state: &mut AppState,
    current_focus: FocusState,
    window_manager: &mut WindowManager,
    backend: &Box<dyn RenderBackend>,
) {
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
            // If no windows, exit immediately
            app_state.should_exit = true;
        }
    } else {
        // Send ESC to terminal
        let _ = window_manager.send_to_focused("\x1b");
    }
}

fn handle_q_key(
    app_state: &mut AppState,
    current_focus: FocusState,
    window_manager: &mut WindowManager,
    backend: &Box<dyn RenderBackend>,
) {
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
            // If no windows, exit immediately
            app_state.should_exit = true;
        }
    } else {
        // Send 'q' to terminal
        let _ = window_manager.send_char_to_focused('q');
    }
}

fn create_terminal_window(
    app_state: &mut AppState,
    window_manager: &mut WindowManager,
    backend: &Box<dyn RenderBackend>,
    maximized: bool,
) {
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
        Ok(window_id) => {
            if maximized {
                window_manager.maximize_window(window_id, cols, rows);
            } else if app_state.auto_tiling_enabled {
                window_manager.auto_position_windows(cols, rows);
            }
        }
        Err(error_msg) => {
            app_state.active_error_dialog = Some(ErrorDialog::new(cols, rows, error_msg));
        }
    }
}

fn handle_save_session(
    app_state: &mut AppState,
    window_manager: &mut WindowManager,
    backend: &Box<dyn RenderBackend>,
    cli_args: &Cli,
    app_config: &AppConfig,
) {
    let (cols, rows) = backend.dimensions();

    if cli_args.no_save {
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
}
