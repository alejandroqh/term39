//! New Terminal widget for the top bar

use super::{Widget, WidgetAlignment, WidgetClickResult, WidgetContext};
use crate::rendering::{Cell, Theme, VideoBuffer};
use crate::ui::button::ButtonState;

/// Widget for creating new terminal windows
pub struct NewTermWidget {
    label: &'static str,
    state: ButtonState,
}

impl NewTermWidget {
    pub fn new() -> Self {
        Self {
            label: "+New Terminal",
            state: ButtonState::Normal,
        }
    }
}

impl Default for NewTermWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for NewTermWidget {
    fn width(&self) -> u16 {
        // "[ +New Terminal ]" = label + 4 for "[ " and " ]"
        (self.label.len() as u16) + 4
    }

    fn render(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, _ctx: &WidgetContext) {
        let (fg_color, bg_color) = match self.state {
            ButtonState::Normal => (theme.button_normal_fg, theme.button_normal_bg),
            ButtonState::Hovered => (theme.button_hovered_fg, theme.button_hovered_bg),
            ButtonState::Pressed => (theme.button_pressed_fg, theme.button_pressed_bg),
        };

        let mut current_x = x;

        // Render "[ "
        buffer.set(current_x, 0, Cell::new_unchecked('[', fg_color, bg_color));
        current_x += 1;
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
        current_x += 1;

        // Render label
        for ch in self.label.chars() {
            buffer.set(current_x, 0, Cell::new_unchecked(ch, fg_color, bg_color));
            current_x += 1;
        }

        // Render " ]"
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, bg_color));
        current_x += 1;
        buffer.set(current_x, 0, Cell::new_unchecked(']', fg_color, bg_color));
    }

    fn is_visible(&self, _ctx: &WidgetContext) -> bool {
        true // Always visible
    }

    fn contains(&self, point_x: u16, point_y: u16, widget_x: u16) -> bool {
        point_y == 0 && point_x >= widget_x && point_x < widget_x + self.width()
    }

    fn update_hover(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) {
        if self.contains(mouse_x, mouse_y, widget_x) {
            self.state = ButtonState::Hovered;
        } else {
            self.state = ButtonState::Normal;
        }
    }

    fn handle_click(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) -> WidgetClickResult {
        if self.contains(mouse_x, mouse_y, widget_x) {
            self.state = ButtonState::Pressed;
            WidgetClickResult::CreateTerminal
        } else {
            WidgetClickResult::NotHandled
        }
    }

    fn reset_state(&mut self) {
        self.state = ButtonState::Normal;
    }

    fn update(&mut self, _ctx: &WidgetContext) {
        // No dynamic state to update
    }

    fn alignment(&self) -> WidgetAlignment {
        WidgetAlignment::Left
    }
}
