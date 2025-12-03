use crate::charset::Charset;
use crate::theme::Theme;
use crate::video_buffer::{Cell, VideoBuffer};
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
}

/// Menu item definition
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<char>,
    pub action: Option<MenuAction>,
    pub is_separator: bool,
}

impl MenuItem {
    pub fn new(label: &str, shortcut: Option<char>, action: MenuAction) -> Self {
        Self {
            label: label.to_string(),
            shortcut,
            action: Some(action),
            is_separator: false,
        }
    }

    #[allow(dead_code)]
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: None,
            action: None,
            is_separator: true,
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
        }
    }

    /// Show the menu at a new position
    pub fn show(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
        self.visible = true;
        self.selected_index = 0;
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

    /// Get currently selected action
    pub fn get_selected_action(&self) -> Option<MenuAction> {
        self.items.get(self.selected_index)?.action
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
        if item_index < self.items.len() && !self.items[item_index].is_separator {
            if self.selected_index != item_index {
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
        (max_label_len + 8) as u16
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

        // Render borders
        // Top border
        buffer.set(
            self.x,
            self.y,
            Cell::new(charset.border_top_left(), border_color, bg_color),
        );
        for dx in 1..width - 1 {
            buffer.set(
                self.x + dx,
                self.y,
                Cell::new(charset.border_horizontal(), border_color, bg_color),
            );
        }
        buffer.set(
            self.x + width - 1,
            self.y,
            Cell::new(charset.border_top_right(), border_color, bg_color),
        );

        // Content rows
        for (i, item) in self.items.iter().enumerate() {
            let row = self.y + 1 + i as u16;
            let is_selected = i == self.selected_index && !item.is_separator;

            // Left border
            buffer.set(
                self.x,
                row,
                Cell::new(charset.border_vertical(), border_color, bg_color),
            );

            if item.is_separator {
                // Render separator
                for dx in 1..width - 1 {
                    buffer.set(
                        self.x + dx,
                        row,
                        Cell::new(charset.border_horizontal(), border_color, bg_color),
                    );
                }
            } else {
                // Render menu item
                let item_fg = if is_selected { selected_fg } else { fg_color };
                let item_bg = if is_selected { selected_bg } else { bg_color };

                // Padding + label
                let mut dx = 1;
                buffer.set(self.x + dx, row, Cell::new(' ', item_fg, item_bg));
                dx += 1;

                for ch in item.label.chars() {
                    buffer.set(self.x + dx, row, Cell::new(ch, item_fg, item_bg));
                    dx += 1;
                }

                // Padding to shortcut
                while dx < width - 3 {
                    buffer.set(self.x + dx, row, Cell::new(' ', item_fg, item_bg));
                    dx += 1;
                }

                // Shortcut
                if let Some(shortcut) = item.shortcut {
                    buffer.set(self.x + dx, row, Cell::new(shortcut, item_fg, item_bg));
                    dx += 1;
                }

                // Fill remaining space
                while dx < width - 1 {
                    buffer.set(self.x + dx, row, Cell::new(' ', item_fg, item_bg));
                    dx += 1;
                }
            }

            // Right border
            buffer.set(
                self.x + width - 1,
                row,
                Cell::new(charset.border_vertical(), border_color, bg_color),
            );
        }

        // Bottom border
        let bottom_y = self.y + height - 1;
        buffer.set(
            self.x,
            bottom_y,
            Cell::new(charset.border_bottom_left(), border_color, bg_color),
        );
        for dx in 1..width - 1 {
            buffer.set(
                self.x + dx,
                bottom_y,
                Cell::new(charset.border_horizontal(), border_color, bg_color),
            );
        }
        buffer.set(
            self.x + width - 1,
            bottom_y,
            Cell::new(charset.border_bottom_right(), border_color, bg_color),
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
        for dy in 1..=height {
            let x = self.x + width;
            let y = self.y + dy;
            buffer.set(x, y, Cell::new(charset.shadow, shadow_fg, shadow_bg));
        }

        // Bottom shadow
        for dx in 2..=width {
            let x = self.x + dx;
            let y = self.y + height;
            buffer.set(x, y, Cell::new(charset.shadow, shadow_fg, shadow_bg));
        }
    }
}
