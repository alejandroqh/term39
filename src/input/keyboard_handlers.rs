use crate::app::app_state::AppState;
use crate::app::cli::Cli;
use crate::app::config;
use crate::app::config_manager::AppConfig;
use crate::rendering::RenderBackend;
use crate::ui::config_window::ConfigWindow;
use crate::ui::error_dialog::ErrorDialog;
use crate::ui::info_window::InfoWindow;
use crate::ui::prompt::{Prompt, PromptAction, PromptButton, PromptType};
use crate::ui::ui_render::CalendarState;
use crate::utils::ClipboardManager;
use crate::window::manager::{FocusState, WindowManager};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

/// Double-backtick threshold in milliseconds
const DOUBLE_BACKTICK_THRESHOLD_MS: u64 = 300;

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
    backend: &dyn RenderBackend,
    app_config: &AppConfig,
    cli_args: &Cli,
) -> bool {
    // Handle F1 to show help (universal help key)
    if key_event.code == KeyCode::F(1)
        && matches!(current_focus, FocusState::Desktop | FocusState::Topbar)
    {
        show_help_window(app_state, backend);
        return true;
    }

    // Handle F2 for window cycling (more compatible than ALT+TAB)
    if key_event.code == KeyCode::F(2) {
        window_manager.cycle_to_next_window();
        return true;
    }

    // Handle F8 to toggle Window Mode (vim-like keyboard control)
    // Only toggle from Desktop/Topbar, or to exit Window Mode if already in it
    // When terminal is focused and NOT in Window Mode: forward F8 to terminal
    if key_event.code == KeyCode::F(8) {
        let in_window_mode = app_state.keyboard_mode.is_window_mode();
        if in_window_mode {
            // Already in Window Mode: F8 exits it
            app_state.keyboard_mode.exit_to_normal();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            return true;
        } else if matches!(current_focus, FocusState::Window(_)) {
            // Terminal is focused and NOT in Window Mode: forward F8 to terminal
            // Let it fall through to forward_to_terminal
            return false;
        } else {
            // Desktop/Topbar focused: F8 enters Window Mode
            app_state.keyboard_mode.toggle();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            return true;
        }
    }

    // Handle backtick (`) with double-press detection
    // Single backtick: toggle Window Mode (only from Desktop/Topbar focus, OR to EXIT from Window Mode)
    // Double backtick (within 300ms): send literal '`' to terminal
    // When terminal is focused and NOT in Window Mode: forward backtick directly to terminal
    // Skip if Alt/Option is pressed (Alt+` is used for window number overlay)
    let has_alt_modifier = key_event.modifiers.contains(KeyModifiers::ALT)
        || key_event.modifiers.contains(KeyModifiers::SUPER);
    if key_event.code == KeyCode::Char('`') && !has_alt_modifier {
        let now = Instant::now();
        let is_double_press = app_state
            .last_backtick_time
            .map(|t| now.duration_since(t) < Duration::from_millis(DOUBLE_BACKTICK_THRESHOLD_MS))
            .unwrap_or(false);

        // Check if we're currently in Window Mode
        let in_window_mode = app_state.keyboard_mode.is_window_mode();

        if is_double_press {
            // Double backtick: send literal '`' to focused terminal
            app_state.last_backtick_time = None;
            // Exit window mode if we're in it
            app_state.keyboard_mode.exit_to_normal();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            // Send the backtick character to the terminal
            let _ = window_manager.send_to_focused("`");
            return true;
        } else if in_window_mode {
            // Already in Window Mode: single backtick exits it
            app_state.last_backtick_time = Some(now);
            app_state.keyboard_mode.exit_to_normal();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            return true;
        } else {
            // Not in Window Mode: single backtick enters Window Mode
            // Works from Desktop, Topbar, or terminal Window focus
            // Users can double-tap backtick to send literal '`' to terminal
            app_state.last_backtick_time = Some(now);
            app_state.keyboard_mode.toggle();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            return true;
        }
    }

    // Platform-aware modifier detection for window selection
    let is_window_select_modifier = if is_macos() {
        key_event.modifiers.contains(KeyModifiers::ALT)
            || key_event.modifiers.contains(KeyModifiers::SUPER)
    } else {
        key_event.modifiers.contains(KeyModifiers::ALT)
    };

    // Handle F10 to toggle window number overlay
    // Shows numbers 1-9 on windows for quick Option+1-9 selection
    if key_event.code == KeyCode::F(10) {
        app_state.show_window_number_overlay = !app_state.show_window_number_overlay;
        return true;
    }

    // Handle ALT+TAB for window cycling (fallback, may be intercepted by OS)
    if key_event.code == KeyCode::Tab && key_event.modifiers.contains(KeyModifiers::ALT) {
        window_manager.cycle_to_next_window();
        return true;
    }

    // Handle Alt+1-9 (or Option+1-9 on macOS) for direct window selection
    // On macOS, Option+number produces special characters WITHOUT the ALT modifier flag
    // So we check for either: Alt modifier + digit, OR the special macOS characters directly
    // Window selection matches the number in the window title (e.g., "Terminal 3" = 3)
    if let KeyCode::Char(c) = key_event.code {
        // Map macOS Option+number special characters to their number equivalents
        // Option+1='¡', Option+2='™', Option+3='£', Option+4='¢', Option+5='∞',
        // Option+6='§', Option+7='¶', Option+8='•', Option+9='ª'
        let num: Option<u32> = match c {
            // Regular digits with Alt/Cmd modifier (Linux/Windows, or Cmd on macOS)
            '1'..='9' if is_window_select_modifier => c.to_digit(10),
            // macOS Option+number special characters (no modifier check needed)
            '¡' => Some(1), // Option+1 on macOS
            '™' => Some(2), // Option+2 on macOS
            '£' => Some(3), // Option+3 on macOS
            '¢' => Some(4), // Option+4 on macOS
            '∞' => Some(5), // Option+5 on macOS
            '§' => Some(6), // Option+6 on macOS
            '¶' => Some(7), // Option+7 on macOS
            '•' => Some(8), // Option+8 on macOS
            'ª' => Some(9), // Option+9 on macOS
            _ => None,
        };
        if let Some(num) = num {
            if (1..=9).contains(&num) {
                // Find window by its title number (e.g., Option+3 selects "Terminal 3")
                if let Some(window_id) = window_manager.find_window_by_title_number(num) {
                    window_manager.restore_and_focus_window(window_id);
                    // Hide overlay after selecting a window
                    app_state.show_window_number_overlay = false;
                }
                // Silently ignore if window with that number doesn't exist
                return true;
            }
        }
    }

    // Handle F3 to save session (more compatible than CTRL+S)
    if key_event.code == KeyCode::F(3) {
        handle_save_session(app_state, window_manager, backend, cli_args, app_config);
        return true;
    }

    // Handle F4 to clear the terminal (alternative to CTRL+L)
    if key_event.code == KeyCode::F(4) && matches!(current_focus, FocusState::Window(_)) {
        let _ = window_manager.send_to_focused("\x0c");
        return true;
    }

    // Handle CTRL+L to clear the terminal (like 'clear' command)
    if key_event.code == KeyCode::Char('l')
        && key_event.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(current_focus, FocusState::Window(_))
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

    // Handle F10 to exit application (classic DOS pattern)
    // Skip if --no-exit flag is set
    if key_event.code == KeyCode::F(10) && !cli_args.no_exit {
        if matches!(current_focus, FocusState::Desktop | FocusState::Topbar) {
            // Determine message based on window count
            let window_count = window_manager.window_count();
            let message = if window_count > 0 {
                format!(
                    "You have {} open terminal{}. Are you sure you want to exit?",
                    window_count,
                    if window_count == 1 { "" } else { "s" }
                )
            } else {
                "Are you sure you want to exit?".to_string()
            };

            // Get dimensions
            let (cols, rows) = backend.dimensions();

            // Create prompt with "Cancel" selected by default (index 0)
            app_state.active_prompt = Some(
                Prompt::new(
                    PromptType::Danger,
                    message,
                    vec![
                        PromptButton::new("Cancel".to_string(), PromptAction::Cancel, false),
                        PromptButton::new("Exit".to_string(), PromptAction::Confirm, true),
                    ],
                    cols,
                    rows,
                )
                .with_selection_indicators(true)
                .with_selected_button(0),
            ); // Select "Cancel"
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
            handle_esc_key(app_state, current_focus, window_manager, backend, cli_args);
            return true;
        }
        // Shift+Q to lock screen (before checking for lowercase 'q' exit)
        KeyCode::Char('Q')
            if key_event.modifiers.contains(KeyModifiers::SHIFT)
                && matches!(current_focus, FocusState::Desktop | FocusState::Topbar) =>
        {
            // Check if lockscreen is enabled in config
            if app_config.lockscreen_enabled && app_state.lockscreen.is_available() {
                app_state.lockscreen.lock();
            } else {
                // Show toast: "To lock the screen, configure in Settings"
                app_state.active_toast = Some(crate::ui::toast::Toast::new(
                    "To lock the screen, configure in Settings",
                ));
            }
            return true;
        }
        KeyCode::Char('q') => {
            handle_q_key(app_state, current_focus, window_manager, backend, cli_args);
            return true;
        }
        KeyCode::Char('h') | KeyCode::Char('?')
            if matches!(current_focus, FocusState::Desktop | FocusState::Topbar) =>
        {
            show_help_window(app_state, backend);
            return true;
        }
        KeyCode::Char('l') if matches!(current_focus, FocusState::Desktop | FocusState::Topbar) => {
            show_about_window(app_state, backend);
            return true;
        }
        KeyCode::Char('c') if matches!(current_focus, FocusState::Desktop | FocusState::Topbar) => {
            app_state.active_calendar = Some(CalendarState::new());
            return true;
        }
        KeyCode::Char('s') if matches!(current_focus, FocusState::Desktop | FocusState::Topbar) => {
            let (cols, rows) = backend.dimensions();
            app_state.active_config_window = Some(ConfigWindow::new(cols, rows));
            return true;
        }
        KeyCode::Char('t') if matches!(current_focus, FocusState::Desktop | FocusState::Topbar) => {
            // When auto-tiling is enabled and this is the first window, create maximized terminal
            let is_first_window = window_manager.window_count() == 0;
            let maximized = app_state.auto_tiling_enabled && is_first_window;
            create_terminal_window(
                app_state,
                window_manager,
                backend,
                maximized,
                app_config.tiling_gaps,
            );
            return true;
        }
        KeyCode::Char('T') if matches!(current_focus, FocusState::Desktop | FocusState::Topbar) => {
            create_terminal_window(
                app_state,
                window_manager,
                backend,
                true,
                app_config.tiling_gaps,
            );
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
            // Check for Shift+Enter - send CSI u sequence for modified Enter
            // This requires keyboard enhancement protocol support in the terminal
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                // CSI u encoding: ESC [ 13 ; 2 u (13 = Enter keycode, 2 = Shift modifier)
                let _ = window_manager.send_to_focused("\x1b[13;2u");
            } else {
                let _ = window_manager.send_to_focused("\r");
            }
        }
        KeyCode::Backspace => {
            let _ = window_manager.send_to_focused("\x7f");
        }
        KeyCode::Tab => {
            let _ = window_manager.send_to_focused("\t");
        }
        KeyCode::BackTab => {
            // Shift+Tab - send ESC [ Z (reverse tab / backtab)
            let _ = window_manager.send_to_focused("\x1b[Z");
        }
        KeyCode::Up => {
            // Check application cursor keys mode (DECCKM)
            let seq = if window_manager.get_focused_application_cursor_keys() {
                "\x1bOA" // Application mode
            } else {
                "\x1b[A" // Normal mode
            };
            let _ = window_manager.send_to_focused(seq);
        }
        KeyCode::Down => {
            let seq = if window_manager.get_focused_application_cursor_keys() {
                "\x1bOB"
            } else {
                "\x1b[B"
            };
            let _ = window_manager.send_to_focused(seq);
        }
        KeyCode::Right => {
            let seq = if window_manager.get_focused_application_cursor_keys() {
                "\x1bOC"
            } else {
                "\x1b[C"
            };
            let _ = window_manager.send_to_focused(seq);
        }
        KeyCode::Left => {
            let seq = if window_manager.get_focused_application_cursor_keys() {
                "\x1bOD"
            } else {
                "\x1b[D"
            };
            let _ = window_manager.send_to_focused(seq);
        }
        KeyCode::Home => {
            let seq = if window_manager.get_focused_application_cursor_keys() {
                "\x1bOH"
            } else {
                "\x1b[H"
            };
            let _ = window_manager.send_to_focused(seq);
        }
        KeyCode::End => {
            let seq = if window_manager.get_focused_application_cursor_keys() {
                "\x1bOF"
            } else {
                "\x1b[F"
            };
            let _ = window_manager.send_to_focused(seq);
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
        KeyCode::F(8) => {
            // F8 - send as escape sequence (CSI 19~)
            let _ = window_manager.send_to_focused("\x1b[19~");
        }
        // Note: Backtick ('`') is handled by KeyCode::Char(c) above
        _ => {}
    }
}

// Helper functions

pub fn show_help_window(app_state: &mut AppState, backend: &dyn RenderBackend) {
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
        {{Y}}'q'/ESC/F10{{W}} - Exit application (from desktop)\n\
        {{Y}}F1{{W}} or {{Y}}'?'{{W}} - Show this help screen\n\
        {{Y}}'l'{{W}}       - Show license and about information\n\
        {{Y}}'s'{{W}}       - Show settings/configuration window\n\
        {{Y}}'c'{{W}}       - Show calendar ({{Y}}\u{2190}\u{2192}{{W}} months, {{Y}}\u{2191}\u{2193}{{W}} years, {{Y}}t{{W}} today)\n\
        {{Y}}CTRL+Space{{W}} - Command launcher (Slight)\n\
        {{Y}}F12{{W}}      - Lock screen (global, works in terminal)\n\
        {{Y}}Shift+Q{{W}}  - Lock screen (from desktop/topbar)\n\
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
        {{Y}}Click bottom bar{{W}}    - Switch windows\n\
        {{Y}}Click [Exit]{{W}}        - Exit application",
        copy_key, paste_key
    );

    app_state.active_help_window = Some(InfoWindow::new(
        "Help".to_string(),
        &help_message,
        cols,
        rows,
    ));
}

pub fn show_about_window(app_state: &mut AppState, backend: &dyn RenderBackend) {
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
    backend: &dyn RenderBackend,
    cli_args: &Cli,
) {
    if matches!(current_focus, FocusState::Desktop | FocusState::Topbar) {
        // Skip exit prompt if --no-exit flag is set
        if cli_args.no_exit {
            return;
        }

        // Determine message based on window count
        let window_count = window_manager.window_count();
        let message = if window_count > 0 {
            format!(
                "You have {} open terminal{}. Are you sure you want to exit?",
                window_count,
                if window_count == 1 { "" } else { "s" }
            )
        } else {
            "Are you sure you want to exit?".to_string()
        };

        // Get dimensions
        let (cols, rows) = backend.dimensions();

        // Create prompt with "Cancel" selected by default (index 0)
        app_state.active_prompt = Some(
            Prompt::new(
                PromptType::Danger,
                message,
                vec![
                    PromptButton::new("Cancel".to_string(), PromptAction::Cancel, false),
                    PromptButton::new("Exit".to_string(), PromptAction::Confirm, true),
                ],
                cols,
                rows,
            )
            .with_selection_indicators(true)
            .with_selected_button(0),
        ); // Select "Cancel"
    } else {
        // Send ESC to terminal
        let _ = window_manager.send_to_focused("\x1b");
    }
}

fn handle_q_key(
    app_state: &mut AppState,
    current_focus: FocusState,
    window_manager: &mut WindowManager,
    backend: &dyn RenderBackend,
    cli_args: &Cli,
) {
    if matches!(current_focus, FocusState::Desktop | FocusState::Topbar) {
        // Skip exit prompt if --no-exit flag is set
        if cli_args.no_exit {
            return;
        }

        // Determine message based on window count
        let window_count = window_manager.window_count();
        let message = if window_count > 0 {
            format!(
                "You have {} open terminal{}. Are you sure you want to exit?",
                window_count,
                if window_count == 1 { "" } else { "s" }
            )
        } else {
            "Are you sure you want to exit?".to_string()
        };

        // Get dimensions
        let (cols, rows) = backend.dimensions();

        // Create prompt with "Cancel" selected by default (index 0)
        app_state.active_prompt = Some(
            Prompt::new(
                PromptType::Danger,
                message,
                vec![
                    PromptButton::new("Cancel".to_string(), PromptAction::Cancel, false),
                    PromptButton::new("Exit".to_string(), PromptAction::Confirm, true),
                ],
                cols,
                rows,
            )
            .with_selection_indicators(true)
            .with_selected_button(0),
        ); // Select "Cancel"
    } else {
        // Send 'q' to terminal
        let _ = window_manager.send_char_to_focused('q');
    }
}

pub fn create_terminal_window(
    app_state: &mut AppState,
    window_manager: &mut WindowManager,
    backend: &dyn RenderBackend,
    maximized: bool,
    tiling_gaps: bool,
) {
    let (cols, rows) = backend.dimensions();

    // Calculate dynamic window size based on screen dimensions
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
        Ok(window_id) => {
            if maximized {
                window_manager.maximize_window(window_id, cols, rows, tiling_gaps);
            } else if app_state.auto_tiling_enabled {
                window_manager.auto_position_windows(cols, rows, tiling_gaps);
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
    backend: &dyn RenderBackend,
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
