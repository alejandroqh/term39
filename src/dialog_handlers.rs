use crate::app_state::AppState;
use crate::command_history::CommandHistory;
use crate::command_indexer::CommandIndexer;
use crate::config_manager::AppConfig;
use crate::config_window::ConfigAction;
use crate::error_dialog::ErrorDialog;
use crate::prompt::PromptAction;
use crate::render_backend::RenderBackend;
use crate::window_manager::WindowManager;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles keyboard events when a prompt is active
/// Returns Some(should_exit) if prompt was handled, None otherwise
pub fn handle_prompt_keyboard(app_state: &mut AppState, key_event: KeyEvent) -> Option<bool> {
    if let Some(ref mut prompt) = app_state.active_prompt {
        match key_event.code {
            KeyCode::Tab => {
                // Tab or Shift+Tab to navigate buttons
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    prompt.select_previous_button();
                } else {
                    prompt.select_next_button();
                }
                return Some(false);
            }
            KeyCode::Left => {
                // Left arrow - previous button
                prompt.select_previous_button();
                return Some(false);
            }
            KeyCode::Right => {
                // Right arrow - next button
                prompt.select_next_button();
                return Some(false);
            }
            KeyCode::Enter => {
                // Enter - activate selected button
                if let Some(action) = prompt.get_selected_action() {
                    match action {
                        PromptAction::Confirm => {
                            // Exit confirmed
                            return Some(true);
                        }
                        PromptAction::Cancel => {
                            // Dismiss prompt
                            app_state.active_prompt = None;
                        }
                        _ => {}
                    }
                }
                return Some(false);
            }
            KeyCode::Esc => {
                // ESC dismisses the prompt
                app_state.active_prompt = None;
                return Some(false);
            }
            _ => {
                // Ignore other keys when prompt is active
                return Some(false);
            }
        }
    }
    None
}

/// Handles keyboard events when an error dialog is active
/// Returns true if event was handled
pub fn handle_error_dialog_keyboard(app_state: &mut AppState, key_event: KeyEvent) -> bool {
    if app_state.active_error_dialog.is_some() {
        match key_event.code {
            KeyCode::Enter | KeyCode::Esc => {
                // Dismiss error dialog
                app_state.active_error_dialog = None;
                return true;
            }
            _ => {
                // Ignore other keys when error dialog is active
                return true;
            }
        }
    }
    false
}

/// Handles keyboard events when Slight input is active
/// Returns true if event was handled
pub fn handle_slight_input_keyboard(
    app_state: &mut AppState,
    key_event: KeyEvent,
    _command_indexer: &CommandIndexer,
    command_history: &mut CommandHistory,
    window_manager: &mut WindowManager,
    backend: &dyn RenderBackend,
    tiling_gaps: bool,
) -> bool {
    if let Some(ref mut slight_input) = app_state.active_slight_input {
        match key_event.code {
            KeyCode::Char(c) => {
                slight_input.insert_char(c);
                return true;
            }
            KeyCode::Backspace => {
                slight_input.delete_char();
                return true;
            }
            KeyCode::Left => {
                slight_input.move_cursor_left();
                return true;
            }
            KeyCode::Right => {
                // If at end of input, accept inline suggestion
                // Otherwise, move cursor right
                if slight_input.cursor_position == slight_input.input_text.len() {
                    slight_input.accept_inline_suggestion();
                } else {
                    slight_input.move_cursor_right();
                }
                return true;
            }
            KeyCode::Up => {
                slight_input.previous_suggestion();
                return true;
            }
            KeyCode::Down => {
                slight_input.next_suggestion();
                return true;
            }
            KeyCode::Tab => {
                slight_input.accept_selected_suggestion();
                return true;
            }
            KeyCode::Home => {
                slight_input.move_cursor_home();
                return true;
            }
            KeyCode::End => {
                slight_input.move_cursor_end();
                return true;
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
                        Some(command),
                    ) {
                        Ok(_terminal_id) => {
                            // Auto-position all windows based on the snap pattern
                            if app_state.auto_tiling_enabled {
                                window_manager.auto_position_windows(cols, rows, tiling_gaps);
                            }
                        }
                        Err(error_msg) => {
                            // Show error dialog
                            app_state.active_error_dialog =
                                Some(ErrorDialog::new(cols, rows, error_msg));
                        }
                    }
                }
                return true;
            }
            KeyCode::Esc => {
                // ESC dismisses the Slight input
                app_state.active_slight_input = None;
                return true;
            }
            _ => {
                // Ignore other keys when Slight input is active
                return true;
            }
        }
    }
    false
}

/// Handles keyboard events when calendar is active
/// Returns true if event was handled
pub fn handle_calendar_keyboard(app_state: &mut AppState, key_event: KeyEvent) -> bool {
    if let Some(ref mut calendar) = app_state.active_calendar {
        match key_event.code {
            KeyCode::Char('<') | KeyCode::Char(',') | KeyCode::Left => {
                // Previous month
                calendar.previous_month();
                return true;
            }
            KeyCode::Char('>') | KeyCode::Char('.') | KeyCode::Right => {
                // Next month
                calendar.next_month();
                return true;
            }
            KeyCode::Up => {
                // Previous year
                calendar.previous_year();
                return true;
            }
            KeyCode::Down => {
                // Next year
                calendar.next_year();
                return true;
            }
            KeyCode::Char('t') | KeyCode::Home => {
                // Reset to today
                calendar.reset_to_today();
                return true;
            }
            KeyCode::Esc => {
                // ESC dismisses the calendar
                app_state.active_calendar = None;
                return true;
            }
            _ => {
                // Ignore other keys when calendar is active
                return true;
            }
        }
    }
    false
}

/// Handles keyboard events when help window is active
/// Returns true if event was handled
pub fn handle_help_window_keyboard(app_state: &mut AppState, key_event: KeyEvent) -> bool {
    if app_state.active_help_window.is_some() {
        match key_event.code {
            KeyCode::Esc => {
                // ESC dismisses the help window
                app_state.active_help_window = None;
                return true;
            }
            _ => {
                // Ignore other keys when help window is active
                return true;
            }
        }
    }
    false
}

/// Handles keyboard events when about window is active
/// Returns true if event was handled
pub fn handle_about_window_keyboard(app_state: &mut AppState, key_event: KeyEvent) -> bool {
    if app_state.active_about_window.is_some() {
        match key_event.code {
            KeyCode::Esc => {
                // ESC dismisses the about window
                app_state.active_about_window = None;
                return true;
            }
            _ => {
                // Ignore other keys when about window is active
                return true;
            }
        }
    }
    false
}

/// Handles keyboard events when Window Mode help window is active
/// Returns true if event was handled
pub fn handle_winmode_help_window_keyboard(app_state: &mut AppState, key_event: KeyEvent) -> bool {
    if app_state.active_winmode_help_window.is_some() {
        match key_event.code {
            KeyCode::Esc => {
                // ESC dismisses the Window Mode help window
                app_state.active_winmode_help_window = None;
                return true;
            }
            _ => {
                // Ignore other keys when Window Mode help window is active
                return true;
            }
        }
    }
    false
}

/// Handles keyboard events when config window is active
/// Returns Some(ConfigAction) if event was handled, None otherwise
pub fn handle_config_window_keyboard(
    app_state: &mut AppState,
    key_event: KeyEvent,
    config: &AppConfig,
) -> Option<ConfigAction> {
    if let Some(ref mut config_win) = app_state.active_config_window {
        match key_event.code {
            KeyCode::Esc => {
                // ESC dismisses the config window
                app_state.active_config_window = None;
                return Some(ConfigAction::Close);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                config_win.focus_previous(config);
                return Some(ConfigAction::None);
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    config_win.focus_previous(config);
                } else {
                    config_win.focus_next(config);
                }
                return Some(ConfigAction::None);
            }
            KeyCode::BackTab => {
                // Shift+Tab
                config_win.focus_previous(config);
                return Some(ConfigAction::None);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                // Activate focused option
                return Some(config_win.get_focused_action());
            }
            KeyCode::Left | KeyCode::Char('h') => {
                // For selector options, cycle backward
                let action = config_win.get_cycle_action(false);
                if action != ConfigAction::None {
                    return Some(action);
                }
                return Some(ConfigAction::None);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                // For selector options, cycle forward
                let action = config_win.get_cycle_action(true);
                if action != ConfigAction::None {
                    return Some(action);
                }
                return Some(ConfigAction::None);
            }
            _ => {
                // Consume other keys when config window is active
                return Some(ConfigAction::None);
            }
        }
    }
    None
}

/// Handles keyboard events when lockscreen is active
/// Returns true if event was handled
pub fn handle_lockscreen_keyboard(app_state: &mut AppState, key_event: KeyEvent) -> bool {
    if !app_state.lockscreen.is_active() {
        return false;
    }

    // Block all input during lockout (but still consume events)
    if app_state.lockscreen.lockout_remaining().is_some() {
        return true;
    }

    match key_event.code {
        KeyCode::Char(c) => {
            app_state.lockscreen.insert_char(c);
            true
        }
        KeyCode::Backspace => {
            app_state.lockscreen.delete_char();
            true
        }
        KeyCode::Delete => {
            // Delete at cursor (not implemented, just consume)
            true
        }
        KeyCode::Left => {
            app_state.lockscreen.move_cursor_left();
            true
        }
        KeyCode::Right => {
            app_state.lockscreen.move_cursor_right();
            true
        }
        KeyCode::Home => {
            app_state.lockscreen.move_cursor_home();
            true
        }
        KeyCode::End => {
            app_state.lockscreen.move_cursor_end();
            true
        }
        KeyCode::Tab => {
            app_state.lockscreen.toggle_focus();
            true
        }
        KeyCode::Enter => {
            app_state.lockscreen.attempt_login();
            true
        }
        KeyCode::Esc => {
            // ESC does nothing on lockscreen - can't dismiss it
            true
        }
        _ => {
            // Consume all other keys when lockscreen is active
            true
        }
    }
}
