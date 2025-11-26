use crate::button::{Button, ButtonState};
use crate::charset::Charset;
use crate::config_manager::AppConfig;
use crate::keyboard_mode::{KeyboardMode, WindowSubMode};
use crate::theme::Theme;
use crate::video_buffer::{Cell, VideoBuffer};
use crate::window_manager::{FocusState, WindowManager};
use chrono::{Datelike, Local, NaiveDate};
use crossterm::style::Color;

#[cfg(feature = "battery")]
mod battery_support {
    use crossterm::style::Color;
    use starship_battery::{Manager, State};
    use std::cell::RefCell;
    use std::time::{Duration, Instant};

    /// Battery information including percentage and charging state
    #[derive(Clone)]
    pub struct BatteryInfo {
        pub percentage: u8,
        pub is_charging: bool,
    }

    /// Cached battery info with last update time
    struct BatteryCache {
        info: Option<BatteryInfo>,
        last_update: Instant,
    }

    thread_local! {
        static BATTERY_CACHE: RefCell<BatteryCache> = RefCell::new(BatteryCache {
            info: None,
            last_update: Instant::now() - Duration::from_secs(2), // Force initial fetch
        });
    }

    /// Get the current battery info or None if no battery is available (cached for 1 second)
    pub fn get_battery_info() -> Option<BatteryInfo> {
        BATTERY_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();

            // Refresh if more than 1 second has passed
            if cache.last_update.elapsed() >= Duration::from_secs(1) {
                cache.info = fetch_battery_info();
                cache.last_update = Instant::now();
            }

            cache.info.clone()
        })
    }

    /// Actually fetch battery info from the system
    fn fetch_battery_info() -> Option<BatteryInfo> {
        let manager = Manager::new().ok()?;
        let mut batteries = manager.batteries().ok()?;
        let battery = batteries.next()?.ok()?;

        let percentage = (battery.state_of_charge().value * 100.0).round() as u8;
        // Show charging icon when plugged in (Charging or Full state)
        let is_charging = matches!(battery.state(), State::Charging | State::Full);

        Some(BatteryInfo {
            percentage,
            is_charging,
        })
    }

    /// Get the color for the battery indicator based on charge level and charging state
    pub fn get_battery_color(percentage: u8, is_charging: bool) -> Color {
        if percentage < 15 {
            Color::Red
        } else if percentage <= 40 {
            if is_charging {
                Color::Yellow
            } else {
                Color::White
            }
        } else {
            // >40%
            if is_charging {
                Color::Green
            } else {
                Color::White
            }
        }
    }
}

#[cfg(feature = "battery")]
use battery_support::{get_battery_color, get_battery_info};

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
    #[cfg(feature = "battery")] battery_hovered: bool,
    #[cfg(not(feature = "battery"))] _battery_hovered: bool,
) {
    let (cols, _rows) = buffer.dimensions();

    // Change background and foreground colors based on focus
    // When desktop has focus, topbar is "focused" (active/bright)
    // When a window has focus, topbar is "unfocused" (inactive/dimmed)
    let (bg_color, fg_color) = match focus {
        FocusState::Desktop => (theme.topbar_bg_focused, theme.topbar_fg_focused),
        FocusState::Window(_) => (theme.topbar_bg_unfocused, theme.topbar_fg_unfocused),
    };

    // Use new_unchecked for performance - theme colors are pre-validated
    let bar_cell = Cell::new_unchecked(' ', fg_color, bg_color);

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

    // Get battery info and create block bar indicator
    #[cfg(feature = "battery")]
    let battery_info = get_battery_info();
    #[cfg(feature = "battery")]
    let battery_width = if battery_info.is_some() { 10u16 } else { 0u16 }; // "| [█████] " or "| [████⚡] " when charging
    #[cfg(not(feature = "battery"))]
    let battery_width = 0u16;

    // Calculate positions (battery comes before clock)
    let total_width = battery_width + clock_width;
    let start_pos = cols.saturating_sub(total_width);

    // Render battery indicator (block bar or percentage text on hover)
    #[cfg(feature = "battery")]
    if let Some(info) = battery_info {
        let pct = info.percentage;
        let is_charging = info.is_charging;
        let battery_color = get_battery_color(pct, is_charging);
        let charging_icon = '≈'; // Approximately equal for charging (CP437 247)
        let charging_color = Color::Yellow;

        if battery_hovered {
            // Show percentage text on hover: "| [ 100%] " or "| [⚡ 89%] " (10 chars)
            let battery_text = if is_charging {
                format!("| [{}{:>4}] ", charging_icon, format!("{}%", pct))
            } else {
                format!("| [{:>5}] ", format!("{}%", pct))
            };
            for (i, ch) in battery_text.chars().enumerate() {
                let fg = if ch == charging_icon {
                    charging_color
                } else if (3..=7).contains(&i) {
                    battery_color
                } else {
                    theme.clock_fg
                };
                buffer.set(
                    start_pos + i as u16,
                    0,
                    Cell::new_unchecked(ch, fg, theme.clock_bg),
                );
            }
        } else {
            // Show block bar: "| [█████] " or "| [███ ≈] " (10 chars)
            let filled = ((pct as f32 / 20.0).round() as usize).min(5);
            let blocks: String = (0..5)
                .map(|i| {
                    if is_charging && i == 3 {
                        ' '
                    } else if is_charging && i == 4 {
                        charging_icon
                    } else if i < filled {
                        '█'
                    } else {
                        '░'
                    }
                })
                .collect();
            let battery_text = format!("| [{}] ", blocks);

            for (i, ch) in battery_text.chars().enumerate() {
                let fg = if ch == charging_icon {
                    charging_color
                } else if ch == '█' {
                    battery_color
                } else if ch == '░' {
                    Color::DarkGrey
                } else {
                    theme.clock_fg
                };
                buffer.set(
                    start_pos + i as u16,
                    0,
                    Cell::new_unchecked(ch, fg, theme.clock_bg),
                );
            }
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

/// Render the keyboard mode indicator in the bottom bar
/// Returns the width of the indicator (0 if not shown)
pub fn render_mode_indicator(
    buffer: &mut VideoBuffer,
    keyboard_mode: &KeyboardMode,
    theme: &Theme,
    x: u16,
    y: u16,
) -> u16 {
    // Build text and get colors based on mode
    let (text, fg, bg): (String, _, _) = match keyboard_mode {
        KeyboardMode::Normal => return 0, // No indicator in normal mode
        KeyboardMode::WindowMode(WindowSubMode::Navigation) => (
            "[WIN]".to_string(),
            theme.mode_indicator_window_fg,
            theme.mode_indicator_window_bg,
        ),
        KeyboardMode::WindowMode(WindowSubMode::Move) => (
            "[WIN:MOVE]".to_string(),
            theme.mode_indicator_move_fg,
            theme.mode_indicator_move_bg,
        ),
        KeyboardMode::WindowMode(WindowSubMode::Resize(_)) => (
            "[WIN:SIZE]".to_string(),
            theme.mode_indicator_resize_fg,
            theme.mode_indicator_resize_bg,
        ),
    };

    for (i, ch) in text.chars().enumerate() {
        buffer.set(x + i as u16, y, Cell::new_unchecked(ch, fg, bg));
    }

    text.len() as u16
}

pub fn render_button_bar(
    buffer: &mut VideoBuffer,
    window_manager: &WindowManager,
    auto_tiling_button: &Button,
    auto_tiling_enabled: bool,
    keyboard_mode: &KeyboardMode,
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

    // Render keyboard mode indicator on the far left (if in Window Mode)
    let mode_indicator_width = render_mode_indicator(buffer, keyboard_mode, theme, 0, bar_y);
    let mode_offset = if mode_indicator_width > 0 {
        mode_indicator_width + 1
    } else {
        0
    };

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

    let mut current_x = 1u16 + mode_offset;

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
    let help_x = if cols > help_text_len {
        let hx = cols - help_text_len - 1;
        for (i, ch) in help_text.chars().enumerate() {
            buffer.set(
                hx + i as u16,
                bar_y,
                Cell::new_unchecked(ch, theme.bottombar_fg, theme.bottombar_bg),
            );
        }
        hx
    } else {
        cols // No room for help text
    };

    // Get list of windows
    let windows = window_manager.get_window_list();
    if windows.is_empty() {
        return;
    }

    // Calculate max position for window buttons (don't overlap help text)
    let max_button_x = help_x.saturating_sub(2);

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

        // Check if there's room for at least the brackets and minimal content
        if current_x + 4 >= max_button_x {
            break; // Not enough room for this button
        }

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
            if current_x >= max_button_x {
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
        if current_x < max_button_x {
            buffer.set(
                current_x,
                bar_y,
                Cell::new_unchecked(' ', button_fg, button_bg),
            );
            current_x += 1;
        }
        if current_x < max_button_x {
            buffer.set(
                current_x,
                bar_y,
                Cell::new_unchecked(close_bracket, button_fg, button_bg),
            );
            current_x += 1;
        }

        // Add space between buttons
        current_x += 1;

        // Stop if we've run out of space (before help text)
        if current_x >= max_button_x {
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
