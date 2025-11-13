use crossterm::style::Color;

// Custom color constants for better readability and maintainability
const PURE_BLACK: Color = Color::Rgb { r: 0, g: 0, b: 0 };
const LIGHT_AMBER: Color = Color::Rgb {
    r: 200,
    g: 175,
    b: 120,
};
const DARK_GREEN_PHOSPHOR: Color = Color::Rgb { r: 0, g: 120, b: 0 };

#[derive(Debug, Clone)]
pub struct Theme {
    // Desktop
    pub desktop_bg: Color,
    pub desktop_fg: Color,

    // Top bar
    pub topbar_bg_desktop: Color,
    pub topbar_bg_window: Color,
    pub topbar_fg: Color,
    pub clock_bg: Color,
    pub clock_fg: Color,

    // Windows
    pub window_title_bg: Color,
    pub window_title_bg_focused: Color,
    pub window_title_fg: Color,
    pub window_border: Color,
    pub window_content_bg: Color,
    #[allow(dead_code)]
    pub window_content_fg: Color,
    pub window_shadow_color: Color,

    // Window controls
    pub button_close_color: Color,
    pub button_maximize_color: Color,
    pub button_minimize_color: Color,
    pub button_bg: Color, // Consistent background for title bar buttons
    pub resize_handle_normal_fg: Color,
    pub resize_handle_normal_bg: Color,
    pub resize_handle_active_fg: Color,
    pub resize_handle_active_bg: Color,

    // UI Buttons
    pub button_normal_fg: Color,
    pub button_normal_bg: Color,
    pub button_hovered_fg: Color,
    pub button_hovered_bg: Color,
    pub button_pressed_fg: Color,
    pub button_pressed_bg: Color,

    // Bottom bar
    pub bottombar_bg: Color,
    pub bottombar_fg: Color,
    pub bottombar_button_normal_fg: Color,
    pub bottombar_button_normal_bg: Color,
    pub bottombar_button_focused_fg: Color,
    pub bottombar_button_focused_bg: Color,
    pub bottombar_button_minimized_fg: Color,
    pub bottombar_button_minimized_bg: Color,

    // Toggle button
    pub toggle_enabled_fg: Color,
    pub toggle_enabled_bg_normal: Color,
    pub toggle_enabled_bg_hovered: Color,
    pub toggle_enabled_bg_pressed: Color,
    pub toggle_disabled_fg: Color,
    pub toggle_disabled_bg_normal: Color,
    pub toggle_disabled_bg_hovered: Color,
    pub toggle_disabled_bg_pressed: Color,

    // Prompts/Dialogs (per type)
    pub prompt_info_bg: Color,
    pub prompt_info_fg: Color,
    pub prompt_success_bg: Color,
    pub prompt_success_fg: Color,
    pub prompt_warning_bg: Color,
    pub prompt_warning_fg: Color,
    pub prompt_danger_bg: Color,
    pub prompt_danger_fg: Color,

    // Dialog buttons
    pub dialog_button_primary_info_fg: Color,
    pub dialog_button_primary_info_bg: Color,
    pub dialog_button_primary_success_fg: Color,
    pub dialog_button_primary_success_bg: Color,
    pub dialog_button_primary_warning_fg: Color,
    pub dialog_button_primary_warning_bg: Color,
    pub dialog_button_primary_danger_fg: Color,
    pub dialog_button_primary_danger_bg: Color,
    pub dialog_button_secondary_fg: Color,
    pub dialog_button_secondary_bg: Color,

    // Config window
    pub config_title_bg: Color,
    pub config_title_fg: Color,
    pub config_border: Color,
    pub config_content_bg: Color,
    pub config_content_fg: Color,
    pub config_instructions_fg: Color,
    pub config_toggle_on_color: Color,
    pub config_toggle_off_color: Color,

    // Calendar
    pub calendar_bg: Color,
    pub calendar_fg: Color,
    pub calendar_title_color: Color,
    pub calendar_today_bg: Color,
    pub calendar_today_fg: Color,

    // Scrollbar
    pub scrollbar_track_fg: Color,
    pub scrollbar_thumb_fg: Color,

    // Snap preview
    pub snap_preview_border: Color,
    pub snap_preview_bg: Color,

    // Splash screen
    pub splash_border: Color,
    pub splash_bg: Color,
    pub splash_fg: Color,
}

impl Theme {
    /// Classic DOS-inspired theme (current default colors)
    pub fn classic() -> Self {
        Self {
            // Desktop
            desktop_bg: Color::Blue,
            desktop_fg: Color::White,

            // Top bar
            topbar_bg_desktop: Color::Cyan,
            topbar_bg_window: Color::Black,
            topbar_fg: Color::White,
            clock_bg: Color::DarkGrey,
            clock_fg: Color::White,

            // Windows
            window_title_bg: Color::DarkGrey,
            window_title_bg_focused: Color::DarkCyan,
            window_title_fg: Color::White,
            window_border: Color::White,
            window_content_bg: Color::DarkBlue,
            window_content_fg: Color::White,
            window_shadow_color: Color::DarkGrey,

            // Window controls
            button_close_color: Color::Red,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Yellow,
            button_bg: Color::Black, // Consistent button background
            resize_handle_normal_fg: Color::Grey,
            resize_handle_normal_bg: Color::Black,
            resize_handle_active_fg: Color::Yellow,
            resize_handle_active_bg: Color::Grey,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::White,
            button_hovered_fg: Color::Black,
            button_hovered_bg: Color::Yellow,
            button_pressed_fg: Color::White,
            button_pressed_bg: Color::DarkGrey,

            // Bottom bar
            bottombar_bg: Color::Black,
            bottombar_fg: Color::White,
            bottombar_button_normal_fg: Color::White,
            bottombar_button_normal_bg: Color::Black,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::Cyan,
            bottombar_button_minimized_fg: Color::DarkGrey,
            bottombar_button_minimized_bg: Color::Black,

            // Toggle button
            toggle_enabled_fg: Color::Green,
            toggle_enabled_bg_normal: Color::DarkGrey,
            toggle_enabled_bg_hovered: Color::Yellow,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::White,
            toggle_disabled_bg_normal: Color::DarkGrey,
            toggle_disabled_bg_hovered: Color::Yellow,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::DarkGrey,
            prompt_info_fg: Color::White,
            prompt_success_bg: Color::Green,
            prompt_success_fg: Color::Black,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: Color::Black,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::White,
            dialog_button_primary_info_bg: Color::DarkCyan,
            dialog_button_primary_success_fg: Color::White,
            dialog_button_primary_success_bg: Color::DarkGreen,
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::DarkYellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::DarkRed,
            dialog_button_secondary_fg: Color::White,
            dialog_button_secondary_bg: Color::DarkGrey,

            // Config window
            config_title_bg: Color::Blue,
            config_title_fg: Color::White,
            config_border: Color::Cyan,
            config_content_bg: Color::Black,
            config_content_fg: Color::White,
            config_instructions_fg: Color::DarkGrey,
            config_toggle_on_color: Color::Green,
            config_toggle_off_color: Color::DarkGrey,

            // Calendar
            calendar_bg: Color::Blue,
            calendar_fg: Color::White,
            calendar_title_color: Color::White,
            calendar_today_bg: Color::Cyan,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkGrey,
            scrollbar_thumb_fg: Color::White,

            // Snap preview
            snap_preview_border: Color::Yellow,
            snap_preview_bg: Color::Black,

            // Splash screen
            splash_border: Color::White,
            splash_bg: Color::DarkBlue,
            splash_fg: Color::White,
        }
    }

    /// Monochrome grayscale theme
    pub fn monochrome() -> Self {
        Self {
            // Desktop
            desktop_bg: Color::DarkGrey,
            desktop_fg: Color::White,

            // Top bar
            topbar_bg_desktop: Color::Grey,
            topbar_bg_window: Color::Black,
            topbar_fg: Color::White,
            clock_bg: Color::Black,
            clock_fg: Color::White,

            // Windows
            window_title_bg: Color::DarkGrey,
            window_title_bg_focused: Color::Grey,
            window_title_fg: Color::White,
            window_border: Color::White,
            window_content_bg: Color::Black,
            window_content_fg: Color::White,
            window_shadow_color: Color::DarkGrey,

            // Window controls
            button_close_color: Color::White,
            button_maximize_color: Color::Grey,
            button_minimize_color: Color::DarkGrey,
            button_bg: Color::Black, // Consistent button background
            resize_handle_normal_fg: Color::Grey,
            resize_handle_normal_bg: Color::Black,
            resize_handle_active_fg: Color::White,
            resize_handle_active_bg: Color::Grey,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::White,
            button_hovered_fg: Color::White,
            button_hovered_bg: Color::Grey,
            button_pressed_fg: Color::White,
            button_pressed_bg: Color::Black,

            // Bottom bar
            bottombar_bg: Color::Black,
            bottombar_fg: Color::White,
            bottombar_button_normal_fg: Color::White,
            bottombar_button_normal_bg: Color::Black,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::White,
            bottombar_button_minimized_fg: Color::DarkGrey,
            bottombar_button_minimized_bg: Color::Black,

            // Toggle button
            toggle_enabled_fg: Color::White,
            toggle_enabled_bg_normal: Color::Grey,
            toggle_enabled_bg_hovered: Color::DarkGrey,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::DarkGrey,
            toggle_disabled_bg_normal: Color::Black,
            toggle_disabled_bg_hovered: Color::DarkGrey,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::Grey,
            prompt_info_fg: Color::Black,
            prompt_success_bg: Color::White,
            prompt_success_fg: Color::Black,
            prompt_warning_bg: Color::Grey,
            prompt_warning_fg: Color::White,
            prompt_danger_bg: Color::DarkGrey,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::White,
            dialog_button_primary_info_bg: Color::DarkGrey,
            dialog_button_primary_success_fg: Color::Black,
            dialog_button_primary_success_bg: Color::White,
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::Grey,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Black,
            dialog_button_secondary_fg: Color::White,
            dialog_button_secondary_bg: Color::DarkGrey,

            // Config window
            config_title_bg: Color::DarkGrey,
            config_title_fg: Color::White,
            config_border: Color::White,
            config_content_bg: Color::Black,
            config_content_fg: Color::White,
            config_instructions_fg: Color::Grey,
            config_toggle_on_color: Color::White,
            config_toggle_off_color: Color::DarkGrey,

            // Calendar
            calendar_bg: Color::DarkGrey,
            calendar_fg: Color::White,
            calendar_title_color: Color::White,
            calendar_today_bg: Color::White,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkGrey,
            scrollbar_thumb_fg: Color::White,

            // Snap preview
            snap_preview_border: Color::White,
            snap_preview_bg: Color::Black,

            // Splash screen
            splash_border: Color::White,
            splash_bg: Color::Black,
            splash_fg: Color::White,
        }
    }

    /// Dark theme inspired by Dracula (draculatheme.com)
    /// Background: #282A36 (Black), Foreground: #F8F8F2 (White)
    /// Accent colors: Cyan #8BE9FD, Purple #BD93F9, Pink #FF79C6, Green #50FA7B, Red #FF5555, Yellow #F1FA8C
    pub fn dark() -> Self {
        Self {
            // Desktop - Dracula background
            desktop_bg: Color::Black,
            desktop_fg: Color::White,

            // Top bar
            topbar_bg_desktop: Color::DarkGrey,
            topbar_bg_window: Color::Black,
            topbar_fg: Color::White,
            clock_bg: Color::Black,
            clock_fg: Color::Magenta, // Purple accent

            // Windows - Dracula purple/cyan accents
            window_title_bg: Color::DarkGrey,
            window_title_bg_focused: Color::DarkMagenta, // Purple accent for focus
            window_title_fg: Color::White,
            window_border: Color::Magenta, // Purple borders
            window_content_bg: Color::Black,
            window_content_fg: Color::White, // Bright foreground like Dracula
            window_shadow_color: Color::DarkGrey,

            // Window controls - Dracula semantic colors
            button_close_color: Color::Red,      // Dracula red #FF5555
            button_maximize_color: Color::Green, // Dracula green #50FA7B
            button_minimize_color: Color::Yellow, // Dracula yellow #F1FA8C
            button_bg: Color::Black,             // Consistent button background
            resize_handle_normal_fg: Color::DarkGrey,
            resize_handle_normal_bg: Color::Black,
            resize_handle_active_fg: Color::Magenta, // Purple accent
            resize_handle_active_bg: Color::DarkGrey,

            // UI Buttons
            button_normal_fg: Color::White,
            button_normal_bg: Color::DarkGrey,
            button_hovered_fg: Color::Cyan, // Dracula cyan #8BE9FD
            button_hovered_bg: Color::DarkGrey,
            button_pressed_fg: Color::White,
            button_pressed_bg: Color::Black,

            // Bottom bar
            bottombar_bg: Color::Black,
            bottombar_fg: Color::White,
            bottombar_button_normal_fg: Color::White,
            bottombar_button_normal_bg: Color::Black,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::Magenta, // Purple accent
            bottombar_button_minimized_fg: Color::DarkGrey,
            bottombar_button_minimized_bg: Color::Black,

            // Toggle button
            toggle_enabled_fg: Color::Green, // Dracula green
            toggle_enabled_bg_normal: Color::DarkGrey,
            toggle_enabled_bg_hovered: Color::DarkGrey,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::DarkGrey,
            toggle_disabled_bg_normal: Color::Black,
            toggle_disabled_bg_hovered: Color::DarkGrey,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::DarkGrey,
            prompt_info_fg: Color::Cyan, // Dracula cyan
            prompt_success_bg: Color::DarkGreen,
            prompt_success_fg: Color::White,
            prompt_warning_bg: Color::DarkYellow,
            prompt_warning_fg: Color::Black,
            prompt_danger_bg: Color::DarkRed,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::Black,
            dialog_button_primary_info_bg: Color::Cyan, // Dracula cyan
            dialog_button_primary_success_fg: Color::Black,
            dialog_button_primary_success_bg: Color::Green, // Dracula green
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::Yellow, // Dracula yellow
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red, // Dracula red
            dialog_button_secondary_fg: Color::White,
            dialog_button_secondary_bg: Color::DarkGrey,

            // Config window
            config_title_bg: Color::DarkMagenta, // Purple title
            config_title_fg: Color::White,
            config_border: Color::Magenta, // Purple borders
            config_content_bg: Color::Black,
            config_content_fg: Color::White,
            config_instructions_fg: Color::DarkGrey,
            config_toggle_on_color: Color::Green, // Dracula green
            config_toggle_off_color: Color::DarkGrey,

            // Calendar
            calendar_bg: Color::Black,
            calendar_fg: Color::White,
            calendar_title_color: Color::Magenta, // Purple accent
            calendar_today_bg: Color::DarkMagenta, // Purple highlight
            calendar_today_fg: Color::White,

            // Scrollbar
            scrollbar_track_fg: Color::DarkGrey,
            scrollbar_thumb_fg: Color::Magenta, // Purple accent

            // Snap preview
            snap_preview_border: Color::Cyan, // Dracula cyan
            snap_preview_bg: Color::Black,

            // Splash screen
            splash_border: Color::Magenta, // Purple borders
            splash_bg: Color::Black,
            splash_fg: Color::Cyan, // Dracula cyan
        }
    }

    /// Green Phosphor theme inspired by classic green monochrome terminals (IBM 5151, VT220)
    /// Resembles the glow of green phosphor CRT displays
    pub fn green_phosphor() -> Self {
        Self {
            // Desktop - black CRT screen with green phosphor
            desktop_bg: Color::Black,
            desktop_fg: Color::Green,

            // Top bar
            topbar_bg_desktop: Color::DarkGreen,
            topbar_bg_window: Color::Black,
            topbar_fg: Color::Green,
            clock_bg: Color::Black,
            clock_fg: Color::Green,

            // Windows
            window_title_bg: Color::DarkGreen,
            window_title_bg_focused: Color::Green, // Bright green for focus
            window_title_fg: Color::Black,
            window_border: Color::Green,
            window_content_bg: Color::Black,
            window_content_fg: Color::Green,
            window_shadow_color: Color::DarkGreen,

            // Window controls
            button_close_color: Color::Green,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Green,
            button_bg: Color::Black,
            resize_handle_normal_fg: Color::DarkGreen,
            resize_handle_normal_bg: Color::Black,
            resize_handle_active_fg: Color::Green,
            resize_handle_active_bg: Color::DarkGreen,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::Green,
            button_hovered_fg: Color::Green,
            button_hovered_bg: Color::DarkGreen,
            button_pressed_fg: Color::Green,
            button_pressed_bg: Color::Black,

            // Bottom bar
            bottombar_bg: Color::Black,
            bottombar_fg: Color::Green,
            bottombar_button_normal_fg: Color::Green,
            bottombar_button_normal_bg: Color::Black,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::Green,
            bottombar_button_minimized_fg: Color::DarkGreen,
            bottombar_button_minimized_bg: Color::Black,

            // Toggle button
            toggle_enabled_fg: Color::Green,
            toggle_enabled_bg_normal: Color::DarkGreen,
            toggle_enabled_bg_hovered: Color::Green,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::DarkGreen,
            toggle_disabled_bg_normal: Color::Black,
            toggle_disabled_bg_hovered: Color::DarkGreen,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::DarkGreen,
            prompt_info_fg: PURE_BLACK, // Black text for contrast
            prompt_success_bg: Color::Green,
            prompt_success_fg: PURE_BLACK,
            prompt_warning_bg: Color::DarkGreen,
            prompt_warning_fg: PURE_BLACK,
            prompt_danger_bg: Color::DarkGreen,
            prompt_danger_fg: PURE_BLACK, // Black text on green for readability

            // Dialog buttons
            dialog_button_primary_info_fg: PURE_BLACK, // Pure black for maximum contrast
            dialog_button_primary_info_bg: Color::Green,
            dialog_button_primary_success_fg: PURE_BLACK,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: PURE_BLACK,
            dialog_button_primary_warning_bg: Color::DarkGreen,
            dialog_button_primary_danger_fg: PURE_BLACK, // Pure black for Exit button
            dialog_button_primary_danger_bg: DARK_GREEN_PHOSPHOR, // Subtle, de-emphasized
            dialog_button_secondary_fg: PURE_BLACK,      // Black text for Cancel button
            dialog_button_secondary_bg: Color::Green,    // Bright, emphasized as safe option

            // Config window
            config_title_bg: Color::DarkGreen,
            config_title_fg: Color::Green,
            config_border: Color::Green,
            config_content_bg: Color::Black,
            config_content_fg: Color::Green,
            config_instructions_fg: Color::DarkGreen,
            config_toggle_on_color: Color::Green,
            config_toggle_off_color: Color::DarkGreen,

            // Calendar
            calendar_bg: Color::Black,
            calendar_fg: Color::Green,
            calendar_title_color: Color::Green,
            calendar_today_bg: Color::Green,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkGreen,
            scrollbar_thumb_fg: Color::Green,

            // Snap preview
            snap_preview_border: Color::Green,
            snap_preview_bg: Color::Black,

            // Splash screen
            splash_border: Color::Green,
            splash_bg: Color::Black,
            splash_fg: Color::Green,
        }
    }

    /// Amber theme inspired by classic amber monochrome terminals (DEC VT100, Wyse terminals)
    /// Resembles the warm glow of amber phosphor CRT displays
    pub fn amber() -> Self {
        Self {
            // Desktop - black CRT screen with amber phosphor
            desktop_bg: Color::Black,
            desktop_fg: Color::Yellow,

            // Top bar
            topbar_bg_desktop: Color::DarkYellow,
            topbar_bg_window: Color::Black,
            topbar_fg: Color::Yellow,
            clock_bg: Color::Black,
            clock_fg: Color::Yellow,

            // Windows
            window_title_bg: Color::DarkYellow,
            window_title_bg_focused: Color::Yellow, // Bright amber for focus
            window_title_fg: Color::Black,
            window_border: Color::Yellow,
            window_content_bg: Color::Black,
            window_content_fg: Color::Yellow,
            window_shadow_color: Color::DarkYellow,

            // Window controls
            button_close_color: Color::Yellow,
            button_maximize_color: Color::Yellow,
            button_minimize_color: Color::Yellow,
            button_bg: Color::Black,
            resize_handle_normal_fg: Color::DarkYellow,
            resize_handle_normal_bg: Color::Black,
            resize_handle_active_fg: Color::Yellow,
            resize_handle_active_bg: Color::DarkYellow,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::Yellow,
            button_hovered_fg: Color::Yellow,
            button_hovered_bg: Color::DarkYellow,
            button_pressed_fg: Color::Yellow,
            button_pressed_bg: Color::Black,

            // Bottom bar
            bottombar_bg: Color::Black,
            bottombar_fg: Color::Yellow,
            bottombar_button_normal_fg: Color::Yellow,
            bottombar_button_normal_bg: Color::Black,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::Yellow,
            bottombar_button_minimized_fg: Color::DarkYellow,
            bottombar_button_minimized_bg: Color::Black,

            // Toggle button
            toggle_enabled_fg: Color::Yellow,
            toggle_enabled_bg_normal: Color::DarkYellow,
            toggle_enabled_bg_hovered: Color::Yellow,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::DarkYellow,
            toggle_disabled_bg_normal: Color::Black,
            toggle_disabled_bg_hovered: Color::DarkYellow,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::DarkYellow,
            prompt_info_fg: PURE_BLACK, // Black text for contrast
            prompt_success_bg: Color::Yellow,
            prompt_success_fg: PURE_BLACK,
            prompt_warning_bg: Color::DarkYellow,
            prompt_warning_fg: PURE_BLACK,
            prompt_danger_bg: Color::DarkYellow,
            prompt_danger_fg: PURE_BLACK, // Black text on amber for readability

            // Dialog buttons
            dialog_button_primary_info_fg: PURE_BLACK, // Pure black for maximum contrast
            dialog_button_primary_info_bg: Color::Yellow,
            dialog_button_primary_success_fg: PURE_BLACK,
            dialog_button_primary_success_bg: Color::Yellow,
            dialog_button_primary_warning_fg: PURE_BLACK,
            dialog_button_primary_warning_bg: Color::DarkYellow,
            dialog_button_primary_danger_fg: PURE_BLACK, // Pure black for Exit button
            dialog_button_primary_danger_bg: LIGHT_AMBER, // Subtle, de-emphasized
            dialog_button_secondary_fg: PURE_BLACK,      // Black text for Cancel button
            dialog_button_secondary_bg: Color::Yellow,   // Bright, emphasized as safe option

            // Config window
            config_title_bg: Color::DarkYellow,
            config_title_fg: Color::Yellow,
            config_border: Color::Yellow,
            config_content_bg: Color::Black,
            config_content_fg: Color::Yellow,
            config_instructions_fg: Color::DarkYellow,
            config_toggle_on_color: Color::Yellow,
            config_toggle_off_color: Color::DarkYellow,

            // Calendar
            calendar_bg: Color::Black,
            calendar_fg: Color::Yellow,
            calendar_title_color: Color::Yellow,
            calendar_today_bg: Color::Yellow,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkYellow,
            scrollbar_thumb_fg: Color::Yellow,

            // Snap preview
            snap_preview_border: Color::Yellow,
            snap_preview_bg: Color::Black,

            // Splash screen
            splash_border: Color::Yellow,
            splash_bg: Color::Black,
            splash_fg: Color::Yellow,
        }
    }

    /// Create a theme from a name string, falling back to Classic if invalid
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "classic" => Self::classic(),
            "monochrome" => Self::monochrome(),
            "dark" => Self::dark(),
            "green" | "green_phosphor" | "greenphosphor" => Self::green_phosphor(),
            "amber" | "orange" => Self::amber(),
            _ => {
                eprintln!("Unknown theme '{}', falling back to 'classic'", name);
                Self::classic()
            }
        }
    }
}
