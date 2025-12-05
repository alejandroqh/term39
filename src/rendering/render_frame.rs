use crate::app::app_state::AppState;
use crate::app::config_manager::AppConfig;
use crate::input::keyboard_mode::KeyboardMode;
use crate::lockscreen::auth::is_os_auth_available;
use crate::ui::ui_render;
use crate::window::manager::WindowManager;
use crate::window::number_overlay;
use super::charset::Charset;
use super::render_backend::RenderBackend;
use super::theme::Theme;
use super::video_buffer::{render_fullscreen_shadow, VideoBuffer};
use std::io::{self, Write};

/// Renders a complete frame to the screen
#[allow(clippy::too_many_arguments)]
pub fn render_frame(
    video_buffer: &mut VideoBuffer,
    backend: &mut Box<dyn RenderBackend>,
    stdout: &mut io::Stdout,
    window_manager: &mut WindowManager,
    app_state: &mut AppState,
    charset: &Charset,
    theme: &Theme,
    app_config: &AppConfig,
) -> io::Result<bool> {
    // Get current dimensions from backend
    let (cols, rows) = backend.dimensions();

    // Render the background (every frame for consistency)
    ui_render::render_background(video_buffer, charset, theme);

    // Render the top bar
    let focus = window_manager.get_focus();
    ui_render::render_top_bar(
        video_buffer,
        focus,
        &app_state.new_terminal_button,
        &app_state.paste_button,
        &app_state.clear_clipboard_button,
        &app_state.copy_button,
        &app_state.clear_selection_button,
        &app_state.exit_button,
        app_config,
        theme,
        app_state.battery_hovered,
    );

    // Render all windows (returns true if any were closed)
    // Pass keyboard mode active state for special border coloring
    let keyboard_mode_active = !matches!(app_state.keyboard_mode, KeyboardMode::Normal);
    let windows_closed = window_manager.render_all(
        video_buffer,
        charset,
        theme,
        app_state.tint_terminal,
        keyboard_mode_active,
    );

    // Render snap preview overlay (if dragging and snap zone is active)
    window_manager.render_snap_preview(video_buffer, charset, theme);

    // Render window number overlay (if Alt/Cmd held for 500ms+)
    if app_state.show_window_number_overlay {
        number_overlay::render_window_numbers(video_buffer, window_manager, theme);
    }

    // Render the pivot for tiled window resizing (only when auto-tiling enabled with gaps and 2-4 windows)
    if app_state.auto_tiling_enabled {
        window_manager.render_pivot(video_buffer, charset, theme, app_config.tiling_gaps);
    }

    // Render the button bar
    ui_render::render_button_bar(
        video_buffer,
        window_manager,
        &app_state.auto_tiling_button,
        app_state.auto_tiling_enabled,
        &app_state.keyboard_mode,
        theme,
    );

    // Check if any modal/dialog is active - apply shadow ONCE if so
    // This avoids redundant O(cols*rows) iterations for each modal
    let has_modal = app_state.active_prompt.is_some()
        || app_state.active_slight_input.is_some()
        || app_state.active_calendar.is_some()
        || app_state.active_config_window.is_some()
        || app_state.active_pin_setup.is_some()
        || app_state.active_help_window.is_some()
        || app_state.active_about_window.is_some()
        || app_state.active_winmode_help_window.is_some()
        || app_state.active_error_dialog.is_some();

    if has_modal {
        render_fullscreen_shadow(video_buffer, theme);
    }

    // Render active prompt (if any) on top of everything
    if let Some(ref prompt) = app_state.active_prompt {
        prompt.render(video_buffer, charset, theme);
    }

    // Render active Slight input (if any) on top of everything
    if let Some(ref slight_input) = app_state.active_slight_input {
        slight_input.render(video_buffer, charset, theme);
    }

    // Render active calendar (if any) on top of everything
    if let Some(ref calendar) = app_state.active_calendar {
        ui_render::render_calendar(video_buffer, calendar, charset, theme, cols, rows);
    }

    // Render active config window (if any) on top of everything
    if let Some(ref config_win) = app_state.active_config_window {
        config_win.render(
            video_buffer,
            charset,
            theme,
            app_config,
            app_state.tint_terminal,
            is_os_auth_available(),
        );
    }

    // Render active PIN setup dialog (if any) on top of everything
    if let Some(ref pin_setup) = app_state.active_pin_setup {
        pin_setup.render(video_buffer, charset, theme);
    }

    // Render active help window (if any)
    if let Some(ref help_win) = app_state.active_help_window {
        help_win.render(video_buffer, charset, theme);
    }

    // Render active about window (if any)
    if let Some(ref about_win) = app_state.active_about_window {
        about_win.render(video_buffer, charset, theme);
    }

    // Render active Window Mode help window (if any)
    if let Some(ref winmode_help_win) = app_state.active_winmode_help_window {
        winmode_help_win.render(video_buffer, charset, theme);
    }

    // Render error dialog (if any) on top of everything
    if let Some(ref error_dialog) = app_state.active_error_dialog {
        error_dialog.render(video_buffer, charset, theme);
    }

    // Render toast notification (if any, auto-expires)
    // Check expiration first, then render if still valid
    let toast_expired = app_state
        .active_toast
        .as_ref()
        .is_some_and(|t| t.is_expired());
    if toast_expired {
        app_state.active_toast = None;
    }
    if let Some(ref toast) = app_state.active_toast {
        toast.render(video_buffer, charset, theme);
    }

    // Render context menu (if visible)
    if app_state.context_menu.visible {
        app_state.context_menu.render(video_buffer, charset, theme);
    }

    // Render taskbar context menu (if visible)
    if app_state.taskbar_menu.visible {
        app_state.taskbar_menu.render(video_buffer, charset, theme);
    }

    // Render lockscreen (highest priority - on top of everything)
    // This completely blocks all other UI when active
    if app_state.lockscreen.is_active() {
        app_state.lockscreen.render(video_buffer, charset, theme);
    }

    // Restore old cursor area before presenting new frame
    backend.restore_cursor_area();

    // Present buffer to screen via rendering backend
    backend.present(video_buffer)?;

    // Update cursor position from mouse input and draw at new position
    backend.update_cursor();
    backend.draw_cursor();

    stdout.flush()?;

    Ok(windows_closed)
}
