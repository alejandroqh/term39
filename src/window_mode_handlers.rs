//! Keyboard handlers for vim-like Window Mode
//!
//! This module handles keyboard input when the application is in Window Mode,
//! allowing full keyboard-only control of windows.

use crate::app_state::AppState;
use crate::info_window::InfoWindow;
use crate::keyboard_mode::{KeyboardMode, ResizeDirection, SnapPosition, WindowSubMode};
use crate::render_backend::RenderBackend;
use crate::window_manager::WindowManager;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

/// Double-backtick threshold in milliseconds
const DOUBLE_BACKTICK_THRESHOLD_MS: u64 = 300;

/// Direction constants for spatial navigation
pub const DIR_LEFT: u8 = 0;
pub const DIR_DOWN: u8 = 1;
pub const DIR_UP: u8 = 2;
pub const DIR_RIGHT: u8 = 3;

/// Handle keyboard input when in Window Mode
/// Returns true if event was consumed
pub fn handle_window_mode_keyboard(
    app_state: &mut AppState,
    key_event: KeyEvent,
    window_manager: &mut WindowManager,
    backend: &dyn RenderBackend,
) -> bool {
    // Only handle if in Window Mode
    let sub_mode = match app_state.keyboard_mode {
        KeyboardMode::Normal => return false,
        KeyboardMode::WindowMode(sub) => sub,
    };

    let (cols, rows) = backend.dimensions();
    let top_y: u16 = 1; // Top bar is row 0

    match sub_mode {
        WindowSubMode::Navigation => {
            handle_navigation_mode(app_state, key_event, window_manager, cols, rows, top_y)
        }
        WindowSubMode::Move => {
            handle_move_mode(app_state, key_event, window_manager, cols, rows, top_y)
        }
        WindowSubMode::Resize(direction) => {
            handle_resize_mode(app_state, key_event, window_manager, direction)
        }
    }
}

/// Handle keyboard in Navigation sub-mode (default Window Mode)
fn handle_navigation_mode(
    app_state: &mut AppState,
    key_event: KeyEvent,
    window_manager: &mut WindowManager,
    cols: u16,
    rows: u16,
    top_y: u16,
) -> bool {
    let has_shift = key_event.modifiers.contains(KeyModifiers::SHIFT);

    match key_event.code {
        // Exit Window Mode (F8 or Esc)
        KeyCode::F(8) | KeyCode::Esc => {
            app_state.keyboard_mode.exit_to_normal();
            app_state.move_state.reset();
            app_state.resize_state.reset();
            true
        }

        // Backtick with double-press detection
        // Single backtick: exit Window Mode
        // Double backtick (within 300ms): send literal '`' to terminal and exit
        KeyCode::Char('`') => {
            let now = Instant::now();
            let is_double_press = app_state
                .last_backtick_time
                .map(|t| {
                    now.duration_since(t) < Duration::from_millis(DOUBLE_BACKTICK_THRESHOLD_MS)
                })
                .unwrap_or(false);

            if is_double_press {
                // Double backtick: send literal '`' to focused terminal
                app_state.last_backtick_time = None;
                app_state.keyboard_mode.exit_to_normal();
                app_state.move_state.reset();
                app_state.resize_state.reset();
                let _ = window_manager.send_to_focused("`");
            } else {
                // Single backtick: just exit Window Mode and record time
                app_state.last_backtick_time = Some(now);
                app_state.keyboard_mode.exit_to_normal();
                app_state.move_state.reset();
                app_state.resize_state.reset();
            }
            true
        }

        // Spatial navigation - focus window in direction
        KeyCode::Char('h') | KeyCode::Left if !has_shift => {
            window_manager.focus_window_in_direction(DIR_LEFT);
            true
        }
        KeyCode::Char('j') | KeyCode::Down if !has_shift => {
            window_manager.focus_window_in_direction(DIR_DOWN);
            true
        }
        KeyCode::Char('k') | KeyCode::Up if !has_shift => {
            window_manager.focus_window_in_direction(DIR_UP);
            true
        }
        KeyCode::Char('l') | KeyCode::Right if !has_shift => {
            window_manager.focus_window_in_direction(DIR_RIGHT);
            true
        }

        // Snap to full halves (Shift + h/j/k/l)
        KeyCode::Char('H') | KeyCode::Left if has_shift => {
            let (x, y, w, h) = SnapPosition::FullLeft.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('J') | KeyCode::Down if has_shift => {
            let (x, y, w, h) = SnapPosition::FullBottom.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('K') | KeyCode::Up if has_shift => {
            let (x, y, w, h) = SnapPosition::FullTop.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('L') | KeyCode::Right if has_shift => {
            let (x, y, w, h) = SnapPosition::FullRight.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }

        // Tab cycling
        KeyCode::Tab if !has_shift => {
            window_manager.cycle_to_next_window();
            true
        }
        KeyCode::BackTab | KeyCode::Tab if has_shift => {
            // TODO: cycle_to_previous_window when implemented
            window_manager.cycle_to_next_window();
            true
        }

        // Enter Move sub-mode
        KeyCode::Char('m') => {
            app_state.keyboard_mode.enter_sub_mode(WindowSubMode::Move);
            app_state.move_state.reset();
            true
        }

        // Enter Resize sub-mode
        KeyCode::Char('r') => {
            app_state
                .keyboard_mode
                .enter_sub_mode(WindowSubMode::Resize(ResizeDirection::Default));
            app_state.resize_state.reset();
            true
        }

        // Close focused window
        KeyCode::Char('x') | KeyCode::Char('q') => {
            let closed = window_manager.close_focused_window();
            // Auto-exit Window Mode if no windows remain
            if closed && window_manager.window_count() == 0 {
                app_state.keyboard_mode.exit_to_normal();
            }
            true
        }

        // Toggle maximize
        KeyCode::Char('z') | KeyCode::Char('+') | KeyCode::Char(' ') => {
            window_manager.toggle_focused_window_maximize(cols, rows);
            true
        }

        // Minimize
        KeyCode::Char('-') | KeyCode::Char('_') => {
            window_manager.toggle_focused_window_minimize();
            true
        }

        // New terminal window
        KeyCode::Char('n') => {
            // Signal to create new window (handled in main.rs)
            // We return false to let main.rs handle creation
            // Actually, we need a way to signal this... for now, skip
            // The user can use 't' from desktop
            false
        }

        // Numpad-style snap positions (1-9)
        KeyCode::Char('1') => {
            let (x, y, w, h) = SnapPosition::BottomLeft.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('2') => {
            let (x, y, w, h) = SnapPosition::BottomCenter.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('3') => {
            let (x, y, w, h) = SnapPosition::BottomRight.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('4') => {
            let (x, y, w, h) = SnapPosition::MiddleLeft.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('5') => {
            let (x, y, w, h) = SnapPosition::Center.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('6') => {
            let (x, y, w, h) = SnapPosition::MiddleRight.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('7') => {
            let (x, y, w, h) = SnapPosition::TopLeft.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('8') => {
            let (x, y, w, h) = SnapPosition::TopCenter.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }
        KeyCode::Char('9') => {
            let (x, y, w, h) = SnapPosition::TopRight.calculate_rect(cols, rows, top_y);
            window_manager.snap_focused_window(x, y, w, h);
            true
        }

        // Help overlay
        KeyCode::Char('?') => {
            show_winmode_help_window(app_state, cols, rows);
            true
        }

        // Consume all other keys - don't let them pass to terminal while in Window Mode
        _ => true,
    }
}

/// Handle keyboard in Move sub-mode
fn handle_move_mode(
    app_state: &mut AppState,
    key_event: KeyEvent,
    window_manager: &mut WindowManager,
    cols: u16,
    rows: u16,
    top_y: u16,
) -> bool {
    let has_shift = key_event.modifiers.contains(KeyModifiers::SHIFT);

    match key_event.code {
        // Exit Move mode (Enter, Esc, F8, m)
        KeyCode::Enter | KeyCode::Esc | KeyCode::F(8) | KeyCode::Char('m') => {
            app_state.keyboard_mode.return_to_navigation();
            app_state.move_state.reset();
            true
        }

        // Backtick with double-press detection in Move mode
        KeyCode::Char('`') => {
            let now = Instant::now();
            let is_double_press = app_state
                .last_backtick_time
                .map(|t| {
                    now.duration_since(t) < Duration::from_millis(DOUBLE_BACKTICK_THRESHOLD_MS)
                })
                .unwrap_or(false);

            if is_double_press {
                // Double backtick: send literal '`' to focused terminal and exit
                app_state.last_backtick_time = None;
                app_state.keyboard_mode.exit_to_normal();
                app_state.move_state.reset();
                app_state.resize_state.reset();
                let _ = window_manager.send_to_focused("`");
            } else {
                // Single backtick: exit to navigation and record time
                app_state.last_backtick_time = Some(now);
                app_state.keyboard_mode.return_to_navigation();
                app_state.move_state.reset();
            }
            true
        }

        // Incremental movement (with adaptive step)
        KeyCode::Char('h') | KeyCode::Left if !has_shift => {
            let step = app_state.move_state.get_step() as i16;
            window_manager.move_focused_window_by(-step, 0, cols, rows, top_y);
            true
        }
        KeyCode::Char('j') | KeyCode::Down if !has_shift => {
            let step = app_state.move_state.get_step() as i16;
            window_manager.move_focused_window_by(0, step, cols, rows, top_y);
            true
        }
        KeyCode::Char('k') | KeyCode::Up if !has_shift => {
            let step = app_state.move_state.get_step() as i16;
            window_manager.move_focused_window_by(0, -step, cols, rows, top_y);
            true
        }
        KeyCode::Char('l') | KeyCode::Right if !has_shift => {
            let step = app_state.move_state.get_step() as i16;
            window_manager.move_focused_window_by(step, 0, cols, rows, top_y);
            true
        }

        // Snap to edges (Shift + h/j/k/l)
        KeyCode::Char('H') | KeyCode::Left if has_shift => {
            // Snap to left edge (x = 0)
            if let Some(win) = window_manager.get_focused_window() {
                let new_x = 0;
                window_manager.snap_focused_window(
                    new_x,
                    win.window.y,
                    win.window.width,
                    win.window.height,
                );
            }
            true
        }
        KeyCode::Char('J') | KeyCode::Down if has_shift => {
            // Snap to bottom edge
            if let Some(win) = window_manager.get_focused_window() {
                let new_y = rows.saturating_sub(win.window.height);
                window_manager.snap_focused_window(
                    win.window.x,
                    new_y,
                    win.window.width,
                    win.window.height,
                );
            }
            true
        }
        KeyCode::Char('K') | KeyCode::Up if has_shift => {
            // Snap to top edge
            if let Some(win) = window_manager.get_focused_window() {
                window_manager.snap_focused_window(
                    win.window.x,
                    top_y,
                    win.window.width,
                    win.window.height,
                );
            }
            true
        }
        KeyCode::Char('L') | KeyCode::Right if has_shift => {
            // Snap to right edge
            if let Some(win) = window_manager.get_focused_window() {
                let new_x = cols.saturating_sub(win.window.width);
                window_manager.snap_focused_window(
                    new_x,
                    win.window.y,
                    win.window.width,
                    win.window.height,
                );
            }
            true
        }

        // Consume all other keys - don't let them pass to terminal while in Move mode
        _ => true,
    }
}

/// Handle keyboard in Resize sub-mode
/// Shift modifier controls which edge is resized (left/top vs right/bottom)
fn handle_resize_mode(
    app_state: &mut AppState,
    key_event: KeyEvent,
    window_manager: &mut WindowManager,
    _resize_direction: ResizeDirection, // Kept for API compatibility
) -> bool {
    let has_shift = key_event.modifiers.contains(KeyModifiers::SHIFT);

    match key_event.code {
        // Exit Resize mode (Enter, Esc, F8, r)
        KeyCode::Enter | KeyCode::Esc | KeyCode::F(8) | KeyCode::Char('r') => {
            app_state.keyboard_mode.return_to_navigation();
            app_state.resize_state.reset();
            true
        }

        // Backtick with double-press detection in Resize mode
        KeyCode::Char('`') => {
            let now = Instant::now();
            let is_double_press = app_state
                .last_backtick_time
                .map(|t| {
                    now.duration_since(t) < Duration::from_millis(DOUBLE_BACKTICK_THRESHOLD_MS)
                })
                .unwrap_or(false);

            if is_double_press {
                // Double backtick: send literal '`' to focused terminal and exit
                app_state.last_backtick_time = None;
                app_state.keyboard_mode.exit_to_normal();
                app_state.move_state.reset();
                app_state.resize_state.reset();
                let _ = window_manager.send_to_focused("`");
            } else {
                // Single backtick: exit to navigation and record time
                app_state.last_backtick_time = Some(now);
                app_state.keyboard_mode.return_to_navigation();
                app_state.resize_state.reset();
            }
            true
        }

        // Incremental resize (with adaptive step)
        // Without Shift: normal resize behavior
        // With Shift: inverted behavior (grow <-> shrink)

        // h/Left = shrink width, Shift+h = GROW width (inverted)
        KeyCode::Char('h') | KeyCode::Left => {
            let step = app_state.resize_state.get_step() as i16;
            if has_shift {
                // Inverted: grow width from left edge
                window_manager.resize_focused_window_from_left(step);
            } else {
                // Normal: shrink width from right edge
                window_manager.resize_focused_window_by(-step, 0);
            }
            true
        }
        // l/Right = grow width, Shift+l = SHRINK width (inverted)
        KeyCode::Char('l') | KeyCode::Right => {
            let step = app_state.resize_state.get_step() as i16;
            if has_shift {
                // Inverted: shrink width from left edge
                window_manager.resize_focused_window_from_left(-step);
            } else {
                // Normal: grow width from right edge
                window_manager.resize_focused_window_by(step, 0);
            }
            true
        }
        // k/Up = shrink height, Shift+k = GROW height (inverted)
        KeyCode::Char('k') | KeyCode::Up => {
            let step = app_state.resize_state.get_step() as i16;
            if has_shift {
                // Inverted: grow height from top edge
                window_manager.resize_focused_window_from_top(step);
            } else {
                // Normal: shrink height from bottom edge
                window_manager.resize_focused_window_by(0, -step);
            }
            true
        }
        // j/Down = grow height, Shift+j = SHRINK height (inverted)
        KeyCode::Char('j') | KeyCode::Down => {
            let step = app_state.resize_state.get_step() as i16;
            if has_shift {
                // Inverted: shrink height from top edge
                window_manager.resize_focused_window_from_top(-step);
            } else {
                // Normal: grow height from bottom edge
                window_manager.resize_focused_window_by(0, step);
            }
            true
        }

        // Consume all other keys - don't let them pass to terminal while in Resize mode
        _ => true,
    }
}

/// Show Window Mode help overlay with all keybindings
pub fn show_winmode_help_window(app_state: &mut AppState, cols: u16, rows: u16) {
    let help_message = "\
{C}WINDOW MODE HELP{W}

Press {Y}`{W} or {Y}F8{W} to toggle Window Mode

{C}NAVIGATION (default){W}

{Y}h{W}/{Y}\u{2190}{W}         Focus window to left
{Y}j{W}/{Y}\u{2193}{W}         Focus window below
{Y}k{W}/{Y}\u{2191}{W}         Focus window above
{Y}l{W}/{Y}\u{2192}{W}         Focus window to right
{Y}Tab{W}         Cycle to next window
{Y}Shift+Tab{W}   Cycle to previous window

{C}SNAP (Shift + h/j/k/l){W}

{Y}H{W}           Snap to left half
{Y}J{W}           Snap to bottom half
{Y}K{W}           Snap to top half
{Y}L{W}           Snap to right half

{C}NUMPAD POSITIONS (1-9){W}

{Y}7{W} {Y}8{W} {Y}9{W}       Top-left, Top-center, Top-right
{Y}4{W} {Y}5{W} {Y}6{W}       Middle-left, Center, Middle-right
{Y}1{W} {Y}2{W} {Y}3{W}       Bottom-left, Bottom-center, Bottom-right

{C}WINDOW ACTIONS{W}

{Y}m{W}           Enter Move mode
{Y}r{W}           Enter Resize mode
{Y}z{W}/{Y}+{W}/{Y}Space{W}   Toggle maximize
{Y}-{W}/{Y}_{W}         Toggle minimize
{Y}x{W}/{Y}q{W}         Close focused window

{C}MOVE MODE (after 'm'){W}

{Y}h/j/k/l{W}     Move window (adaptive speed)
{Y}Shift+H/J/K/L{W} Snap to edge
{Y}Enter{W}/{Y}Esc{W}/{Y}m{W} Exit Move mode

{C}RESIZE MODE (after 'r'){W}

{Y}h{W}/{Y}l{W}         Shrink/Grow width
{Y}k{W}/{Y}j{W}         Shrink/Grow height
{Y}Shift{W}       Invert direction
{Y}Enter{W}/{Y}Esc{W}/{Y}r{W} Exit Resize mode

{C}EXIT WINDOW MODE{W}

{Y}`{W}/{Y}F8{W}/{Y}Esc{W}    Return to Normal mode";

    app_state.active_winmode_help_window = Some(InfoWindow::new(
        "Window Mode Help".to_string(),
        help_message,
        cols,
        rows,
    ));
}
