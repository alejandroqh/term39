//! Command Center widget for the top bar
//!
//! A dropdown menu button on the right side of the topbar that provides
//! access to various system actions like Exit.

use super::{Widget, WidgetAlignment, WidgetClickResult, WidgetContext};
use crate::rendering::{Cell, Theme, VideoBuffer};
use crate::ui::button::ButtonState;
use crate::window::manager::FocusState;

/// Widget for the Command Center dropdown menu
pub struct CommandCenterWidget {
    label: &'static str,
    state: ButtonState,
    /// Whether the dropdown menu is currently open
    menu_open: bool,
}

impl CommandCenterWidget {
    const LABEL: &'static str = "Command Center";

    pub fn new() -> Self {
        Self {
            label: Self::LABEL,
            state: ButtonState::Normal,
            menu_open: false,
        }
    }

    /// Close the dropdown menu
    pub fn close_menu(&mut self) {
        self.menu_open = false;
    }

    /// Toggle the dropdown menu
    pub fn toggle_menu(&mut self) {
        self.menu_open = !self.menu_open;
    }
}

impl Default for CommandCenterWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for CommandCenterWidget {
    fn width(&self) -> u16 {
        // "[ Command Center ]" = label + 4 for "[ " and " ]"
        (self.label.len() as u16) + 4
    }

    fn render(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, ctx: &WidgetContext) {
        // Use same colors as datetime widget for consistency
        let bg_color = match ctx.focus {
            FocusState::Desktop | FocusState::Topbar => theme.topbar_bg_focused,
            FocusState::Window(_) => theme.topbar_bg_unfocused,
        };

        let (fg_color, btn_bg) = match self.state {
            ButtonState::Normal => (theme.window_border_unfocused_fg, bg_color),
            ButtonState::Hovered => (theme.button_hovered_fg, theme.button_hovered_bg),
            ButtonState::Pressed => (theme.button_pressed_fg, theme.button_pressed_bg),
        };

        // If menu is open, show as pressed
        let (fg_color, btn_bg) = if self.menu_open {
            (theme.button_pressed_fg, theme.button_pressed_bg)
        } else {
            (fg_color, btn_bg)
        };

        let mut current_x = x;

        // Render "[ "
        buffer.set(current_x, 0, Cell::new_unchecked('[', fg_color, btn_bg));
        current_x += 1;
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, btn_bg));
        current_x += 1;

        // Render label
        for ch in self.label.chars() {
            buffer.set(current_x, 0, Cell::new_unchecked(ch, fg_color, btn_bg));
            current_x += 1;
        }

        // Render " ]"
        buffer.set(current_x, 0, Cell::new_unchecked(' ', fg_color, btn_bg));
        current_x += 1;
        buffer.set(current_x, 0, Cell::new_unchecked(']', fg_color, btn_bg));
    }

    fn is_visible(&self, _ctx: &WidgetContext) -> bool {
        true // Always visible
    }

    fn contains(&self, point_x: u16, point_y: u16, widget_x: u16) -> bool {
        point_y == 0 && point_x >= widget_x && point_x < widget_x + self.width()
    }

    fn update_hover(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) {
        if self.contains(mouse_x, mouse_y, widget_x) {
            if self.state != ButtonState::Pressed {
                self.state = ButtonState::Hovered;
            }
        } else if !self.menu_open {
            self.state = ButtonState::Normal;
        }
    }

    fn handle_click(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) -> WidgetClickResult {
        if self.contains(mouse_x, mouse_y, widget_x) {
            self.state = ButtonState::Pressed;
            self.toggle_menu();
            WidgetClickResult::ToggleCommandCenter
        } else {
            WidgetClickResult::NotHandled
        }
    }

    fn reset_state(&mut self) {
        self.state = ButtonState::Normal;
        // Don't close menu on reset - that's handled separately
    }

    fn update(&mut self, _ctx: &WidgetContext) {
        // No dynamic state to update
    }

    fn alignment(&self) -> WidgetAlignment {
        WidgetAlignment::Right
    }
}
