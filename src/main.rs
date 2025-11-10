#[macro_use]
mod debug_log;
mod ansi_handler;
mod button;
mod charset;
mod prompt;
mod term_grid;
mod terminal_emulator;
mod terminal_window;
mod video_buffer;
mod window;
mod window_manager;

use button::Button;
use charset::Charset;
use chrono::{Datelike, Local, NaiveDate};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind},
    execute,
    style::Color,
    terminal::{self, ClearType},
};
use prompt::{Prompt, PromptAction, PromptButton, PromptType};
use std::io::{self, Write};
use std::time::Duration;
use std::{thread, time};
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
    // Initialize debug logging
    debug_log::init_debug_log();
    debug_log!("Application started");

    // Parse command-line arguments for charset configuration
    let charset = Charset::from_args();
    debug_log!("Using charset mode: {:?}", charset.mode);

    let mut stdout = io::stdout();

    // Enter raw mode for low-level terminal control
    terminal::enable_raw_mode()?;

    // Hide cursor and enable mouse capture
    execute!(stdout, cursor::Hide, event::EnableMouseCapture)?;

    // Clear the screen
    execute!(stdout, terminal::Clear(ClearType::All))?;

    // Initialize video buffer
    let (cols, rows) = terminal::size()?;
    let mut video_buffer = VideoBuffer::new(cols, rows);

    // Initialize window manager
    let mut window_manager = WindowManager::new();

    // Create the "New Terminal" button
    let mut new_terminal_button = Button::new(1, 0, "+New Terminal".to_string());

    // Prompt state (None when no prompt is active)
    let mut active_prompt: Option<Prompt> = None;

    // Calendar state (None when calendar is not shown)
    let mut active_calendar: Option<CalendarState> = None;

    // Show splash screen for 1 second
    show_splash_screen(&mut video_buffer, &mut stdout, &charset)?;

    // Start with desktop focused - no windows yet
    // User can press 't' to create windows

    // Main loop
    loop {
        // Get current terminal size
        let (cols, rows) = terminal::size()?;

        // Render the background (every frame for consistency)
        render_background(&mut video_buffer, &charset);

        // Render the top bar
        let focus = window_manager.get_focus();
        render_top_bar(&mut video_buffer, focus, &new_terminal_button);

        // Render all windows
        window_manager.render_all(&mut video_buffer, &charset);

        // Render the button bar
        render_button_bar(&mut video_buffer, &window_manager);

        // Render active prompt (if any) on top of everything
        if let Some(ref prompt) = active_prompt {
            prompt.render(&mut video_buffer, &charset);
        }

        // Render active calendar (if any) on top of everything
        if let Some(ref calendar) = active_calendar {
            render_calendar(&mut video_buffer, calendar, &charset, cols, rows);
        }

        // Present buffer to screen
        video_buffer.present(&mut stdout)?;
        stdout.flush()?;

        // Check for input (non-blocking with ~60fps)
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key_event) => {
                    debug_log!("Key event: {:?}", key_event);
                    let current_focus = window_manager.get_focus();
                    debug_log!("Current focus: {:?}", current_focus);

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

                    // Handle ALT+TAB for window cycling
                    if key_event.code == KeyCode::Tab
                        && key_event.modifiers.contains(KeyModifiers::ALT)
                    {
                        debug_log!("ALT+TAB detected - cycling to next window");
                        window_manager.cycle_to_next_window();
                        continue;
                    }

                    match key_event.code {
                        KeyCode::Esc => {
                            // ESC exits only from desktop (prompts are handled above)
                            if current_focus == FocusState::Desktop {
                                // If windows are open, show confirmation
                                if window_manager.window_count() > 0 {
                                    let (cols, rows) = terminal::size()?;
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
                                    let (cols, rows) = terminal::size()?;
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
                                let (cols, rows) = terminal::size()?;
                                let help_message = "KEYBOARD SHORTCUTS\n\
                                    \n\
                                    't'       - Create new terminal window\n\
                                    'T'       - Create new maximized terminal window\n\
                                    'q'/ESC   - Exit application (from desktop)\n\
                                    'h'       - Show this help screen\n\
                                    'l'       - Show license and about information\n\
                                    'c'       - Show calendar (\u{2190}\u{2192} months, \u{2191}\u{2193} years, t today)\n\
                                    ALT+TAB   - Switch between windows\n\
                                    \n\
                                    POPUP DIALOG CONTROLS\n\
                                    \n\
                                    TAB/Arrow keys - Navigate between buttons\n\
                                    ENTER          - Activate selected button\n\
                                    ESC            - Close dialog\n\
                                    \n\
                                    MOUSE CONTROLS\n\
                                    \n\
                                    Click title bar  - Drag window\n\
                                    Click [X]        - Close window\n\
                                    Drag ╬ handle    - Resize window\n\
                                    Click window     - Focus window\n\
                                    Click bottom bar - Switch windows";

                                active_prompt = Some(Prompt::new(
                                    PromptType::Success,
                                    help_message.to_string(),
                                    vec![PromptButton::new(
                                        "Close".to_string(),
                                        PromptAction::Cancel,
                                        true,
                                    )],
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
                                let (cols, rows) = terminal::size()?;
                                let license_message = "TERM39 - Terminal UI Windows Manager\n\
                                    \n\
                                    A low-level terminal UI windows manager built with Rust.\n\
                                    \n\
                                    Version: 0.1.0\n\
                                    Author: Alejandro Quintanar\n\
                                    Repository: https://github.com/alejandroqh/term39\n\
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
                                    All dependencies are used under their respective licenses.";

                                active_prompt = Some(Prompt::new(
                                    PromptType::Info,
                                    license_message.to_string(),
                                    vec![PromptButton::new(
                                        "Close".to_string(),
                                        PromptAction::Cancel,
                                        true,
                                    )],
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
                        KeyCode::Char('t') => {
                            // Only create new window if desktop is focused
                            if current_focus == FocusState::Desktop {
                                debug_log!("Creating new terminal window");
                                // Create a new terminal window
                                let (cols, rows) = terminal::size()?;

                                // Window size: 2.5x larger (60*2.5=150, 20*2.5=50)
                                let width = 150;
                                let height = 50;

                                // Center the window
                                let x = (cols.saturating_sub(width)) / 2;
                                let y = (rows.saturating_sub(height)) / 2;

                                window_manager.create_window(
                                    x,
                                    y,
                                    width,
                                    height,
                                    format!("Terminal {}", window_manager.window_count() + 1),
                                );
                                debug_log!("Terminal window created");
                            } else {
                                // Send 't' to terminal
                                debug_log!("Sending 't' to terminal");
                                let _ = window_manager.send_char_to_focused('t');
                            }
                        }
                        KeyCode::Char('T') => {
                            // Only create maximized window if desktop is focused
                            if current_focus == FocusState::Desktop {
                                debug_log!("Creating new maximized terminal window");
                                // Create a new terminal window
                                let (cols, rows) = terminal::size()?;

                                // Window size: 2.5x larger (60*2.5=150, 20*2.5=50)
                                let width = 150;
                                let height = 50;

                                // Center the window (will be maximized immediately)
                                let x = (cols.saturating_sub(width)) / 2;
                                let y = (rows.saturating_sub(height)) / 2;

                                let window_id = window_manager.create_window(
                                    x,
                                    y,
                                    width,
                                    height,
                                    format!("Terminal {}", window_manager.window_count() + 1),
                                );
                                debug_log!(
                                    "Maximized terminal window created with ID: {}",
                                    window_id
                                );

                                // Maximize the newly created window
                                window_manager.maximize_window(window_id, cols, rows);
                            } else {
                                // Send 'T' to terminal
                                debug_log!("Sending 'T' to terminal");
                                let _ = window_manager.send_char_to_focused('T');
                            }
                        }
                        KeyCode::Char(c) => {
                            // Send character to focused terminal
                            if current_focus != FocusState::Desktop {
                                debug_log!("Sending char: {:?}", c);
                                let result = window_manager.send_char_to_focused(c);
                                debug_log!("Send char result: {:?}", result);
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
                Event::Mouse(mouse_event) => {
                    let (_, rows) = terminal::size()?;
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

                    // Update button hover state on mouse movement (always active)
                    if !handled {
                        if new_terminal_button.contains(mouse_event.column, mouse_event.row) {
                            new_terminal_button.set_state(button::ButtonState::Hovered);
                        } else {
                            new_terminal_button.set_state(button::ButtonState::Normal);
                        }
                    }

                    // Check if click is on the New Terminal button in the top bar (only if no prompt)
                    if !handled
                        && active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                        && new_terminal_button.contains(mouse_event.column, mouse_event.row)
                    {
                        debug_log!("New Terminal button clicked");
                        new_terminal_button.set_state(button::ButtonState::Pressed);

                        // Create a new terminal window (same as pressing 't')
                        let (cols, rows) = terminal::size()?;
                        let width = 150;
                        let height = 50;
                        let x = (cols.saturating_sub(width)) / 2;
                        let y = (rows.saturating_sub(height)) / 2;

                        window_manager.create_window(
                            x,
                            y,
                            width,
                            height,
                            format!("Terminal {}", window_manager.window_count() + 1),
                        );
                        handled = true;
                    }

                    // Check if click is on button bar (only if no prompt)
                    if !handled
                        && active_prompt.is_none()
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                    {
                        handled = window_manager
                            .button_bar_click(mouse_event.column, bar_y, mouse_event.row)
                            .is_some();
                    }

                    // If not handled by buttons, let window manager handle it (only if no prompt)
                    if !handled && active_prompt.is_none() {
                        window_manager.handle_mouse_event(&mut video_buffer, mouse_event);
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

fn render_background(buffer: &mut VideoBuffer, charset: &Charset) {
    let (cols, rows) = buffer.dimensions();

    // Use the background character from charset configuration
    let background_cell = Cell::new(charset.background, Color::White, Color::Blue);

    // Fill entire screen with the background character
    for y in 0..rows {
        for x in 0..cols {
            buffer.set(x, y, background_cell);
        }
    }
}

fn render_top_bar(buffer: &mut VideoBuffer, focus: FocusState, new_terminal_button: &Button) {
    let (cols, _rows) = buffer.dimensions();

    // Change background color based on focus
    let bg_color = match focus {
        FocusState::Desktop => Color::Cyan,
        FocusState::Window(_) => Color::Black,
    };

    let bar_cell = Cell::new(' ', Color::White, bg_color);

    // Create a blank top bar
    for x in 0..cols {
        buffer.set(x, 0, bar_cell);
    }

    // Left section - New Terminal button (always visible)
    new_terminal_button.render(buffer);

    // Right section - Clock
    let now = Local::now();
    let time_str = now.format("%H:%M:%S").to_string();
    let time_pos = cols.saturating_sub(time_str.len() as u16 + 1);

    for (i, ch) in time_str.chars().enumerate() {
        buffer.set(
            time_pos + i as u16,
            0,
            Cell::new(ch, Color::White, bg_color),
        );
    }
}

fn show_splash_screen(
    buffer: &mut VideoBuffer,
    stdout: &mut io::Stdout,
    charset: &Charset,
) -> io::Result<()> {
    let (cols, rows) = buffer.dimensions();

    // Clear screen to black
    let black_cell = Cell::new(' ', Color::White, Color::Black);
    for y in 0..rows {
        for x in 0..cols {
            buffer.set(x, y, black_cell);
        }
    }

    // Choose ASCII art based on charset mode
    let ascii_art = match charset.mode {
        charset::CharsetMode::Unicode => vec![
            "████████╗███████╗██████╗ ███╗   ███╗██████╗  █████╗ ",
            "╚══██╔══╝██╔════╝██╔══██╗████╗ ████║╚════██╗██╔══██╗",
            "   ██║   █████╗  ██████╔╝██╔████╔██║ █████╔╝╚██████║",
            "   ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║ ╚═══██╗ ╚═══██║",
            "   ██║   ███████╗██║  ██║██║ ╚═╝ ██║██████╔╝ █████╔╝",
            "   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═════╝  ╚════╝ ",
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
    let license_lines = ["", "MIT License", "Copyright (c) 2025 Alejandro Quintanar"];

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

    let border_color = Color::White;
    let content_bg = Color::DarkBlue;

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
                Cell::new(' ', Color::White, content_bg),
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

    // Draw shadow (right and bottom) using charset
    let shadow_char = charset.shadow;
    let shadow_color = Color::DarkGrey;

    // Right shadow
    for y in 1..=window_height {
        if window_y + y < rows {
            buffer.set(
                window_x + window_width,
                window_y + y,
                Cell::new(shadow_char, shadow_color, shadow_color),
            );
        }
    }

    // Bottom shadow
    for x in 1..=window_width {
        if window_x + x < cols {
            buffer.set(
                window_x + x,
                window_y + window_height,
                Cell::new(shadow_char, shadow_color, shadow_color),
            );
        }
    }

    // Render ASCII art centered in the window
    let content_start_y = window_y + 2; // Start after top border and padding
    let content_x = window_x + 3; // Left padding

    for (i, line) in ascii_art.iter().enumerate() {
        for (j, ch) in line.chars().enumerate() {
            buffer.set(
                content_x + j as u16,
                content_start_y + i as u16,
                Cell::new(ch, Color::Yellow, content_bg),
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
                Cell::new(ch, Color::White, content_bg),
            );
        }
    }

    // Present to screen
    buffer.present(stdout)?;
    stdout.flush()?;

    // Wait for 1 second
    thread::sleep(time::Duration::from_secs(1));

    Ok(())
}

fn render_button_bar(buffer: &mut VideoBuffer, window_manager: &WindowManager) {
    let (cols, rows) = buffer.dimensions();
    let bar_y = rows - 1;

    // Fill button bar with black background
    let bar_cell = Cell::new(' ', Color::White, Color::Black);
    for x in 0..cols {
        buffer.set(x, bar_y, bar_cell);
    }

    // Render help text on the right side
    let help_text = " > Press 'h' for Help < ";
    let help_text_len = help_text.len() as u16;
    if cols > help_text_len {
        let help_x = cols - help_text_len - 1;
        for (i, ch) in help_text.chars().enumerate() {
            buffer.set(
                help_x + i as u16,
                bar_y,
                Cell::new(ch, Color::White, Color::Black),
            );
        }
    }

    // Get list of windows
    let windows = window_manager.get_window_list();
    if windows.is_empty() {
        return;
    }

    let mut current_x = 1u16; // Start at position 1

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
            ('(', ')', Color::Black, Color::DarkGrey)
        } else if is_focused {
            // Focused window: cyan background
            ('[', ']', Color::Cyan, Color::Black)
        } else {
            // Normal unfocused window: white text
            ('[', ']', Color::Black, Color::White)
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
    let bg_color = Color::Blue;
    let fg_color = Color::White;
    let title_color = Color::White;
    let today_bg = Color::Cyan;
    let today_fg = Color::Black;

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
                Cell::new(ch, Color::DarkGrey, bg_color),
            );
        }
    }

    // Add shadow effect
    let shadow_char = charset.shadow;
    for sy in 1..height {
        buffer.set(
            x + width,
            y + sy,
            Cell::new(shadow_char, Color::DarkGrey, Color::Black),
        );
    }
    for sx in 1..=width {
        buffer.set(
            x + sx,
            y + height,
            Cell::new(shadow_char, Color::DarkGrey, Color::Black),
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
