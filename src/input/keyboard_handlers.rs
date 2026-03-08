use crate::app::app_state::AppState;
use crate::app::cli::Cli;
use crate::app::config;
use crate::app::config_manager::AppConfig;
use crate::input::keybinding_profile::{KeybindingProfile, matches_any};
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

/// Returns the escape sequence for a function key (F1-F12)
/// F1-F4 use SS3 format (ESC O), F5+ use CSI format (ESC [ n ~)
fn get_function_key_sequence(n: u8) -> Option<&'static str> {
    match n {
        1 => Some("\x1bOP"),    // F1
        2 => Some("\x1bOQ"),    // F2
        3 => Some("\x1bOR"),    // F3
        4 => Some("\x1bOS"),    // F4
        5 => Some("\x1b[15~"),  // F5
        6 => Some("\x1b[17~"),  // F6
        7 => Some("\x1b[18~"),  // F7
        8 => Some("\x1b[19~"),  // F8
        9 => Some("\x1b[20~"),  // F9
        10 => Some("\x1b[21~"), // F10
        11 => Some("\x1b[23~"), // F11
        12 => Some("\x1b[24~"), // F12
        _ => None,
    }
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
    app_config: &mut AppConfig,
    cli_args: &Cli,
    profile: &KeybindingProfile,
) -> bool {
    let code = key_event.code;
    let modifiers = key_event.modifiers;
    let on_desktop = matches!(current_focus, FocusState::Desktop | FocusState::Topbar);

    // Handle Shift+F1-F12 to send function key sequences to terminal
    // This allows users to send F-keys to terminal apps while F-keys are used for app shortcuts
    if let KeyCode::F(n) = code {
        if modifiers.contains(KeyModifiers::SHIFT) {
            if let FocusState::Window(_) = current_focus {
                if let Some(seq) = get_function_key_sequence(n) {
                    let _ = window_manager.send_to_focused(seq);
                    return true;
                }
            }
        }
    }

    // -- Direct-mode actions (Alt-modifier, work from any focus) --
    // These are checked BEFORE terminal forwarding so they intercept input
    if profile.has_direct_bindings() {
        if matches_any(&profile.direct_close_window, code, modifiers) {
            window_manager.request_close_focused_window();
            return true;
        }
        if matches_any(&profile.direct_new_terminal, code, modifiers) {
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
        if matches_any(&profile.direct_new_terminal_maximized, code, modifiers) {
            create_terminal_window(
                app_state,
                window_manager,
                backend,
                true,
                app_config.tiling_gaps,
            );
            return true;
        }
        if matches_any(&profile.direct_focus_left, code, modifiers) {
            window_manager.focus_window_in_direction(crate::window::mode_handlers::DIR_LEFT);
            return true;
        }
        if matches_any(&profile.direct_focus_down, code, modifiers) {
            window_manager.focus_window_in_direction(crate::window::mode_handlers::DIR_DOWN);
            return true;
        }
        if matches_any(&profile.direct_focus_up, code, modifiers) {
            window_manager.focus_window_in_direction(crate::window::mode_handlers::DIR_UP);
            return true;
        }
        if matches_any(&profile.direct_focus_right, code, modifiers) {
            window_manager.focus_window_in_direction(crate::window::mode_handlers::DIR_RIGHT);
            return true;
        }
        if matches_any(&profile.direct_snap_left, code, modifiers) {
            let (cols, rows) = backend.dimensions();
            let top_y: u16 = 1;
            let (x, y, w, h) = crate::input::keyboard_mode::SnapPosition::FullLeft
                .calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            return true;
        }
        if matches_any(&profile.direct_snap_down, code, modifiers) {
            let (cols, rows) = backend.dimensions();
            let top_y: u16 = 1;
            let (x, y, w, h) = crate::input::keyboard_mode::SnapPosition::FullBottom
                .calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            return true;
        }
        if matches_any(&profile.direct_snap_up, code, modifiers) {
            let (cols, rows) = backend.dimensions();
            let top_y: u16 = 1;
            let (x, y, w, h) = crate::input::keyboard_mode::SnapPosition::FullTop
                .calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            return true;
        }
        if matches_any(&profile.direct_snap_right, code, modifiers) {
            let (cols, rows) = backend.dimensions();
            let top_y: u16 = 1;
            let (x, y, w, h) = crate::input::keyboard_mode::SnapPosition::FullRight
                .calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            return true;
        }
        if matches_any(&profile.direct_maximize, code, modifiers) {
            let (cols, rows) = backend.dimensions();
            window_manager.toggle_focused_window_maximize(cols, rows, app_config.tiling_gaps);
            return true;
        }
        if matches_any(&profile.direct_toggle_auto_tiling, code, modifiers) {
            toggle_auto_tiling(app_state, app_config, window_manager, backend);
            return true;
        }
        if matches_any(&profile.direct_settings, code, modifiers) {
            let (cols, rows) = backend.dimensions();
            app_state.active_config_window = Some(ConfigWindow::new(cols, rows));
            return true;
        }
    }

    // Handle help (F1 always from desktop; profile bindings from desktop)
    if matches_any(&profile.help, code, modifiers) && on_desktop {
        show_help_window(app_state, backend, profile);
        return true;
    }

    // Handle window cycling
    if matches_any(&profile.cycle_window, code, modifiers) {
        window_manager.cycle_to_next_window();
        return true;
    }

    // Handle toggle Window Mode
    // F8 and backtick have special logic - handle them explicitly
    if code == KeyCode::F(8) && matches_any(&profile.toggle_window_mode, code, modifiers) {
        let in_window_mode = app_state.keyboard_mode.is_window_mode();
        if in_window_mode {
            app_state.keyboard_mode.exit_to_normal();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            return true;
        } else if matches!(current_focus, FocusState::Window(_)) {
            // Terminal is focused and NOT in Window Mode: forward F8 to terminal
            return false;
        } else {
            app_state.keyboard_mode.toggle();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            return true;
        }
    }

    // Handle backtick with double-press detection (only if backtick is in toggle_window_mode bindings)
    let has_backtick_binding = profile
        .toggle_window_mode
        .iter()
        .any(|b| b.code == KeyCode::Char('`'));
    let has_alt_modifier =
        modifiers.contains(KeyModifiers::ALT) || modifiers.contains(KeyModifiers::SUPER);
    if has_backtick_binding && code == KeyCode::Char('`') && !has_alt_modifier {
        let now = Instant::now();
        let is_double_press = app_state
            .last_backtick_time
            .map(|t| now.duration_since(t) < Duration::from_millis(DOUBLE_BACKTICK_THRESHOLD_MS))
            .unwrap_or(false);
        let in_window_mode = app_state.keyboard_mode.is_window_mode();

        if is_double_press {
            app_state.last_backtick_time = None;
            app_state.keyboard_mode.exit_to_normal();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            let _ = window_manager.send_to_focused("`");
            return true;
        } else if in_window_mode {
            app_state.last_backtick_time = Some(now);
            app_state.keyboard_mode.exit_to_normal();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            return true;
        } else {
            app_state.last_backtick_time = Some(now);
            app_state.keyboard_mode.toggle();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            return true;
        }
    }

    // Platform-aware modifier detection for window selection
    let is_window_select_modifier = if is_macos() {
        modifiers.contains(KeyModifiers::ALT) || modifiers.contains(KeyModifiers::SUPER)
    } else {
        modifiers.contains(KeyModifiers::ALT)
    };

    // Handle F10 to toggle window number overlay
    if code == KeyCode::F(10) && !matches_any(&profile.exit, code, modifiers) {
        app_state.show_window_number_overlay = !app_state.show_window_number_overlay;
        return true;
    }

    // Handle Alt+1-9 (or Option+1-9 on macOS) for direct window selection
    if let KeyCode::Char(c) = code {
        let num: Option<u32> = match c {
            '1'..='9' if is_window_select_modifier => c.to_digit(10),
            '¡' => Some(1),
            '™' => Some(2),
            '£' => Some(3),
            '¢' => Some(4),
            '∞' => Some(5),
            '§' => Some(6),
            '¶' => Some(7),
            '•' => Some(8),
            'ª' => Some(9),
            _ => None,
        };
        if let Some(num) = num {
            if (1..=9).contains(&num) {
                if let Some(window_id) = window_manager.find_window_by_title_number(num) {
                    window_manager.restore_and_focus_window(window_id);
                    app_state.show_window_number_overlay = false;
                }
                return true;
            }
        }
    }

    // Handle save session
    if matches_any(&profile.save_session, code, modifiers) {
        handle_save_session(app_state, window_manager, backend, cli_args, app_config);
        return true;
    }

    // Handle F4 to clear the terminal (alternative to CTRL+L)
    if code == KeyCode::F(4) && matches!(current_focus, FocusState::Window(_)) {
        let _ = window_manager.send_to_focused("\x0c");
        return true;
    }

    // Handle CTRL+L to clear the terminal (like 'clear' command)
    if code == KeyCode::Char('l')
        && modifiers.contains(KeyModifiers::CONTROL)
        && matches!(current_focus, FocusState::Window(_))
    {
        let _ = window_manager.send_to_focused("\x0c");
        return true;
    }

    // Handle CTRL+Space / Option+Space to open Slight input popup
    let is_launcher_shortcut = (code == KeyCode::Char(' ')
        && (modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::ALT)))
        || code == KeyCode::Char('\0')
        || code == KeyCode::Char('\u{00a0}');
    if is_launcher_shortcut {
        return true; // Signal to open Slight input (handled in main)
    }

    // Handle copy (F5)
    if matches_any(&profile.copy, code, modifiers) {
        if let FocusState::Window(window_id) = current_focus {
            if let Some(text) = window_manager.get_selected_text(window_id) {
                if clipboard_manager.copy(text).is_ok() {
                    window_manager.clear_selection(window_id);
                }
            }
        }
        return true;
    }

    // Handle paste (F6)
    if matches_any(&profile.paste, code, modifiers) {
        if let FocusState::Window(window_id) = current_focus {
            if let Ok(text) = clipboard_manager.paste() {
                let _ = window_manager.paste_to_window(window_id, &text);
                window_manager.clear_selection(window_id);
            }
        }
        return true;
    }

    // Handle new terminal (F7 always; bare key from desktop)
    if code == KeyCode::F(7) {
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

    // Platform-aware copy shortcut
    let is_copy_shortcut = if is_macos() {
        code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::SUPER)
    } else {
        code == KeyCode::Char('C')
            && modifiers.contains(KeyModifiers::CONTROL)
            && modifiers.contains(KeyModifiers::SHIFT)
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
        code == KeyCode::Char('v') && modifiers.contains(KeyModifiers::SUPER)
    } else {
        code == KeyCode::Char('V')
            && modifiers.contains(KeyModifiers::CONTROL)
            && modifiers.contains(KeyModifiers::SHIFT)
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

    // Handle exit (profile-based: ESC and q for term39; ESC and F10 for hyprland)
    if matches_any(&profile.exit, code, modifiers) && !cli_args.no_exit {
        // ESC: from desktop shows prompt, from window sends ESC
        if code == KeyCode::Esc {
            handle_esc_key(
                app_state,
                current_focus,
                window_manager,
                backend,
                cli_args,
                app_config,
            );
            return true;
        }
        // 'q' key
        if code == KeyCode::Char('q') && modifiers == KeyModifiers::NONE {
            handle_q_key(
                app_state,
                current_focus,
                window_manager,
                backend,
                cli_args,
                app_config,
            );
            return true;
        }
        // F10 key
        if code == KeyCode::F(10) && on_desktop {
            show_exit_prompt(app_state, window_manager, backend, app_config);
            return true;
        }
    }
    // Handle ESC even if it's not in exit bindings (still needed to send to terminal)
    if code == KeyCode::Esc && !matches_any(&profile.exit, code, modifiers) {
        if matches!(current_focus, FocusState::Window(_)) {
            let _ = window_manager.send_to_focused("\x1b");
        }
        return true;
    }

    // Handle lock screen
    if matches_any(&profile.lock_screen, code, modifiers) && on_desktop {
        if app_config.lockscreen_enabled && app_state.lockscreen.is_available() {
            app_state.lockscreen.lock();
        } else {
            app_state.active_toast = Some(crate::ui::toast::Toast::new(
                "To lock the screen, configure in Settings",
            ));
        }
        return true;
    }

    // Handle character keys that only work from Desktop/Topbar (bare keys without modifiers)
    if on_desktop {
        if matches_any(&profile.help, code, modifiers) {
            show_help_window(app_state, backend, profile);
            return true;
        }
        if matches_any(&profile.about, code, modifiers) && code != KeyCode::F(1) {
            show_about_window(app_state, backend);
            return true;
        }
        if matches_any(&profile.calendar, code, modifiers) {
            app_state.active_calendar = Some(CalendarState::new());
            return true;
        }
        if matches_any(&profile.settings, code, modifiers) {
            let (cols, rows) = backend.dimensions();
            app_state.active_config_window = Some(ConfigWindow::new(cols, rows));
            return true;
        }
        if matches_any(&profile.new_terminal, code, modifiers) {
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
        if matches_any(&profile.new_terminal_maximized, code, modifiers) {
            create_terminal_window(
                app_state,
                window_manager,
                backend,
                true,
                app_config.tiling_gaps,
            );
            return true;
        }
    }

    false
}

/// Forwards keyboard input to the focused terminal window
pub fn forward_to_terminal(key_event: KeyEvent, window_manager: &mut WindowManager) {
    match key_event.code {
        KeyCode::Char(c) => {
            // Windows: Handle AltGr combinations (reported as CTRL+ALT)
            // AltGr is used for special characters on international keyboards
            // (e.g., @ on German keyboards, € on many European keyboards)
            #[cfg(target_os = "windows")]
            {
                let is_altgr = key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && key_event.modifiers.contains(KeyModifiers::ALT);
                if is_altgr {
                    // AltGr combination - send the character directly
                    let _ = window_manager.send_char_to_focused(c);
                    return;
                }
            }

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
            // Send carriage return for both Enter and Shift+Enter
            let _ = window_manager.send_to_focused("\r");
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

pub fn show_help_window(
    app_state: &mut AppState,
    backend: &dyn RenderBackend,
    profile: &KeybindingProfile,
) {
    let (cols, rows) = backend.dimensions();

    // Platform-specific modifier key text
    let (copy_key, paste_key) = if is_macos() {
        ("CMD+C", "CMD+V")
    } else {
        ("CTRL+SHIFT+C", "CTRL+SHIFT+V")
    };

    let help_message = if profile.name == "hyprland" {
        format!(
            "{{C}}KEYBOARD SHORTCUTS (Hyprland){{W}}\n\
            \n\
            {{C}}DIRECT MODE (Alt-modifier, works from any focus){{W}}\n\
            \n\
            {{Y}}Alt+Enter{{W}}    - Create new terminal window\n\
            {{Y}}Alt+Shift+Enter{{W}} - Create new maximized terminal\n\
            {{Y}}Alt+Q{{W}}        - Close focused window\n\
            {{Y}}Alt+H/J/K/L{{W}} - Focus window left/down/up/right\n\
            {{Y}}Alt+Shift+H/J/K/L{{W}} - Snap to half screen\n\
            {{Y}}Alt+F{{W}}        - Toggle maximize\n\
            {{Y}}Alt+V{{W}}        - Toggle auto-tiling\n\
            {{Y}}Alt+S{{W}}        - Settings\n\
            \n\
            {{C}}DESKTOP SHORTCUTS (from desktop/topbar){{W}}\n\
            \n\
            {{Y}}ESC{{W}}/{{Y}}F10{{W}}    - Exit application\n\
            {{Y}}F1{{W}} or {{Y}}'?'{{W}}  - Show this help screen\n\
            {{Y}}'l'{{W}}          - Show about information\n\
            {{Y}}'s'{{W}}          - Settings window\n\
            {{Y}}'c'{{W}}          - Calendar\n\
            {{Y}}CTRL+Space{{W}}   - Command launcher (Slight)\n\
            {{Y}}F12{{W}}          - Lock screen (global)\n\
            {{Y}}Shift+Q{{W}}      - Lock screen\n\
            \n\
            {{C}}WINDOW & SESSION{{W}}\n\
            \n\
            {{Y}}Alt+TAB{{W}}/{{Y}}F2{{W}} - Switch between windows\n\
            {{Y}}F3{{W}}             - Save session\n\
            {{Y}}F4{{W}}/{{Y}}CTRL+L{{W}}  - Clear terminal\n\
            {{Y}}F8{{W}}             - Toggle Window Mode\n\
            {{Y}}Shift+F1-F12{{W}}   - Send F-key to terminal\n\
            \n\
            {{C}}COPY & PASTE{{W}}\n\
            \n\
            {{Y}}{}{{W}} or {{Y}}F5{{W}} - Copy selected text\n\
            {{Y}}{}{{W}} or {{Y}}F6{{W}} - Paste from clipboard\n\
            \n\
            {{C}}MOUSE CONTROLS{{W}}\n\
            \n\
            {{Y}}Click title bar{{W}}     - Drag window\n\
            {{Y}}CTRL+Drag{{W}}          - Drag without snap\n\
            {{Y}}Click [X]{{W}}           - Close window\n\
            {{Y}}Drag border{{W}}         - Resize window\n\
            {{Y}}Click window{{W}}        - Focus window",
            copy_key, paste_key
        )
    } else {
        format!(
            "{{C}}KEYBOARD SHORTCUTS (Term39){{W}}\n\
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
            {{Y}}F7{{W}}              - Create new terminal window\n\
            {{Y}}Shift+F1-F12{{W}}    - Send F-key to terminal\n\
            \n\
            {{C}}COPY & PASTE{{W}}\n\
            \n\
            {{Y}}{}{{W}} or {{Y}}F5{{W}} - Copy selected text\n\
            {{Y}}{}{{W}} or {{Y}}F6{{W}} - Paste from clipboard\n\
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
        )
    };

    app_state.active_help_window = Some(InfoWindow::new(
        "Help".to_string(),
        &help_message,
        cols,
        rows,
    ));
}

/// Helper to show exit prompt
fn show_exit_prompt(
    app_state: &mut AppState,
    window_manager: &WindowManager,
    backend: &dyn RenderBackend,
    app_config: &AppConfig,
) {
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
    let (cols, rows) = backend.dimensions();
    let mut buttons = vec![
        PromptButton::new("Cancel".to_string(), PromptAction::Cancel, false),
        PromptButton::new("Exit".to_string(), PromptAction::Confirm, true),
    ];
    // Add "Exit & Kill Daemon" option when persist mode is active and enabled
    #[cfg(unix)]
    if app_config.persist_enabled && window_manager.has_persist_client() {
        buttons.push(PromptButton::new(
            "Exit & Kill Daemon".to_string(),
            PromptAction::Custom(1),
            true,
        ));
    }
    #[cfg(not(unix))]
    let _ = app_config;
    app_state.active_prompt = Some(
        Prompt::new(PromptType::Danger, message, buttons, cols, rows)
            .with_selection_indicators(true)
            .with_selected_button(0),
    );
}

/// Helper to toggle auto-tiling (shared between desktop and direct mode)
fn toggle_auto_tiling(
    app_state: &mut AppState,
    app_config: &mut AppConfig,
    window_manager: &mut WindowManager,
    backend: &dyn RenderBackend,
) {
    app_config.toggle_auto_tiling_on_startup();
    app_state.auto_tiling_enabled = app_config.auto_tiling_on_startup;
    let auto_tiling_text = if app_state.auto_tiling_enabled {
        "█ on] Auto Tiling"
    } else {
        "off ░] Auto Tiling"
    };
    let (cols, rows) = backend.dimensions();
    app_state.auto_tiling_button =
        crate::ui::button::Button::new(1, rows - 1, auto_tiling_text.to_string());
    if app_state.auto_tiling_enabled {
        window_manager.auto_position_windows(cols, rows, app_config.tiling_gaps);
    }
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
    app_config: &AppConfig,
) {
    if matches!(current_focus, FocusState::Desktop | FocusState::Topbar) {
        // Skip exit prompt if --no-exit flag is set
        if cli_args.no_exit {
            return;
        }

        show_exit_prompt(app_state, window_manager, backend, app_config);
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
    app_config: &AppConfig,
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
        let mut buttons = vec![
            PromptButton::new("Cancel".to_string(), PromptAction::Cancel, false),
            PromptButton::new("Exit".to_string(), PromptAction::Confirm, true),
        ];
        // Add "Exit & Kill Daemon" option when persist mode is active and enabled
        #[cfg(unix)]
        if app_config.persist_enabled && window_manager.has_persist_client() {
            buttons.push(PromptButton::new(
                "Exit & Kill Daemon".to_string(),
                PromptAction::Custom(1),
                true,
            ));
        }
        #[cfg(not(unix))]
        let _ = app_config;
        app_state.active_prompt = Some(
            Prompt::new(PromptType::Danger, message, buttons, cols, rows)
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
