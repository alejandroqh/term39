//! Widget system for the top bar
//!
//! This module provides a widget-based architecture for the top bar,
//! allowing modular and reusable UI components.

use crate::rendering::{Theme, VideoBuffer};
use crate::window::manager::FocusState;

pub mod battery;
pub mod clipboard;
pub mod datetime;
pub mod exit;
pub mod new_term;
pub mod topbar;

// Re-export main types
pub use battery::BatteryWidget;
pub use clipboard::ClipboardWidget;
pub use datetime::DateTimeWidget;
pub use exit::ExitWidget;
pub use new_term::NewTermWidget;
pub use topbar::TopBar;

/// Result from widget click handling
#[derive(Debug, Clone)]
pub enum WidgetClickResult {
    /// Click was not handled by this widget
    NotHandled,
    /// Click was handled, no further action needed
    Handled,
    /// Widget requests showing exit prompt
    ShowExitPrompt(String, u16, u16),
    /// Widget requests opening calendar
    OpenCalendar,
    /// Widget requests creating new terminal
    CreateTerminal,
    /// Widget requests copying selected text
    CopySelection,
    /// Widget requests clearing selection
    ClearSelection,
    /// Widget requests pasting from clipboard
    Paste,
    /// Widget requests clearing clipboard
    ClearClipboard,
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
pub struct WidgetContext {
    pub cols: u16,
    pub rows: u16,
    pub focus: FocusState,
    pub has_clipboard_content: bool,
    pub has_selection: bool,
    pub show_date_in_clock: bool,
}

impl WidgetContext {
    pub fn new(
        cols: u16,
        rows: u16,
        focus: FocusState,
        has_clipboard_content: bool,
        has_selection: bool,
        show_date_in_clock: bool,
    ) -> Self {
        Self {
            cols,
            rows,
            focus,
            has_clipboard_content,
            has_selection,
            show_date_in_clock,
        }
    }
}

/// Core Widget trait that all topbar widgets implement
pub trait Widget {
    /// Return the widget's display width in characters
    fn width(&self) -> u16;

    /// Render the widget at the given x position (y is always 0 for topbar)
    fn render(&self, buffer: &mut VideoBuffer, x: u16, theme: &Theme, focus: FocusState);

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
    fn alignment(&self) -> WidgetAlignment;
}
