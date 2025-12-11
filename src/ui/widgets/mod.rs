//! Widget system for the top bar
//!
//! This module provides a widget-based architecture for the top bar,
//! allowing modular and reusable UI components.

use crate::rendering::{Charset, Theme, VideoBuffer};
use crate::window::manager::FocusState;

pub mod battery;
pub mod command_center;
pub mod datetime;
pub mod network;
pub mod new_term;
pub mod topbar;

// Re-export main types
pub use battery::BatteryWidget;
pub use command_center::CommandCenterWidget;
pub use datetime::DateTimeWidget;
pub use network::NetworkWidget;
pub use new_term::NewTermWidget;
pub use topbar::TopBar;

/// Result from widget click handling
#[derive(Debug, Clone)]
pub enum WidgetClickResult {
    /// Click was not handled by this widget
    NotHandled,
    /// Widget requests opening calendar
    OpenCalendar,
    /// Widget requests creating new terminal
    CreateTerminal,
    /// Widget requests toggling Command Center menu
    ToggleCommandCenter,
}

/// Alignment of widget within its container
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WidgetAlignment {
    Left,
    Center,
    Right,
}

/// Context passed to widgets for rendering and updates
#[derive(Clone, Copy, Debug)]
pub struct WidgetContext<'a> {
    pub cols: u16,
    #[allow(dead_code)]
    pub rows: u16,
    pub focus: FocusState,
    #[allow(dead_code)]
    pub has_clipboard_content: bool,
    #[allow(dead_code)]
    pub has_selection: bool,
    pub show_date_in_clock: bool,
    pub charset: &'a Charset,
}

impl<'a> WidgetContext<'a> {
    pub fn new(
        cols: u16,
        rows: u16,
        focus: FocusState,
        has_clipboard_content: bool,
        has_selection: bool,
        show_date_in_clock: bool,
        charset: &'a Charset,
    ) -> Self {
        Self {
            cols,
            rows,
            focus,
            has_clipboard_content,
            has_selection,
            show_date_in_clock,
            charset,
        }
    }
}

/// Core Widget trait that all topbar widgets implement
pub trait Widget {
    /// Return the widget's display width in characters
    fn width(&self) -> u16;

    /// Render the widget at the given x position (y is always 0 for topbar)
    fn render(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, ctx: &WidgetContext);

    /// Check if the widget should be visible given current context
    fn is_visible(&self, ctx: &WidgetContext) -> bool;

    /// Check if point (x, y) is within widget bounds
    fn contains(&self, point_x: u16, point_y: u16, widget_x: u16) -> bool;

    /// Handle mouse hover - update internal hover state
    fn update_hover(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16);

    /// Handle mouse click - returns result indicating action to take
    fn handle_click(&mut self, mouse_x: u16, mouse_y: u16, widget_x: u16) -> WidgetClickResult;

    /// Reset all hover/pressed states to normal
    fn reset_state(&mut self);

    /// Update widget state based on context (e.g., button enabled states)
    fn update(&mut self, ctx: &WidgetContext);

    /// Get widget alignment preference
    #[allow(dead_code)]
    fn alignment(&self) -> WidgetAlignment;
}
