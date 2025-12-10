//! Exit widget - code only, not rendered in the top bar
//!
//! This widget exists for future use (e.g., keyboard shortcuts, programmatic exit)
//! but is NOT rendered in the top bar per design requirements.

use super::{Widget, WidgetAlignment, WidgetClickResult, WidgetContext};
use crate::rendering::{Theme, VideoBuffer};
use crate::ui::button::ButtonState;
use crate::window::manager::FocusState;

/// Exit widget - provides exit functionality but is NOT rendered in the top bar
pub struct ExitWidget {
    state: ButtonState,
    no_exit_mode: bool,
}

impl ExitWidget {
    const LABEL: &'static str = "Exit";

    pub fn new() -> Self {
        Self {
            state: ButtonState::Normal,
            no_exit_mode: false,
        }
    }

    /// Enable/disable no-exit mode
    pub fn set_no_exit_mode(&mut self, enabled: bool) {
        self.no_exit_mode = enabled;
    }

    /// Trigger exit confirmation - can be called programmatically
    pub fn trigger_exit(&mut self, window_count: usize, cols: u16, rows: u16) -> WidgetClickResult {
        if self.no_exit_mode {
            return WidgetClickResult::NotHandled;
        }

        let message = if window_count > 0 {
            "Exit with open windows?\nAll terminal sessions will be closed.".to_string()
        } else {
            "Exit term39?".to_string()
        };

        WidgetClickResult::ShowExitPrompt(message, cols, rows)
    }
}

impl Default for ExitWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ExitWidget {
    fn width(&self) -> u16 {
        // "[ Exit ]" = label + 4
        (Self::LABEL.len() as u16) + 4
    }

    fn render(&self, _buffer: &mut VideoBuffer, _x: u16, _theme: &Theme, _focus: FocusState) {
        // NOT RENDERED - code only widget
    }

    fn is_visible(&self, _ctx: &WidgetContext) -> bool {
        false // Never visible - code only
    }

    fn contains(&self, _point_x: u16, _point_y: u16, _widget_x: u16) -> bool {
        false // Not interactive via mouse since not rendered
    }

    fn update_hover(&mut self, _mouse_x: u16, _mouse_y: u16, _widget_x: u16) {
        // No-op
    }

    fn handle_click(&mut self, _mouse_x: u16, _mouse_y: u16, _widget_x: u16) -> WidgetClickResult {
        WidgetClickResult::NotHandled
    }

    fn reset_state(&mut self) {
        self.state = ButtonState::Normal;
    }

    fn update(&mut self, _ctx: &WidgetContext) {
        // No-op
    }

    fn alignment(&self) -> WidgetAlignment {
        WidgetAlignment::Right
    }
}
