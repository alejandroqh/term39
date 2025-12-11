//! Mouse event handlers extracted from main.rs
//! Handles button hover states, modal dialogs, top bar buttons, menus, and text selection.

use crate::app::app_state::AppState;
use crate::app::config_manager::AppConfig;
use crate::lockscreen::PinSetupState;
use crate::rendering::{Charset, Theme};
use crate::term_emu::SelectionType;
use crate::ui::button::ButtonState;
use crate::ui::config_action_handler::{apply_config_result, process_config_action};
use crate::ui::config_window::ConfigAction;
use crate::ui::context_menu::MenuAction;
use crate::ui::error_dialog::ErrorDialog;
use crate::ui::prompt::PromptAction;
use crate::ui::widgets::{WidgetClickResult, WidgetContext};
use crate::utils::ClipboardManager;
use crate::window::manager::{FocusState, WindowManager};
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
use crossterm::event::Event;
use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::time::Instant;

// ============================================================================
// Framebuffer Mouse Event Mapping
// ============================================================================

/// Maps a framebuffer button event to a crossterm Event.
/// event_type: 0=Down, 1=Up, 2=Drag
/// button_id: 0=Left, 1=Right, 2=Middle
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub fn map_fb_button_event(
    event_type: u8,
    button_id: u8,
    col: u16,
    row: u16,
    swap_buttons: bool,
) -> Event {
    // Map button ID to MouseButton, applying swap if configured
    let button = match button_id {
        0 => {
            if swap_buttons {
                MouseButton::Right
            } else {
                MouseButton::Left
            }
        }
        1 => {
            if swap_buttons {
                MouseButton::Left
            } else {
                MouseButton::Right
            }
        }
        2 => MouseButton::Middle,
        _ => MouseButton::Left, // Fallback
    };

    // Map event type to MouseEventKind
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

    Event::Mouse(mouse_event)
}

/// Maps a framebuffer scroll event to a crossterm Event.
/// scroll_direction: 0=Up, 1=Down
#[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
pub fn map_fb_scroll_event(scroll_direction: u8, col: u16, row: u16) -> Event {
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

    Event::Mouse(mouse_event)
}

// ============================================================================
// Button Hover State Management
// ============================================================================

/// Updates hover states for all top and bottom bar buttons based on mouse position.
/// Uses early exit optimization when mouse is not on bar areas.
#[allow(clippy::too_many_arguments)]
pub fn update_bar_button_hover_states(
    app_state: &mut AppState,
    mouse_col: u16,
    mouse_row: u16,
    cols: u16,
    rows: u16,
    show_date_in_clock: bool,
    has_clipboard_content: bool,
    has_selection: bool,
    focus: FocusState,
    charset: &crate::rendering::Charset,
) {
    let bar_y = rows.saturating_sub(1);

    // Fast path: if mouse is not on top or bottom bar, reset all buttons
    if mouse_row != 0 && mouse_row != bar_y {
        // Reset TopBar widget hover states
        let ctx = WidgetContext::new(
            cols,
            rows,
            focus,
            has_clipboard_content,
            has_selection,
            show_date_in_clock,
            charset,
        );
        app_state.top_bar.update_hover(mouse_col, mouse_row, &ctx);
        app_state.auto_tiling_button.set_state(ButtonState::Normal);

        // Legacy: keep battery_hovered in sync
        app_state.battery_hovered = app_state.top_bar.is_battery_hovered();
        return;
    }

    if mouse_row == 0 {
        // Top bar - use widget-based hover
        let ctx = WidgetContext::new(
            cols,
            rows,
            focus,
            has_clipboard_content,
            has_selection,
            show_date_in_clock,
            charset,
        );
        app_state.top_bar.update_hover(mouse_col, mouse_row, &ctx);

        // Legacy: keep battery_hovered in sync
        app_state.battery_hovered = app_state.top_bar.is_battery_hovered();

        // Reset bottom bar button when on top bar
        app_state.auto_tiling_button.set_state(ButtonState::Normal);
    } else {
        // Bottom bar - check bottom bar button only
        let button_start_x = 1u16;
        let button_text_width = app_state.auto_tiling_button.label.len() as u16 + 3;
        let button_end_x = button_start_x + button_text_width;

        if mouse_col >= button_start_x && mouse_col < button_end_x {
            app_state.auto_tiling_button.set_state(ButtonState::Hovered);
        } else {
            app_state.auto_tiling_button.set_state(ButtonState::Normal);
        }

        // Reset TopBar widget hover states when on bottom bar
        let ctx = WidgetContext::new(
            cols,
            rows,
            focus,
            has_clipboard_content,
            has_selection,
            show_date_in_clock,
            charset,
        );
        app_state.top_bar.update_hover(mouse_col, mouse_row, &ctx);
        app_state.battery_hovered = false;
    }
}

// ============================================================================
// Modal Mouse Handlers
// ============================================================================

/// Result of handling a modal mouse event.
pub enum ModalMouseResult {
    /// Event was not handled by this modal
    NotHandled,
    /// Event was handled, continue processing
    Handled,
    /// Event triggered exit (e.g., confirm on exit prompt)
    Exit,
}

/// Handles mouse events on the active prompt dialog.
/// Returns ModalMouseResult indicating how the event was handled.
pub fn handle_prompt_mouse(
    app_state: &mut AppState,
    mouse_event: &MouseEvent,
    charset: &Charset,
) -> ModalMouseResult {
    if let Some(ref prompt) = app_state.active_prompt {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
            if let Some(action) = prompt.handle_click(mouse_event.column, mouse_event.row, charset)
            {
                match action {
                    PromptAction::Confirm => {
                        return ModalMouseResult::Exit;
                    }
                    PromptAction::Cancel => {
                        app_state.active_prompt = None;
                        return ModalMouseResult::Handled;
                    }
                    _ => {
                        return ModalMouseResult::Handled;
                    }
                }
            } else if prompt.contains_point(mouse_event.column, mouse_event.row) {
                // Click inside prompt but not on a button - consume the event
                return ModalMouseResult::Handled;
            }
        }
    }
    ModalMouseResult::NotHandled
}

/// Handles mouse events on the PIN setup dialog.
/// Returns true if the event was handled.
pub fn handle_pin_setup_mouse(
    app_state: &mut AppState,
    app_config: &mut AppConfig,
    mouse_event: &MouseEvent,
    cols: u16,
    rows: u16,
    charset: &Charset,
) -> bool {
    if let Some(ref mut pin_setup) = app_state.active_pin_setup {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
            if pin_setup.handle_click(mouse_event.column, mouse_event.row, cols, rows, charset) {
                // Button was clicked, check state
                match pin_setup.state().clone() {
                    PinSetupState::Complete { hash, salt } => {
                        app_config.set_pin(hash, salt);
                        app_state.update_lockscreen_auth(app_config);
                        app_state.active_pin_setup = None;
                    }
                    PinSetupState::Cancelled => {
                        app_state.active_pin_setup = None;
                    }
                    _ => {}
                }
                return true;
            } else if pin_setup.contains_point(mouse_event.column, mouse_event.row, cols, rows) {
                // Click inside dialog but not on a button - consume the event
                return true;
            }
        }
    }
    false
}

/// Handles mouse events on the error dialog.
/// Returns true if the event was handled.
pub fn handle_error_dialog_mouse(app_state: &mut AppState, mouse_event: &MouseEvent) -> bool {
    if let Some(ref error_dialog) = app_state.active_error_dialog {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
            && error_dialog.is_ok_button_clicked(mouse_event.column, mouse_event.row)
        {
            app_state.active_error_dialog = None;
            return true;
        }
    }
    false
}

/// Handles mouse events on the config window.
/// Returns true if the event was handled.
pub fn handle_config_window_mouse(
    app_state: &mut AppState,
    app_config: &mut AppConfig,
    mouse_event: &MouseEvent,
    rows: u16,
    charset: &mut Charset,
    theme: &mut Theme,
) -> bool {
    if let Some(ref config_win) = app_state.active_config_window {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
            let action = config_win.handle_click(mouse_event.column, mouse_event.row, app_config);
            match action {
                ConfigAction::Close => {
                    app_state.active_config_window = None;
                    return true;
                }
                ConfigAction::None => {
                    // Check if click is inside config window
                    if config_win.contains_point(mouse_event.column, mouse_event.row) {
                        // Click inside config window but not on an option - consume the event
                        return true;
                    }
                }
                _ => {
                    // Process config action using shared handler
                    let result = process_config_action(action, app_state, app_config, rows);
                    apply_config_result(&result, charset, theme);
                    return true;
                }
            }
        }
    }
    false
}

/// Handles mouse events on the help window.
/// Returns true if the event was handled.
pub fn handle_help_window_mouse(app_state: &mut AppState, mouse_event: &MouseEvent) -> bool {
    if let Some(ref help_window) = app_state.active_help_window {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
            && help_window.is_close_button_click(mouse_event.column, mouse_event.row)
        {
            app_state.active_help_window = None;
            return true;
        }
        // Consume clicks inside the window
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
            && help_window.contains_point(mouse_event.column, mouse_event.row)
        {
            return true;
        }
    }
    false
}

/// Handles mouse events on the about window.
/// Returns true if the event was handled.
pub fn handle_about_window_mouse(app_state: &mut AppState, mouse_event: &MouseEvent) -> bool {
    if let Some(ref about_window) = app_state.active_about_window {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
            && about_window.is_close_button_click(mouse_event.column, mouse_event.row)
        {
            app_state.active_about_window = None;
            return true;
        }
        // Consume clicks inside the window
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
            && about_window.contains_point(mouse_event.column, mouse_event.row)
        {
            return true;
        }
    }
    false
}

/// Handles mouse events on the window mode help window.
/// Returns true if the event was handled.
pub fn handle_winmode_help_window_mouse(
    app_state: &mut AppState,
    mouse_event: &MouseEvent,
) -> bool {
    if let Some(ref winmode_help_window) = app_state.active_winmode_help_window {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
            && winmode_help_window.is_close_button_click(mouse_event.column, mouse_event.row)
        {
            app_state.active_winmode_help_window = None;
            return true;
        }
        // Consume clicks inside the window
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
            && winmode_help_window.contains_point(mouse_event.column, mouse_event.row)
        {
            return true;
        }
    }
    false
}

/// Handles mouse events on the calendar.
/// Returns true if the event was handled.
pub fn handle_calendar_mouse(
    app_state: &mut AppState,
    mouse_event: &MouseEvent,
    cols: u16,
    rows: u16,
) -> bool {
    use crate::ui::ui_render::is_calendar_close_button_click;

    if app_state.active_calendar.is_some() {
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
            && is_calendar_close_button_click(mouse_event.column, mouse_event.row, cols, rows)
        {
            app_state.active_calendar = None;
            return true;
        }
        // Consume clicks inside the calendar area
        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
            // Calendar dimensions (must match render_calendar)
            let width = 42u16;
            let height = 18u16;
            let x = (cols.saturating_sub(width)) / 2;
            let y = (rows.saturating_sub(height)) / 2;
            if mouse_event.column >= x
                && mouse_event.column < x + width
                && mouse_event.row >= y
                && mouse_event.row < y + height
            {
                return true;
            }
        }
    }
    false
}

// ============================================================================
// Top Bar Button Click Handlers
// ============================================================================

/// Result of handling a top bar button click.
pub enum TopBarClickResult {
    /// Click was not on any top bar button
    NotHandled,
    /// Click was handled
    Handled,
}

/// Handles clicks on top bar buttons (New Terminal, Copy, Paste, etc.).
/// Returns TopBarClickResult indicating what action was taken.
#[allow(clippy::too_many_arguments)]
pub fn handle_topbar_click(
    app_state: &mut AppState,
    window_manager: &mut WindowManager,
    clipboard_manager: &mut ClipboardManager,
    mouse_event: &MouseEvent,
    cols: u16,
    rows: u16,
    tiling_gaps: bool,
    _no_exit: bool,
    _show_date_in_clock: bool,
) -> TopBarClickResult {
    // Only handle left clicks on row 0
    if mouse_event.kind != MouseEventKind::Down(MouseButton::Left) || mouse_event.row != 0 {
        return TopBarClickResult::NotHandled;
    }

    // Use the widget-based click handling
    let widget_result = app_state
        .top_bar
        .handle_click(mouse_event.column, mouse_event.row);

    // Process the widget result
    match widget_result {
        WidgetClickResult::NotHandled => TopBarClickResult::NotHandled,

        WidgetClickResult::CreateTerminal => {
            // Check if this will be the first window
            let is_first_window = window_manager.window_count() == 0;

            let (width, height) = WindowManager::calculate_window_size(cols, rows);
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
                    // When auto-tiling is enabled and this is the first window, maximize it
                    if app_state.auto_tiling_enabled && is_first_window {
                        window_manager.maximize_window(window_id, cols, rows, tiling_gaps);
                    } else if app_state.auto_tiling_enabled {
                        // For subsequent windows, use auto-positioning
                        window_manager.auto_position_windows(cols, rows, tiling_gaps);
                    }
                }
                Err(error_msg) => {
                    app_state.active_error_dialog = Some(ErrorDialog::new(cols, rows, error_msg));
                }
            }
            TopBarClickResult::Handled
        }

        WidgetClickResult::OpenCalendar => {
            app_state.active_calendar = Some(crate::ui::ui_render::CalendarState::new());
            TopBarClickResult::Handled
        }

        WidgetClickResult::ToggleCommandCenter => {
            // Toggle the Command Center dropdown menu
            if app_state.command_center_menu.visible {
                app_state.command_center_menu.hide();
                app_state.top_bar.close_command_center();
            } else {
                // Update enabled states based on current context
                let has_selection = window_manager.focused_window_has_selection();
                let has_clipboard_content = clipboard_manager.has_content();

                app_state
                    .command_center_menu
                    .set_item_enabled(MenuAction::CopySelection, has_selection);
                app_state
                    .command_center_menu
                    .set_item_enabled(MenuAction::PasteClipboard, has_clipboard_content);
                app_state
                    .command_center_menu
                    .set_item_enabled(MenuAction::ClearClipboard, has_clipboard_content);

                let button_x = app_state.top_bar.get_command_center_x();
                // Use show_bounded to auto-adjust position if menu would overflow
                app_state
                    .command_center_menu
                    .show_bounded(button_x, 1, cols);
            }
            TopBarClickResult::Handled
        }
    }
}

/// Handles click on the auto-tiling toggle button in the bottom bar.
/// Returns true if the event was handled.
pub fn handle_auto_tiling_click(
    app_state: &mut AppState,
    app_config: &mut AppConfig,
    mouse_event: &MouseEvent,
    rows: u16,
) -> bool {
    if mouse_event.kind != MouseEventKind::Down(MouseButton::Left) {
        return false;
    }

    let bar_y = rows - 1;
    let button_start_x = 1u16;
    let button_text_width = app_state.auto_tiling_button.label.len() as u16 + 3;
    let button_end_x = button_start_x + button_text_width;

    if mouse_event.row == bar_y
        && mouse_event.column >= button_start_x
        && mouse_event.column < button_end_x
    {
        app_state.auto_tiling_button.set_state(ButtonState::Pressed);

        // Toggle the auto-tiling state and save to config
        app_config.toggle_auto_tiling_on_startup();
        app_state.auto_tiling_enabled = app_config.auto_tiling_on_startup;

        // Update button label to reflect new state
        let new_label = if app_state.auto_tiling_enabled {
            "\u{2588} on] Auto Tiling".to_string()
        } else {
            "off \u{2591}] Auto Tiling".to_string()
        };
        app_state.auto_tiling_button = crate::ui::button::Button::new(1, bar_y, new_label);

        return true;
    }

    false
}

// ============================================================================
// Context Menu and Taskbar Menu Handlers
// ============================================================================

/// Handles context menu mouse interactions (show/hide, item selection).
/// Returns true if the event was handled.
pub fn handle_context_menu_mouse(
    app_state: &mut AppState,
    window_manager: &mut WindowManager,
    clipboard_manager: &mut ClipboardManager,
    mouse_event: &MouseEvent,
) -> bool {
    if !app_state.context_menu.visible {
        return false;
    }

    if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
        if app_state
            .context_menu
            .contains_point(mouse_event.column, mouse_event.row)
        {
            // Update selection to clicked item before getting action
            app_state
                .context_menu
                .update_selection_from_mouse(mouse_event.column, mouse_event.row);

            if let Some(action) = app_state.context_menu.get_selected_action() {
                if let FocusState::Window(window_id) = window_manager.get_focus() {
                    match action {
                        MenuAction::Copy => {
                            if let Some(text) = window_manager.get_selected_text(window_id) {
                                let _ = clipboard_manager.copy(text);
                                window_manager.clear_selection(window_id);
                            }
                        }
                        MenuAction::Paste => {
                            if let Ok(text) = clipboard_manager.paste() {
                                let _ = window_manager.paste_to_window(window_id, &text);
                            }
                        }
                        MenuAction::SelectAll => {
                            window_manager.select_all(window_id);
                        }
                        MenuAction::Close
                        | MenuAction::Restore
                        | MenuAction::Maximize
                        | MenuAction::CloseWindow
                        | MenuAction::Exit
                        | MenuAction::CopySelection
                        | MenuAction::PasteClipboard
                        | MenuAction::ClearClipboard
                        | MenuAction::Settings
                        | MenuAction::Help
                        | MenuAction::About => {}
                    }
                }
            }
            app_state.context_menu.hide();
            return true;
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

    false
}

/// Handles taskbar menu mouse interactions (restore/maximize/close window).
/// Returns true if the event was handled.
pub fn handle_taskbar_menu_mouse(
    app_state: &mut AppState,
    window_manager: &mut WindowManager,
    mouse_event: &MouseEvent,
    cols: u16,
    rows: u16,
    tiling_gaps: bool,
) -> bool {
    if !app_state.taskbar_menu.visible {
        return false;
    }

    if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
        if app_state
            .taskbar_menu
            .contains_point(mouse_event.column, mouse_event.row)
        {
            // Update selection to clicked item before getting action
            app_state
                .taskbar_menu
                .update_selection_from_mouse(mouse_event.column, mouse_event.row);

            if let Some(action) = app_state.taskbar_menu.get_selected_action() {
                if let Some(window_id) = app_state.taskbar_menu_window_id {
                    match action {
                        MenuAction::Restore => {
                            window_manager.restore_and_focus_window(window_id);
                        }
                        MenuAction::Maximize => {
                            window_manager.maximize_window(
                                window_id,
                                cols,
                                rows - 2, // Account for top and bottom bars
                                tiling_gaps,
                            );
                        }
                        MenuAction::CloseWindow => {
                            window_manager.close_window(window_id);
                        }
                        MenuAction::Copy
                        | MenuAction::Paste
                        | MenuAction::SelectAll
                        | MenuAction::Close
                        | MenuAction::Exit
                        | MenuAction::CopySelection
                        | MenuAction::PasteClipboard
                        | MenuAction::ClearClipboard
                        | MenuAction::Settings
                        | MenuAction::Help
                        | MenuAction::About => {}
                    }
                }
            }
            app_state.taskbar_menu.hide();
            app_state.taskbar_menu_window_id = None;
            return true;
        } else {
            // Clicked outside menu - hide it
            app_state.taskbar_menu.hide();
            app_state.taskbar_menu_window_id = None;
        }
    } else if mouse_event.kind == MouseEventKind::Moved {
        // Update menu selection on hover
        app_state
            .taskbar_menu
            .update_selection_from_mouse(mouse_event.column, mouse_event.row);
    }

    false
}

/// Shows the context menu at the specified position (for right-click inside windows).
/// Returns true if the menu was shown.
pub fn show_context_menu(
    app_state: &mut AppState,
    window_manager: &WindowManager,
    mouse_event: &MouseEvent,
) -> bool {
    if mouse_event.kind == MouseEventKind::Down(MouseButton::Right) {
        if let FocusState::Window(_) = window_manager.get_focus() {
            app_state
                .context_menu
                .show(mouse_event.column, mouse_event.row);
            return true;
        }
    }
    false
}

/// Result of handling a command center menu mouse event.
pub enum CommandCenterMenuResult {
    /// Event was not handled
    NotHandled,
    /// Event was handled
    Handled,
    /// Settings was requested - show config window
    ShowSettings,
    /// Help was requested - show help window
    ShowHelp,
    /// About was requested - show about window
    ShowAbout,
    /// Exit was requested - show confirmation prompt
    ShowExitPrompt,
}

/// Handles Command Center menu mouse interactions.
/// Returns CommandCenterMenuResult indicating what action was taken.
pub fn handle_command_center_menu_mouse(
    app_state: &mut AppState,
    window_manager: &mut WindowManager,
    clipboard_manager: &mut ClipboardManager,
    mouse_event: &MouseEvent,
) -> CommandCenterMenuResult {
    if !app_state.command_center_menu.visible {
        return CommandCenterMenuResult::NotHandled;
    }

    if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
        if app_state
            .command_center_menu
            .contains_point(mouse_event.column, mouse_event.row)
        {
            // Update selection to clicked item before getting action
            app_state
                .command_center_menu
                .update_selection_from_mouse(mouse_event.column, mouse_event.row);

            let mut result = CommandCenterMenuResult::Handled;

            if let Some(action) = app_state.command_center_menu.get_selected_action() {
                match action {
                    MenuAction::Exit => {
                        // Return ShowExitPrompt to trigger confirmation dialog
                        result = CommandCenterMenuResult::ShowExitPrompt;
                    }
                    MenuAction::Settings => {
                        // Return ShowSettings to trigger config window
                        result = CommandCenterMenuResult::ShowSettings;
                    }
                    MenuAction::Help => {
                        // Return ShowHelp to trigger help window
                        result = CommandCenterMenuResult::ShowHelp;
                    }
                    MenuAction::About => {
                        // Return ShowAbout to trigger about window
                        result = CommandCenterMenuResult::ShowAbout;
                    }
                    MenuAction::CopySelection => {
                        if let FocusState::Window(window_id) = window_manager.get_focus() {
                            if let Some(text) = window_manager.get_selected_text(window_id) {
                                let _ = clipboard_manager.copy(text);
                                window_manager.clear_selection(window_id);
                            }
                        }
                    }
                    MenuAction::PasteClipboard => {
                        if let FocusState::Window(window_id) = window_manager.get_focus() {
                            if let Ok(text) = clipboard_manager.paste() {
                                let _ = window_manager.paste_to_window(window_id, &text);
                            }
                        }
                    }
                    MenuAction::ClearClipboard => {
                        clipboard_manager.clear();
                    }
                    _ => {}
                }
            }
            app_state.command_center_menu.hide();
            app_state.top_bar.close_command_center();
            return result;
        } else {
            // Clicked outside menu - hide it
            app_state.command_center_menu.hide();
            app_state.top_bar.close_command_center();
        }
    } else if mouse_event.kind == MouseEventKind::Moved {
        // Update menu selection on hover
        app_state
            .command_center_menu
            .update_selection_from_mouse(mouse_event.column, mouse_event.row);
    }

    CommandCenterMenuResult::NotHandled
}

/// Shows the taskbar menu for a window button right-click.
/// Returns true if the menu was shown.
pub fn show_taskbar_menu(
    app_state: &mut AppState,
    window_manager: &WindowManager,
    mouse_event: &MouseEvent,
    bar_y: u16,
    window_buttons_start: u16,
) -> bool {
    if mouse_event.kind == MouseEventKind::Down(MouseButton::Right) && mouse_event.row == bar_y {
        if let Some(window_id) = window_manager.button_bar_get_window_at(
            mouse_event.column,
            bar_y,
            mouse_event.row,
            window_buttons_start,
        ) {
            // Position menu above the click point (menu height is 5: 3 items + 2 borders)
            let menu_y = mouse_event.row.saturating_sub(5);
            app_state.taskbar_menu.show(mouse_event.column, menu_y);
            app_state.taskbar_menu_window_id = Some(window_id);
            return true;
        }
    }
    false
}

// ============================================================================
// Text Selection Handling
// ============================================================================

/// Handles mouse events for text selection (single/double/triple click, drag).
/// Returns true if the event was handled.
pub fn handle_selection_mouse(
    app_state: &mut AppState,
    window_manager: &mut WindowManager,
    mouse_event: &MouseEvent,
) -> bool {
    // Check if we should forward mouse to the terminal child process
    // Don't forward if a close confirmation dialog is active - dialog must capture clicks
    let forward_to_terminal = window_manager.focused_has_mouse_tracking()
        && !window_manager.is_dragging_or_resizing()
        && !window_manager.is_point_on_drag_or_resize_area(mouse_event.column, mouse_event.row)
        && !window_manager.focused_has_close_confirmation();

    if forward_to_terminal {
        // Forward mouse event to child process (e.g., dialog, vim)
        let (button, action) = match mouse_event.kind {
            MouseEventKind::Down(MouseButton::Left) => (0u8, 0u8),
            MouseEventKind::Down(MouseButton::Middle) => (1u8, 0u8),
            MouseEventKind::Down(MouseButton::Right) => (2u8, 0u8),
            MouseEventKind::Up(MouseButton::Left) => (0u8, 1u8),
            MouseEventKind::Up(MouseButton::Middle) => (1u8, 1u8),
            MouseEventKind::Up(MouseButton::Right) => (2u8, 1u8),
            MouseEventKind::Drag(MouseButton::Left) => (0u8, 2u8),
            MouseEventKind::Drag(MouseButton::Middle) => (1u8, 2u8),
            MouseEventKind::Drag(MouseButton::Right) => (2u8, 2u8),
            MouseEventKind::Moved => (0u8, 2u8), // Motion with no button
            MouseEventKind::ScrollUp => (64u8, 0u8),
            MouseEventKind::ScrollDown => (65u8, 0u8),
            MouseEventKind::ScrollLeft => (66u8, 0u8),
            MouseEventKind::ScrollRight => (67u8, 0u8),
        };

        if window_manager.forward_mouse_to_focused(
            mouse_event.column,
            mouse_event.row,
            button,
            action,
        ) {
            return true;
        }
    }

    // Skip selection if a close confirmation dialog is active - let window manager handle it
    if window_manager.focused_has_close_confirmation() {
        return false;
    }

    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Skip selection if clicking on title bar or resize edge (would start drag/resize)
            if window_manager.is_point_on_drag_or_resize_area(mouse_event.column, mouse_event.row) {
                return false;
            }

            // Check if clicking on a window
            let clicked_window_id = window_manager.window_at(mouse_event.column, mouse_event.row);

            // If clicking on empty space (no window), let window manager handle it
            // This allows focus_desktop() to be called
            if clicked_window_id.is_none() {
                return false;
            }

            // If clicking on a different window than the focused one,
            // let the window manager handle focus change first
            if let Some(clicked_id) = clicked_window_id {
                if let FocusState::Window(focused_id) = window_manager.get_focus() {
                    if clicked_id != focused_id {
                        return false;
                    }
                }
            }

            if let FocusState::Window(window_id) = window_manager.get_focus() {
                // Track click timing and position for double/triple-click detection
                let now = Instant::now();
                let click_x = mouse_event.column;
                let click_y = mouse_event.row;

                // Check if this click is close enough in time and position
                // to be considered a multi-click (within 500ms and 2 chars)
                let is_multi_click = if let (Some(last_time), Some((last_x, last_y))) =
                    (app_state.last_click_time, app_state.last_click_pos)
                {
                    let time_ok = now.duration_since(last_time).as_millis() < 500;
                    let pos_ok = click_x.abs_diff(last_x) <= 2 && click_y.abs_diff(last_y) <= 2;
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
                        let sel_type = if mouse_event.modifiers.contains(KeyModifiers::ALT) {
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

                if app_state.click_count <= 1 || selection_type == SelectionType::Block {
                    app_state.selection_active = true;
                }
                return true;
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            // Don't update selection while dragging/resizing a window
            if app_state.selection_active && !window_manager.is_dragging_or_resizing() {
                if let FocusState::Window(window_id) = window_manager.get_focus() {
                    window_manager.update_selection(window_id, mouse_event.column, mouse_event.row);
                    return true;
                }
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if app_state.selection_active {
                if let FocusState::Window(window_id) = window_manager.get_focus() {
                    window_manager.complete_selection(window_id);
                }
                app_state.selection_active = false;
                return true;
            }
        }
        _ => {}
    }

    false
}
