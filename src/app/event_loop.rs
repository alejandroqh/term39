use crate::app::{AppConfig, AppState};
use crate::input::mouse_handlers::{
    CommandCenterMenuResult, ModalMouseResult, TopBarClickResult, handle_about_window_mouse,
    handle_auto_tiling_click, handle_calendar_mouse, handle_command_center_menu_mouse,
    handle_config_window_mouse, handle_context_menu_mouse, handle_error_dialog_mouse,
    handle_help_window_mouse, handle_pin_setup_mouse, handle_prompt_mouse, handle_selection_mouse,
    handle_taskbar_menu_mouse, handle_topbar_click, handle_winmode_help_window_mouse,
    show_context_menu, show_taskbar_menu, update_bar_button_hover_states,
};
use crate::lockscreen::PinSetupState;
use crate::rendering::RenderBackend;
use crate::ui::config_action_handler::{apply_config_result, process_config_action};
use crate::ui::config_window::ConfigWindow;
use crate::ui::prompt::{Prompt, PromptAction, PromptButton, PromptType};
use crate::ui::slight_input::SlightInput;
use crate::utils::{ClipboardManager, CommandHistory, CommandIndexer};
use crate::window::{FocusState, WindowManager};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind},
    terminal::{self, ClearType},
};
use std::io;
use std::time::Duration;

#[allow(clippy::too_many_arguments)]
pub fn run(
    backend: &mut Box<dyn RenderBackend>,
    video_buffer: &mut crate::rendering::VideoBuffer,
    stdout: &mut io::Stdout,
    window_manager: &mut WindowManager,
    app_state: &mut AppState,
    app_config: &mut AppConfig,
    charset: &mut crate::rendering::Charset,
    theme: &mut crate::rendering::Theme,
    mouse_input_manager: &mut crate::input::mouse::MouseInputManager,
    cli_args: &crate::app::cli::Cli,
    command_indexer: &CommandIndexer,
    command_history: &mut CommandHistory,
    clipboard_manager: &mut ClipboardManager,
    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    #[allow(unused_variables)]
    _gpm_disable_connection: &Option<crate::input::gpm_control::GpmConnection>,
    #[cfg(not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    #[allow(unused_variables)]
    _gpm_disable_connection: &Option<()>,
) -> io::Result<()> {
    // Load framebuffer configuration (for swap_buttons, etc.)
    #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
    #[allow(unused_variables)] // Only used on Linux with framebuffer
    let fb_config = crate::framebuffer::fb_config::FramebufferConfig::load();

    // Main loop
    loop {
        // Check for external lock request (via SIGUSR1 signal)
        if crate::lockscreen::signal_handler::check_and_clear() {
            if app_state.lockscreen.is_available() {
                app_state.lockscreen.lock();
            }
        }

        // Update lockscreen state (check lockout timer)
        app_state.lockscreen.update();
        // Check if backend was resized and recreate buffer if needed
        if let Some((new_cols, new_rows)) = backend.check_resize()? {
            // Clear the terminal screen to remove artifacts
            use crossterm::execute;
            execute!(stdout, terminal::Clear(ClearType::All))?;
            *video_buffer = crate::app::initialization::initialize_video_buffer(backend.as_ref());
            app_state.update_auto_tiling_button_position(new_rows);

            // Update mouse input manager bounds for the new size
            mouse_input_manager.set_bounds(new_cols, new_rows);

            // Reposition windows to fit the new screen dimensions
            if app_state.auto_tiling_enabled {
                window_manager.auto_position_windows(new_cols, new_rows, app_config.tiling_gaps);
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
        let windows_closed = crate::rendering::render_frame(
            video_buffer,
            backend,
            stdout,
            window_manager,
            app_state,
            charset,
            theme,
            app_config,
            has_clipboard_content,
            has_selection,
        )?;

        // Auto-reposition remaining windows if any were closed
        if windows_closed && app_state.auto_tiling_enabled {
            let (cols, rows) = backend.dimensions();
            window_manager.auto_position_windows(cols, rows, app_config.tiling_gaps);
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

        // Process framebuffer mouse events if available
        #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
        if injected_event.is_none() {
            if let Some((event_type, button_id, col, row)) = backend.get_mouse_button_event() {
                injected_event = Some(crate::input::mouse_handlers::map_fb_button_event(
                    event_type,
                    button_id,
                    col,
                    row,
                    fb_config.mouse.swap_buttons,
                ));
            }
        }

        #[cfg(all(target_os = "linux", feature = "framebuffer-backend"))]
        if injected_event.is_none() {
            if let Some((scroll_direction, col, row)) = backend.get_mouse_scroll_event() {
                injected_event = Some(crate::input::mouse_handlers::map_fb_scroll_event(
                    scroll_direction,
                    col,
                    row,
                ));
            }
        }

        // Process all available events before next frame (batch processing for responsiveness)
        // This prevents input lag on Windows where events can queue up between frames
        const MAX_EVENTS_PER_FRAME: usize = 50;
        let mut events_processed = 0;
        let mut should_break_main_loop = false;

        while events_processed < MAX_EVENTS_PER_FRAME {
            // Check for available events (non-blocking after first iteration)
            // Windows console I/O is slower, so use shorter timeout for faster input response
            let poll_timeout = if events_processed == 0 {
                #[cfg(target_os = "windows")]
                {
                    Duration::from_millis(8) // Windows: faster polling for responsive input
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Duration::from_millis(16) // Other platforms: standard 60fps timing
                }
            } else {
                Duration::from_millis(0) // Subsequent: non-blocking
            };

            #[cfg(target_os = "linux")]
            let has_event = injected_event.is_some() || event::poll(poll_timeout)?;
            #[cfg(not(target_os = "linux"))]
            let has_event = event::poll(poll_timeout)?;

            if !has_event {
                break; // No more events available
            }

            events_processed += 1;
            // Track whether this event is injected (raw/FB) to avoid double-scaling
            #[cfg(target_os = "linux")]
            let is_injected = injected_event.is_some();
            #[cfg(not(target_os = "linux"))]
            let is_injected = false;

            #[cfg(target_os = "linux")]
            let current_event = if let Some(evt) = injected_event.take() {
                evt
            } else {
                event::read()?
            };
            #[cfg(not(target_os = "linux"))]
            let current_event = event::read()?;

            match current_event {
                Event::Key(key_event) => {
                    // Windows sends KeyEventKind::Press, Release, AND Repeat
                    // - Press: initial key down
                    // - Repeat: key held down (auto-repeat)
                    // - Release: key up (should be ignored)
                    //
                    // For character keys: only process Press events to avoid duplicates
                    // when typing fast (Windows can generate spurious Repeat events)
                    // For navigation keys (arrows, etc.): process both Press and Repeat
                    // so holding the key continues to work
                    if key_event.kind == KeyEventKind::Release {
                        continue;
                    }

                    // Skip Repeat events for character keys to prevent duplicates
                    // Allow Repeat for navigation/control keys (arrows, Page Up/Down, etc.)
                    if key_event.kind == KeyEventKind::Repeat {
                        let is_navigation_key = matches!(
                            key_event.code,
                            KeyCode::Up
                                | KeyCode::Down
                                | KeyCode::Left
                                | KeyCode::Right
                                | KeyCode::PageUp
                                | KeyCode::PageDown
                                | KeyCode::Home
                                | KeyCode::End
                                | KeyCode::Backspace
                                | KeyCode::Delete
                        );
                        if !is_navigation_key {
                            continue;
                        }
                    }

                    let current_focus = window_manager.get_focus();

                    // Handle lockscreen keyboard events (highest priority - blocks all other input)
                    if crate::ui::dialog_handlers::handle_lockscreen_keyboard(app_state, key_event)
                    {
                        continue;
                    }

                    // F12 - Global lockscreen shortcut (works from anywhere, even in terminal)
                    if key_event.code == KeyCode::F(12) {
                        if app_config.lockscreen_enabled && app_state.lockscreen.is_available() {
                            app_state.lockscreen.lock();
                        } else {
                            // Show toast: "To lock the screen, configure in Settings"
                            app_state.active_toast = Some(crate::ui::toast::Toast::new(
                                "To lock the screen, configure in Settings",
                            ));
                        }
                        continue;
                    }

                    // Dismiss toast on any key press (if active and not just created)
                    // Check if toast was created more than 100ms ago to avoid dismissing
                    // toasts that were just created by the same key press
                    if let Some(ref toast) = app_state.active_toast {
                        if toast.created_at.elapsed() > std::time::Duration::from_millis(100) {
                            app_state.active_toast = None;
                        }
                    }

                    // Handle prompt keyboard navigation
                    if let Some(should_exit) =
                        crate::ui::dialog_handlers::handle_prompt_keyboard(app_state, key_event)
                    {
                        if should_exit {
                            should_break_main_loop = true;
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
                                    window_manager.auto_position_windows(
                                        cols,
                                        rows,
                                        app_config.tiling_gaps,
                                    );
                                }
                            }
                            continue; // Handled
                        }
                    }

                    // Handle error dialog keyboard events
                    if crate::ui::dialog_handlers::handle_error_dialog_keyboard(
                        app_state, key_event,
                    ) {
                        continue;
                    }

                    // Handle PIN setup dialog keyboard events
                    if let Some(ref mut pin_setup) = app_state.active_pin_setup {
                        pin_setup.handle_key(key_event);
                        match pin_setup.state().clone() {
                            PinSetupState::Complete { hash, salt } => {
                                // Save PIN to config
                                app_config.set_pin(hash, salt);
                                app_state.update_lockscreen_auth(app_config);
                                app_state.active_pin_setup = None;
                            }
                            PinSetupState::Cancelled => {
                                app_state.active_pin_setup = None;
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // Handle Slight input keyboard events
                    if crate::ui::dialog_handlers::handle_slight_input_keyboard(
                        app_state,
                        key_event,
                        command_indexer,
                        command_history,
                        window_manager,
                        backend.as_ref(),
                        app_config.tiling_gaps,
                    ) {
                        continue;
                    }

                    // Handle calendar keyboard navigation
                    if crate::ui::dialog_handlers::handle_calendar_keyboard(app_state, key_event) {
                        continue;
                    }

                    // Handle help window keyboard events
                    if crate::ui::dialog_handlers::handle_help_window_keyboard(app_state, key_event)
                    {
                        continue;
                    }

                    // Handle about window keyboard events
                    if crate::ui::dialog_handlers::handle_about_window_keyboard(
                        app_state, key_event,
                    ) {
                        continue;
                    }

                    // Handle config window keyboard events
                    if let Some(action) = crate::ui::dialog_handlers::handle_config_window_keyboard(
                        app_state, key_event, app_config,
                    ) {
                        let (_, rows) = backend.dimensions();
                        let result = process_config_action(action, app_state, app_config, rows);
                        apply_config_result(&result, charset, theme);
                        continue;
                    }

                    // Handle Window Mode help window keyboard events
                    if crate::ui::dialog_handlers::handle_winmode_help_window_keyboard(
                        app_state, key_event,
                    ) {
                        continue;
                    }

                    // Handle Window Mode keyboard events (vim-like window control)
                    if crate::window::mode_handlers::handle_window_mode_keyboard(
                        app_state,
                        app_config,
                        key_event,
                        window_manager,
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
                    if crate::input::keyboard_handlers::handle_desktop_keyboard(
                        app_state,
                        key_event,
                        current_focus,
                        window_manager,
                        clipboard_manager,
                        backend.as_ref(),
                        app_config,
                        cli_args,
                    ) {
                        // Check if exit was requested
                        if app_state.should_exit {
                            should_break_main_loop = true;
                            break;
                        }
                        continue;
                    }

                    // Forward input to terminal window if a window is focused
                    if matches!(current_focus, FocusState::Window(_)) {
                        crate::input::keyboard_handlers::forward_to_terminal(
                            key_event,
                            window_manager,
                        );
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

                    // Handle modal dialogs (prompt, PIN setup, error, config window)
                    match handle_prompt_mouse(app_state, &mouse_event, charset) {
                        ModalMouseResult::Exit => {
                            should_break_main_loop = true;
                            break;
                        }
                        ModalMouseResult::Handled => handled = true,
                        ModalMouseResult::NotHandled => {}
                    }

                    let (cols, rows) = backend.dimensions();
                    if !handled
                        && handle_pin_setup_mouse(
                            app_state,
                            app_config,
                            &mouse_event,
                            cols,
                            rows,
                            charset,
                        )
                    {
                        handled = true;
                    }

                    if !handled && handle_error_dialog_mouse(app_state, &mouse_event) {
                        handled = true;
                    }

                    if !handled
                        && handle_config_window_mouse(
                            app_state,
                            app_config,
                            &mouse_event,
                            rows,
                            charset,
                            theme,
                        )
                    {
                        handled = true;
                    }

                    // Handle help window mouse events
                    if !handled && handle_help_window_mouse(app_state, &mouse_event) {
                        handled = true;
                    }

                    // Handle about window mouse events
                    if !handled && handle_about_window_mouse(app_state, &mouse_event) {
                        handled = true;
                    }

                    // Handle window mode help window mouse events
                    if !handled && handle_winmode_help_window_mouse(app_state, &mouse_event) {
                        handled = true;
                    }

                    // Handle calendar mouse events
                    if !handled && handle_calendar_mouse(app_state, &mouse_event, cols, rows) {
                        handled = true;
                    }

                    // Update button hover states (always active)
                    if !handled {
                        // Get clipboard and selection state for hover updates
                        let hover_clipboard = clipboard_manager.has_content();
                        let hover_selection =
                            window_manager.focused_window_has_meaningful_selection();
                        let focus = window_manager.get_focus();
                        update_bar_button_hover_states(
                            app_state,
                            mouse_event.column,
                            mouse_event.row,
                            cols,
                            rows,
                            app_config.show_date_in_clock,
                            hover_clipboard,
                            hover_selection,
                            focus,
                        );
                    }

                    // Handle top bar button clicks (if no prompt active)
                    if !handled && app_state.active_prompt.is_none() {
                        match handle_topbar_click(
                            app_state,
                            window_manager,
                            clipboard_manager,
                            &mouse_event,
                            cols,
                            rows,
                            app_config.tiling_gaps,
                            cli_args.no_exit,
                            app_config.show_date_in_clock,
                        ) {
                            TopBarClickResult::Handled => handled = true,
                            TopBarClickResult::NotHandled => {}
                        }
                    }

                    // Handle auto-tiling toggle button click
                    if !handled
                        && app_state.active_prompt.is_none()
                        && handle_auto_tiling_click(app_state, app_config, &mouse_event, rows)
                    {
                        handled = true;
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

                    // Handle right-click on button bar for taskbar context menu
                    if !handled && app_state.active_prompt.is_none() {
                        let window_buttons_start =
                            1 + 1 + app_state.auto_tiling_button.label.len() as u16 + 1 + 2;
                        if show_taskbar_menu(
                            app_state,
                            window_manager,
                            &mouse_event,
                            bar_y,
                            window_buttons_start,
                        ) {
                            handled = true;
                        }
                    }

                    // Handle right-click for context menu (inside windows)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && show_context_menu(app_state, window_manager, &mouse_event)
                    {
                        handled = true;
                    }

                    // Handle context menu interactions
                    if !handled
                        && handle_context_menu_mouse(
                            app_state,
                            window_manager,
                            clipboard_manager,
                            &mouse_event,
                        )
                    {
                        handled = true;
                    }

                    // Handle taskbar menu interactions
                    if !handled
                        && handle_taskbar_menu_mouse(
                            app_state,
                            window_manager,
                            &mouse_event,
                            cols,
                            rows,
                            app_config.tiling_gaps,
                        )
                    {
                        handled = true;
                    }

                    // Handle command center menu interactions
                    if !handled {
                        match handle_command_center_menu_mouse(
                            app_state,
                            window_manager,
                            clipboard_manager,
                            &mouse_event,
                        ) {
                            CommandCenterMenuResult::Handled => handled = true,
                            CommandCenterMenuResult::ShowSettings => {
                                // Open the config window (same as pressing 's' on desktop)
                                app_state.active_config_window =
                                    Some(ConfigWindow::new(cols, rows));
                                handled = true;
                            }
                            CommandCenterMenuResult::ShowAbout => {
                                // Open the about window (same as pressing 'l' on desktop)
                                crate::input::keyboard_handlers::show_about_window(
                                    app_state,
                                    backend.as_ref(),
                                );
                                handled = true;
                            }
                            CommandCenterMenuResult::ShowExitPrompt => {
                                // Build exit confirmation message
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

                                app_state.active_prompt = Some(
                                    Prompt::new(
                                        PromptType::Danger,
                                        message,
                                        vec![
                                            PromptButton::new(
                                                "Cancel".to_string(),
                                                PromptAction::Cancel,
                                                false,
                                            ),
                                            PromptButton::new(
                                                "Exit".to_string(),
                                                PromptAction::Confirm,
                                                true,
                                            ),
                                        ],
                                        cols,
                                        rows,
                                    )
                                    .with_selection_indicators(true)
                                    .with_selected_button(0),
                                );
                                handled = true;
                            }
                            CommandCenterMenuResult::NotHandled => {}
                        }
                    }

                    // Handle text selection (left-click, drag, mouse forwarding)
                    // Skip selection handling if clicking on the pivot (let window manager handle it)
                    let on_pivot = app_state.auto_tiling_enabled
                        && app_config.tiling_gaps
                        && window_manager.is_point_on_pivot(
                            mouse_event.column,
                            mouse_event.row,
                            cols,
                            rows,
                            app_config.tiling_gaps,
                        );
                    if !handled
                        && !on_pivot
                        && app_state.active_prompt.is_none()
                        && !app_state.context_menu.visible
                        && handle_selection_mouse(app_state, window_manager, &mouse_event)
                    {
                        handled = true;
                    }

                    // If not handled by buttons, let window manager handle it (only if no prompt)
                    if !handled
                        && app_state.active_prompt.is_none()
                        && !app_state.context_menu.visible
                    {
                        let window_closed = window_manager.handle_mouse_event(
                            video_buffer,
                            mouse_event,
                            charset,
                            app_config.tiling_gaps,
                            app_state.auto_tiling_enabled,
                        );
                        // Auto-reposition remaining windows if a window was closed
                        if window_closed && app_state.auto_tiling_enabled {
                            let (cols, rows) = backend.dimensions();
                            window_manager.auto_position_windows(
                                cols,
                                rows,
                                app_config.tiling_gaps,
                            );
                        }
                    }

                    // Check if exit was requested (from Exit button)
                    if app_state.should_exit {
                        should_break_main_loop = true;
                        break;
                    }
                }
                _ => {}
            }
        } // End of while events loop

        // Flush all buffered terminal input once after processing the event batch
        // This avoids per-keystroke I/O overhead (especially important on Windows)
        window_manager.flush_all_terminal_input();

        // Check if we need to exit the main loop
        if should_break_main_loop {
            break;
        }
    }

    Ok(())
}
