//! TopBar container that manages widget layout and rendering

use super::{
    BatteryWidget, CommandCenterWidget, DateTimeWidget, NetworkWidget, NewTermWidget, Widget,
    WidgetAlignment, WidgetClickResult, WidgetContext,
};
use crate::rendering::{Cell, Theme, VideoBuffer};
use crate::window::manager::FocusState;

/// Position of a widget in the top bar
#[derive(Clone, Debug)]
struct WidgetPosition {
    alignment: WidgetAlignment,
    index: usize,
    x: u16,
}

/// Top bar container that manages widget layout and rendering
pub struct TopBar {
    // Left-aligned widgets
    new_term: NewTermWidget,

    // Center-aligned widgets
    datetime: DateTimeWidget,

    // Right-aligned widgets (from left to right: battery, network, command_center)
    battery: BatteryWidget,
    network: NetworkWidget,
    command_center: CommandCenterWidget,

    // Cached positions (updated each frame)
    positions: Vec<WidgetPosition>,
}

impl TopBar {
    pub fn new(show_date_in_clock: bool) -> Self {
        Self {
            new_term: NewTermWidget::new(),
            datetime: DateTimeWidget::new(show_date_in_clock),
            battery: BatteryWidget::new(),
            network: NetworkWidget::new(),
            command_center: CommandCenterWidget::new(),
            positions: Vec::new(),
        }
    }

    /// Configure the network widget with interface name and enabled state
    pub fn configure_network(&mut self, interface: &str, enabled: bool) {
        self.network.configure(interface, enabled);
    }

    /// Update widget state and calculate positions
    pub fn update(&mut self, ctx: &WidgetContext) {
        // Update all widgets
        self.new_term.update(ctx);
        self.datetime.update(ctx);
        self.battery.update(ctx);
        self.network.update(ctx);
        self.command_center.update(ctx);

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
            });
        }

        // Right section: Command Center (rightmost), then Network, then Battery
        // Position from right edge: Command Center first (rightmost)
        let mut right_x = ctx.cols;

        // Command Center (always visible, rightmost with 1 char padding from edge)
        if self.command_center.is_visible(ctx) {
            let cc_width = self.command_center.width();
            right_x = right_x.saturating_sub(cc_width + 1); // +1 for right edge padding
            self.positions.push(WidgetPosition {
                alignment: WidgetAlignment::Right,
                index: 1, // index 1 for command center
                x: right_x,
            });
        }

        // Network (left of command center)
        if self.network.is_visible(ctx) {
            let network_width = self.network.width();
            right_x = right_x.saturating_sub(network_width);
            self.positions.push(WidgetPosition {
                alignment: WidgetAlignment::Right,
                index: 2, // index 2 for network
                x: right_x,
            });
        }

        // Battery (left of network)
        if self.battery.is_visible(ctx) {
            let battery_width = self.battery.width();
            right_x = right_x.saturating_sub(battery_width);
            self.positions.push(WidgetPosition {
                alignment: WidgetAlignment::Right,
                index: 0, // index 0 for battery
                x: right_x,
            });
        }

        // Center section: DateTime only
        // Calculate true screen center (not remaining space center)
        let datetime_width = self.datetime.width();

        // Center the widget on the total screen width
        let center_x = ctx.cols.saturating_sub(datetime_width) / 2;

        // Position datetime (center)
        if self.datetime.is_visible(ctx) {
            self.positions.push(WidgetPosition {
                alignment: WidgetAlignment::Center,
                index: 0,
                x: center_x,
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

        // Render each positioned widget
        for pos in &self.positions {
            match (pos.alignment, pos.index) {
                (WidgetAlignment::Left, 0) => self.new_term.render(buffer, pos.x, theme, ctx.focus),
                (WidgetAlignment::Center, 0) => {
                    self.datetime.render(buffer, pos.x, theme, ctx.focus)
                }
                (WidgetAlignment::Right, 0) => self.battery.render(buffer, pos.x, theme, ctx.focus),
                (WidgetAlignment::Right, 1) => {
                    self.command_center.render(buffer, pos.x, theme, ctx.focus)
                }
                (WidgetAlignment::Right, 2) => self.network.render(buffer, pos.x, theme, ctx.focus),
                _ => {}
            }
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
                    self.datetime.update_hover(mouse_x, mouse_y, pos.x);
                }
                (WidgetAlignment::Right, 0) => {
                    self.battery.update_hover(mouse_x, mouse_y, pos.x);
                }
                (WidgetAlignment::Right, 1) => {
                    self.command_center.update_hover(mouse_x, mouse_y, pos.x);
                }
                (WidgetAlignment::Right, 2) => {
                    self.network.update_hover(mouse_x, mouse_y, pos.x);
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
                (WidgetAlignment::Center, 0) => self.datetime.handle_click(mouse_x, mouse_y, pos.x),
                (WidgetAlignment::Right, 0) => self.battery.handle_click(mouse_x, mouse_y, pos.x),
                (WidgetAlignment::Right, 1) => {
                    self.command_center.handle_click(mouse_x, mouse_y, pos.x)
                }
                (WidgetAlignment::Right, 2) => self.network.handle_click(mouse_x, mouse_y, pos.x),
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
        self.datetime.reset_state();
        self.battery.reset_state();
        self.network.reset_state();
        self.command_center.reset_state();
    }

    /// Check if the battery widget is hovered (for compatibility)
    pub fn is_battery_hovered(&self) -> bool {
        self.battery.is_hovered()
    }

    /// Close command center menu
    pub fn close_command_center(&mut self) {
        self.command_center.close_menu();
    }

    /// Get the X position of the command center widget for menu positioning
    pub fn get_command_center_x(&self) -> u16 {
        for pos in &self.positions {
            if pos.alignment == WidgetAlignment::Right && pos.index == 1 {
                return pos.x;
            }
        }
        0
    }
}

impl Default for TopBar {
    fn default() -> Self {
        Self::new(false)
    }
}
