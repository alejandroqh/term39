//! Date/Time widget for the top bar

use super::{Widget, WidgetAlignment, WidgetClickResult, WidgetContext};
use crate::rendering::{Cell, Theme, VideoBuffer};
use crate::window::manager::FocusState;
use chrono::Local;

/// Widget displaying date and/or time
pub struct DateTimeWidget {
    show_date: bool,
    hovered: bool,
}

impl DateTimeWidget {
    pub fn new(show_date: bool) -> Self {
        Self {
            show_date,
            hovered: false,
        }
    }

    fn get_time_string(&self) -> String {
        let now = Local::now();
        if self.show_date {
            // Show date and time: "Tue Nov 11, 09:21"
            now.format("%a %b %d, %H:%M").to_string()
        } else {
            // Show time only with seconds: "09:21:45"
            now.format("%H:%M:%S").to_string()
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
        let time_str = self.get_time_string();
        (time_str.len() + 2) as u16 // " " + time + " "
    }

    fn render(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, focus: FocusState) {
        let time_str = format!(" {} ", self.get_time_string());

        // Use topbar background with window border fg color for text
        let bg_color = match focus {
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
        self.show_date = ctx.show_date_in_clock;
    }

    fn alignment(&self) -> WidgetAlignment {
        WidgetAlignment::Center
    }
}
