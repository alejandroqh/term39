use crate::button::{Button, ButtonState};
use crate::charset::Charset;
use crate::config_manager::AppConfig;
use crate::theme::Theme;
use crate::video_buffer::{Cell, VideoBuffer};
use crate::window_manager::{FocusState, WindowManager};
use battery::Manager;
use chrono::{Datelike, Local, NaiveDate};
use crossterm::style::Color;

/// Get the current battery percentage (0-100) or None if no battery is available
fn get_battery_percentage() -> Option<u8> {
    let manager = Manager::new().ok()?;
    let mut batteries = manager.batteries().ok()?;
    let battery = batteries.next()?.ok()?;

    let percentage = battery.state_of_charge().value * 100.0;
    Some(percentage.round() as u8)
}

/// Get the color for the battery indicator based on charge level
fn get_battery_color(percentage: u8) -> Color {
    if percentage > 40 {
        Color::White
    } else if percentage >= 20 {
        Color::DarkGrey
    } else {
        Color::Red
    }
}

// Calendar state structure
pub struct CalendarState {
    year: i32,
    month: u32,
    today: NaiveDate,
}

impl CalendarState {
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            year: today.year(),
            month: today.month(),
            today,
        }
    }

    pub fn next_month(&mut self) {
        if self.month == 12 {
            self.month = 1;
            self.year += 1;
        } else {
            self.month += 1;
        }
    }

    pub fn previous_month(&mut self) {
        if self.month == 1 {
            self.month = 12;
            self.year -= 1;
        } else {
            self.month -= 1;
        }
    }

    pub fn next_year(&mut self) {
        self.year += 1;
    }

    pub fn previous_year(&mut self) {
        self.year -= 1;
    }

    pub fn reset_to_today(&mut self) {
        self.year = self.today.year();
        self.month = self.today.month();
    }

    pub fn month_name(&self) -> &'static str {
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

impl Default for CalendarState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_background(buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
    let (cols, rows) = buffer.dimensions();

    // Use the background character from charset configuration
    // Use new_unchecked for performance - theme colors are pre-validated
    let background_cell =
        Cell::new_unchecked(charset.background, theme.desktop_fg, theme.desktop_bg);

    // Fill entire screen with the background character
    for y in 0..rows {
        for x in 0..cols {
            buffer.set(x, y, background_cell);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_top_bar(
    buffer: &mut VideoBuffer,
    focus: FocusState,
    new_terminal_button: &Button,
    paste_button: &Button,
    clear_clipboard_button: &Button,
    copy_button: &Button,
    clear_selection_button: &Button,
    exit_button: &Button,
    app_config: &AppConfig,
    theme: &Theme,
) {
    let (cols, _rows) = buffer.dimensions();

    // Change background color based on focus
    let bg_color = match focus {
        FocusState::Desktop => theme.topbar_bg_desktop,
        FocusState::Window(_) => theme.topbar_bg_window,
    };

    // Use new_unchecked for performance - theme colors are pre-validated
    let bar_cell = Cell::new_unchecked(' ', theme.topbar_fg, bg_color);

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

    // Right section - Exit button (before clock)
    exit_button.render(buffer, theme);

    // Right section - Battery indicator and Clock with dark background
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

    // Get battery percentage and create block bar indicator
    let battery_percentage = get_battery_percentage();
    let battery_width = if battery_percentage.is_some() { 10u16 } else { 0u16 }; // "| [█████] "

    // Calculate positions (battery comes before clock)
    let total_width = battery_width + clock_width;
    let start_pos = cols.saturating_sub(total_width);

    // Render battery indicator with block bar
    if let Some(pct) = battery_percentage {
        let battery_color = get_battery_color(pct);
        let filled_blocks = ((pct as f32 / 20.0).round() as usize).min(5);

        // Format: "| [█████] " (10 chars)
        // Positions: 0='|', 1=' ', 2='[', 3-7=blocks, 8=']', 9=' '
        let prefix = "| [";
        let suffix = "] ";

        // Render prefix with clock colors
        for (i, ch) in prefix.chars().enumerate() {
            buffer.set(
                start_pos + i as u16,
                0,
                Cell::new_unchecked(ch, theme.clock_fg, theme.clock_bg),
            );
        }

        // Render battery blocks
        let block_start = start_pos + prefix.len() as u16;
        for i in 0..5 {
            let (ch, fg) = if i < filled_blocks {
                ('█', battery_color) // Filled block with battery color
            } else {
                ('░', Color::DarkGrey) // Empty block
            };
            buffer.set(
                block_start + i as u16,
                0,
                Cell::new_unchecked(ch, fg, theme.clock_bg),
            );
        }

        // Render suffix with clock colors
        let suffix_start = block_start + 5;
        for (i, ch) in suffix.chars().enumerate() {
            buffer.set(
                suffix_start + i as u16,
                0,
                Cell::new_unchecked(ch, theme.clock_fg, theme.clock_bg),
            );
        }
    }

    // Render clock with dark background
    let clock_pos = start_pos + battery_width;
    // Use new_unchecked for performance - theme colors are pre-validated
    for (i, ch) in clock_with_separator.chars().enumerate() {
        buffer.set(
            clock_pos + i as u16,
            0,
            Cell::new_unchecked(ch, theme.clock_fg, theme.clock_bg),
        );
    }
}

pub fn render_button_bar(
    buffer: &mut VideoBuffer,
    window_manager: &WindowManager,
    auto_tiling_button: &Button,
    auto_tiling_enabled: bool,
    theme: &Theme,
) {
    let (cols, rows) = buffer.dimensions();
    let bar_y = rows - 1;

    // Fill button bar with black background
    // Use new_unchecked for performance - theme colors are pre-validated
    let bar_cell = Cell::new_unchecked(' ', theme.bottombar_fg, theme.bottombar_bg);
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
        ButtonState::Normal => {
            if auto_tiling_enabled {
                theme.toggle_enabled_bg_normal
            } else {
                theme.toggle_disabled_bg_normal
            }
        }
        ButtonState::Hovered => {
            if auto_tiling_enabled {
                theme.toggle_enabled_bg_hovered
            } else {
                theme.toggle_disabled_bg_hovered
            }
        }
        ButtonState::Pressed => {
            if auto_tiling_enabled {
                theme.toggle_enabled_bg_pressed
            } else {
                theme.toggle_disabled_bg_pressed
            }
        }
    };

    let mut current_x = 1u16;

    // Render "[ "
    // Use new_unchecked for performance - theme colors are pre-validated
    buffer.set(
        current_x,
        bar_y,
        Cell::new_unchecked('[', toggle_color, toggle_bg),
    );
    current_x += 1;

    // Render label
    for ch in auto_tiling_button.label.chars() {
        buffer.set(
            current_x,
            bar_y,
            Cell::new_unchecked(ch, toggle_color, toggle_bg),
        );
        current_x += 1;
    }

    // Render " ]"
    buffer.set(
        current_x,
        bar_y,
        Cell::new_unchecked(' ', toggle_color, toggle_bg),
    );
    current_x += 1;

    // Add spacing after toggle
    current_x += 2;

    // Render help text on the right side
    let help_text = " > F1 Help | 's' Settings | F10 Exit < ";
    let help_text_len = help_text.len() as u16;
    if cols > help_text_len {
        let help_x = cols - help_text_len - 1;
        for (i, ch) in help_text.chars().enumerate() {
            buffer.set(
                help_x + i as u16,
                bar_y,
                Cell::new_unchecked(ch, theme.bottombar_fg, theme.bottombar_bg),
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
        // Use new_unchecked for performance - theme colors are pre-validated
        buffer.set(
            current_x,
            bar_y,
            Cell::new_unchecked(open_bracket, button_fg, button_bg),
        );
        current_x += 1;
        buffer.set(
            current_x,
            bar_y,
            Cell::new_unchecked(' ', button_fg, button_bg),
        );
        current_x += 1;

        // Render title
        for ch in button_title.chars() {
            if current_x >= cols - 1 {
                break;
            }
            buffer.set(
                current_x,
                bar_y,
                Cell::new_unchecked(ch, button_fg, button_bg),
            );
            current_x += 1;
        }

        // Render space and closing bracket
        if current_x < cols - 1 {
            buffer.set(
                current_x,
                bar_y,
                Cell::new_unchecked(' ', button_fg, button_bg),
            );
            current_x += 1;
        }
        if current_x < cols - 1 {
            buffer.set(
                current_x,
                bar_y,
                Cell::new_unchecked(close_bracket, button_fg, button_bg),
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

pub fn render_calendar(
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
    // Use new_unchecked for performance - theme colors are pre-validated
    let bg_cell = Cell::new_unchecked(' ', fg_color, bg_color);
    for cy in 0..height {
        for cx in 0..width {
            buffer.set(x + cx, y + cy, bg_cell);
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
            buffer.set(
                char_x,
                y + 1,
                Cell::new_unchecked(ch, title_color, bg_color),
            );
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
            buffer.set(char_x, y + 3, Cell::new_unchecked(ch, fg_color, bg_color));
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
                buffer.set(
                    day_x + i as u16,
                    day_y,
                    Cell::new_unchecked(ch, day_fg, day_bg),
                );
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
                Cell::new_unchecked(ch, theme.config_instructions_fg, bg_color),
            );
        }
    }

    // Add shadow effect
    // Use new_unchecked for performance - shadow colors are intentionally low contrast
    let shadow_char = charset.shadow;
    for sy in 1..height {
        buffer.set(
            x + width,
            y + sy,
            Cell::new_unchecked(shadow_char, theme.window_shadow_color, Color::Black),
        );
    }
    for sx in 1..=width {
        buffer.set(
            x + sx,
            y + height,
            Cell::new_unchecked(shadow_char, theme.window_shadow_color, Color::Black),
        );
    }
}
