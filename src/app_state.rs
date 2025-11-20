use crate::button::Button;
use crate::config_window::ConfigWindow;
use crate::context_menu::ContextMenu;
use crate::error_dialog::ErrorDialog;
use crate::info_window::InfoWindow;
use crate::prompt::Prompt;
use crate::slight_input::SlightInput;
use crate::ui_render::CalendarState;
use std::time::Instant;

/// Centralizes all mutable application state
pub struct AppState {
    // Dialog/Popup State
    pub active_prompt: Option<Prompt>,
    pub active_calendar: Option<CalendarState>,
    pub active_config_window: Option<ConfigWindow>,
    pub active_help_window: Option<InfoWindow>,
    pub active_about_window: Option<InfoWindow>,
    pub active_slight_input: Option<SlightInput>,
    pub active_error_dialog: Option<ErrorDialog>,
    pub context_menu: ContextMenu,

    // Top Bar Buttons
    pub new_terminal_button: Button,
    pub paste_button: Button,
    pub clear_clipboard_button: Button,
    pub copy_button: Button,
    pub clear_selection_button: Button,
    pub exit_button: Button,

    // Bottom Bar Buttons
    pub auto_tiling_button: Button,

    // Application Settings
    pub auto_tiling_enabled: bool,
    pub tint_terminal: bool,

    // Selection State
    pub selection_active: bool,
    pub last_click_time: Option<Instant>,
    pub click_count: u32,

    // Exit flag
    pub should_exit: bool,
}

impl AppState {
    /// Creates a new AppState with initial button positions
    pub fn new(cols: u16, rows: u16, auto_tiling_on_startup: bool, tint_terminal: bool) -> Self {
        // Create the "New Terminal" button
        let new_terminal_button = Button::new(1, 0, "+New Terminal".to_string());

        // Paste button (positioned in center, will be repositioned each frame)
        let paste_label = "Paste".to_string();
        let paste_button_width = (paste_label.len() as u16) + 4; // "[ Label ]"
        let paste_x = (cols.saturating_sub(paste_button_width + 5)) / 2; // +5 for [ X ] button
        let paste_button = Button::new(paste_x, 0, paste_label);

        // Clear clipboard button (X)
        let clear_label = "X".to_string();
        let clear_clipboard_button = Button::new(paste_x + paste_button_width, 0, clear_label);

        // Copy button (shows when text is selected)
        let copy_label = "Copy".to_string();
        let copy_button = Button::new(0, 0, copy_label);

        // Clear selection button (X) (shows when text is selected)
        let clear_selection_label = "X".to_string();
        let clear_selection_button = Button::new(0, 0, clear_selection_label);

        // Exit button (positioned in top bar, will be repositioned each frame)
        let exit_button = Button::new(0, 0, "Exit".to_string());

        // Auto-tiling toggle button (bottom bar)
        let auto_tiling_text = if auto_tiling_on_startup {
            "█ on] Auto Tiling"
        } else {
            "off ░] Auto Tiling"
        };
        let auto_tiling_button = Button::new(1, rows - 1, auto_tiling_text.to_string());

        // Context menu (initially at 0, 0, not visible)
        let context_menu = ContextMenu::new(0, 0);

        Self {
            // Dialog/Popup State
            active_prompt: None,
            active_calendar: None,
            active_config_window: None,
            active_help_window: None,
            active_about_window: None,
            active_slight_input: None,
            active_error_dialog: None,
            context_menu,

            // Top Bar Buttons
            new_terminal_button,
            paste_button,
            clear_clipboard_button,
            copy_button,
            clear_selection_button,
            exit_button,

            // Bottom Bar Button
            auto_tiling_button,

            // Application Settings
            auto_tiling_enabled: auto_tiling_on_startup,
            tint_terminal,

            // Selection State
            selection_active: false,
            last_click_time: None,
            click_count: 0,

            // Exit flag
            should_exit: false,
        }
    }

    /// Updates button positions and states based on current clipboard and selection state
    pub fn update_button_states(
        &mut self,
        cols: u16,
        has_clipboard_content: bool,
        has_selection: bool,
    ) {
        // Update button enabled states
        self.paste_button.enabled = has_clipboard_content;
        self.clear_clipboard_button.enabled = has_clipboard_content;
        self.copy_button.enabled = has_selection;
        self.clear_selection_button.enabled = has_selection;

        // Position exit button on the right side of top bar (before clock area)
        // Clock with date format "| Thu Nov 20, 09:21 " is about 22 chars
        // Clock without date "| 09:21:45 " is about 12 chars
        // Use the larger value to ensure no overlap, plus padding
        let exit_button_width = self.exit_button.width();
        self.exit_button.x = cols.saturating_sub(exit_button_width + 24);
        self.exit_button.y = 0;

        // Calculate button widths
        let paste_width = self.paste_button.width();
        let clear_clip_width = self.clear_clipboard_button.width();
        let copy_width = self.copy_button.width();
        let clear_sel_width = self.clear_selection_button.width();

        // Position paste and clear clipboard buttons together in center
        let paste_clear_total_width = paste_width + clear_clip_width;
        let paste_x = (cols.saturating_sub(paste_clear_total_width)) / 2;
        self.paste_button.x = paste_x;
        self.clear_clipboard_button.x = paste_x + paste_width;

        // Position copy and clear selection buttons
        if has_selection && has_clipboard_content {
            // Both visible: put copy[X] to the left of paste[X]
            let copy_clear_sel_width = copy_width + clear_sel_width;
            let total_width = copy_clear_sel_width + 1 + paste_clear_total_width; // +1 for gap
            let start_x = (cols.saturating_sub(total_width)) / 2;
            self.copy_button.x = start_x;
            self.clear_selection_button.x = start_x + copy_width;
            self.paste_button.x = start_x + copy_clear_sel_width + 1;
            self.clear_clipboard_button.x = self.paste_button.x + paste_width;
        } else if has_selection {
            // Only copy[X] visible: center it
            let copy_clear_sel_width = copy_width + clear_sel_width;
            let start_x = (cols.saturating_sub(copy_clear_sel_width)) / 2;
            self.copy_button.x = start_x;
            self.clear_selection_button.x = start_x + copy_width;
        }
    }

    /// Updates the auto-tiling button position (call after resize)
    pub fn update_auto_tiling_button_position(&mut self, rows: u16) {
        self.auto_tiling_button.y = rows - 1;
    }
}
