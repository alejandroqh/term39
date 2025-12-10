//! Battery widget for the top bar
//!
//! Shows battery level as a block bar or percentage on hover.
//! Only available when the "battery" feature is enabled.

use super::{Widget, WidgetAlignment, WidgetClickResult, WidgetContext};
use crate::rendering::{Cell, Theme, VideoBuffer};
use crate::window::manager::FocusState;
use crossterm::style::Color;

#[cfg(feature = "battery")]
use crate::ui::ui_render::battery_support::{BatteryInfo, get_battery_color, get_battery_info};

/// Widget displaying battery status
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

    #[cfg(feature = "battery")]
    fn render_bar(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, info: &BatteryInfo) {
        let pct = info.percentage;
        let is_charging = info.is_charging;
        let battery_color = get_battery_color(pct, is_charging);
        let charging_icon = '\u{2248}'; // ≈
        let charging_color = Color::Yellow;

        let filled = ((pct as f32 / 20.0).round() as usize).min(5);
        let blocks: String = (0..5)
            .map(|i| {
                if is_charging && i == 3 {
                    ' '
                } else if is_charging && i == 4 {
                    charging_icon
                } else if i < filled {
                    '\u{2588}' // █
                } else {
                    '\u{2591}' // ░
                }
            })
            .collect();

        let battery_text = format!("| [{}] ", blocks);

        for (i, ch) in battery_text.chars().enumerate() {
            let fg = if ch == charging_icon {
                charging_color
            } else if ch == '\u{2588}' {
                battery_color
            } else if ch == '\u{2591}' {
                Color::DarkGrey
            } else {
                theme.clock_fg
            };

            buffer.set(x + i as u16, 0, Cell::new_unchecked(ch, fg, theme.clock_bg));
        }
    }

    #[cfg(feature = "battery")]
    fn render_percentage(
        &self,
        buffer: &mut VideoBuffer,
        x: u16,
        theme: &Theme,
        info: &BatteryInfo,
    ) {
        let pct = info.percentage;
        let is_charging = info.is_charging;
        let battery_color = get_battery_color(pct, is_charging);
        let charging_icon = '\u{2248}'; // ≈
        let charging_color = Color::Yellow;

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

            buffer.set(x + i as u16, 0, Cell::new_unchecked(ch, fg, theme.clock_bg));
        }
    }
}

impl Default for BatteryWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for BatteryWidget {
    fn width(&self) -> u16 {
        10 // "| [█████] " or "| [ 100%] "
    }

    fn render(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, _focus: FocusState) {
        #[cfg(feature = "battery")]
        if let Some(ref info) = self.cached_info {
            if self.hovered {
                self.render_percentage(buffer, x, theme, info);
            } else {
                self.render_bar(buffer, x, theme, info);
            }
        }

        #[cfg(not(feature = "battery"))]
        {
            let _ = (buffer, x, theme);
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
