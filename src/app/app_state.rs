use super::config_manager::AppConfig;
use crate::input::keyboard_mode::{KeyboardMode, MovementState};
use crate::lockscreen::{LockScreen, PinSetupDialog};
use crate::ui::button::Button;
use crate::ui::config_window::ConfigWindow;
use crate::ui::context_menu::ContextMenu;
use crate::ui::error_dialog::ErrorDialog;
use crate::ui::info_window::InfoWindow;
use crate::ui::prompt::Prompt;
use crate::ui::slight_input::SlightInput;
use crate::ui::toast::Toast;
use crate::ui::ui_render::CalendarState;
use crate::ui::widgets::TopBar;
use std::time::Instant;

/// Centralizes all mutable application state
pub struct AppState {
    // Dialog/Popup State
    pub active_prompt: Option<Prompt>,
    pub active_calendar: Option<CalendarState>,
    pub active_config_window: Option<ConfigWindow>,
    pub active_help_window: Option<InfoWindow>,
    pub active_about_window: Option<InfoWindow>,
    pub active_winmode_help_window: Option<InfoWindow>,
    pub active_slight_input: Option<SlightInput>,
    pub active_error_dialog: Option<ErrorDialog>,
    pub active_toast: Option<Toast>,
    pub context_menu: ContextMenu,
    pub taskbar_menu: ContextMenu,
    pub taskbar_menu_window_id: Option<u32>,
    pub command_center_menu: ContextMenu,

    // Top Bar Buttons (legacy - will be replaced by TopBar)
    #[allow(dead_code)]
    pub new_terminal_button: Button,
    pub paste_button: Button,
    pub clear_clipboard_button: Button,
    pub copy_button: Button,
    pub clear_selection_button: Button,
    pub exit_button: Button,

    // New Widget-based Top Bar
    pub top_bar: TopBar,

    // Battery indicator hover state (legacy - TopBar manages this internally)
    pub battery_hovered: bool,

    // Bottom Bar Buttons
    pub auto_tiling_button: Button,

    // Application Settings
    pub auto_tiling_enabled: bool,
    pub tint_terminal: bool,

    // Selection State
    pub selection_active: bool,
    pub last_click_time: Option<Instant>,
    pub last_click_pos: Option<(u16, u16)>,
    pub click_count: u32,

    // Exit flag
    pub should_exit: bool,

    // Keyboard Mode State (vim-like window control)
    pub keyboard_mode: KeyboardMode,
    pub move_state: MovementState,
    pub resize_state: MovementState,

    // Double-backtick detection for literal backtick input
    pub last_backtick_time: Option<Instant>,

    // Window number overlay (F10 toggle, Option+1-9 selection)
    /// Whether to show window number overlay
    pub show_window_number_overlay: bool,

    // Lockscreen
    pub lockscreen: LockScreen,
    pub active_pin_setup: Option<PinSetupDialog>,
}

impl AppState {
    /// Creates a new AppState with initial button positions
    pub fn new(cols: u16, rows: u16, config: &AppConfig) -> Self {
        let auto_tiling_on_startup = config.auto_tiling_on_startup;
        let tint_terminal = config.tint_terminal;
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

        // Taskbar context menu (initially at 0, 0, not visible)
        let taskbar_menu = ContextMenu::new_taskbar_menu(0, 0);

        // Command Center menu (width matches the button: "[ Command Center ]" = 18 chars)
        let command_center_menu = ContextMenu::new_command_center_menu(0, 1, 18);

        Self {
            // Dialog/Popup State
            active_prompt: None,
            active_calendar: None,
            active_config_window: None,
            active_help_window: None,
            active_about_window: None,
            active_winmode_help_window: None,
            active_slight_input: None,
            active_error_dialog: None,
            active_toast: None,
            context_menu,
            taskbar_menu,
            taskbar_menu_window_id: None,
            command_center_menu,

            // Top Bar Buttons (legacy)
            new_terminal_button,
            paste_button,
            clear_clipboard_button,
            copy_button,
            clear_selection_button,
            exit_button,

            // New Widget-based Top Bar
            top_bar: TopBar::new(config.show_date_in_clock),

            // Battery indicator hover state (legacy)
            battery_hovered: false,

            // Bottom Bar Button
            auto_tiling_button,

            // Application Settings
            auto_tiling_enabled: auto_tiling_on_startup,
            tint_terminal,

            // Selection State
            selection_active: false,
            last_click_time: None,
            last_click_pos: None,
            click_count: 0,

            // Exit flag
            should_exit: false,

            // Keyboard Mode State
            keyboard_mode: KeyboardMode::Normal,
            move_state: MovementState::new(),
            resize_state: MovementState::new(),

            // Double-backtick detection
            last_backtick_time: None,

            // Window number overlay (F10 toggle)
            show_window_number_overlay: false,

            // Lockscreen - initialize with config settings
            lockscreen: LockScreen::new_with_mode(
                config.lockscreen_auth_mode,
                config.lockscreen_pin_hash.clone(),
                config.lockscreen_salt.clone(),
            ),
            active_pin_setup: None,
        }
    }

    /// Starts the PIN setup dialog
    pub fn start_pin_setup(&mut self, salt: String) {
        self.active_pin_setup = Some(PinSetupDialog::new(salt));
    }

    /// Updates the lockscreen authentication mode from config
    pub fn update_lockscreen_auth(&mut self, config: &AppConfig) {
        self.lockscreen.update_auth_mode(
            config.lockscreen_auth_mode,
            config.lockscreen_pin_hash.clone(),
            config.lockscreen_salt.clone(),
        );
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

        // Position exit button on the right side of top bar (before battery and clock)
        // Battery indicator "| [ 89%] " is 10 chars
        // Clock with date format "| Thu Nov 20, 09:21 " is about 22 chars
        // Clock without date "| 09:21:45 " is about 12 chars
        // Total max: 10 + 22 = 32 chars, use 34 for safe padding
        let exit_button_width = self.exit_button.width();
        self.exit_button.x = cols.saturating_sub(exit_button_width + 34);
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
