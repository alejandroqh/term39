//! TopBar container that manages widget layout and rendering

use super::{
    BatteryWidget, ClipboardWidget, DateTimeWidget, ExitWidget, NewTermWidget, Widget,
    WidgetAlignment, WidgetClickResult, WidgetContext,
};
use crate::rendering::{Cell, Theme, VideoBuffer};
use crate::window::manager::FocusState;

/// Separator character between widget sections
const SEPARATOR: char = '|';

/// Position of a widget in the top bar
#[derive(Clone, Debug)]
struct WidgetPosition {
    alignment: WidgetAlignment,
    index: usize,
    x: u16,
    width: u16,
}

/// Top bar container that manages widget layout and rendering
pub struct TopBar {
    // Left-aligned widgets
    new_term: NewTermWidget,

    // Center-aligned widgets
    clipboard: ClipboardWidget,

    // Right-aligned widgets
    battery: BatteryWidget,
    datetime: DateTimeWidget,

    // Exit widget (code only, not rendered)
    exit: ExitWidget,

    // Cached positions (updated each frame)
    positions: Vec<WidgetPosition>,
}

impl TopBar {
    pub fn new(show_date_in_clock: bool) -> Self {
        Self {
            new_term: NewTermWidget::new(),
            clipboard: ClipboardWidget::new(),
            battery: BatteryWidget::new(),
            datetime: DateTimeWidget::new(show_date_in_clock),
            exit: ExitWidget::new(),
            positions: Vec::new(),
        }
    }

    /// Get mutable reference to exit widget for programmatic exit
    pub fn exit_widget_mut(&mut self) -> &mut ExitWidget {
        &mut self.exit
    }

    /// Update widget state and calculate positions
    pub fn update(&mut self, ctx: &WidgetContext) {
        // Update all widgets
        self.new_term.update(ctx);
        self.clipboard.update(ctx);
        self.battery.update(ctx);
        self.datetime.update(ctx);
        self.exit.update(ctx);

        // Recalculate layout
        self.layout(ctx);
    }

    /// Calculate widget positions based on current context
    fn layout(&mut self, ctx: &WidgetContext) {
        self.positions.clear();

        // Left section: NewTerm widget at x=1
        let left_x = 1u16;
        if self.new_term.is_visible(ctx) {
            self.positions.push(WidgetPosition {
                alignment: WidgetAlignment::Left,
                index: 0,
                x: left_x,
                width: self.new_term.width(),
            });
        }
        let left_end = left_x + self.new_term.width();

        // Right section: Battery only (positioned from right edge)
        let mut right_total_width = 0u16;
        if self.battery.is_visible(ctx) {
            let battery_width = self.battery.width();
            right_total_width = battery_width;
            let battery_x = ctx.cols.saturating_sub(battery_width);
            self.positions.push(WidgetPosition {
                alignment: WidgetAlignment::Right,
                index: 0,
                x: battery_x,
                width: battery_width,
            });
        }

        // Center section: DateTime (and Clipboard if visible)
        // Calculate center area boundaries
        let center_start = left_end + 2; // +2 for separator space
        let center_end = ctx.cols.saturating_sub(right_total_width + 2); // +2 for separator space
        let available_center = center_end.saturating_sub(center_start);

        // Calculate total center width needed
        let datetime_width = self.datetime.width();
        let clipboard_width = if self.clipboard.is_visible(ctx) {
            self.clipboard.width() + 1 // +1 for gap between clipboard and datetime
        } else {
            0
        };
        let total_center_width = datetime_width + clipboard_width;

        // Center the widgets in available space
        let center_x = center_start + available_center.saturating_sub(total_center_width) / 2;

        // Position clipboard first (left of datetime) if visible
        let mut current_x = center_x;
        if self.clipboard.is_visible(ctx) {
            let clip_width = self.clipboard.width();
            self.positions.push(WidgetPosition {
                alignment: WidgetAlignment::Center,
                index: 0,
                x: current_x,
                width: clip_width,
            });
            current_x += clip_width + 1; // +1 for gap
        }

        // Position datetime (center)
        if self.datetime.is_visible(ctx) {
            self.positions.push(WidgetPosition {
                alignment: WidgetAlignment::Center,
                index: 1, // index 1 for datetime in center
                x: current_x,
                width: datetime_width,
            });
        }

        // Sort positions by x coordinate for rendering
        self.positions.sort_by_key(|p| p.x);
    }

    /// Render the complete top bar
    pub fn render(&self, buffer: &mut VideoBuffer, theme: &Theme, ctx: &WidgetContext) {
        let (cols, _) = buffer.dimensions();

        // Determine colors based on focus
        let (bg_color, fg_color) = match ctx.focus {
            FocusState::Desktop | FocusState::Topbar => {
                (theme.topbar_bg_focused, theme.topbar_fg_focused)
            }
            FocusState::Window(_) => (theme.topbar_bg_unfocused, theme.topbar_fg_unfocused),
        };

        // Clear top bar with background
        let bar_cell = Cell::new_unchecked(' ', fg_color, bg_color);
        for x in 0..cols {
            buffer.set(x, 0, bar_cell);
        }

        // Track last widget end position for separator placement
        let mut last_end: Option<(u16, WidgetAlignment)> = None;

        // Render each positioned widget
        for pos in &self.positions {
            // Draw separator between different alignment sections
            if let Some((prev_end, prev_align)) = last_end {
                if prev_align != pos.alignment && pos.x > prev_end + 1 {
                    // Draw separator between sections
                    let sep_x = (prev_end + pos.x) / 2;
                    buffer.set(sep_x, 0, Cell::new_unchecked(SEPARATOR, fg_color, bg_color));
                }
            }

            // Render the widget
            match (pos.alignment, pos.index) {
                (WidgetAlignment::Left, 0) => self.new_term.render(buffer, pos.x, theme, ctx.focus),
                (WidgetAlignment::Center, 0) => {
                    self.clipboard.render(buffer, pos.x, theme, ctx.focus)
                }
                (WidgetAlignment::Center, 1) => {
                    self.datetime.render(buffer, pos.x, theme, ctx.focus)
                }
                (WidgetAlignment::Right, 0) => self.battery.render(buffer, pos.x, theme, ctx.focus),
                _ => {}
            }

            last_end = Some((pos.x + pos.width, pos.alignment));
        }
    }

    /// Update hover states for all widgets based on mouse position
    pub fn update_hover(&mut self, mouse_x: u16, mouse_y: u16, _ctx: &WidgetContext) {
        if mouse_y != 0 {
            self.reset_all_states();
            return;
        }

        // Update hover for each widget at its calculated position
        for pos in &self.positions {
            match (pos.alignment, pos.index) {
                (WidgetAlignment::Left, 0) => {
                    self.new_term.update_hover(mouse_x, mouse_y, pos.x);
                }
                (WidgetAlignment::Center, 0) => {
                    self.clipboard.update_hover(mouse_x, mouse_y, pos.x);
                }
                (WidgetAlignment::Center, 1) => {
                    self.datetime.update_hover(mouse_x, mouse_y, pos.x);
                }
                (WidgetAlignment::Right, 0) => {
                    self.battery.update_hover(mouse_x, mouse_y, pos.x);
                }
                _ => {}
            }
        }
    }

    /// Handle click on topbar
    pub fn handle_click(&mut self, mouse_x: u16, mouse_y: u16) -> WidgetClickResult {
        if mouse_y != 0 {
            return WidgetClickResult::NotHandled;
        }

        // Check each widget at its calculated position
        for pos in &self.positions {
            let result = match (pos.alignment, pos.index) {
                (WidgetAlignment::Left, 0) => self.new_term.handle_click(mouse_x, mouse_y, pos.x),
                (WidgetAlignment::Center, 0) => {
                    self.clipboard.handle_click(mouse_x, mouse_y, pos.x)
                }
                (WidgetAlignment::Center, 1) => self.datetime.handle_click(mouse_x, mouse_y, pos.x),
                (WidgetAlignment::Right, 0) => self.battery.handle_click(mouse_x, mouse_y, pos.x),
                _ => WidgetClickResult::NotHandled,
            };

            if !matches!(result, WidgetClickResult::NotHandled) {
                return result;
            }
        }

        WidgetClickResult::NotHandled
    }

    /// Reset all widget states
    fn reset_all_states(&mut self) {
        self.new_term.reset_state();
        self.clipboard.reset_state();
        self.battery.reset_state();
        self.datetime.reset_state();
        self.exit.reset_state();
    }

    /// Check if the battery widget is hovered (for compatibility)
    pub fn is_battery_hovered(&self) -> bool {
        self.battery.is_hovered()
    }
}

impl Default for TopBar {
    fn default() -> Self {
        Self::new(false)
    }
}
