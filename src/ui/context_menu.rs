use crate::rendering::{Cell, Charset, Theme, VideoBuffer};
use crossterm::style::Color;

/// Menu item action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Copy,
    Paste,
    SelectAll,
    #[allow(dead_code)]
    Close,
    // Taskbar menu actions
    Restore,
    Maximize,
    CloseWindow,
    // Command Center menu actions
    CopySelection,
    PasteClipboard,
    ClearClipboard,
    Settings,
    About,
    Exit,
}

/// Menu item definition
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<char>,
    pub action: Option<MenuAction>,
    pub is_separator: bool,
    pub enabled: bool,
}

impl MenuItem {
    pub fn new(label: &str, shortcut: Option<char>, action: MenuAction) -> Self {
        Self {
            label: label.to_string(),
            shortcut,
            action: Some(action),
            is_separator: false,
            enabled: true,
        }
    }

    pub fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: None,
            action: None,
            is_separator: true,
            enabled: true,
        }
    }
}

/// Context menu state and rendering
pub struct ContextMenu {
    pub x: u16,
    pub y: u16,
    pub items: Vec<MenuItem>,
    pub selected_index: usize,
    pub visible: bool,
    /// Optional minimum width for the menu (used by Command Center)
    min_width: Option<u16>,
}

impl ContextMenu {
    /// Create a new context menu at position
    pub fn new(x: u16, y: u16) -> Self {
        let items = vec![
            MenuItem::new("Copy", None, MenuAction::Copy),
            MenuItem::new("Paste", None, MenuAction::Paste),
            MenuItem::new("Select All", None, MenuAction::SelectAll),
        ];

        Self {
            x,
            y,
            items,
            selected_index: 0,
            visible: false,
            min_width: None,
        }
    }

    /// Create a taskbar context menu for window buttons in the bottom bar
    pub fn new_taskbar_menu(x: u16, y: u16) -> Self {
        let items = vec![
            MenuItem::new("Restore", None, MenuAction::Restore),
            MenuItem::new("Maximize", None, MenuAction::Maximize),
            MenuItem::new("Close", None, MenuAction::CloseWindow),
        ];

        Self {
            x,
            y,
            items,
            selected_index: 0,
            visible: false,
            min_width: None,
        }
    }

    /// Create a Command Center dropdown menu
    pub fn new_command_center_menu(x: u16, y: u16, menu_width: u16) -> Self {
        // Clipboard operations + Settings + About + Exit
        // Copy, Paste, Clear Clipboard are always visible but enabled/disabled based on context
        // Icons: ⧉ (U+29C9) Copy, ⧠ (U+29E0) Paste, ⌫ (U+232B) Clear, ⚙ (U+2699) Settings, ⓘ (U+24D8) About, ⏻ (U+23FB) Power/Exit
        let items = vec![
            MenuItem::new("Copy", Some('\u{29C9}'), MenuAction::CopySelection),
            MenuItem::new("Paste", Some('\u{29E0}'), MenuAction::PasteClipboard),
            MenuItem::new(
                "Clear Clipboard",
                Some('\u{232B}'),
                MenuAction::ClearClipboard,
            ),
            MenuItem::separator(),
            MenuItem::new("Settings...", Some('\u{2699}'), MenuAction::Settings),
            MenuItem::new("About...", Some('\u{24D8}'), MenuAction::About),
            MenuItem::separator(),
            MenuItem::new("Exit", Some('\u{23FB}'), MenuAction::Exit),
        ];

        Self {
            x,
            y,
            items,
            selected_index: 0,
            visible: false,
            min_width: Some(menu_width),
        }
    }

    /// Update the enabled state of a menu item by action
    pub fn set_item_enabled(&mut self, action: MenuAction, enabled: bool) {
        for item in &mut self.items {
            if item.action == Some(action) {
                item.enabled = enabled;
            }
        }
    }

    /// Show the menu at a new position
    pub fn show(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
        self.visible = true;
        self.selected_index = 0;
    }

    /// Show the menu with bounds checking to prevent overflow
    /// Adjusts x position if menu would exceed screen width
    pub fn show_bounded(&mut self, x: u16, y: u16, screen_width: u16) {
        let total_width = self.total_width_with_shadow();
        // Ensure menu + shadow fits within screen
        self.x = if x + total_width > screen_width {
            screen_width.saturating_sub(total_width)
        } else {
            x
        };
        self.y = y;
        self.visible = true;
        self.selected_index = 0;
    }

    /// Get total width including shadow (for bounds calculations)
    pub fn total_width_with_shadow(&self) -> u16 {
        self.calculate_width() + 1 // +1 for shadow column
    }

    /// Hide the menu
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Move selection up
    #[allow(dead_code)]
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            // Skip separators
            while self.selected_index > 0 && self.items[self.selected_index].is_separator {
                self.selected_index -= 1;
            }
        }
    }

    /// Move selection down
    #[allow(dead_code)]
    pub fn select_next(&mut self) {
        if self.selected_index < self.items.len() - 1 {
            self.selected_index += 1;
            // Skip separators
            while self.selected_index < self.items.len() - 1
                && self.items[self.selected_index].is_separator
            {
                self.selected_index += 1;
            }
        }
    }

    /// Get currently selected action (returns None if item is disabled)
    pub fn get_selected_action(&self) -> Option<MenuAction> {
        let item = self.items.get(self.selected_index)?;
        if item.enabled { item.action } else { None }
    }

    /// Check if a click position is inside the menu
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        if !self.visible {
            return false;
        }

        let width = self.calculate_width();
        let height = self.items.len() as u16 + 2; // +2 for borders

        x >= self.x && x < self.x + width && y >= self.y && y < self.y + height
    }

    /// Update selection based on mouse position, returns true if selection changed
    /// Skips disabled items and separators
    pub fn update_selection_from_mouse(&mut self, x: u16, y: u16) -> bool {
        if !self.visible {
            return false;
        }

        let width = self.calculate_width();

        // Check if within menu bounds (excluding borders)
        if x <= self.x || x >= self.x + width - 1 {
            return false;
        }

        // Calculate which item row the mouse is on (accounting for top border)
        if y <= self.y || y > self.y + self.items.len() as u16 {
            return false;
        }

        let item_index = (y - self.y - 1) as usize;
        if item_index < self.items.len() {
            let item = &self.items[item_index];
            // Only select enabled, non-separator items
            if !item.is_separator && item.enabled && self.selected_index != item_index {
                self.selected_index = item_index;
                return true;
            }
        }

        false
    }

    /// Calculate menu width based on content
    fn calculate_width(&self) -> u16 {
        let max_label_len = self
            .items
            .iter()
            .map(|item| item.label.len())
            .max()
            .unwrap_or(0);

        // Width = border + padding + label + padding + shortcut + padding + border
        let content_width = (max_label_len + 8) as u16;

        // Use min_width if set and larger than content width
        self.min_width
            .map_or(content_width, |min| min.max(content_width))
    }

    /// Render the menu to video buffer
    pub fn render(&self, buffer: &mut VideoBuffer, charset: &Charset, theme: &Theme) {
        if !self.visible {
            return;
        }

        let width = self.calculate_width();
        let height = self.items.len() as u16 + 2; // +2 for borders

        let fg_color = theme.menu_fg;
        let bg_color = theme.menu_bg;
        let border_color = theme.menu_border;
        let selected_fg = theme.menu_selected_fg;
        let selected_bg = theme.menu_selected_bg;
        let disabled_fg = theme.menu_disabled_fg;

        // Render borders
        // Top border
        // Use new_unchecked for performance - theme colors are pre-validated
        buffer.set(
            self.x,
            self.y,
            Cell::new_unchecked(charset.border_top_left(), border_color, bg_color),
        );
        for dx in 1..width - 1 {
            buffer.set(
                self.x + dx,
                self.y,
                Cell::new_unchecked(charset.border_horizontal(), border_color, bg_color),
            );
        }
        buffer.set(
            self.x + width - 1,
            self.y,
            Cell::new_unchecked(charset.border_top_right(), border_color, bg_color),
        );

        // Content rows
        for (i, item) in self.items.iter().enumerate() {
            let row = self.y + 1 + i as u16;
            // Only show selection highlight for enabled items
            let is_selected = i == self.selected_index && !item.is_separator && item.enabled;

            // Left border
            buffer.set(
                self.x,
                row,
                Cell::new_unchecked(charset.border_vertical(), border_color, bg_color),
            );

            if item.is_separator {
                // Render separator
                for dx in 1..width - 1 {
                    buffer.set(
                        self.x + dx,
                        row,
                        Cell::new_unchecked(charset.border_horizontal(), border_color, bg_color),
                    );
                }
            } else {
                // Render menu item - use disabled color if not enabled
                let item_fg = if !item.enabled {
                    disabled_fg
                } else if is_selected {
                    selected_fg
                } else {
                    fg_color
                };
                let item_bg = if is_selected { selected_bg } else { bg_color };

                // Padding + label
                let mut dx = 1;
                buffer.set(self.x + dx, row, Cell::new_unchecked(' ', item_fg, item_bg));
                dx += 1;

                for ch in item.label.chars() {
                    buffer.set(self.x + dx, row, Cell::new_unchecked(ch, item_fg, item_bg));
                    dx += 1;
                }

                // Padding to shortcut
                while dx < width - 3 {
                    buffer.set(self.x + dx, row, Cell::new_unchecked(' ', item_fg, item_bg));
                    dx += 1;
                }

                // Shortcut
                if let Some(shortcut) = item.shortcut {
                    buffer.set(
                        self.x + dx,
                        row,
                        Cell::new_unchecked(shortcut, item_fg, item_bg),
                    );
                    dx += 1;
                }

                // Fill remaining space
                while dx < width - 1 {
                    buffer.set(self.x + dx, row, Cell::new_unchecked(' ', item_fg, item_bg));
                    dx += 1;
                }
            }

            // Right border
            buffer.set(
                self.x + width - 1,
                row,
                Cell::new_unchecked(charset.border_vertical(), border_color, bg_color),
            );
        }

        // Bottom border
        let bottom_y = self.y + height - 1;
        buffer.set(
            self.x,
            bottom_y,
            Cell::new_unchecked(charset.border_bottom_left(), border_color, bg_color),
        );
        for dx in 1..width - 1 {
            buffer.set(
                self.x + dx,
                bottom_y,
                Cell::new_unchecked(charset.border_horizontal(), border_color, bg_color),
            );
        }
        buffer.set(
            self.x + width - 1,
            bottom_y,
            Cell::new_unchecked(charset.border_bottom_right(), border_color, bg_color),
        );

        // Render shadow
        self.render_shadow(buffer, width, height, charset, theme);
    }

    /// Render drop shadow for menu
    fn render_shadow(
        &self,
        buffer: &mut VideoBuffer,
        width: u16,
        height: u16,
        charset: &Charset,
        theme: &Theme,
    ) {
        let shadow_fg = theme.menu_shadow_fg;
        let shadow_bg = Color::Black;

        // Right shadow
        // Use new_unchecked for performance - shadow colors are intentionally low contrast
        for dy in 1..=height {
            let x = self.x + width;
            let y = self.y + dy;
            buffer.set(
                x,
                y,
                Cell::new_unchecked(charset.shadow, shadow_fg, shadow_bg),
            );
        }

        // Bottom shadow
        for dx in 2..=width {
            let x = self.x + dx;
            let y = self.y + height;
            buffer.set(
                x,
                y,
                Cell::new_unchecked(charset.shadow, shadow_fg, shadow_bg),
            );
        }
    }
}
