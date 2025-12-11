//! Battery widget for the top bar
//!
//! Shows battery icon with percentage and charging indicator.
//! Only available when the "battery" feature is enabled.

use super::{Widget, WidgetAlignment, WidgetClickResult, WidgetContext};
use crate::rendering::{Theme, VideoBuffer};

#[cfg(feature = "battery")]
use crate::rendering::Cell;
#[cfg(feature = "battery")]
use crate::ui::ui_render::battery_support::{BatteryInfo, get_battery_color, get_battery_info};
#[cfg(feature = "battery")]
use crate::window::manager::FocusState;
#[cfg(feature = "battery")]
use crossterm::style::Color;

/// Widget displaying battery status with icon and percentage
pub struct BatteryWidget {
    hovered: bool,
    #[cfg(feature = "battery")]
    cached_info: Option<BatteryInfo>,
    #[cfg(not(feature = "battery"))]
    _phantom: (),
}

impl BatteryWidget {
    pub fn new() -> Self {
        Self {
            hovered: false,
            #[cfg(feature = "battery")]
            cached_info: None,
            #[cfg(not(feature = "battery"))]
            _phantom: (),
        }
    }

    /// Returns whether the battery widget is currently hovered
    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Get the battery icon based on charge level using charset
    #[cfg(feature = "battery")]
    fn get_battery_icon(percentage: u8, charset: &crate::rendering::Charset) -> char {
        // Battery icons showing different fill levels
        if percentage >= 75 {
            charset.battery_full // full battery
        } else if percentage >= 50 {
            charset.battery_high // 3/4 battery
        } else if percentage >= 25 {
            charset.battery_medium // half battery
        } else if percentage >= 10 {
            charset.battery_low // low battery
        } else {
            charset.battery_critical // critical
        }
    }

    #[cfg(feature = "battery")]
    fn render_widget(
        &self,
        buffer: &mut VideoBuffer,
        x: u16,
        theme: &Theme,
        info: &BatteryInfo,
        ctx: &WidgetContext,
    ) {
        let pct = info.percentage;
        let is_charging = info.is_charging;
        let battery_color = get_battery_color(pct, is_charging);
        let charset = ctx.charset;

        // Use same colors as datetime widget
        let bg_color = match ctx.focus {
            FocusState::Desktop | FocusState::Topbar => theme.topbar_bg_focused,
            FocusState::Window(_) => theme.topbar_bg_unfocused,
        };
        let fg_color = theme.window_border_unfocused_fg;

        // Charging indicator from charset
        let charging_icon = charset.battery_charging;
        let charging_color = Color::Yellow;

        // Battery body using box drawing: [███]+
        let battery_icon = Self::get_battery_icon(pct, charset);

        // Build the display string: " 85% ↯[███]+ " or " 85% [███]+ "
        let mut current_x = x;

        // Leading space
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
        current_x += 1;

        // Percentage text first (right-aligned in 4 chars: "100%" or " 85%")
        let pct_str = format!("{:>3}%", pct);
        for ch in pct_str.chars() {
            buffer.set(
                current_x,
                0,
                Cell::new_unchecked(ch, battery_color, bg_color),
            );
            current_x += 1;
        }

        // Space before battery icon
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
        current_x += 1;

        // Charging icon (if charging)
        if is_charging {
            buffer.set(
                current_x,
                0,
                Cell::new_unchecked(charging_icon, charging_color, bg_color),
            );
            current_x += 1;
        }

        // Battery body: [███]+
        // Opening bracket
        buffer.set(current_x, 0, Cell::new_unchecked('[', fg_color, bg_color));
        current_x += 1;

        // Battery fill (3 characters)
        let filled_count = ((pct as f32 / 100.0) * 3.0).ceil() as usize;
        for i in 0..3 {
            let ch = if i < filled_count { battery_icon } else { ' ' };
            let fg = if i < filled_count {
                battery_color
            } else {
                Color::DarkGrey
            };
            buffer.set(current_x, 0, Cell::new_unchecked(ch, fg, bg_color));
            current_x += 1;
        }

        // Closing bracket and terminal
        buffer.set(current_x, 0, Cell::new_unchecked(']', fg_color, bg_color));
        current_x += 1;
        buffer.set(current_x, 0, Cell::new_unchecked('+', fg_color, bg_color));
        current_x += 1;

        // Trailing space
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
    }
}

impl Default for BatteryWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for BatteryWidget {
    fn width(&self) -> u16 {
        // " 100% [███]+ " = 13 chars (without charging icon)
        // " 100% ↯[███]+ " = 14 chars (with charging icon)
        // Use max width to ensure consistent layout
        #[cfg(feature = "battery")]
        {
            if let Some(ref info) = self.cached_info {
                if info.is_charging {
                    14 // " 100% ↯[███]+ "
                } else {
                    13 // " 100% [███]+ "
                }
            } else {
                13
            }
        }
        #[cfg(not(feature = "battery"))]
        {
            0
        }
    }

    fn render(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, ctx: &WidgetContext) {
        #[cfg(feature = "battery")]
        if let Some(ref info) = self.cached_info {
            self.render_widget(buffer, x, theme, info, ctx);
        }

        #[cfg(not(feature = "battery"))]
        {
            let _ = (buffer, x, theme, ctx);
        }
    }

    fn is_visible(&self, _ctx: &WidgetContext) -> bool {
        #[cfg(feature = "battery")]
        {
            self.cached_info.is_some()
        }
        #[cfg(not(feature = "battery"))]
        {
            false
        }
    }

    fn contains(&self, point_x: u16, point_y: u16, widget_x: u16) -> bool {
        point_y == 0 && point_x >= widget_x && point_x < widget_x + self.width()
    }

    fn update_hover(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) {
        self.hovered = self.contains(mouse_x, mouse_y, widget_x);
    }

    fn handle_click(&mut self, _mouse_x: u16, _mouse_y: u16, _widget_x: u16) -> WidgetClickResult {
        // Battery doesn't respond to clicks
        WidgetClickResult::NotHandled
    }

    fn reset_state(&mut self) {
        self.hovered = false;
    }

    fn update(&mut self, _ctx: &WidgetContext) {
        #[cfg(feature = "battery")]
        {
            self.cached_info = get_battery_info();
        }
    }

    fn alignment(&self) -> WidgetAlignment {
        WidgetAlignment::Right
    }
}
