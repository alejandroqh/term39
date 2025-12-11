//! Date/Time widget for the top bar

use super::{Widget, WidgetAlignment, WidgetClickResult, WidgetContext};
use crate::rendering::{Cell, Theme, VideoBuffer};
use crate::window::manager::FocusState;
use chrono::{Local, Timelike};

/// Widget displaying date and/or time
pub struct DateTimeWidget {
    show_date: bool,
    hovered: bool,
    /// Cached formatted time string to avoid repeated allocations
    cached_time: String,
    /// Last second value to detect when to refresh
    last_second: u32,
}

impl DateTimeWidget {
    pub fn new(show_date: bool) -> Self {
        Self {
            show_date,
            hovered: false,
            cached_time: String::new(),
            last_second: 60, // Invalid value to force initial update
        }
    }

    /// Refresh the cached time string if the second has changed
    fn refresh_time_if_needed(&mut self) {
        let now = Local::now();
        let current_second = now.second();

        // Only regenerate when second changes or cache is empty
        if current_second != self.last_second || self.cached_time.is_empty() {
            self.last_second = current_second;
            self.cached_time = if self.show_date {
                // Show date and time: "Tue Nov 11, 09:21"
                now.format("%a %b %d, %H:%M").to_string()
            } else {
                // Show time only with seconds: "09:21:45"
                now.format("%H:%M:%S").to_string()
            };
        }
    }
}

impl Default for DateTimeWidget {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Widget for DateTimeWidget {
    fn width(&self) -> u16 {
        // Just the time string with padding: " Tue Nov 11, 09:21 " or " 09:21:45 "
        // Use cached string length (add 2 for padding spaces)
        (self.cached_time.len() + 2) as u16
    }

    fn render(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, ctx: &WidgetContext) {
        let time_str = format!(" {} ", &self.cached_time);

        // Use topbar background with window border fg color for text
        let bg_color = match ctx.focus {
            FocusState::Desktop | FocusState::Topbar => theme.topbar_bg_focused,
            FocusState::Window(_) => theme.topbar_bg_unfocused,
        };
        let fg_color = theme.window_border_unfocused_fg;

        for (i, ch) in time_str.chars().enumerate() {
            buffer.set(x + i as u16, 0, Cell::new_unchecked(ch, fg_color, bg_color));
        }
    }

    fn is_visible(&self, _ctx: &WidgetContext) -> bool {
        true // Always visible
    }

    fn contains(&self, point_x: u16, point_y: u16, widget_x: u16) -> bool {
        point_y == 0 && point_x >= widget_x && point_x < widget_x + self.width()
    }

    fn update_hover(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) {
        self.hovered = self.contains(mouse_x, mouse_y, widget_x);
    }

    fn handle_click(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) -> WidgetClickResult {
        if self.contains(mouse_x, mouse_y, widget_x) {
            WidgetClickResult::OpenCalendar
        } else {
            WidgetClickResult::NotHandled
        }
    }

    fn reset_state(&mut self) {
        self.hovered = false;
    }

    fn update(&mut self, ctx: &WidgetContext) {
        // Check if show_date setting changed - if so, force cache refresh
        if self.show_date != ctx.show_date_in_clock {
            self.show_date = ctx.show_date_in_clock;
            self.last_second = 60; // Force refresh on next call
        }
        // Refresh time string if needed (only when second changes)
        self.refresh_time_if_needed();
    }

    fn alignment(&self) -> WidgetAlignment {
        WidgetAlignment::Center
    }
}
