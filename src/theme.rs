use crossterm::style::Color;

// Custom color constants for better readability and maintainability
// Pure colors
const PURE_BLACK: Color = Color::Rgb { r: 0, g: 0, b: 0 };

// Amber/Yellow variants - authentic P3 amber phosphor colors (602nm wavelength)
const LIGHT_AMBER: Color = Color::Rgb {
    r: 255,
    g: 176,
    b: 0,
};
const MID_AMBER: Color = Color::Rgb {
    r: 255,
    g: 213,
    b: 128,
};
const BRIGHT_AMBER: Color = Color::Rgb {
    r: 255,
    g: 191,
    b: 0,
};

// Green variants - authentic P39 phosphor colors (525nm wavelength)
const DARK_GREEN_PHOSPHOR: Color = Color::Rgb { r: 0, g: 120, b: 0 };
const MID_GREEN_PHOSPHOR: Color = Color::Rgb {
    r: 158,
    g: 211,
    b: 177,
};
const LIGHT_GREEN_PHOSPHOR: Color = Color::Rgb {
    r: 51,
    g: 255,
    b: 51,
};

// Dracula theme colors - official palette from draculatheme.com
const DRACULA_BACKGROUND: Color = Color::Rgb {
    r: 40,
    g: 42,
    b: 54,
};
const DRACULA_CURRENT_LINE: Color = Color::Rgb {
    r: 68,
    g: 71,
    b: 90,
};
const DRACULA_FOREGROUND: Color = Color::Rgb {
    r: 248,
    g: 248,
    b: 242,
};
const DRACULA_COMMENT: Color = Color::Rgb {
    r: 98,
    g: 114,
    b: 164,
};
const DRACULA_CYAN: Color = Color::Rgb {
    r: 139,
    g: 233,
    b: 253,
};
const DRACULA_GREEN: Color = Color::Rgb {
    r: 80,
    g: 250,
    b: 123,
};
const DRACULA_ORANGE: Color = Color::Rgb {
    r: 255,
    g: 184,
    b: 108,
};
#[allow(dead_code)]
const DRACULA_PINK: Color = Color::Rgb {
    r: 255,
    g: 121,
    b: 198,
};
const DRACULA_PURPLE: Color = Color::Rgb {
    r: 189,
    g: 147,
    b: 249,
};
const DRACULA_RED: Color = Color::Rgb {
    r: 255,
    g: 85,
    b: 85,
};
const DRACULA_YELLOW: Color = Color::Rgb {
    r: 241,
    g: 250,
    b: 140,
};

// IntelliJ Darcula theme colors - official JetBrains IDE dark theme
const DARCULA_BACKGROUND: Color = Color::Rgb {
    r: 43,
    g: 43,
    b: 43,
};
const DARCULA_UI_BACKGROUND: Color = Color::Rgb {
    r: 60,
    g: 63,
    b: 65,
};
const DARCULA_FOREGROUND: Color = Color::Rgb {
    r: 169,
    g: 183,
    b: 198,
};
const DARCULA_ORANGE: Color = Color::Rgb {
    r: 204,
    g: 120,
    b: 50,
};
#[allow(dead_code)]
const DARCULA_STRING_GREEN: Color = Color::Rgb {
    r: 106,
    g: 135,
    b: 89,
};
const DARCULA_NUMBER_BLUE: Color = Color::Rgb {
    r: 104,
    g: 151,
    b: 187,
};
const DARCULA_COMMENT: Color = Color::Rgb {
    r: 128,
    g: 128,
    b: 128,
};
const DARCULA_FUNCTION_YELLOW: Color = Color::Rgb {
    r: 255,
    g: 198,
    b: 109,
};
const DARCULA_DOC_GREEN: Color = Color::Rgb {
    r: 98,
    g: 151,
    b: 85,
};
#[allow(dead_code)]
const DARCULA_CARET_ROW: Color = Color::Rgb {
    r: 50,
    g: 50,
    b: 50,
};

#[derive(Debug, Clone)]
pub struct Theme {
    // Desktop
    pub desktop_bg: Color,
    pub desktop_fg: Color,

    // Top bar
    pub topbar_bg_unfocused: Color,
    pub topbar_bg_focused: Color,
    pub topbar_fg_unfocused: Color,
    pub topbar_fg_focused: Color,
    pub clock_bg: Color,
    pub clock_fg: Color,

    // Windows - Title bar colors
    pub window_title_unfocused_fg: Color,
    pub window_title_unfocused_bg: Color,
    pub window_title_focused_fg: Color,
    pub window_title_focused_bg: Color,
    // Windows - Border colors
    pub window_border_unfocused_fg: Color,
    pub window_border_unfocused_bg: Color,
    pub window_border_focused_fg: Color,
    pub window_border_focused_bg: Color,
    // Windows - Content area
    pub window_content_bg: Color,
    pub window_content_fg: Color,
    pub window_shadow_color: Color,

    // Window controls
    pub button_close_color: Color,
    pub button_maximize_color: Color,
    pub button_minimize_color: Color,
    pub button_bg: Color, // Consistent background for title bar buttons
    // Resize handle colors (for future implementation of active resize state)
    #[allow(dead_code)]
    pub resize_handle_normal_fg: Color,
    #[allow(dead_code)]
    pub resize_handle_normal_bg: Color,
    pub resize_handle_active_fg: Color, // Used in terminal_window.rs for color tinting
    #[allow(dead_code)]
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

    // Context menu
    pub menu_bg: Color,
    pub menu_fg: Color,
    pub menu_border: Color,
    pub menu_selected_bg: Color,
    pub menu_selected_fg: Color,
    pub menu_shadow_fg: Color,

    // Snap preview
    pub snap_preview_border: Color,
    pub snap_preview_bg: Color,

    // Splash screen
    pub splash_border: Color,
    pub splash_bg: Color,
    pub splash_fg: Color,

    // Slight input popup
    pub slight_bg: Color,
    pub slight_fg: Color,
    pub slight_border: Color,
    pub slight_input_bg: Color,
    pub slight_input_fg: Color,
    pub slight_suggestion_fg: Color, // Gray text for inline suggestions
    pub slight_dropdown_bg: Color,
    pub slight_dropdown_fg: Color,
    pub slight_dropdown_selected_bg: Color,
    pub slight_dropdown_selected_fg: Color,
}

impl Theme {
    /// Classic DOS-inspired theme (current default colors)
    pub fn classic() -> Self {
        Self {
            // Desktop
            desktop_bg: Color::Blue,
            desktop_fg: Color::White,

            // Top bar
            topbar_bg_focused: Color::Cyan,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::White,
            topbar_fg_unfocused: Color::White,
            clock_bg: Color::DarkGrey,
            clock_fg: Color::White,

            // Windows - Title bar
            window_title_unfocused_fg: Color::White,
            window_title_unfocused_bg: Color::DarkGrey,
            window_title_focused_fg: Color::White,
            window_title_focused_bg: Color::DarkCyan,
            // Windows - Border
            window_border_unfocused_fg: Color::White,
            window_border_unfocused_bg: Color::DarkGrey,
            window_border_focused_fg: Color::Cyan,
            window_border_focused_bg: Color::DarkCyan,
            // Windows - Content
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
            bottombar_bg: Color::DarkGrey,
            bottombar_fg: Color::White,
            bottombar_button_normal_fg: Color::White,
            bottombar_button_normal_bg: Color::DarkGrey,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::Cyan,
            bottombar_button_minimized_fg: Color::Black,
            bottombar_button_minimized_bg: Color::DarkGrey,

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

            // Context menu
            menu_bg: Color::Black,
            menu_fg: Color::White,
            menu_border: Color::White,
            menu_selected_bg: Color::Cyan,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::DarkGrey,

            // Snap preview
            snap_preview_border: Color::Yellow,
            snap_preview_bg: Color::Black,

            // Splash screen
            splash_border: Color::White,
            splash_bg: PURE_BLACK,
            splash_fg: Color::White,

            // Slight input popup - dark Spotlight-like style
            slight_bg: Color::Black,
            slight_fg: Color::White,
            slight_border: Color::Cyan, // Primary theme color (topbar_bg_unfocused)
            slight_input_bg: Color::DarkGrey,
            slight_input_fg: Color::White,
            slight_suggestion_fg: Color::Yellow, // Yellow for clear distinction from input
            slight_dropdown_bg: Color::Black,    // Dark background
            slight_dropdown_fg: Color::White,
            slight_dropdown_selected_bg: Color::Cyan, // Cyan highlight
            slight_dropdown_selected_fg: Color::Black,
        }
    }

    /// Monochrome grayscale theme
    pub fn monochrome() -> Self {
        Self {
            // Desktop
            desktop_bg: Color::Black,
            desktop_fg: Color::White,

            // Top bar
            topbar_bg_focused: Color::Grey,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::White,
            topbar_fg_unfocused: Color::White,
            clock_bg: Color::Black,
            clock_fg: Color::White,

            // Windows - Title bar
            window_title_unfocused_fg: Color::White,
            window_title_unfocused_bg: Color::DarkGrey,
            window_title_focused_fg: Color::White,
            window_title_focused_bg: Color::Grey,
            // Windows - Border
            window_border_unfocused_fg: Color::Grey,
            window_border_unfocused_bg: Color::DarkGrey,
            window_border_focused_fg: Color::White,
            window_border_focused_bg: Color::Grey,
            // Windows - Content
            window_content_bg: Color::Black,
            window_content_fg: Color::White,
            window_shadow_color: Color::DarkGrey,

            // Window controls
            button_close_color: Color::White,
            button_maximize_color: Color::Grey,
            button_minimize_color: Color::Grey, // Changed from DarkGrey for better contrast on Black
            button_bg: Color::Black,            // Consistent button background
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

            // Context menu
            menu_bg: Color::Black,
            menu_fg: Color::White,
            menu_border: Color::White,
            menu_selected_bg: Color::White,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::DarkGrey,

            // Snap preview
            snap_preview_border: Color::White,
            snap_preview_bg: Color::Black,

            // Splash screen
            splash_border: Color::White,
            splash_bg: PURE_BLACK,
            splash_fg: Color::White,

            // Slight input popup - dark Spotlight-like style
            slight_bg: Color::Black,
            slight_fg: Color::White,
            slight_border: Color::Grey, // Primary theme color (topbar_bg_unfocused)
            slight_input_bg: Color::DarkGrey,
            slight_input_fg: Color::White,
            slight_suggestion_fg: Color::Grey, // Grey for distinction from white input
            slight_dropdown_bg: Color::Black,  // Dark background
            slight_dropdown_fg: Color::White,
            slight_dropdown_selected_bg: Color::White, // Inverted
            slight_dropdown_selected_fg: Color::Black,
        }
    }

    /// Dark theme inspired by Dracula (draculatheme.com)
    /// Background: #282A36, Foreground: #F8F8F2
    /// Accent colors: Cyan #8BE9FD, Purple #BD93F9, Pink #FF79C6, Green #50FA7B, Red #FF5555, Yellow #F1FA8C
    pub fn dark() -> Self {
        Self {
            // Desktop - Dracula background
            desktop_bg: DRACULA_BACKGROUND,
            desktop_fg: DRACULA_FOREGROUND,

            // Top bar
            topbar_bg_focused: DRACULA_CURRENT_LINE,
            topbar_bg_unfocused: DRACULA_BACKGROUND,
            topbar_fg_focused: DRACULA_FOREGROUND,
            topbar_fg_unfocused: DRACULA_FOREGROUND,
            clock_bg: DRACULA_BACKGROUND,
            clock_fg: DRACULA_PURPLE, // Purple accent

            // Windows - Title bar (Dracula purple/cyan accents)
            window_title_unfocused_fg: DRACULA_FOREGROUND,
            window_title_unfocused_bg: DRACULA_BACKGROUND,
            window_title_focused_fg: DRACULA_FOREGROUND,
            window_title_focused_bg: DRACULA_CURRENT_LINE, // Selection color for focus
            // Windows - Border
            window_border_unfocused_fg: DRACULA_COMMENT, // Dim purple/blue for unfocused
            window_border_unfocused_bg: DRACULA_BACKGROUND,
            window_border_focused_fg: DRACULA_CYAN, // Bright cyan for focused
            window_border_focused_bg: DRACULA_CURRENT_LINE,
            // Windows - Content
            window_content_bg: DRACULA_BACKGROUND, // True dark background
            window_content_fg: DRACULA_FOREGROUND, // Good contrast on dark background
            window_shadow_color: PURE_BLACK,

            // Window controls - Dracula semantic colors
            button_close_color: DRACULA_RED, // Dracula red #FF5555
            button_maximize_color: DRACULA_GREEN, // Dracula green #50FA7B
            button_minimize_color: DRACULA_YELLOW, // Dracula yellow #F1FA8C
            button_bg: DRACULA_BACKGROUND,   // Consistent button background
            resize_handle_normal_fg: DRACULA_COMMENT,
            resize_handle_normal_bg: DRACULA_BACKGROUND,
            resize_handle_active_fg: DRACULA_PURPLE, // Purple accent
            resize_handle_active_bg: DRACULA_CURRENT_LINE,

            // UI Buttons
            button_normal_fg: DRACULA_FOREGROUND,
            button_normal_bg: DRACULA_CURRENT_LINE,
            button_hovered_fg: DRACULA_CYAN, // Dracula cyan #8BE9FD
            button_hovered_bg: DRACULA_CURRENT_LINE,
            button_pressed_fg: DRACULA_FOREGROUND,
            button_pressed_bg: DRACULA_BACKGROUND,

            // Bottom bar
            bottombar_bg: DRACULA_BACKGROUND,
            bottombar_fg: DRACULA_FOREGROUND,
            bottombar_button_normal_fg: DRACULA_FOREGROUND,
            bottombar_button_normal_bg: DRACULA_BACKGROUND,
            bottombar_button_focused_fg: DRACULA_BACKGROUND,
            bottombar_button_focused_bg: DRACULA_PURPLE, // Purple accent
            bottombar_button_minimized_fg: DRACULA_COMMENT,
            bottombar_button_minimized_bg: DRACULA_BACKGROUND,

            // Toggle button
            toggle_enabled_fg: DRACULA_GREEN, // Dracula green
            toggle_enabled_bg_normal: DRACULA_CURRENT_LINE,
            toggle_enabled_bg_hovered: DRACULA_CURRENT_LINE,
            toggle_enabled_bg_pressed: DRACULA_BACKGROUND,
            toggle_disabled_fg: DRACULA_COMMENT,
            toggle_disabled_bg_normal: DRACULA_BACKGROUND,
            toggle_disabled_bg_hovered: DRACULA_CURRENT_LINE,
            toggle_disabled_bg_pressed: DRACULA_BACKGROUND,

            // Prompts/Dialogs
            prompt_info_bg: DRACULA_CURRENT_LINE,
            prompt_info_fg: DRACULA_CYAN, // Dracula cyan
            prompt_success_bg: DRACULA_GREEN,
            prompt_success_fg: DRACULA_BACKGROUND,
            prompt_warning_bg: DRACULA_ORANGE,
            prompt_warning_fg: DRACULA_BACKGROUND,
            prompt_danger_bg: DRACULA_RED,
            prompt_danger_fg: DRACULA_FOREGROUND,

            // Dialog buttons
            dialog_button_primary_info_fg: DRACULA_BACKGROUND,
            dialog_button_primary_info_bg: DRACULA_CYAN, // Dracula cyan
            dialog_button_primary_success_fg: DRACULA_BACKGROUND,
            dialog_button_primary_success_bg: DRACULA_GREEN, // Dracula green
            dialog_button_primary_warning_fg: DRACULA_BACKGROUND,
            dialog_button_primary_warning_bg: DRACULA_YELLOW, // Dracula yellow
            dialog_button_primary_danger_fg: DRACULA_FOREGROUND,
            dialog_button_primary_danger_bg: DRACULA_RED, // Dracula red
            dialog_button_secondary_fg: DRACULA_FOREGROUND,
            dialog_button_secondary_bg: DRACULA_CURRENT_LINE,

            // Config window
            config_title_bg: DRACULA_CURRENT_LINE, // Selection color for title
            config_title_fg: DRACULA_FOREGROUND,
            config_border: DRACULA_PURPLE, // Purple borders
            config_content_bg: DRACULA_BACKGROUND,
            config_content_fg: DRACULA_FOREGROUND,
            config_instructions_fg: DRACULA_COMMENT,
            config_toggle_on_color: DRACULA_GREEN, // Dracula green
            config_toggle_off_color: DRACULA_COMMENT,

            // Calendar
            calendar_bg: DRACULA_BACKGROUND,
            calendar_fg: DRACULA_FOREGROUND,
            calendar_title_color: DRACULA_PURPLE, // Purple accent
            calendar_today_bg: DRACULA_PURPLE,    // Purple highlight
            calendar_today_fg: DRACULA_FOREGROUND,

            // Scrollbar
            scrollbar_track_fg: DRACULA_COMMENT,
            scrollbar_thumb_fg: DRACULA_PURPLE, // Purple accent

            // Context menu
            menu_bg: DRACULA_BACKGROUND,
            menu_fg: DRACULA_FOREGROUND,
            menu_border: DRACULA_PURPLE,
            menu_selected_bg: DRACULA_PURPLE,
            menu_selected_fg: DRACULA_FOREGROUND,
            menu_shadow_fg: PURE_BLACK,

            // Snap preview
            snap_preview_border: DRACULA_CYAN, // Dracula cyan
            snap_preview_bg: DRACULA_BACKGROUND,

            // Splash screen - theme background with primary accent
            splash_border: DRACULA_PURPLE,
            splash_bg: DRACULA_BACKGROUND,
            splash_fg: DRACULA_CYAN, // Primary cyan accent

            // Slight input popup - dark Spotlight-like style
            slight_bg: DRACULA_BACKGROUND,
            slight_fg: DRACULA_FOREGROUND,
            slight_border: DRACULA_CURRENT_LINE, // Current line color
            slight_input_bg: DRACULA_CURRENT_LINE,
            slight_input_fg: DRACULA_CYAN,        // Dracula cyan
            slight_suggestion_fg: DRACULA_YELLOW, // Yellow for clear distinction
            slight_dropdown_bg: DRACULA_BACKGROUND,
            slight_dropdown_fg: DRACULA_CYAN, // Dracula cyan
            slight_dropdown_selected_bg: DRACULA_PURPLE, // Purple highlight
            slight_dropdown_selected_fg: DRACULA_FOREGROUND,
        }
    }

    /// Dracu theme inspired by IntelliJ IDEA Darcula (JetBrains IDE dark theme)
    /// Background: #2B2B2B, Foreground: #A9B7C6
    /// Accent colors: Orange #CC7832, Blue #6897BB, Green #6A8759, Yellow #FFC66D
    pub fn dracu() -> Self {
        Self {
            // Desktop - Darcula background
            desktop_bg: DARCULA_BACKGROUND,
            desktop_fg: DARCULA_FOREGROUND,

            // Top bar
            topbar_bg_focused: DARCULA_UI_BACKGROUND,
            topbar_bg_unfocused: DARCULA_BACKGROUND,
            topbar_fg_focused: DARCULA_FOREGROUND,
            topbar_fg_unfocused: DARCULA_FOREGROUND,
            clock_bg: DARCULA_BACKGROUND,
            clock_fg: DARCULA_NUMBER_BLUE, // Blue accent

            // Windows - Title bar (Darcula orange/blue accents)
            window_title_unfocused_fg: DARCULA_FOREGROUND,
            window_title_unfocused_bg: DARCULA_BACKGROUND,
            window_title_focused_fg: DARCULA_FOREGROUND,
            window_title_focused_bg: DARCULA_UI_BACKGROUND, // UI background for focus
            // Windows - Border
            window_border_unfocused_fg: DARCULA_COMMENT, // Grey for unfocused
            window_border_unfocused_bg: DARCULA_BACKGROUND,
            window_border_focused_fg: DARCULA_ORANGE, // Orange for focused
            window_border_focused_bg: DARCULA_UI_BACKGROUND,
            // Windows - Content
            window_content_bg: DARCULA_BACKGROUND, // True dark background
            window_content_fg: DARCULA_FOREGROUND, // Good contrast on dark background
            window_shadow_color: PURE_BLACK,

            // Window controls - Darcula semantic colors
            button_close_color: Color::Rgb {
                r: 255,
                g: 75,
                b: 75,
            }, // Red for close
            button_maximize_color: DARCULA_DOC_GREEN, // Green for maximize
            button_minimize_color: DARCULA_FUNCTION_YELLOW, // Yellow for minimize
            button_bg: DARCULA_BACKGROUND,            // Consistent button background
            resize_handle_normal_fg: DARCULA_COMMENT,
            resize_handle_normal_bg: DARCULA_BACKGROUND,
            resize_handle_active_fg: DARCULA_ORANGE, // Orange accent
            resize_handle_active_bg: DARCULA_UI_BACKGROUND,

            // UI Buttons
            button_normal_fg: DARCULA_FOREGROUND,
            button_normal_bg: DARCULA_UI_BACKGROUND,
            button_hovered_fg: DARCULA_FUNCTION_YELLOW, // Yellow hover
            button_hovered_bg: DARCULA_UI_BACKGROUND,
            button_pressed_fg: DARCULA_FOREGROUND,
            button_pressed_bg: DARCULA_BACKGROUND,

            // Bottom bar
            bottombar_bg: DARCULA_BACKGROUND,
            bottombar_fg: DARCULA_FOREGROUND,
            bottombar_button_normal_fg: DARCULA_FOREGROUND,
            bottombar_button_normal_bg: DARCULA_BACKGROUND,
            bottombar_button_focused_fg: DARCULA_BACKGROUND,
            bottombar_button_focused_bg: DARCULA_ORANGE, // Orange accent
            bottombar_button_minimized_fg: DARCULA_COMMENT,
            bottombar_button_minimized_bg: DARCULA_BACKGROUND,

            // Toggle button
            toggle_enabled_fg: DARCULA_DOC_GREEN, // Green for enabled
            toggle_enabled_bg_normal: DARCULA_UI_BACKGROUND,
            toggle_enabled_bg_hovered: DARCULA_UI_BACKGROUND,
            toggle_enabled_bg_pressed: DARCULA_BACKGROUND,
            toggle_disabled_fg: DARCULA_COMMENT,
            toggle_disabled_bg_normal: DARCULA_BACKGROUND,
            toggle_disabled_bg_hovered: DARCULA_UI_BACKGROUND,
            toggle_disabled_bg_pressed: DARCULA_BACKGROUND,

            // Prompts/Dialogs
            prompt_info_bg: DARCULA_UI_BACKGROUND,
            prompt_info_fg: DARCULA_NUMBER_BLUE, // Blue for info
            prompt_success_bg: DARCULA_DOC_GREEN,
            prompt_success_fg: DARCULA_BACKGROUND,
            prompt_warning_bg: DARCULA_ORANGE,
            prompt_warning_fg: DARCULA_BACKGROUND,
            prompt_danger_bg: Color::Rgb {
                r: 255,
                g: 75,
                b: 75,
            },
            prompt_danger_fg: DARCULA_FOREGROUND,

            // Dialog buttons
            dialog_button_primary_info_fg: DARCULA_BACKGROUND,
            dialog_button_primary_info_bg: DARCULA_NUMBER_BLUE, // Blue
            dialog_button_primary_success_fg: DARCULA_BACKGROUND,
            dialog_button_primary_success_bg: DARCULA_DOC_GREEN, // Green
            dialog_button_primary_warning_fg: DARCULA_BACKGROUND,
            dialog_button_primary_warning_bg: DARCULA_FUNCTION_YELLOW, // Yellow
            dialog_button_primary_danger_fg: DARCULA_FOREGROUND,
            dialog_button_primary_danger_bg: Color::Rgb {
                r: 255,
                g: 75,
                b: 75,
            }, // Red
            dialog_button_secondary_fg: DARCULA_FOREGROUND,
            dialog_button_secondary_bg: DARCULA_UI_BACKGROUND,

            // Config window
            config_title_bg: DARCULA_UI_BACKGROUND, // UI background for title
            config_title_fg: DARCULA_FOREGROUND,
            config_border: DARCULA_ORANGE, // Orange borders
            config_content_bg: DARCULA_BACKGROUND,
            config_content_fg: DARCULA_FOREGROUND,
            config_instructions_fg: DARCULA_COMMENT,
            config_toggle_on_color: DARCULA_DOC_GREEN, // Green
            config_toggle_off_color: DARCULA_COMMENT,

            // Calendar
            calendar_bg: DARCULA_BACKGROUND,
            calendar_fg: DARCULA_FOREGROUND,
            calendar_title_color: DARCULA_ORANGE, // Orange accent
            calendar_today_bg: DARCULA_ORANGE,    // Orange highlight
            calendar_today_fg: DARCULA_BACKGROUND,

            // Scrollbar
            scrollbar_track_fg: DARCULA_COMMENT,
            scrollbar_thumb_fg: DARCULA_ORANGE, // Orange accent

            // Context menu
            menu_bg: DARCULA_BACKGROUND,
            menu_fg: DARCULA_FOREGROUND,
            menu_border: DARCULA_ORANGE,
            menu_selected_bg: DARCULA_ORANGE,
            menu_selected_fg: DARCULA_BACKGROUND,
            menu_shadow_fg: PURE_BLACK,

            // Snap preview
            snap_preview_border: DARCULA_NUMBER_BLUE, // Blue
            snap_preview_bg: DARCULA_BACKGROUND,

            // Splash screen - theme background with primary accent
            splash_border: DARCULA_ORANGE,
            splash_bg: DARCULA_BACKGROUND,
            splash_fg: DARCULA_ORANGE, // Primary orange accent

            // Slight input popup - dark Spotlight-like style
            slight_bg: DARCULA_BACKGROUND,
            slight_fg: DARCULA_FOREGROUND,
            slight_border: DARCULA_UI_BACKGROUND, // UI background color
            slight_input_bg: DARCULA_UI_BACKGROUND,
            slight_input_fg: DARCULA_NUMBER_BLUE, // Blue
            slight_suggestion_fg: DARCULA_FUNCTION_YELLOW, // Yellow for clear distinction
            slight_dropdown_bg: DARCULA_BACKGROUND,
            slight_dropdown_fg: DARCULA_NUMBER_BLUE, // Blue
            slight_dropdown_selected_bg: DARCULA_ORANGE, // Orange highlight
            slight_dropdown_selected_fg: DARCULA_BACKGROUND,
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
            topbar_bg_focused: MID_GREEN_PHOSPHOR,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::Green,
            topbar_fg_unfocused: Color::Green,
            clock_bg: Color::Black,
            clock_fg: Color::Green,

            // Windows - Title bar
            window_title_unfocused_fg: Color::Green,
            window_title_unfocused_bg: Color::Black,
            window_title_focused_fg: Color::Black,
            window_title_focused_bg: MID_GREEN_PHOSPHOR,
            // Windows - Border
            window_border_unfocused_fg: Color::Green,
            window_border_unfocused_bg: Color::Black,
            window_border_focused_fg: Color::Black,
            window_border_focused_bg: Color::Green,
            // Windows - Content
            window_content_bg: Color::Black,
            window_content_fg: Color::Green,
            window_shadow_color: Color::DarkGreen,

            // Window controls - vary brightness for semantic distinction
            button_close_color: Color::Green, // Bright green for close (primary action)
            button_maximize_color: MID_GREEN_PHOSPHOR, // Mid green for maximize
            button_minimize_color: DARK_GREEN_PHOSPHOR, // Dimmest for minimize
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

            // Context menu
            menu_bg: Color::Black,
            menu_fg: Color::Green,
            menu_border: Color::Green,
            menu_selected_bg: Color::Green,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::DarkGreen,

            // Snap preview
            snap_preview_border: Color::Green,
            snap_preview_bg: Color::Black,

            // Splash screen
            splash_border: Color::Green,
            splash_bg: PURE_BLACK,
            splash_fg: Color::Green,

            // Slight input popup - dark Spotlight-like style
            slight_bg: Color::Black,
            slight_fg: Color::Green,
            slight_border: Color::DarkGreen, // Primary theme color (topbar_bg_unfocused)
            slight_input_bg: Color::DarkGreen,
            slight_input_fg: Color::Green,
            slight_suggestion_fg: LIGHT_GREEN_PHOSPHOR, // Light green for clear distinction
            slight_dropdown_bg: Color::Black,
            slight_dropdown_fg: Color::Green,
            slight_dropdown_selected_bg: Color::Green, // Inverted
            slight_dropdown_selected_fg: Color::Black,
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
            topbar_bg_focused: MID_AMBER,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::Black,
            topbar_fg_unfocused: Color::Yellow,
            clock_bg: Color::Black,
            clock_fg: Color::Yellow,

            // Windows - Title bar
            window_title_unfocused_fg: Color::Yellow,
            window_title_unfocused_bg: Color::Black,
            window_title_focused_fg: Color::Black,
            window_title_focused_bg: MID_AMBER, // Mid amber for focus
            // Windows - Border
            window_border_unfocused_fg: Color::DarkYellow,
            window_border_unfocused_bg: Color::Black,
            window_border_focused_fg: Color::Yellow,
            window_border_focused_bg: Color::DarkYellow,
            // Windows - Content
            window_content_bg: Color::Black,
            window_content_fg: Color::Yellow,
            window_shadow_color: Color::DarkYellow,

            // Window controls - vary brightness for semantic distinction
            button_close_color: Color::Yellow, // Bright amber for close (primary action)
            button_maximize_color: MID_AMBER, // Mid amber for maximize
            button_minimize_color: LIGHT_AMBER, // Dimmest for minimize
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

            // Context menu
            menu_bg: Color::Black,
            menu_fg: Color::Yellow,
            menu_border: Color::Yellow,
            menu_selected_bg: Color::Yellow,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::DarkYellow,

            // Snap preview
            snap_preview_border: Color::Yellow,
            snap_preview_bg: Color::Black,

            // Splash screen
            splash_border: Color::Yellow,
            splash_bg: PURE_BLACK,
            splash_fg: Color::Yellow,

            // Slight input popup - dark Spotlight-like style
            slight_bg: Color::Black,
            slight_fg: Color::Yellow,
            slight_border: Color::DarkYellow, // Primary theme color (topbar_bg_unfocused)
            slight_input_bg: Color::DarkYellow,
            slight_input_fg: Color::Yellow,
            slight_suggestion_fg: BRIGHT_AMBER, // Bright amber for clear distinction
            slight_dropdown_bg: Color::Black,
            slight_dropdown_fg: Color::Yellow,
            slight_dropdown_selected_bg: Color::Yellow, // Inverted
            slight_dropdown_selected_fg: Color::Black,
        }
    }

    /// NDD theme - inspired by classic disk utility interfaces
    /// Cyan text on dark blue backgrounds with white accents
    pub fn ndd() -> Self {
        Self {
            // Desktop - classic blue
            desktop_bg: Color::Black,
            desktop_fg: Color::Cyan,

            // Top bar
            topbar_bg_focused: Color::Cyan,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::Black,
            topbar_fg_unfocused: Color::Black,
            clock_bg: Color::DarkCyan,
            clock_fg: Color::White,

            // Windows - Title bar
            window_title_unfocused_fg: Color::Black,
            window_title_unfocused_bg: Color::DarkCyan,
            window_title_focused_fg: Color::Black,
            window_title_focused_bg: Color::Cyan,
            // Windows - Border
            window_border_unfocused_fg: Color::DarkCyan,
            window_border_unfocused_bg: Color::DarkCyan,
            window_border_focused_fg: Color::Cyan,
            window_border_focused_bg: Color::Cyan,
            // Windows - Content
            window_content_bg: Color::DarkBlue,
            window_content_fg: Color::Cyan,
            window_shadow_color: Color::Black,

            // Window controls
            button_close_color: Color::Red,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Yellow,
            button_bg: Color::Black,
            resize_handle_normal_fg: Color::DarkCyan,
            resize_handle_normal_bg: Color::DarkBlue,
            resize_handle_active_fg: Color::Cyan,
            resize_handle_active_bg: Color::DarkCyan,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::Cyan,
            button_hovered_fg: Color::Black,
            button_hovered_bg: Color::White,
            button_pressed_fg: Color::White,
            button_pressed_bg: Color::DarkCyan,

            // Bottom bar
            bottombar_bg: Color::Black,
            bottombar_fg: Color::Cyan,
            bottombar_button_normal_fg: Color::Cyan,
            bottombar_button_normal_bg: Color::Black,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::Cyan,
            bottombar_button_minimized_fg: Color::DarkCyan,
            bottombar_button_minimized_bg: Color::Black,

            // Toggle button
            toggle_enabled_fg: Color::Green,
            toggle_enabled_bg_normal: Color::DarkCyan,
            toggle_enabled_bg_hovered: Color::Cyan,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::White,
            toggle_disabled_bg_normal: Color::DarkCyan,
            toggle_disabled_bg_hovered: Color::Cyan,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::DarkCyan,
            prompt_info_fg: Color::White,
            prompt_success_bg: Color::Green,
            prompt_success_fg: Color::Black,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: Color::Black,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::Black,
            dialog_button_primary_info_bg: Color::Cyan,
            dialog_button_primary_success_fg: Color::Black,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: Color::White,
            dialog_button_secondary_bg: Color::DarkCyan,

            // Config window
            config_title_bg: Color::Cyan,
            config_title_fg: Color::Black,
            config_border: Color::Cyan,
            config_content_bg: Color::DarkBlue,
            config_content_fg: Color::Cyan,
            config_instructions_fg: Color::DarkCyan,
            config_toggle_on_color: Color::Green,
            config_toggle_off_color: Color::DarkCyan,

            // Calendar
            calendar_bg: Color::DarkBlue,
            calendar_fg: Color::Cyan,
            calendar_title_color: Color::White,
            calendar_today_bg: Color::Cyan,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkCyan,
            scrollbar_thumb_fg: Color::Cyan,

            // Context menu
            menu_bg: Color::DarkBlue,
            menu_fg: Color::Cyan,
            menu_border: Color::Cyan,
            menu_selected_bg: Color::Cyan,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::Black,

            // Snap preview
            snap_preview_border: Color::White,
            snap_preview_bg: Color::Black,

            // Splash screen - theme background with primary accent
            splash_border: Color::Cyan,
            splash_bg: Color::DarkBlue,
            splash_fg: Color::Cyan, // Primary cyan accent

            // Slight input popup
            slight_bg: Color::DarkBlue,
            slight_fg: Color::Cyan,
            slight_border: Color::Cyan,
            slight_input_bg: Color::DarkCyan,
            slight_input_fg: Color::White,
            slight_suggestion_fg: Color::Yellow,
            slight_dropdown_bg: Color::DarkBlue,
            slight_dropdown_fg: Color::Cyan,
            slight_dropdown_selected_bg: Color::Cyan,
            slight_dropdown_selected_fg: Color::Black,
        }
    }

    /// QBasic theme - inspired by classic BASIC IDE interfaces
    /// Yellow/white text on blue backgrounds
    pub fn qbasic() -> Self {
        Self {
            // Desktop - classic blue
            desktop_bg: Color::Black,
            desktop_fg: Color::White,

            // Top bar
            topbar_bg_focused: Color::White,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::Black,
            topbar_fg_unfocused: Color::Black,
            clock_bg: Color::DarkGrey,
            clock_fg: Color::White,

            // Windows - Title bar
            window_title_unfocused_fg: Color::Black,
            window_title_unfocused_bg: Color::DarkGrey,
            window_title_focused_fg: Color::Black,
            window_title_focused_bg: Color::White,
            // Windows - Border
            window_border_unfocused_fg: Color::Grey,
            window_border_unfocused_bg: Color::DarkGrey,
            window_border_focused_fg: Color::White,
            window_border_focused_bg: Color::White,
            window_content_bg: Color::DarkBlue,
            window_content_fg: Color::Yellow,
            window_shadow_color: Color::Black,

            // Window controls
            button_close_color: Color::Red,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Yellow,
            button_bg: Color::Black,
            resize_handle_normal_fg: Color::Grey,
            resize_handle_normal_bg: Color::DarkBlue,
            resize_handle_active_fg: Color::White,
            resize_handle_active_bg: Color::Grey,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::White,
            button_hovered_fg: Color::White,
            button_hovered_bg: Color::DarkGrey,
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
            toggle_enabled_fg: Color::Green,
            toggle_enabled_bg_normal: Color::DarkGrey,
            toggle_enabled_bg_hovered: Color::White,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::White,
            toggle_disabled_bg_normal: Color::DarkGrey,
            toggle_disabled_bg_hovered: Color::White,
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
            dialog_button_primary_info_fg: Color::Black,
            dialog_button_primary_info_bg: Color::White,
            dialog_button_primary_success_fg: Color::Black,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: Color::White,
            dialog_button_secondary_bg: Color::DarkGrey,

            // Config window
            config_title_bg: Color::White,
            config_title_fg: Color::Black,
            config_border: Color::White,
            config_content_bg: Color::DarkBlue,
            config_content_fg: Color::Yellow,
            config_instructions_fg: Color::Grey,
            config_toggle_on_color: Color::Green,
            config_toggle_off_color: Color::DarkGrey,

            // Calendar
            calendar_bg: Color::DarkBlue,
            calendar_fg: Color::Yellow,
            calendar_title_color: Color::White,
            calendar_today_bg: Color::White,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkGrey,
            scrollbar_thumb_fg: Color::White,

            // Context menu
            menu_bg: Color::DarkBlue,
            menu_fg: Color::White,
            menu_border: Color::White,
            menu_selected_bg: Color::White,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::Black,

            // Snap preview
            snap_preview_border: Color::Yellow,
            snap_preview_bg: Color::Black,

            // Splash screen - theme background with primary accent
            splash_border: Color::White,
            splash_bg: Color::DarkBlue,
            splash_fg: Color::Yellow, // Primary yellow accent

            // Slight input popup
            slight_bg: Color::DarkBlue,
            slight_fg: Color::Yellow,
            slight_border: Color::White,
            slight_input_bg: Color::DarkGrey,
            slight_input_fg: Color::White,
            slight_suggestion_fg: Color::Cyan,
            slight_dropdown_bg: Color::DarkBlue,
            slight_dropdown_fg: Color::Yellow,
            slight_dropdown_selected_bg: Color::White,
            slight_dropdown_selected_fg: Color::Black,
        }
    }

    /// TurboP theme - inspired by classic Pascal IDE interfaces
    /// Yellow text on grey/blue backgrounds
    pub fn turbo() -> Self {
        Self {
            // Desktop - grey background
            desktop_bg: Color::Black,
            desktop_fg: Color::Yellow,

            // Top bar
            topbar_bg_focused: Color::Yellow,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::Black,
            topbar_fg_unfocused: Color::Black,
            clock_bg: Color::Black,
            clock_fg: Color::Yellow,

            // Windows - Title bar
            window_title_unfocused_fg: Color::Black,
            window_title_unfocused_bg: Color::Grey,
            window_title_focused_fg: Color::Black,
            window_title_focused_bg: Color::Yellow,
            // Windows - Border
            window_border_unfocused_fg: Color::Grey,
            window_border_unfocused_bg: Color::Grey,
            window_border_focused_fg: Color::Yellow,
            window_border_focused_bg: Color::Yellow,
            // Windows - Content
            window_content_bg: Color::DarkBlue,
            window_content_fg: Color::Yellow,
            window_shadow_color: Color::Black,

            // Window controls
            button_close_color: Color::Red,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Cyan,
            button_bg: Color::Black,
            resize_handle_normal_fg: Color::Grey,
            resize_handle_normal_bg: Color::DarkBlue,
            resize_handle_active_fg: Color::Yellow,
            resize_handle_active_bg: Color::Grey,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::Yellow,
            button_hovered_fg: Color::Yellow,
            button_hovered_bg: Color::DarkGrey,
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
            toggle_enabled_fg: Color::Green,
            toggle_enabled_bg_normal: Color::Grey,
            toggle_enabled_bg_hovered: Color::Yellow,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::Yellow,
            toggle_disabled_bg_normal: Color::Grey,
            toggle_disabled_bg_hovered: Color::Yellow,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::Grey,
            prompt_info_fg: Color::Black,
            prompt_success_bg: Color::Green,
            prompt_success_fg: Color::Black,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: Color::Black,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::Black,
            dialog_button_primary_info_bg: Color::Yellow,
            dialog_button_primary_success_fg: Color::Black,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: Color::Black,
            dialog_button_secondary_bg: Color::Grey,

            // Config window
            config_title_bg: Color::Yellow,
            config_title_fg: Color::Black,
            config_border: Color::Yellow,
            config_content_bg: Color::DarkBlue,
            config_content_fg: Color::Yellow,
            config_instructions_fg: Color::Grey,
            config_toggle_on_color: Color::Green,
            config_toggle_off_color: Color::Grey,

            // Calendar
            calendar_bg: Color::DarkBlue,
            calendar_fg: Color::Yellow,
            calendar_title_color: Color::White,
            calendar_today_bg: Color::Yellow,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::Grey,
            scrollbar_thumb_fg: Color::Yellow,

            // Context menu
            menu_bg: Color::DarkBlue,
            menu_fg: Color::Yellow,
            menu_border: Color::Yellow,
            menu_selected_bg: Color::Yellow,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::Black,

            // Snap preview
            snap_preview_border: Color::Yellow,
            snap_preview_bg: Color::Black,

            // Splash screen - theme background with primary accent
            splash_border: Color::Yellow,
            splash_bg: Color::DarkGrey,
            splash_fg: Color::Yellow, // Primary yellow accent

            // Slight input popup
            slight_bg: Color::DarkBlue,
            slight_fg: Color::Yellow,
            slight_border: Color::Yellow,
            slight_input_bg: Color::Grey,
            slight_input_fg: Color::Black,
            slight_suggestion_fg: Color::White,
            slight_dropdown_bg: Color::DarkBlue,
            slight_dropdown_fg: Color::Yellow,
            slight_dropdown_selected_bg: Color::Yellow,
            slight_dropdown_selected_fg: Color::Black,
        }
    }

    /// NCC theme - inspired by classic file manager interfaces
    /// Cyan panels on blue background with yellow highlights
    pub fn norton_commander() -> Self {
        Self {
            // Desktop - blue background
            desktop_bg: Color::Blue,
            desktop_fg: Color::Cyan,

            // Top bar
            topbar_bg_focused: Color::Cyan,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::Black,
            topbar_fg_unfocused: Color::Black,
            clock_bg: Color::Black,
            clock_fg: Color::Cyan,

            // Windows - Title bar (cyan panels on blue background)
            window_title_unfocused_fg: Color::Black,
            window_title_unfocused_bg: Color::Black,
            window_title_focused_fg: Color::Black,
            window_title_focused_bg: Color::Cyan,
            // Windows - Border
            window_border_unfocused_fg: Color::Cyan,
            window_border_unfocused_bg: Color::Black,
            window_border_focused_fg: Color::Yellow,
            window_border_focused_bg: Color::Cyan,
            // Windows - Content
            window_content_bg: Color::Blue,
            window_content_fg: Color::Cyan,
            window_shadow_color: Color::Black,

            // Window controls
            button_close_color: Color::Red,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Yellow,
            button_bg: Color::Black,
            resize_handle_normal_fg: Color::Cyan,
            resize_handle_normal_bg: Color::Blue,
            resize_handle_active_fg: Color::Yellow,
            resize_handle_active_bg: Color::Cyan,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::Cyan,
            button_hovered_fg: Color::Black,
            button_hovered_bg: Color::Yellow,
            button_pressed_fg: Color::Cyan,
            button_pressed_bg: Color::Black,

            // Bottom bar
            bottombar_bg: Color::Black,
            bottombar_fg: Color::Cyan,
            bottombar_button_normal_fg: Color::Cyan,
            bottombar_button_normal_bg: Color::Black,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::Cyan,
            bottombar_button_minimized_fg: Color::DarkCyan,
            bottombar_button_minimized_bg: Color::Black,

            // Toggle button
            toggle_enabled_fg: Color::Yellow,
            toggle_enabled_bg_normal: Color::Blue,
            toggle_enabled_bg_hovered: Color::Cyan,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::DarkCyan,
            toggle_disabled_bg_normal: Color::Blue,
            toggle_disabled_bg_hovered: Color::Cyan,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::Cyan,
            prompt_info_fg: Color::Black,
            prompt_success_bg: Color::Green,
            prompt_success_fg: Color::Black,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: Color::Black,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::Black,
            dialog_button_primary_info_bg: Color::Cyan,
            dialog_button_primary_success_fg: Color::Black,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: Color::Cyan,
            dialog_button_secondary_bg: Color::Blue,

            // Config window
            config_title_bg: Color::Cyan,
            config_title_fg: Color::Black,
            config_border: Color::Yellow,
            config_content_bg: Color::Blue,
            config_content_fg: Color::Cyan,
            config_instructions_fg: Color::DarkCyan,
            config_toggle_on_color: Color::Yellow,
            config_toggle_off_color: Color::DarkCyan,

            // Calendar
            calendar_bg: Color::Blue,
            calendar_fg: Color::Cyan,
            calendar_title_color: Color::Yellow,
            calendar_today_bg: Color::Cyan,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkCyan,
            scrollbar_thumb_fg: Color::Cyan,

            // Context menu
            menu_bg: Color::Blue,
            menu_fg: Color::Cyan,
            menu_border: Color::Yellow,
            menu_selected_bg: Color::Cyan,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::Black,

            // Snap preview
            snap_preview_border: Color::Yellow,
            snap_preview_bg: Color::Black,

            // Splash screen - theme background with primary accent
            splash_border: Color::Yellow,
            splash_bg: Color::Blue,
            splash_fg: Color::Cyan, // Primary cyan accent

            // Slight input popup
            slight_bg: Color::Blue,
            slight_fg: Color::Cyan,
            slight_border: Color::Cyan,
            slight_input_bg: Color::Black,
            slight_input_fg: Color::Cyan,
            slight_suggestion_fg: Color::Yellow,
            slight_dropdown_bg: Color::Blue,
            slight_dropdown_fg: Color::Cyan,
            slight_dropdown_selected_bg: Color::Cyan,
            slight_dropdown_selected_fg: Color::Black,
        }
    }

    /// XT theme - inspired by golden file manager interfaces
    /// Yellow on blue with white borders
    pub fn xtree() -> Self {
        Self {
            // Desktop - blue background
            desktop_bg: Color::Blue,
            desktop_fg: Color::Yellow,

            // Top bar
            topbar_bg_focused: Color::Yellow,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::Black,
            topbar_fg_unfocused: Color::Black,
            clock_bg: Color::Black,
            clock_fg: Color::Yellow,

            // Windows - Title bar (yellow/white on blue)
            window_title_unfocused_fg: Color::Black,
            window_title_unfocused_bg: Color::Black,
            window_title_focused_fg: Color::Black,
            window_title_focused_bg: Color::Yellow,
            // Windows - Border
            window_border_unfocused_fg: Color::White,
            window_border_unfocused_bg: Color::Black,
            window_border_focused_fg: Color::Yellow,
            window_border_focused_bg: Color::Yellow,
            window_content_bg: Color::Blue,
            window_content_fg: Color::Yellow,
            window_shadow_color: Color::Black,

            // Window controls
            button_close_color: Color::Red,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Cyan,
            button_bg: Color::Black,
            resize_handle_normal_fg: Color::White,
            resize_handle_normal_bg: Color::Blue,
            resize_handle_active_fg: Color::Yellow,
            resize_handle_active_bg: Color::White,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::Yellow,
            button_hovered_fg: Color::Black,
            button_hovered_bg: Color::White,
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
            toggle_enabled_bg_normal: Color::Blue,
            toggle_enabled_bg_hovered: Color::White,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::DarkYellow,
            toggle_disabled_bg_normal: Color::Blue,
            toggle_disabled_bg_hovered: Color::White,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::White,
            prompt_info_fg: Color::Black,
            prompt_success_bg: Color::Green,
            prompt_success_fg: Color::Black,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: Color::Black,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::Black,
            dialog_button_primary_info_bg: Color::Yellow,
            dialog_button_primary_success_fg: Color::Black,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: Color::Yellow,
            dialog_button_secondary_bg: Color::Blue,

            // Config window
            config_title_bg: Color::Yellow,
            config_title_fg: Color::Black,
            config_border: Color::White,
            config_content_bg: Color::Blue,
            config_content_fg: Color::Yellow,
            config_instructions_fg: Color::DarkYellow,
            config_toggle_on_color: Color::Yellow,
            config_toggle_off_color: Color::DarkYellow,

            // Calendar
            calendar_bg: Color::Blue,
            calendar_fg: Color::Yellow,
            calendar_title_color: Color::White,
            calendar_today_bg: Color::Yellow,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkYellow,
            scrollbar_thumb_fg: Color::Yellow,

            // Context menu
            menu_bg: Color::Blue,
            menu_fg: Color::Yellow,
            menu_border: Color::White,
            menu_selected_bg: Color::Yellow,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::Black,

            // Snap preview
            snap_preview_border: Color::White,
            snap_preview_bg: Color::Black,

            // Splash screen - theme background with primary accent
            splash_border: Color::White,
            splash_bg: Color::Blue,
            splash_fg: Color::Yellow, // Primary yellow accent

            // Slight input popup
            slight_bg: Color::Blue,
            slight_fg: Color::Yellow,
            slight_border: Color::White,
            slight_input_bg: Color::Black,
            slight_input_fg: Color::Yellow,
            slight_suggestion_fg: Color::White,
            slight_dropdown_bg: Color::Blue,
            slight_dropdown_fg: Color::Yellow,
            slight_dropdown_selected_bg: Color::Yellow,
            slight_dropdown_selected_fg: Color::Black,
        }
    }

    /// WP theme - inspired by classic word processor interfaces
    /// White/cyan on blue background
    pub fn wordperfect() -> Self {
        Self {
            // Desktop - deep blue background (iconic WP 5.1 blue screen)
            desktop_bg: Color::Blue,
            desktop_fg: Color::White,

            // Top bar
            topbar_bg_focused: Color::Cyan,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::Black,
            topbar_fg_unfocused: Color::Black,
            clock_bg: Color::Black,
            clock_fg: Color::Cyan,

            // Windows - Title bar (white/cyan on blue)
            window_title_unfocused_fg: Color::Black,
            window_title_unfocused_bg: Color::Black,
            window_title_focused_fg: Color::Black,
            window_title_focused_bg: Color::Cyan,
            // Windows - Border
            window_border_unfocused_fg: Color::Cyan,
            window_border_unfocused_bg: Color::Black,
            window_border_focused_fg: Color::White,
            window_border_focused_bg: Color::Cyan,
            // Windows - Content
            window_content_bg: Color::Blue,
            window_content_fg: Color::White,
            window_shadow_color: Color::Black,

            // Window controls
            button_close_color: Color::Red,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Yellow,
            button_bg: Color::Black,
            resize_handle_normal_fg: Color::Cyan,
            resize_handle_normal_bg: Color::Blue,
            resize_handle_active_fg: Color::White,
            resize_handle_active_bg: Color::Cyan,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::Cyan,
            button_hovered_fg: Color::Black,
            button_hovered_bg: Color::White,
            button_pressed_fg: Color::Cyan,
            button_pressed_bg: Color::Black,

            // Bottom bar - status line style
            bottombar_bg: Color::Black,
            bottombar_fg: Color::White,
            bottombar_button_normal_fg: Color::White,
            bottombar_button_normal_bg: Color::Black,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::Cyan,
            bottombar_button_minimized_fg: Color::DarkGrey,
            bottombar_button_minimized_bg: Color::Black,

            // Toggle button
            toggle_enabled_fg: Color::White,
            toggle_enabled_bg_normal: Color::Blue,
            toggle_enabled_bg_hovered: Color::Cyan,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::DarkCyan,
            toggle_disabled_bg_normal: Color::Blue,
            toggle_disabled_bg_hovered: Color::Cyan,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::Cyan,
            prompt_info_fg: Color::Black,
            prompt_success_bg: Color::Green,
            prompt_success_fg: Color::Black,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: Color::Black,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::Black,
            dialog_button_primary_info_bg: Color::Cyan,
            dialog_button_primary_success_fg: Color::Black,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: Color::White,
            dialog_button_secondary_bg: Color::Blue,

            // Config window
            config_title_bg: Color::Cyan,
            config_title_fg: Color::Black,
            config_border: Color::White,
            config_content_bg: Color::Blue,
            config_content_fg: Color::White,
            config_instructions_fg: Color::Cyan,
            config_toggle_on_color: Color::White,
            config_toggle_off_color: Color::DarkCyan,

            // Calendar
            calendar_bg: Color::Blue,
            calendar_fg: Color::White,
            calendar_title_color: Color::Cyan,
            calendar_today_bg: Color::Cyan,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkCyan,
            scrollbar_thumb_fg: Color::Cyan,

            // Context menu
            menu_bg: Color::Blue,
            menu_fg: Color::White,
            menu_border: Color::Cyan,
            menu_selected_bg: Color::Cyan,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::Black,

            // Snap preview
            snap_preview_border: Color::White,
            snap_preview_bg: Color::Black,

            // Splash screen - theme background with primary accent
            splash_border: Color::Cyan,
            splash_bg: Color::Blue,
            splash_fg: Color::Cyan, // Primary cyan accent

            // Slight input popup
            slight_bg: Color::Blue,
            slight_fg: Color::White,
            slight_border: Color::Cyan,
            slight_input_bg: Color::Black,
            slight_input_fg: Color::Cyan,
            slight_suggestion_fg: Color::White,
            slight_dropdown_bg: Color::Blue,
            slight_dropdown_fg: Color::White,
            slight_dropdown_selected_bg: Color::Cyan,
            slight_dropdown_selected_fg: Color::Black,
        }
    }

    /// dB theme - inspired by classic database manager interfaces
    /// Cyan on blue with white accents
    pub fn dbase() -> Self {
        Self {
            // Desktop - blue background
            desktop_bg: Color::Blue,
            desktop_fg: Color::Cyan,

            // Top bar
            topbar_bg_focused: Color::White,
            topbar_bg_unfocused: Color::Black,
            topbar_fg_focused: Color::Black,
            topbar_fg_unfocused: Color::Black,
            clock_bg: Color::Black,
            clock_fg: Color::White,

            // Windows - Title bar (cyan on blue)
            window_title_unfocused_fg: Color::Black,
            window_title_unfocused_bg: Color::Black,
            window_title_focused_fg: Color::Black,
            window_title_focused_bg: Color::White,
            // Windows - Border
            window_border_unfocused_fg: Color::Cyan,
            window_border_unfocused_bg: Color::Black,
            window_border_focused_fg: Color::White,
            window_border_focused_bg: Color::White,
            // Windows - Content
            window_content_bg: Color::Blue,
            window_content_fg: Color::Cyan,
            window_shadow_color: Color::Black,

            // Window controls
            button_close_color: Color::Red,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Yellow,
            button_bg: Color::Black,
            resize_handle_normal_fg: Color::Cyan,
            resize_handle_normal_bg: Color::Blue,
            resize_handle_active_fg: Color::White,
            resize_handle_active_bg: Color::Cyan,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::White,
            button_hovered_fg: Color::Black,
            button_hovered_bg: Color::Cyan,
            button_pressed_fg: Color::White,
            button_pressed_bg: Color::Black,

            // Bottom bar
            bottombar_bg: Color::Black,
            bottombar_fg: Color::Cyan,
            bottombar_button_normal_fg: Color::Cyan,
            bottombar_button_normal_bg: Color::Black,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::White,
            bottombar_button_minimized_fg: Color::DarkCyan,
            bottombar_button_minimized_bg: Color::Black,

            // Toggle button
            toggle_enabled_fg: Color::White,
            toggle_enabled_bg_normal: Color::Blue,
            toggle_enabled_bg_hovered: Color::Cyan,
            toggle_enabled_bg_pressed: Color::Black,
            toggle_disabled_fg: Color::DarkCyan,
            toggle_disabled_bg_normal: Color::Blue,
            toggle_disabled_bg_hovered: Color::Cyan,
            toggle_disabled_bg_pressed: Color::Black,

            // Prompts/Dialogs
            prompt_info_bg: Color::White,
            prompt_info_fg: Color::Black,
            prompt_success_bg: Color::Green,
            prompt_success_fg: Color::Black,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: Color::Black,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::Black,
            dialog_button_primary_info_bg: Color::White,
            dialog_button_primary_success_fg: Color::Black,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: Color::Cyan,
            dialog_button_secondary_bg: Color::Blue,

            // Config window
            config_title_bg: Color::White,
            config_title_fg: Color::Black,
            config_border: Color::Cyan,
            config_content_bg: Color::Blue,
            config_content_fg: Color::Cyan,
            config_instructions_fg: Color::DarkCyan,
            config_toggle_on_color: Color::White,
            config_toggle_off_color: Color::DarkCyan,

            // Calendar
            calendar_bg: Color::Blue,
            calendar_fg: Color::Cyan,
            calendar_title_color: Color::White,
            calendar_today_bg: Color::White,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkCyan,
            scrollbar_thumb_fg: Color::Cyan,

            // Context menu
            menu_bg: Color::Blue,
            menu_fg: Color::Cyan,
            menu_border: Color::White,
            menu_selected_bg: Color::White,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::Black,

            // Snap preview
            snap_preview_border: Color::White,
            snap_preview_bg: Color::Black,

            // Splash screen - theme background with primary accent
            splash_border: Color::White,
            splash_bg: Color::Blue,
            splash_fg: Color::Cyan, // Primary cyan accent

            // Slight input popup
            slight_bg: Color::Blue,
            slight_fg: Color::Cyan,
            slight_border: Color::White,
            slight_input_bg: Color::Black,
            slight_input_fg: Color::Cyan,
            slight_suggestion_fg: Color::White,
            slight_dropdown_bg: Color::Blue,
            slight_dropdown_fg: Color::Cyan,
            slight_dropdown_selected_bg: Color::White,
            slight_dropdown_selected_fg: Color::Black,
        }
    }

    /// Create a theme from a name string, falling back to Classic if invalid
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "classic" => Self::classic(),
            "monochrome" => Self::monochrome(),
            "dark" => Self::dark(),
            "dracu" | "darcula" | "intellij" => Self::dracu(),
            "green" | "green_phosphor" | "greenphosphor" => Self::green_phosphor(),
            "amber" | "orange" => Self::amber(),
            "ndd" => Self::ndd(),
            "qbasic" | "basic" | "edit" => Self::qbasic(),
            "turbo" | "pascal" => Self::turbo(),
            "norton_commander" | "nortoncommander" | "nc" => Self::norton_commander(),
            "xtree" | "xtree_gold" | "xtreegold" => Self::xtree(),
            "wordperfect" | "wp" | "wp51" => Self::wordperfect(),
            "dbase" | "dbase4" | "dbaseiv" => Self::dbase(),
            _ => {
                eprintln!("Unknown theme '{}', falling back to 'classic'", name);
                Self::classic()
            }
        }
    }
}
