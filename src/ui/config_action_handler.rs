//! Handler for ConfigAction processing.
//!
//! This module consolidates the handling of ConfigAction values, which is
//! shared between keyboard and mouse event handlers.

use crate::app::app_state::AppState;
use crate::app::config_manager::AppConfig;
use crate::lockscreen::auth::is_os_auth_available;
use crate::rendering::{Charset, Theme};
use crate::ui::button::Button;
use crate::ui::config_window::ConfigAction;
use crate::window::manager::WindowManager;

/// Result of processing a config action
#[derive(Default)]
pub struct ConfigActionResult {
    /// New theme if theme was changed
    pub new_theme: Option<Theme>,
    /// New background character if changed
    pub new_background: Option<char>,
}

/// Process a ConfigAction and apply changes to app_state and app_config.
///
/// This function handles all ConfigAction variants and applies the appropriate
/// changes. It returns a ConfigActionResult indicating any values that need
/// to be applied at the call site (like theme changes).
///
/// # Arguments
/// * `action` - The config action to process
/// * `app_state` - Mutable reference to app state
/// * `app_config` - Mutable reference to app config
/// * `rows` - Current terminal height (for button positioning)
///
/// # Returns
/// A ConfigActionResult indicating any changes that need external handling
#[allow(clippy::too_many_lines)]
pub fn process_config_action(
    action: ConfigAction,
    app_state: &mut AppState,
    app_config: &mut AppConfig,
    rows: u16,
) -> ConfigActionResult {
    let mut result = ConfigActionResult::default();

    match action {
        ConfigAction::Close => {
            // Already handled externally
        }
        ConfigAction::ToggleAutoTiling => {
            app_config.toggle_auto_tiling_on_startup();
            app_state.auto_tiling_enabled = app_config.auto_tiling_on_startup;
            let auto_tiling_text = if app_state.auto_tiling_enabled {
                "█ on] Auto Tiling"
            } else {
                "off ░] Auto Tiling"
            };
            app_state.auto_tiling_button = Button::new(1, rows - 1, auto_tiling_text.to_string());
            if let Some(ref mut config_win) = app_state.active_config_window {
                config_win.ensure_focus_valid(app_config);
            }
        }
        ConfigAction::ToggleTilingGaps => {
            app_config.toggle_tiling_gaps();
        }
        ConfigAction::ToggleShowDate => {
            app_config.toggle_show_date_in_clock();
        }
        ConfigAction::CycleTheme => {
            let next_theme = match app_config.theme.as_str() {
                "classic" => "monochrome",
                "monochrome" => "dark",
                "dark" => "dracu",
                "dracu" => "green_phosphor",
                "green_phosphor" => "amber",
                "amber" => "ndd",
                "ndd" => "qbasic",
                "qbasic" => "turbo",
                "turbo" => "norton_commander",
                "norton_commander" => "xtree",
                "xtree" => "wordperfect",
                "wordperfect" => "dbase",
                "dbase" => "system",
                "system" => "classic",
                _ => "classic",
            };
            app_config.theme = next_theme.to_string();
            let _ = app_config.save();
            result.new_theme = Some(Theme::from_name(&app_config.theme));
        }
        ConfigAction::CycleThemeBackward => {
            let prev_theme = match app_config.theme.as_str() {
                "classic" => "system",
                "system" => "dbase",
                "monochrome" => "classic",
                "dark" => "monochrome",
                "dracu" => "dark",
                "green_phosphor" => "dracu",
                "amber" => "green_phosphor",
                "ndd" => "amber",
                "qbasic" => "ndd",
                "turbo" => "qbasic",
                "norton_commander" => "turbo",
                "xtree" => "norton_commander",
                "wordperfect" => "xtree",
                "dbase" => "wordperfect",
                _ => "classic",
            };
            app_config.theme = prev_theme.to_string();
            let _ = app_config.save();
            result.new_theme = Some(Theme::from_name(&app_config.theme));
        }
        ConfigAction::CycleBackgroundChar => {
            app_config.cycle_background_char();
            result.new_background = Some(app_config.get_background_char());
        }
        ConfigAction::CycleBackgroundCharBackward => {
            app_config.cycle_background_char_backward();
            result.new_background = Some(app_config.get_background_char());
        }
        ConfigAction::ToggleTintTerminal => {
            app_config.toggle_tint_terminal();
            app_state.tint_terminal = app_config.tint_terminal;
        }
        ConfigAction::ToggleAutoSave => {
            app_config.toggle_auto_save();
            if !app_config.auto_save {
                let _ = WindowManager::clear_session_file();
            }
        }
        ConfigAction::ToggleLockscreen => {
            app_config.toggle_lockscreen_enabled();
            if let Some(ref mut config_win) = app_state.active_config_window {
                config_win.ensure_focus_valid(app_config);
            }
        }
        ConfigAction::CycleLockscreenAuthMode => {
            app_config.cycle_lockscreen_auth_mode(is_os_auth_available());
            app_state.update_lockscreen_auth(app_config);
            if let Some(ref mut config_win) = app_state.active_config_window {
                config_win.ensure_focus_valid(app_config);
            }
        }
        ConfigAction::SetupPin => {
            app_state.active_config_window = None;
            let salt = app_config.get_or_create_salt();
            app_state.start_pin_setup(salt);
        }
        ConfigAction::None => {
            // Just navigation, no action needed
        }
    }

    result
}

/// Apply the result of a config action to the charset and theme.
///
/// This is a helper to apply changes that require mutable access to
/// charset and theme which are owned by the caller.
pub fn apply_config_result(
    result: &ConfigActionResult,
    charset: &mut Charset,
    theme: &mut Theme,
) {
    if let Some(ref new_theme) = result.new_theme {
        *theme = new_theme.clone();
    }
    if let Some(new_bg) = result.new_background {
        charset.set_background(new_bg);
    }
}
