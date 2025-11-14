#![allow(clippy::collapsible_if)]

mod ansi_handler;
mod button;
mod charset;
mod cli;
mod clipboard_manager;
mod config;
mod config_manager;
mod config_window;
mod context_menu;
#[cfg(feature = "framebuffer-backend")]
mod framebuffer;
#[cfg(target_os = "linux")]
mod gpm_handler;
mod info_window;
mod prompt;
mod render_backend;
mod selection;
mod term_grid;
mod terminal_emulator;
mod terminal_window;
mod theme;
mod video_buffer;
mod window;
mod window_manager;

use button::Button;
use charset::Charset;
use chrono::{Datelike, Local, NaiveDate};
use clipboard_manager::ClipboardManager;
use config_manager::AppConfig;
use config_window::{ConfigAction, ConfigWindow};
use context_menu::{ContextMenu, MenuAction};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    style::Color,
    terminal::{self, ClearType},
};
use info_window::InfoWindow;
use prompt::{Prompt, PromptAction, PromptButton, PromptType};
#[cfg(feature = "framebuffer-backend")]
use render_backend::FramebufferBackend;
use render_backend::{RenderBackend, TerminalBackend};
use selection::SelectionType;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::{thread, time};
use theme::Theme;
use video_buffer::{Cell, VideoBuffer};
use window_manager::{FocusState, WindowManager};

// Calendar state structure
struct CalendarState {
    year: i32,
    month: u32,
    today: NaiveDate,
}

impl CalendarState {
    fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            year: today.year(),
            month: today.month(),
            today,
        }
    }

    fn next_month(&mut self) {
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
    }

    fn previous_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
    }

    fn next_year(&mut self) {
        self.year += 1;
    }

    fn previous_year(&mut self) {
        self.year -= 1;
    }

    fn reset_to_today(&mut self) {
        self.year = self.today.year();
        self.month = self.today.month();
    }

    fn month_name(&self) -> &'static str {
        match self.month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown",
        }
    }
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

    // Create charset based on CLI arguments
    let mut charset = if cli_args.ascii {
        Charset::ascii()
    } else {
        Charset::unicode()
    };

    // Load application configuration
    let mut app_config = AppConfig::load();

    // Set the background character from config
    charset.set_background(app_config.get_background_char());

    // Use theme from CLI args if provided, otherwise use config theme
    let theme_name = cli_args.theme.as_ref().unwrap_or(&app_config.theme);
    let mut theme = Theme::from_name(theme_name);

    // Initialize rendering backend (framebuffer or terminal)
    #[cfg(feature = "framebuffer-backend")]
    let mut backend: Box<dyn RenderBackend> = if cli_args.framebuffer {
        use framebuffer::text_modes::{TextMode, TextModeKind};

        // Parse the framebuffer mode from CLI args
        let mode_kind = TextModeKind::from_str(&cli_args.fb_mode).unwrap_or_else(|| {
            eprintln!(
                "Warning: Invalid framebuffer mode '{}', using default 80x25",
                cli_args.fb_mode
            );
            TextModeKind::Mode80x25
        });

        let mode = TextMode::new(mode_kind);

        // Parse the scale factor from CLI args
        let scale = cli_args.fb_scale.as_ref().and_then(|s| {
            if s == "auto" {
                None // Auto-calculate scale
            } else {
                s.parse::<usize>().ok().filter(|&n| (1..=8).contains(&n))
            }
        });

        // Get font name from CLI args
        let font_name = cli_args.fb_font.as_deref();

        // Get mouse device from CLI args
        let mouse_device = cli_args.mouse_device.as_deref();

        // Try to initialize framebuffer backend
        match FramebufferBackend::new(mode, scale, font_name, mouse_device) {
            Ok(fb_backend) => {
                println!("Framebuffer backend initialized: {}", mode_kind);
                Box::new(fb_backend)
            }
            Err(e) => {
                eprintln!("Failed to initialize framebuffer: {}", e);
                eprintln!("Falling back to terminal backend...");
                std::thread::sleep(std::time::Duration::from_secs(2));
                Box::new(TerminalBackend::new()?)
            }
        }
    } else {
        // Framebuffer feature is enabled but flag not set - use terminal backend
        Box::new(TerminalBackend::new()?)
    };

    #[cfg(not(feature = "framebuffer-backend"))]
    let mut backend: Box<dyn RenderBackend> = Box::new(TerminalBackend::new()?);

    let mut stdout = io::stdout();

    // Enter raw mode for low-level terminal control
    terminal::enable_raw_mode()?;

    // Hide cursor and enable mouse capture
    execute!(stdout, cursor::Hide, event::EnableMouseCapture)?;

    // Initialize GPM (General Purpose Mouse) for Linux console if available
    #[cfg(target_os = "linux")]
    let gpm_connection = gpm_handler::GpmConnection::open();

    // Clear the screen
    execute!(stdout, terminal::Clear(ClearType::All))?;

    // Initialize video buffer using backend dimensions
    let (cols, rows) = backend.dimensions();
    let mut video_buffer = VideoBuffer::new(cols, rows);

    // Initialize window manager
    let mut window_manager = WindowManager::new();

    // Create the "New Terminal" button
    let mut new_terminal_button = Button::new(1, 0, "+New Terminal".to_string());

    // Create clipboard-related buttons (centered in top bar)
    let (initial_cols, initial_rows) = backend.dimensions();

    // Paste button
    let paste_label = "Paste".to_string();
    let paste_button_width = (paste_label.len() as u16) + 4; // "[ Label ]"
    let paste_x = (initial_cols.saturating_sub(paste_button_width + 5)) / 2; // +5 for [ X ] button
    let mut paste_button = Button::new(paste_x, 0, paste_label);

    // Clear clipboard button (X)
    let clear_label = "X".to_string();
    let mut clear_clipboard_button = Button::new(paste_x + paste_button_width, 0, clear_label);

    // Copy button (shows when text is selected)
    let copy_label = "Copy".to_string();
    let mut copy_button = Button::new(0, 0, copy_label);

    // Clear selection button (X) (shows when text is selected)
    let clear_selection_label = "X".to_string();
    let mut clear_selection_button = Button::new(0, 0, clear_selection_label);

    // Auto-tiling toggle state and button (initialized from config)
    // Button is on bottom bar (rows - 1), left side
    let mut auto_tiling_enabled = app_config.auto_tiling_on_startup;
    let auto_tiling_text = if auto_tiling_enabled {
        "█ on] Auto Tiling"
    } else {
        "off ░] Auto Tiling"
    };
    let mut auto_tiling_button = Button::new(1, initial_rows - 1, auto_tiling_text.to_string());

    // Terminal tinting toggle state (initialized from config, CLI arg can override)
    let mut tint_terminal = app_config.tint_terminal || cli_args.tint_terminal;

    // Prompt state (None when no prompt is active)
    let mut active_prompt: Option<Prompt> = None;

    // Calendar state (None when calendar is not shown)
    let mut active_calendar: Option<CalendarState> = None;

    // Config window state (None when not shown)
    let mut active_config_window: Option<ConfigWindow> = None;

    // Help window state (None when not shown)
    let mut active_help_window: Option<InfoWindow> = None;

    // About window state (None when not shown)
    let mut active_about_window: Option<InfoWindow> = None;

    // Clipboard manager
    let mut clipboard_manager = ClipboardManager::new();

    // Context menu state (None when not shown)
    let mut context_menu = ContextMenu::new(0, 0);

    // Selection state
    let mut selection_active = false;
    let mut last_click_time: Option<Instant> = None;
    let mut click_count = 0;

    // Show splash screen for 1 second
    show_splash_screen(&mut video_buffer, &mut backend, &charset, &theme)?;

    // Start with desktop focused - no windows yet
    // User can press 't' to create windows

    // Main loop
    loop {
        // Check if backend was resized and recreate buffer if needed
        if let Some((new_cols, new_rows)) = backend.check_resize()? {
            // Clear the terminal screen to remove artifacts
            execute!(stdout, terminal::Clear(ClearType::All))?;
            video_buffer = VideoBuffer::new(new_cols, new_rows);
        }

        // Get current dimensions from backend
        let (cols, rows) = backend.dimensions();

        // Render the background (every frame for consistency)
        render_background(&mut video_buffer, &charset, &theme);

        // Update clipboard buttons state and position
        let has_clipboard_content = clipboard_manager.has_content();
        let has_selection = window_manager.focused_window_has_meaningful_selection();

        paste_button.enabled = has_clipboard_content;
        clear_clipboard_button.enabled = has_clipboard_content;
        copy_button.enabled = has_selection;
        clear_selection_button.enabled = has_selection;

        // Calculate button positions (centered)
        let paste_width = paste_button.width();
        let clear_clip_width = clear_clipboard_button.width();
        let copy_width = copy_button.width();
        let clear_sel_width = clear_selection_button.width();

        // Position paste and clear clipboard buttons together in center
        let paste_clear_total_width = paste_width + clear_clip_width;
        let paste_x = (cols.saturating_sub(paste_clear_total_width)) / 2;
        paste_button.x = paste_x;
        clear_clipboard_button.x = paste_x + paste_width;

        // Position copy and clear selection buttons
        if has_selection && has_clipboard_content {
            // Both visible: put copy[X] to the left of paste[X]
            let copy_clear_sel_width = copy_width + clear_sel_width;
            let total_width = copy_clear_sel_width + 1 + paste_clear_total_width; // +1 for gap
            let start_x = (cols.saturating_sub(total_width)) / 2;
            copy_button.x = start_x;
            clear_selection_button.x = start_x + copy_width;
            paste_button.x = start_x + copy_clear_sel_width + 1;
            clear_clipboard_button.x = paste_button.x + paste_width;
        } else if has_selection {
            // Only copy[X] visible: center it
            let copy_clear_sel_width = copy_width + clear_sel_width;
            let start_x = (cols.saturating_sub(copy_clear_sel_width)) / 2;
            copy_button.x = start_x;
            clear_selection_button.x = start_x + copy_width;
        }

        // Render the top bar
        let focus = window_manager.get_focus();
        render_top_bar(
            &mut video_buffer,
            focus,
            &new_terminal_button,
            &paste_button,
            &clear_clipboard_button,
            &copy_button,
            &clear_selection_button,
            &app_config,
            &theme,
        );

        // Render all windows (returns true if any were closed)
        let windows_closed =
            window_manager.render_all(&mut video_buffer, &charset, &theme, tint_terminal);

        // Auto-reposition remaining windows if any were closed
        if windows_closed && auto_tiling_enabled {
            let (cols, rows) = backend.dimensions();
            window_manager.auto_position_windows(cols, rows);
        }

        // Render snap preview overlay (if dragging and snap zone is active)
        window_manager.render_snap_preview(&mut video_buffer, &charset, &theme);

        // Render the button bar
        render_button_bar(
            &mut video_buffer,
            &window_manager,
            &auto_tiling_button,
            auto_tiling_enabled,
            &theme,
        );

        // Render active prompt (if any) on top of everything
        if let Some(ref prompt) = active_prompt {
            video_buffer::render_fullscreen_shadow(&mut video_buffer);
            prompt.render(&mut video_buffer, &charset, &theme);
        }

        // Render active calendar (if any) on top of everything
        if let Some(ref calendar) = active_calendar {
            video_buffer::render_fullscreen_shadow(&mut video_buffer);
            render_calendar(&mut video_buffer, calendar, &charset, &theme, cols, rows);
        }

        // Render active config window (if any) on top of everything
        if let Some(ref config_win) = active_config_window {
            video_buffer::render_fullscreen_shadow(&mut video_buffer);
            config_win.render(
                &mut video_buffer,
                &charset,
                &theme,
                &app_config,
                tint_terminal,
            );
        }

        // Render active help window (if any)
        if let Some(ref help_win) = active_help_window {
            video_buffer::render_fullscreen_shadow(&mut video_buffer);
            help_win.render(&mut video_buffer, &charset, &theme);
        }

        // Render active about window (if any)
        if let Some(ref about_win) = active_about_window {
            video_buffer::render_fullscreen_shadow(&mut video_buffer);
            about_win.render(&mut video_buffer, &charset, &theme);
        }

        // Render context menu (if visible)
        if context_menu.visible {
            context_menu.render(&mut video_buffer, &charset);
        }

        // Restore old cursor area before presenting new frame
        backend.restore_cursor_area();

        // Present buffer to screen via rendering backend
        backend.present(&mut video_buffer)?;

        // Update cursor position from mouse input and draw at new position
        backend.update_cursor();
        backend.draw_cursor();

        stdout.flush()?;

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
                    if let Some(ref mut prompt) = active_prompt {
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
                                            active_prompt = None;
                                        }
                                        _ => {}
                                    }
                                }
                                continue;
                            }
                            KeyCode::Esc => {
                                // ESC dismisses the prompt
                                active_prompt = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when prompt is active
                                continue;
                            }
                        }
                    }

                    // Handle calendar keyboard navigation if calendar is active
                    if let Some(ref mut calendar) = active_calendar {
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
                                active_calendar = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when calendar is active
                                continue;
                            }
                        }
                    }

                    // Handle help window keyboard events if help window is active
                    if active_help_window.is_some() {
                        match key_event.code {
                            KeyCode::Esc => {
                                // ESC dismisses the help window
                                active_help_window = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when help window is active
                                continue;
                            }
                        }
                    }

                    // Handle about window keyboard events if about window is active
                    if active_about_window.is_some() {
                        match key_event.code {
                            KeyCode::Esc => {
                                // ESC dismisses the about window
                                active_about_window = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when about window is active
                                continue;
                            }
                        }
                    }

                    // Handle config window keyboard events if config window is active
                    if active_config_window.is_some() {
                        match key_event.code {
                            KeyCode::Esc => {
                                // ESC dismisses the config window
                                active_config_window = None;
                                continue;
                            }
                            _ => {
                                // Ignore other keys when config window is active
                                continue;
                            }
                        }
                    }

                    // Handle ALT+TAB for window cycling
                    if key_event.code == KeyCode::Tab
                        && key_event.modifiers.contains(KeyModifiers::ALT)
                    {
                        window_manager.cycle_to_next_window();
                        continue;
                    }

                    // Handle CTRL+L to clear the terminal (like 'clear' command)
                    // Check for both Ctrl+L and the control character form
                    if key_event.code == KeyCode::Char('l')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        if current_focus != FocusState::Desktop {
                            // Send Ctrl+L (form feed, 0x0c) to the shell
                            // Most shells (bash, zsh, etc.) interpret this as "clear screen"
                            let _ = window_manager.send_to_focused("\x0c");
                        }
                        continue;
                    }

                    // Handle Ctrl+Shift+C to copy selection
                    if key_event.code == KeyCode::Char('C')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                        && key_event.modifiers.contains(KeyModifiers::SHIFT)
                    {
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

                    // Handle Ctrl+Shift+V to paste
                    if key_event.code == KeyCode::Char('V')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                        && key_event.modifiers.contains(KeyModifiers::SHIFT)
                    {
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
                                    active_prompt = Some(Prompt::new(
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
                                    active_prompt = Some(Prompt::new(
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
                                let help_message = "{C}KEYBOARD SHORTCUTS{W}\n\
                                    \n\
                                    {Y}'t'{W}       - Create new terminal window\n\
                                    {Y}'T'{W}       - Create new maximized terminal window\n\
                                    {Y}'q'/ESC{W}   - Exit application (from desktop)\n\
                                    {Y}'h'{W}       - Show this help screen\n\
                                    {Y}'l'{W}       - Show license and about information\n\
                                    {Y}'s'{W}       - Show settings/configuration window\n\
                                    {Y}'c'{W}       - Show calendar ({Y}\u{2190}\u{2192}{W} months, {Y}\u{2191}\u{2193}{W} years, {Y}t{W} today)\n\
                                    {Y}CTRL+L{W}    - Clear terminal (like 'clear' command)\n\
                                    {Y}ALT+TAB{W}   - Switch between windows\n\
                                    \n\
                                    {C}POPUP DIALOG CONTROLS{W}\n\
                                    \n\
                                    {Y}TAB/Arrow keys{W} - Navigate between buttons\n\
                                    {Y}ENTER{W}          - Activate selected button\n\
                                    {Y}ESC{W}            - Close dialog\n\
                                    \n\
                                    {C}MOUSE CONTROLS{W}\n\
                                    \n\
                                    {Y}Click title bar{W}     - Drag window\n\
                                    {Y}CTRL+Drag{W}          - Drag without snap\n\
                                    {Y}Click [X]{W}           - Close window\n\
                                    {Y}Drag ╬ handle{W}       - Resize window\n\
                                    {Y}Click window{W}        - Focus window\n\
                                    {Y}Click bottom bar{W}    - Switch windows";

                                active_help_window = Some(InfoWindow::new(
                                    "Help".to_string(),
                                    help_message,
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

                                active_about_window = Some(InfoWindow::new(
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
                                active_calendar = Some(CalendarState::new());
                            } else if current_focus != FocusState::Desktop {
                                // Send 'c' to terminal
                                let _ = window_manager.send_char_to_focused('c');
                            }
                        }
                        KeyCode::Char('s') => {
                            // Show settings/config window if desktop is focused
                            if current_focus == FocusState::Desktop {
                                let (cols, rows) = backend.dimensions();
                                active_config_window = Some(ConfigWindow::new(cols, rows));
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

                                // Center the window (ensuring y >= 1 to avoid overlapping top bar)
                                let x = (cols.saturating_sub(width)) / 2;
                                let y = ((rows.saturating_sub(height)) / 2).max(1);

                                window_manager.create_window(
                                    x,
                                    y,
                                    width,
                                    height,
                                    format!("Terminal {}", window_manager.window_count() + 1),
                                );

                                // Auto-position all windows based on the snap pattern
                                if auto_tiling_enabled {
                                    window_manager.auto_position_windows(cols, rows);
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

                                // Center the window (will be maximized immediately, ensuring y >= 1)
                                let x = (cols.saturating_sub(width)) / 2;
                                let y = ((rows.saturating_sub(height)) / 2).max(1);

                                let window_id = window_manager.create_window(
                                    x,
                                    y,
                                    width,
                                    height,
                                    format!("Terminal {}", window_manager.window_count() + 1),
                                );

                                // Maximize the newly created window
                                window_manager.maximize_window(window_id, cols, rows);
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
                    if let Some(ref prompt) = active_prompt {
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
                                        active_prompt = None;
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

                    // Check if there's an active config window (after prompt, before other events)
                    #[allow(clippy::collapsible_if)]
                    if !handled {
                        if let Some(ref config_win) = active_config_window {
                            if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                                let action =
                                    config_win.handle_click(mouse_event.column, mouse_event.row);
                                match action {
                                    ConfigAction::Close => {
                                        active_config_window = None;
                                        handled = true;
                                    }
                                    ConfigAction::ToggleAutoTiling => {
                                        app_config.toggle_auto_tiling_on_startup();
                                        // Update runtime state to match config
                                        auto_tiling_enabled = app_config.auto_tiling_on_startup;
                                        // Update button text
                                        let (_, rows) = backend.dimensions();
                                        let auto_tiling_text = if auto_tiling_enabled {
                                            "█ on] Auto Tiling"
                                        } else {
                                            "off ░] Auto Tiling"
                                        };
                                        auto_tiling_button =
                                            Button::new(1, rows - 1, auto_tiling_text.to_string());
                                        handled = true;
                                    }
                                    ConfigAction::ToggleShowDate => {
                                        app_config.toggle_show_date_in_clock();
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
                                        handled = true;
                                    }
                                    ConfigAction::CycleBackgroundChar => {
                                        // Cycle to the next background character
                                        app_config.cycle_background_char();
                                        // Update charset with new background character
                                        charset.set_background(app_config.get_background_char());
                                        handled = true;
                                    }
                                    ConfigAction::ToggleTintTerminal => {
                                        // Toggle terminal tinting and save
                                        app_config.toggle_tint_terminal();
                                        tint_terminal = app_config.tint_terminal;
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
                        if new_terminal_button.contains(mouse_event.column, mouse_event.row) {
                            new_terminal_button.set_state(button::ButtonState::Hovered);
                        } else {
                            new_terminal_button.set_state(button::ButtonState::Normal);
                        }

                        // Clipboard buttons hover state
                        if paste_button.contains(mouse_event.column, mouse_event.row) {
                            paste_button.set_state(button::ButtonState::Hovered);
                        } else {
                            paste_button.set_state(button::ButtonState::Normal);
                        }

                        if clear_clipboard_button.contains(mouse_event.column, mouse_event.row) {
                            clear_clipboard_button.set_state(button::ButtonState::Hovered);
                        } else {
                            clear_clipboard_button.set_state(button::ButtonState::Normal);
                        }

                        if copy_button.contains(mouse_event.column, mouse_event.row) {
                            copy_button.set_state(button::ButtonState::Hovered);
                        } else {
                            copy_button.set_state(button::ButtonState::Normal);
                        }

                        if clear_selection_button.contains(mouse_event.column, mouse_event.row) {
                            clear_selection_button.set_state(button::ButtonState::Hovered);
                        } else {
                            clear_selection_button.set_state(button::ButtonState::Normal);
                        }

                        // Calculate position for toggle button hover detection (bottom bar, left side)
                        let (_, rows) = backend.dimensions();
                        let bar_y = rows - 1;
                        let button_start_x = 1u16;
                        let button_text_width = auto_tiling_button.label.len() as u16 + 3; // +1 for "[", +1 for label, +1 for " "
                        let button_end_x = button_start_x + button_text_width;

                        if mouse_event.row == bar_y
                            && mouse_event.column >= button_start_x
                            && mouse_event.column < button_end_x
                        {
                            auto_tiling_button.set_state(button::ButtonState::Hovered);
                        } else {
                            auto_tiling_button.set_state(button::ButtonState::Normal);
                        }
                    }

                    // Check if click is on the New Terminal button in the top bar (only if no prompt)
                    if !handled
                        && active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && new_terminal_button.contains(mouse_event.column, mouse_event.row)
                    {
                        new_terminal_button.set_state(button::ButtonState::Pressed);

                        // Create a new terminal window (same as pressing 't')
                        let (cols, rows) = backend.dimensions();
                        let width = 150;
                        let height = 50;
                        let x = (cols.saturating_sub(width)) / 2;
                        let y = ((rows.saturating_sub(height)) / 2).max(1);

                        window_manager.create_window(
                            x,
                            y,
                            width,
                            height,
                            format!("Terminal {}", window_manager.window_count() + 1),
                        );

                        // Auto-position all windows based on the snap pattern
                        if auto_tiling_enabled {
                            window_manager.auto_position_windows(cols, rows);
                        }

                        handled = true;
                    }

                    // Check if click is on the Copy button in the top bar (only if no prompt)
                    if !handled
                        && active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && copy_button.contains(mouse_event.column, mouse_event.row)
                    {
                        copy_button.set_state(button::ButtonState::Pressed);

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
                        && active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && clear_selection_button.contains(mouse_event.column, mouse_event.row)
                    {
                        clear_selection_button.set_state(button::ButtonState::Pressed);

                        // Clear the selection
                        if let FocusState::Window(window_id) = window_manager.get_focus() {
                            window_manager.clear_selection(window_id);
                        }

                        handled = true;
                    }

                    // Check if click is on the Paste button in the top bar (only if no prompt)
                    if !handled
                        && active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && paste_button.contains(mouse_event.column, mouse_event.row)
                    {
                        paste_button.set_state(button::ButtonState::Pressed);

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
                        && active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && clear_clipboard_button.contains(mouse_event.column, mouse_event.row)
                    {
                        clear_clipboard_button.set_state(button::ButtonState::Pressed);

                        // Clear the clipboard
                        clipboard_manager.clear();

                        handled = true;
                    }

                    // Check if click is on the clock in the top bar (only if no prompt)
                    if !handled
                        && active_prompt.is_none()
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
                            active_calendar = Some(CalendarState::new());
                            handled = true;
                        }
                    }

                    // Check if click is on the Auto-Tiling toggle button (only if no prompt)
                    if !handled
                        && active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                    {
                        // Calculate position for toggle button click detection (bottom bar, left side)
                        let (_, rows) = backend.dimensions();
                        let bar_y = rows - 1;
                        let button_start_x = 1u16;
                        let button_text_width = auto_tiling_button.label.len() as u16 + 3; // +1 for "[", +1 for label, +1 for " "
                        let button_end_x = button_start_x + button_text_width;

                        if mouse_event.row == bar_y
                            && mouse_event.column >= button_start_x
                            && mouse_event.column < button_end_x
                        {
                            auto_tiling_button.set_state(button::ButtonState::Pressed);

                            // Toggle the auto-tiling state
                            auto_tiling_enabled = !auto_tiling_enabled;

                            // Update button label to reflect new state
                            let new_label = if auto_tiling_enabled {
                                "█ on] Auto Tiling".to_string()
                            } else {
                                "off ░] Auto Tiling".to_string()
                            };
                            auto_tiling_button = Button::new(1, bar_y, new_label);

                            handled = true;
                        }
                    }

                    // Check if click is on button bar (only if no prompt)
                    if !handled
                        && active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                    {
                        // Calculate where window buttons start (after auto-tiling button)
                        // Format: "[label ]" + 2 spaces
                        let window_buttons_start =
                            1 + 1 + auto_tiling_button.label.len() as u16 + 1 + 2;

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
                        && active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Right)
                    {
                        if let FocusState::Window(_) = window_manager.get_focus() {
                            context_menu.show(mouse_event.column, mouse_event.row);
                            handled = true;
                        }
                    }

                    // Handle context menu interactions
                    if !handled && context_menu.visible {
                        if mouse_event.kind == MouseEventKind::Down(MouseButton::Left) {
                            if context_menu.contains_point(mouse_event.column, mouse_event.row) {
                                if let Some(action) = context_menu.get_selected_action() {
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
                                context_menu.hide();
                                handled = true;
                            } else {
                                // Clicked outside menu - hide it
                                context_menu.hide();
                            }
                        } else if mouse_event.kind == MouseEventKind::Moved {
                            // Update menu selection on hover
                            if context_menu.contains_point(mouse_event.column, mouse_event.row) {
                                // TODO: Update hover state if needed
                            }
                        }
                    }

                    // Handle selection events (left-click, drag)
                    if !handled && active_prompt.is_none() && !context_menu.visible {
                        match mouse_event.kind {
                            MouseEventKind::Down(MouseButton::Left) => {
                                // Check if click is in a window content area
                                if let FocusState::Window(window_id) = window_manager.get_focus() {
                                    // Track click timing for double/triple-click detection
                                    let now = Instant::now();
                                    let is_multi_click = if let Some(last_time) = last_click_time {
                                        now.duration_since(last_time).as_millis() < 500
                                    } else {
                                        false
                                    };

                                    if is_multi_click {
                                        click_count += 1;
                                    } else {
                                        click_count = 1;
                                    }
                                    last_click_time = Some(now);

                                    // Start or expand selection based on click count
                                    let selection_type = match click_count {
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

                                    if click_count <= 1 || selection_type == SelectionType::Block {
                                        selection_active = true;
                                    }
                                }
                            }
                            MouseEventKind::Drag(MouseButton::Left) => {
                                if selection_active {
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
                                if selection_active {
                                    if let FocusState::Window(window_id) =
                                        window_manager.get_focus()
                                    {
                                        window_manager.complete_selection(window_id);
                                    }
                                    selection_active = false;
                                }
                            }
                            _ => {}
                        }
                    }

                    // If not handled by buttons, let window manager handle it (only if no prompt)
                    if !handled && active_prompt.is_none() && !context_menu.visible {
                        let window_closed =
                            window_manager.handle_mouse_event(&mut video_buffer, mouse_event);
                        // Auto-reposition remaining windows if a window was closed
                        if window_closed && auto_tiling_enabled {
                            let (cols, rows) = backend.dimensions();
                            window_manager.auto_position_windows(cols, rows);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Cleanup: restore terminal
    cleanup(&mut stdout)?;

    Ok(())
}

fn render_background(buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
    let (cols, rows) = buffer.dimensions();

    // Use the background character from charset configuration
    let background_cell = Cell::new(charset.background, theme.desktop_fg, theme.desktop_bg);

    // Fill entire screen with the background character
    for y in 0..rows {
        for x in 0..cols {
            buffer.set(x, y, background_cell);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_top_bar(
    buffer: &mut VideoBuffer,
    focus: FocusState,
    new_terminal_button: &Button,
    paste_button: &Button,
    clear_clipboard_button: &Button,
    copy_button: &Button,
    clear_selection_button: &Button,
    app_config: &AppConfig,
    theme: &Theme,
) {
    let (cols, _rows) = buffer.dimensions();

    // Change background color based on focus
    let bg_color = match focus {
        FocusState::Desktop => theme.topbar_bg_desktop,
        FocusState::Window(_) => theme.topbar_bg_window,
    };

    let bar_cell = Cell::new(' ', theme.topbar_fg, bg_color);

    // Create a blank top bar
    for x in 0..cols {
        buffer.set(x, 0, bar_cell);
    }

    // Left section - New Terminal button (always visible)
    new_terminal_button.render(buffer, theme);

    // Center section - Copy/Paste/Clear buttons (visible based on state)
    copy_button.render(buffer, theme);
    clear_selection_button.render(buffer, theme);
    paste_button.render(buffer, theme);
    clear_clipboard_button.render(buffer, theme);

    // Right section - Clock with dark background
    let now = Local::now();

    // Format clock based on configuration
    let time_str = if app_config.show_date_in_clock {
        // Show date and time: "Tue Nov 11, 09:21"
        now.format("%a %b %d, %H:%M").to_string()
    } else {
        // Show time only with seconds: "09:21:45"
        now.format("%H:%M:%S").to_string()
    };

    // Format: "| Tue Nov 11, 09:21 " or "| 09:21:45 " (with separator and trailing space)
    let clock_with_separator = format!("| {} ", time_str);
    let clock_width = clock_with_separator.len() as u16;
    let time_pos = cols.saturating_sub(clock_width);

    // Render clock with dark background
    for (i, ch) in clock_with_separator.chars().enumerate() {
        buffer.set(
            time_pos + i as u16,
            0,
            Cell::new(ch, theme.clock_fg, theme.clock_bg),
        );
    }
}

fn show_splash_screen(
    buffer: &mut VideoBuffer,
    backend: &mut Box<dyn RenderBackend>,
    charset: &Charset,
    theme: &Theme,
) -> io::Result<()> {
    let (cols, rows) = buffer.dimensions();

    // Clear screen to black
    let black_cell = Cell::new(' ', theme.splash_fg, Color::Black);
    for y in 0..rows {
        for x in 0..cols {
            buffer.set(x, y, black_cell);
        }
    }

    // Choose ASCII art based on charset mode
    let ascii_art = match charset.mode {
        charset::CharsetMode::Unicode => vec![
            " ███████████ ██████████ ███████████   ██████   ██████  ████████   ████████ ",
            "░█░░░███░░░█░░███░░░░░█░░███░░░░░███ ░░██████ ██████  ███░░░░███ ███░░░░███",
            "░   ░███  ░  ░███  █ ░  ░███    ░███  ░███░█████░███ ░░░    ░███░███   ░███",
            "    ░███     ░██████    ░██████████   ░███░░███ ░███    ██████░ ░░█████████",
            "    ░███     ░███░░█    ░███░░░░░███  ░███ ░░░  ░███   ░░░░░░███ ░░░░░░░███",
            "    ░███     ░███ ░   █ ░███    ░███  ░███      ░███  ███   ░███ ███   ░███",
            "    █████    ██████████ █████   █████ █████     █████░░████████ ░░████████ ",
            "   ░░░░░    ░░░░░░░░░░ ░░░░░   ░░░░░ ░░░░░     ░░░░░  ░░░░░░░░   ░░░░░░░░  ",
        ],
        charset::CharsetMode::Ascii => vec![
            "TTTTTTT EEEEEEE RRRRRR  M     M  333333   999999 ",
            "  TTT   EE      RR   RR MM   MM       33 99   99",
            "  TTT   EEEEE   RRRRRR  M M M M  333333  9999999",
            "  TTT   EE      RR  RR  M  M  M       33      99",
            "  TTT   EEEEEEE RR   RR M     M  333333   99999 ",
        ],
    };

    // License information to display below ASCII art
    let license_lines = [
        "",
        &format!("Version {}", config::VERSION),
        "MIT License",
        &format!("Copyright (c) 2025 {}", config::AUTHORS),
    ];

    // Calculate window dimensions
    let art_width = ascii_art[0].chars().count() as u16;
    let art_height = ascii_art.len() as u16;
    let license_height = license_lines.len() as u16;
    let total_content_height = art_height + license_height;

    let window_width = art_width + 6; // 2 for borders + 4 for padding
    let window_height = total_content_height + 4; // Top border + padding + content + padding + bottom border

    // Center the window
    let window_x = (cols.saturating_sub(window_width)) / 2;
    let window_y = (rows.saturating_sub(window_height)) / 2;

    let border_color = theme.splash_border;
    let content_bg = theme.splash_bg;

    // Draw top border using charset
    buffer.set(
        window_x,
        window_y,
        Cell::new(charset.border_top_left, border_color, content_bg),
    );
    for x in 1..window_width - 1 {
        buffer.set(
            window_x + x,
            window_y,
            Cell::new(charset.border_horizontal, border_color, content_bg),
        );
    }
    buffer.set(
        window_x + window_width - 1,
        window_y,
        Cell::new(charset.border_top_right, border_color, content_bg),
    );

    // Draw middle rows (content area)
    for y in 1..window_height - 1 {
        // Left border
        buffer.set(
            window_x,
            window_y + y,
            Cell::new(charset.border_vertical, border_color, content_bg),
        );

        // Content
        for x in 1..window_width - 1 {
            buffer.set(
                window_x + x,
                window_y + y,
                Cell::new(' ', theme.splash_fg, content_bg),
            );
        }

        // Right border
        buffer.set(
            window_x + window_width - 1,
            window_y + y,
            Cell::new(charset.border_vertical, border_color, content_bg),
        );
    }

    // Draw bottom border using charset
    buffer.set(
        window_x,
        window_y + window_height - 1,
        Cell::new(charset.border_bottom_left, border_color, content_bg),
    );
    for x in 1..window_width - 1 {
        buffer.set(
            window_x + x,
            window_y + window_height - 1,
            Cell::new(charset.border_horizontal, border_color, content_bg),
        );
    }
    buffer.set(
        window_x + window_width - 1,
        window_y + window_height - 1,
        Cell::new(charset.border_bottom_right, border_color, content_bg),
    );

    // Draw shadow (right and bottom) using shared function
    video_buffer::render_shadow(
        buffer,
        window_x,
        window_y,
        window_width,
        window_height,
        charset,
        theme,
    );

    // Render ASCII art centered in the window
    let content_start_y = window_y + 2; // Start after top border and padding
    let content_x = window_x + 3; // Left padding

    for (i, line) in ascii_art.iter().enumerate() {
        for (j, ch) in line.chars().enumerate() {
            buffer.set(
                content_x + j as u16,
                content_start_y + i as u16,
                Cell::new(ch, theme.splash_fg, content_bg),
            );
        }
    }

    // Render license information below ASCII art
    let license_start_y = content_start_y + art_height;
    for (i, line) in license_lines.iter().enumerate() {
        // Center each license line
        let line_len = line.chars().count() as u16;
        let line_x = if line_len < art_width {
            content_x + (art_width - line_len) / 2
        } else {
            content_x
        };

        for (j, ch) in line.chars().enumerate() {
            buffer.set(
                line_x + j as u16,
                license_start_y + i as u16,
                Cell::new(ch, theme.splash_fg, content_bg),
            );
        }
    }

    // Present to screen
    backend.present(&mut *buffer)?;

    // Wait for 1 second
    thread::sleep(time::Duration::from_secs(1));

    Ok(())
}

fn render_button_bar(
    buffer: &mut VideoBuffer,
    window_manager: &WindowManager,
    auto_tiling_button: &Button,
    auto_tiling_enabled: bool,
    theme: &Theme,
) {
    let (cols, rows) = buffer.dimensions();
    let bar_y = rows - 1;

    // Fill button bar with black background
    let bar_cell = Cell::new(' ', theme.bottombar_fg, theme.bottombar_bg);
    for x in 0..cols {
        buffer.set(x, bar_y, bar_cell);
    }

    // Render Auto Tiling toggle on the left side
    let toggle_color = if auto_tiling_enabled {
        theme.toggle_enabled_fg
    } else {
        theme.toggle_disabled_fg
    };

    let toggle_bg = match auto_tiling_button.state {
        button::ButtonState::Normal => {
            if auto_tiling_enabled {
                theme.toggle_enabled_bg_normal
            } else {
                theme.toggle_disabled_bg_normal
            }
        }
        button::ButtonState::Hovered => {
            if auto_tiling_enabled {
                theme.toggle_enabled_bg_hovered
            } else {
                theme.toggle_disabled_bg_hovered
            }
        }
        button::ButtonState::Pressed => {
            if auto_tiling_enabled {
                theme.toggle_enabled_bg_pressed
            } else {
                theme.toggle_disabled_bg_pressed
            }
        }
    };

    let mut current_x = 1u16;

    // Render "[ "
    buffer.set(current_x, bar_y, Cell::new('[', toggle_color, toggle_bg));
    current_x += 1;

    // Render label
    for ch in auto_tiling_button.label.chars() {
        buffer.set(current_x, bar_y, Cell::new(ch, toggle_color, toggle_bg));
        current_x += 1;
    }

    // Render " ]"
    buffer.set(current_x, bar_y, Cell::new(' ', toggle_color, toggle_bg));
    current_x += 1;

    // Add spacing after toggle
    current_x += 2;

    // Render help text on the right side
    let help_text = " > 'h' Help | 's' Settings < ";
    let help_text_len = help_text.len() as u16;
    if cols > help_text_len {
        let help_x = cols - help_text_len - 1;
        for (i, ch) in help_text.chars().enumerate() {
            buffer.set(
                help_x + i as u16,
                bar_y,
                Cell::new(ch, theme.bottombar_fg, theme.bottombar_bg),
            );
        }
    }

    // Get list of windows
    let windows = window_manager.get_window_list();
    if windows.is_empty() {
        return;
    }

    for (_id, title, is_focused, is_minimized) in windows {
        // Max button width is 18 chars total
        let max_title_len = 14;
        let button_title = if title.len() > max_title_len {
            &title[..max_title_len]
        } else {
            title
        };

        // Button format: [ Title ] for normal, ( Title ) for minimized
        // Use different brackets and colors for minimized windows
        let (open_bracket, close_bracket, button_bg, button_fg) = if is_minimized {
            // Minimized windows: use parentheses and grey color
            (
                '(',
                ')',
                theme.bottombar_button_minimized_bg,
                theme.bottombar_button_minimized_fg,
            )
        } else if is_focused {
            // Focused window: cyan background
            (
                '[',
                ']',
                theme.bottombar_button_focused_bg,
                theme.bottombar_button_focused_fg,
            )
        } else {
            // Normal unfocused window: white text
            (
                '[',
                ']',
                theme.bottombar_button_normal_bg,
                theme.bottombar_button_normal_fg,
            )
        };

        // Render opening bracket and space
        buffer.set(
            current_x,
            bar_y,
            Cell::new(open_bracket, button_fg, button_bg),
        );
        current_x += 1;
        buffer.set(current_x, bar_y, Cell::new(' ', button_fg, button_bg));
        current_x += 1;

        // Render title
        for ch in button_title.chars() {
            if current_x >= cols - 1 {
                break;
            }
            buffer.set(current_x, bar_y, Cell::new(ch, button_fg, button_bg));
            current_x += 1;
        }

        // Render space and closing bracket
        if current_x < cols - 1 {
            buffer.set(current_x, bar_y, Cell::new(' ', button_fg, button_bg));
            current_x += 1;
        }
        if current_x < cols - 1 {
            buffer.set(
                current_x,
                bar_y,
                Cell::new(close_bracket, button_fg, button_bg),
            );
            current_x += 1;
        }

        // Add space between buttons
        current_x += 1;

        // Stop if we've run out of space
        if current_x >= cols - 1 {
            break;
        }
    }
}

fn render_calendar(
    buffer: &mut VideoBuffer,
    calendar: &CalendarState,
    charset: &Charset,
    theme: &Theme,
    cols: u16,
    rows: u16,
) {
    // Calendar dimensions
    let width = 42u16;
    let height = 18u16;
    let x = (cols.saturating_sub(width)) / 2;
    let y = (rows.saturating_sub(height)) / 2;

    // Get the first day of the month
    let first_day = match NaiveDate::from_ymd_opt(calendar.year, calendar.month, 1) {
        Some(date) => date,
        None => return, // Invalid date, don't render
    };

    // Get the number of days in the month
    let days_in_month = if calendar.month == 12 {
        match NaiveDate::from_ymd_opt(calendar.year + 1, 1, 1) {
            Some(next_month) => (next_month - chrono::Duration::days(1)).day(),
            None => 31,
        }
    } else {
        match NaiveDate::from_ymd_opt(calendar.year, calendar.month + 1, 1) {
            Some(next_month) => (next_month - chrono::Duration::days(1)).day(),
            None => 31,
        }
    };

    // Get the weekday of the first day (0 = Sunday, 6 = Saturday)
    let first_weekday = first_day.weekday().num_days_from_sunday() as u16;

    // Colors
    let bg_color = theme.calendar_bg;
    let fg_color = theme.calendar_fg;
    let title_color = theme.calendar_title_color;
    let today_bg = theme.calendar_today_bg;
    let today_fg = theme.calendar_today_fg;

    // Fill calendar background
    for cy in 0..height {
        for cx in 0..width {
            buffer.set(x + cx, y + cy, Cell::new(' ', fg_color, bg_color));
        }
    }

    // Render title (Month Year)
    let title = format!("{} {}", calendar.month_name(), calendar.year);
    let title_len = title.len() as u16;
    let title_x = if title_len < width {
        x + (width - title_len) / 2
    } else {
        x + 1
    };
    for (i, ch) in title.chars().enumerate() {
        let char_x = title_x + i as u16;
        if char_x < x + width {
            buffer.set(char_x, y + 1, Cell::new(ch, title_color, bg_color));
        }
    }

    // Render day headers (Su  Mo  Tu  We  Th  Fr  Sa)
    let day_headers = "Su   Mo   Tu   We   Th   Fr   Sa";
    let header_len = day_headers.len() as u16;
    let header_x = if header_len < width {
        x + (width - header_len) / 2
    } else {
        x + 1
    };
    for (i, ch) in day_headers.chars().enumerate() {
        let char_x = header_x + i as u16;
        if char_x < x + width {
            buffer.set(char_x, y + 3, Cell::new(ch, fg_color, bg_color));
        }
    }

    // Render calendar days
    let mut day = 1u32;
    let calendar_start_y = y + 5;

    for week in 0..6 {
        for weekday in 0..7 {
            let day_x = header_x + (weekday * 5);
            let day_y = calendar_start_y + (week * 2);

            if (week == 0 && weekday < first_weekday) || day > days_in_month {
                continue;
            }

            // Check if this is today
            let is_today = calendar.today.year() == calendar.year
                && calendar.today.month() == calendar.month
                && calendar.today.day() == day;

            let (day_fg, day_bg) = if is_today {
                (today_fg, today_bg)
            } else {
                (fg_color, bg_color)
            };

            // Render day number (right-aligned in 2-char space)
            let day_str = format!("{:>2}", day);
            for (i, ch) in day_str.chars().enumerate() {
                buffer.set(day_x + i as u16, day_y, Cell::new(ch, day_fg, day_bg));
            }

            day += 1;
        }
    }

    // Render navigation hints at bottom
    let hint = "\u{2190}\u{2192} Month | \u{2191}\u{2193} Year | T Today | ESC Close";
    let hint_len = hint.chars().count() as u16;
    let hint_x = if hint_len < width {
        x + (width - hint_len) / 2
    } else {
        x + 1
    };
    for (i, ch) in hint.chars().enumerate() {
        let char_x = hint_x + i as u16;
        if char_x < x + width {
            buffer.set(
                char_x,
                y + height - 1,
                Cell::new(ch, theme.config_instructions_fg, bg_color),
            );
        }
    }

    // Add shadow effect
    let shadow_char = charset.shadow;
    for sy in 1..height {
        buffer.set(
            x + width,
            y + sy,
            Cell::new(shadow_char, theme.window_shadow_color, Color::Black),
        );
    }
    for sx in 1..=width {
        buffer.set(
            x + sx,
            y + height,
            Cell::new(shadow_char, theme.window_shadow_color, Color::Black),
        );
    }
}

fn cleanup(stdout: &mut io::Stdout) -> io::Result<()> {
    // Disable mouse capture
    execute!(stdout, event::DisableMouseCapture)?;

    // Clear screen
    execute!(stdout, terminal::Clear(ClearType::All))?;

    // Show cursor
    execute!(stdout, cursor::Show)?;

    // Disable raw mode
    terminal::disable_raw_mode()?;

    Ok(())
}
