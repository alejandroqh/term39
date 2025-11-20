use crate::app_state::AppState;
use crate::charset::Charset;
use crate::config_manager::AppConfig;
use crate::render_backend::RenderBackend;
use crate::theme::Theme;
use crate::ui_render;
use crate::video_buffer::{self, VideoBuffer};
use crate::window_manager::WindowManager;
use std::io::{self, Write};

/// Renders a complete frame to the screen
#[allow(clippy::too_many_arguments)]
pub fn render_frame(
    video_buffer: &mut VideoBuffer,
    backend: &mut Box<dyn RenderBackend>,
    stdout: &mut io::Stdout,
    window_manager: &mut WindowManager,
    app_state: &AppState,
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
    );

    // Render all windows (returns true if any were closed)
    let windows_closed =
        window_manager.render_all(video_buffer, charset, theme, app_state.tint_terminal);

    // Render snap preview overlay (if dragging and snap zone is active)
    window_manager.render_snap_preview(video_buffer, charset, theme);

    // Render the button bar
    ui_render::render_button_bar(
        video_buffer,
        window_manager,
        &app_state.auto_tiling_button,
        app_state.auto_tiling_enabled,
        theme,
    );

    // Render active prompt (if any) on top of everything
    if let Some(ref prompt) = app_state.active_prompt {
        video_buffer::render_fullscreen_shadow(video_buffer, theme);
        prompt.render(video_buffer, charset, theme);
    }

    // Render active Slight input (if any) on top of everything
    if let Some(ref slight_input) = app_state.active_slight_input {
        video_buffer::render_fullscreen_shadow(video_buffer, theme);
        slight_input.render(video_buffer, charset, theme);
    }

    // Render active calendar (if any) on top of everything
    if let Some(ref calendar) = app_state.active_calendar {
        video_buffer::render_fullscreen_shadow(video_buffer, theme);
        ui_render::render_calendar(video_buffer, calendar, charset, theme, cols, rows);
    }

    // Render active config window (if any) on top of everything
    if let Some(ref config_win) = app_state.active_config_window {
        video_buffer::render_fullscreen_shadow(video_buffer, theme);
        config_win.render(
            video_buffer,
            charset,
            theme,
            app_config,
            app_state.tint_terminal,
        );
    }

    // Render active help window (if any)
    if let Some(ref help_win) = app_state.active_help_window {
        video_buffer::render_fullscreen_shadow(video_buffer, theme);
        help_win.render(video_buffer, charset, theme);
    }

    // Render active about window (if any)
    if let Some(ref about_win) = app_state.active_about_window {
        video_buffer::render_fullscreen_shadow(video_buffer, theme);
        about_win.render(video_buffer, charset, theme);
    }

    // Render error dialog (if any) on top of everything
    if let Some(ref error_dialog) = app_state.active_error_dialog {
        video_buffer::render_fullscreen_shadow(video_buffer, theme);
        error_dialog.render(video_buffer, charset, theme);
    }

    // Render context menu (if visible)
    if app_state.context_menu.visible {
        app_state.context_menu.render(video_buffer, charset, theme);
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
