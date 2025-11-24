use crossterm::style::Color;

// Custom color constants for better readability and maintainability
// Pure colors
const PURE_BLACK: Color = Color::Rgb { r: 0, g: 0, b: 0 };

// NDD (Norton Desktop) theme colors
const NDD_LIGHT_PURPLE: Color = Color::Rgb {
    r: 87,
    g: 87,
    b: 246,
};
const NDD_LIGHT_GRAY: Color = Color::Rgb {
    r: 168,
    g: 168,
    b: 168,
};

const NDD_DARK_GRAY: Color = Color::Rgb {
    r: 108,
    g: 108,
    b: 108,
};

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

// dBase IV theme colors - authentic Borland dBASE IV 2.0 colors
const DB_BLUE: Color = Color::Rgb { r: 0, g: 0, b: 170 }; // #0000AA - main/description background
const DB_GREY: Color = Color::Rgb {
    r: 192,
    g: 192,
    b: 192,
}; // #C0C0C0 - dialog background, bars
const DB_LIGHT_GREY: Color = Color::Rgb {
    r: 170,
    g: 170,
    b: 170,
}; // #AAAAAA - shadow border, UI shading
const DB_YELLOW: Color = Color::Rgb {
    r: 255,
    g: 255,
    b: 85,
}; // #FFFF55 - F2-DO-IT, Description label
const DB_BRIGHT_RED: Color = Color::Rgb {
    r: 255,
    g: 85,
    b: 85,
}; // #FF5555 - ENTER-Change, ESC-Previous

// WordPerfect theme colors - authentic WP 5.1 VGA colors
const WP_BLUE: Color = Color::Rgb { r: 0, g: 0, b: 170 }; // #0000AA - main background
const WP_LIGHT_GREY: Color = Color::Rgb {
    r: 192,
    g: 192,
    b: 192,
}; // #C0C0C0 - menu bar, pop-ups
const WP_RED: Color = Color::Rgb { r: 170, g: 0, b: 0 }; // #AA0000 - menu highlight
const WP_CYAN: Color = Color::Rgb {
    r: 0,
    g: 170,
    b: 170,
}; // #00AAAA - selection boxes
const WP_BRIGHT_CYAN: Color = Color::Rgb {
    r: 85,
    g: 255,
    b: 255,
}; // #55FFFF - inline color coding
const WP_BRIGHT_BLUE: Color = Color::Rgb {
    r: 85,
    g: 85,
    b: 255,
}; // #5555FF - borders, underlines

// XTree Gold theme colors - authentic XTree Gold colors
const XT_DARK_BLUE: Color = Color::Rgb {
    r: 28,
    g: 28,
    b: 28,
}; // #1C1C1C - main background
const XT_CYAN: Color = Color::Rgb {
    r: 190,
    g: 190,
    b: 50,
}; // #BEBE32 - file tree text
const XT_YELLOW: Color = Color::Rgb {
    r: 255,
    g: 255,
    b: 0,
}; // #FFFF00 - highlight bar, borders, tree lines
const XT_ORANGE: Color = Color::Rgb {
    r: 255,
    g: 123,
    b: 0,
}; // #FF7B00 - orange accent
const XT_LIGHT_PURPLE: Color = Color::Rgb {
    r: 90,
    g: 255,
    b: 255,
}; // #5AFFFF - cyan accent

// Norton Commander theme colors - authentic NC 5.0 colors
const NC_BLUE: Color = Color::Rgb { r: 0, g: 0, b: 175 }; // #0000AF - main panel background
const NC_CYAN: Color = Color::Rgb {
    r: 80,
    g: 255,
    b: 255,
}; // #50FFFF - file list text
const NC_TEAL: Color = Color::Rgb {
    r: 0,
    g: 168,
    b: 175,
}; // #00A8AF - headers, menus
const NC_GREY: Color = Color::Rgb {
    r: 175,
    g: 168,
    b: 175,
}; // #AFA8AF - menu bar
const NC_YELLOW: Color = Color::Rgb {
    r: 255,
    g: 255,
    b: 80,
}; // #FFFF50 - selection highlight
const NC_ORANGE_RED: Color = Color::Rgb {
    r: 255,
    g: 87,
    b: 80,
}; // #FF5750 - cursor/selection bar

// Turbo Pascal theme colors - authentic Borland Turbo Pascal 7.0 colors
const TURBO_DARK_BLUE: Color = Color::Rgb { r: 0, g: 0, b: 123 }; // #00007B - desktop/editor background
const TURBO_BLUE_PURPLE: Color = Color::Rgb {
    r: 62,
    g: 59,
    b: 149,
}; // #3E3B95 - title bar
const TURBO_TEAL: Color = Color::Rgb {
    r: 0,
    g: 132,
    b: 132,
}; // #008484 - help window body
const TURBO_DARK_TEAL: Color = Color::Rgb {
    r: 0,
    g: 123,
    b: 123,
}; // #007B7B - help window title
const TURBO_LIGHT_GREY: Color = Color::Rgb {
    r: 181,
    g: 177,
    b: 189,
}; // #B5B1BD - menu bar
const TURBO_BEIGE: Color = Color::Rgb {
    r: 231,
    g: 231,
    b: 206,
}; // #E7E7CE - status bar

// QBasic theme colors - authentic MS-DOS QBasic colors
const QBASIC_ROYAL_BLUE: Color = Color::Rgb { r: 0, g: 0, b: 170 }; // #0000AA - main background
const QBASIC_LIGHT_GREY: Color = Color::Rgb {
    r: 192,
    g: 192,
    b: 192,
}; // #C0C0C0 - dialog background
const QBASIC_DARK_GREY: Color = Color::Rgb {
    r: 128,
    g: 128,
    b: 128,
}; // #808080 - dialog border
const QBASIC_PALE_GREY: Color = Color::Rgb {
    r: 224,
    g: 224,
    b: 224,
}; // #E0E0E0 - menu bar
const QBASIC_CYAN: Color = Color::Rgb {
    r: 0,
    g: 255,
    b: 255,
}; // #00FFFF - status line

// Dracula theme colors - official palette from draculatheme.com
const DRACULA_BACKGROUND: Color = Color::Rgb {
    r: 40,
    g: 42,
    b: 54,
};
// Selection color #44475A - used for highlighting
const DRACULA_SELECTION: Color = Color::Rgb {
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

// IntelliJ Darcula Darker theme colors - darker variant of JetBrains IDE theme
// Based on Darcula Darker plugin: darker backgrounds with vibrant accents
const DARCULA_BACKGROUND: Color = Color::Rgb {
    r: 30,
    g: 30,
    b: 30,
}; // #1E1E1E - darker than standard #2B2B2B
const DARCULA_UI_BACKGROUND: Color = Color::Rgb {
    r: 43,
    g: 43,
    b: 43,
}; // #2B2B2B - used for panels/UI elements
#[allow(dead_code)]
const DARCULA_SELECTION: Color = Color::Rgb {
    r: 33,
    g: 66,
    b: 131,
}; // #214283 - selection highlight
const DARCULA_FOREGROUND: Color = Color::Rgb {
    r: 169,
    g: 183,
    b: 198,
}; // #A9B7C6
const DARCULA_ORANGE: Color = Color::Rgb {
    r: 204,
    g: 120,
    b: 50,
}; // #CC7832 - keywords
const DARCULA_STRING_GREEN: Color = Color::Rgb {
    r: 165,
    g: 194,
    b: 92,
}; // #A5C25C - strings
const DARCULA_NUMBER_BLUE: Color = Color::Rgb {
    r: 104,
    g: 151,
    b: 187,
}; // #6897BB - numbers
#[allow(dead_code)]
const DARCULA_PURPLE: Color = Color::Rgb {
    r: 152,
    g: 118,
    b: 170,
}; // #9876AA - attributes/types
const DARCULA_COMMENT: Color = Color::Rgb {
    r: 128,
    g: 128,
    b: 128,
}; // #808080
const DARCULA_FUNCTION_YELLOW: Color = Color::Rgb {
    r: 255,
    g: 198,
    b: 109,
}; // #FFC66D
#[allow(dead_code)]
const DARCULA_DOC_GREEN: Color = Color::Rgb {
    r: 98,
    g: 151,
    b: 85,
}; // #629755
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
    /// Background: #282A36, Foreground: #F8F8F2, Selection: #44475A
    /// Accent colors: Cyan #8BE9FD, Purple #BD93F9, Pink #FF79C6, Green #50FA7B, Red #FF5555, Yellow #F1FA8C
    pub fn dark() -> Self {
        Self {
            // Desktop - Dracula background with purple accent
            desktop_bg: DRACULA_BACKGROUND,
            desktop_fg: DRACULA_PURPLE,

            // Top bar - Purple accent when focused (signature Dracula color)
            topbar_bg_focused: DRACULA_PURPLE,
            topbar_bg_unfocused: DRACULA_BACKGROUND,
            topbar_fg_focused: DRACULA_BACKGROUND,
            topbar_fg_unfocused: DRACULA_FOREGROUND,
            clock_bg: DRACULA_SELECTION,
            clock_fg: DRACULA_CYAN,

            // Windows - Title bar with vibrant Dracula accents
            window_title_unfocused_fg: DRACULA_COMMENT,
            window_title_unfocused_bg: DRACULA_BACKGROUND,
            window_title_focused_fg: DRACULA_BACKGROUND,
            window_title_focused_bg: DRACULA_PURPLE, // Signature purple for focus
            // Windows - Border
            window_border_unfocused_fg: DRACULA_COMMENT,
            window_border_unfocused_bg: DRACULA_BACKGROUND,
            window_border_focused_fg: DRACULA_BACKGROUND,
            window_border_focused_bg: DRACULA_PURPLE,
            // Windows - Content
            window_content_bg: DRACULA_BACKGROUND,
            window_content_fg: DRACULA_FOREGROUND,
            window_shadow_color: PURE_BLACK,

            // Window controls - Dracula semantic colors
            button_close_color: DRACULA_RED,
            button_maximize_color: DRACULA_GREEN,
            button_minimize_color: DRACULA_YELLOW,
            button_bg: DRACULA_BACKGROUND,
            resize_handle_normal_fg: DRACULA_COMMENT,
            resize_handle_normal_bg: DRACULA_BACKGROUND,
            resize_handle_active_fg: DRACULA_CYAN,
            resize_handle_active_bg: DRACULA_SELECTION,

            // UI Buttons
            button_normal_fg: DRACULA_FOREGROUND,
            button_normal_bg: DRACULA_SELECTION,
            button_hovered_fg: DRACULA_BACKGROUND,
            button_hovered_bg: DRACULA_CYAN,
            button_pressed_fg: DRACULA_FOREGROUND,
            button_pressed_bg: DRACULA_BACKGROUND,

            // Bottom bar
            bottombar_bg: DRACULA_BACKGROUND,
            bottombar_fg: DRACULA_FOREGROUND,
            bottombar_button_normal_fg: DRACULA_FOREGROUND,
            bottombar_button_normal_bg: DRACULA_BACKGROUND,
            bottombar_button_focused_fg: DRACULA_BACKGROUND,
            bottombar_button_focused_bg: DRACULA_PURPLE,
            bottombar_button_minimized_fg: DRACULA_COMMENT,
            bottombar_button_minimized_bg: DRACULA_BACKGROUND,

            // Toggle button
            toggle_enabled_fg: DRACULA_GREEN,
            toggle_enabled_bg_normal: DRACULA_SELECTION,
            toggle_enabled_bg_hovered: DRACULA_SELECTION,
            toggle_enabled_bg_pressed: DRACULA_BACKGROUND,
            toggle_disabled_fg: DRACULA_COMMENT,
            toggle_disabled_bg_normal: DRACULA_BACKGROUND,
            toggle_disabled_bg_hovered: DRACULA_SELECTION,
            toggle_disabled_bg_pressed: DRACULA_BACKGROUND,

            // Prompts/Dialogs
            prompt_info_bg: DRACULA_SELECTION,
            prompt_info_fg: DRACULA_CYAN,
            prompt_success_bg: DRACULA_GREEN,
            prompt_success_fg: DRACULA_BACKGROUND,
            prompt_warning_bg: DRACULA_ORANGE,
            prompt_warning_fg: DRACULA_BACKGROUND,
            prompt_danger_bg: DRACULA_RED,
            prompt_danger_fg: DRACULA_FOREGROUND,

            // Dialog buttons
            dialog_button_primary_info_fg: DRACULA_BACKGROUND,
            dialog_button_primary_info_bg: DRACULA_CYAN,
            dialog_button_primary_success_fg: DRACULA_BACKGROUND,
            dialog_button_primary_success_bg: DRACULA_GREEN,
            dialog_button_primary_warning_fg: DRACULA_BACKGROUND,
            dialog_button_primary_warning_bg: DRACULA_YELLOW,
            dialog_button_primary_danger_fg: DRACULA_FOREGROUND,
            dialog_button_primary_danger_bg: DRACULA_RED,
            dialog_button_secondary_fg: DRACULA_FOREGROUND,
            dialog_button_secondary_bg: DRACULA_SELECTION,

            // Config window
            config_title_bg: DRACULA_PURPLE,
            config_title_fg: DRACULA_BACKGROUND,
            config_border: DRACULA_PURPLE,
            config_content_bg: DRACULA_BACKGROUND,
            config_content_fg: DRACULA_FOREGROUND,
            config_instructions_fg: DRACULA_COMMENT,
            config_toggle_on_color: DRACULA_GREEN,
            config_toggle_off_color: DRACULA_COMMENT,

            // Calendar
            calendar_bg: DRACULA_BACKGROUND,
            calendar_fg: DRACULA_FOREGROUND,
            calendar_title_color: DRACULA_PINK,
            calendar_today_bg: DRACULA_PURPLE,
            calendar_today_fg: DRACULA_FOREGROUND,

            // Scrollbar
            scrollbar_track_fg: DRACULA_COMMENT,
            scrollbar_thumb_fg: DRACULA_PURPLE,

            // Context menu
            menu_bg: DRACULA_BACKGROUND,
            menu_fg: DRACULA_FOREGROUND,
            menu_border: DRACULA_PURPLE,
            menu_selected_bg: DRACULA_PURPLE,
            menu_selected_fg: DRACULA_FOREGROUND,
            menu_shadow_fg: PURE_BLACK,

            // Snap preview
            snap_preview_border: DRACULA_CYAN,
            snap_preview_bg: DRACULA_BACKGROUND,

            // Splash screen
            splash_border: DRACULA_PURPLE,
            splash_bg: DRACULA_BACKGROUND,
            splash_fg: DRACULA_PURPLE,

            // Slight input popup
            slight_bg: DRACULA_BACKGROUND,
            slight_fg: DRACULA_FOREGROUND,
            slight_border: DRACULA_PURPLE,
            slight_input_bg: DRACULA_SELECTION,
            slight_input_fg: DRACULA_CYAN,
            slight_suggestion_fg: DRACULA_YELLOW,
            slight_dropdown_bg: DRACULA_BACKGROUND,
            slight_dropdown_fg: DRACULA_FOREGROUND,
            slight_dropdown_selected_bg: DRACULA_PURPLE,
            slight_dropdown_selected_fg: DRACULA_FOREGROUND,
        }
    }

    /// Dracu theme inspired by IntelliJ IDEA Darcula Darker (JetBrains IDE dark theme)
    /// Background: #1E1E1E, UI: #2B2B2B, Foreground: #A9B7C6
    /// Main: Gray shades, Accent: Orange #CC7832
    pub fn dracu() -> Self {
        Self {
            // Desktop - Dark gray background
            desktop_bg: DARCULA_BACKGROUND,
            desktop_fg: DARCULA_COMMENT,

            // Top bar - Gray shades, orange only as accent
            topbar_bg_focused: DARCULA_UI_BACKGROUND,
            topbar_bg_unfocused: DARCULA_BACKGROUND,
            topbar_fg_focused: DARCULA_FOREGROUND,
            topbar_fg_unfocused: DARCULA_COMMENT,
            clock_bg: DARCULA_BACKGROUND,
            clock_fg: DARCULA_ORANGE, // Orange accent for clock

            // Windows - Title bar with gray shades
            window_title_unfocused_fg: DARCULA_COMMENT,
            window_title_unfocused_bg: DARCULA_BACKGROUND,
            window_title_focused_fg: DARCULA_FOREGROUND,
            window_title_focused_bg: DARCULA_UI_BACKGROUND,
            // Windows - Border - gray with orange accent on focus
            window_border_unfocused_fg: DARCULA_COMMENT,
            window_border_unfocused_bg: DARCULA_BACKGROUND,
            window_border_focused_fg: DARCULA_ORANGE, // Orange accent border
            window_border_focused_bg: DARCULA_UI_BACKGROUND,
            // Windows - Content
            window_content_bg: DARCULA_BACKGROUND,
            window_content_fg: DARCULA_FOREGROUND,
            window_shadow_color: PURE_BLACK,

            // Window controls - Darcula semantic colors
            button_close_color: Color::Rgb {
                r: 255,
                g: 75,
                b: 75,
            },
            button_maximize_color: DARCULA_STRING_GREEN,
            button_minimize_color: DARCULA_FUNCTION_YELLOW,
            button_bg: DARCULA_BACKGROUND,
            resize_handle_normal_fg: DARCULA_COMMENT,
            resize_handle_normal_bg: DARCULA_BACKGROUND,
            resize_handle_active_fg: DARCULA_ORANGE, // Orange accent
            resize_handle_active_bg: DARCULA_UI_BACKGROUND,

            // UI Buttons - gray base, orange hover accent
            button_normal_fg: DARCULA_FOREGROUND,
            button_normal_bg: DARCULA_UI_BACKGROUND,
            button_hovered_fg: DARCULA_BACKGROUND,
            button_hovered_bg: DARCULA_ORANGE, // Orange accent on hover
            button_pressed_fg: DARCULA_FOREGROUND,
            button_pressed_bg: DARCULA_BACKGROUND,

            // Bottom bar - gray shades
            bottombar_bg: DARCULA_BACKGROUND,
            bottombar_fg: DARCULA_FOREGROUND,
            bottombar_button_normal_fg: DARCULA_FOREGROUND,
            bottombar_button_normal_bg: DARCULA_BACKGROUND,
            bottombar_button_focused_fg: DARCULA_BACKGROUND,
            bottombar_button_focused_bg: DARCULA_ORANGE, // Orange accent for focused
            bottombar_button_minimized_fg: DARCULA_COMMENT,
            bottombar_button_minimized_bg: DARCULA_BACKGROUND,

            // Toggle button
            toggle_enabled_fg: DARCULA_ORANGE, // Orange for enabled state
            toggle_enabled_bg_normal: DARCULA_UI_BACKGROUND,
            toggle_enabled_bg_hovered: DARCULA_UI_BACKGROUND,
            toggle_enabled_bg_pressed: DARCULA_BACKGROUND,
            toggle_disabled_fg: DARCULA_COMMENT,
            toggle_disabled_bg_normal: DARCULA_BACKGROUND,
            toggle_disabled_bg_hovered: DARCULA_UI_BACKGROUND,
            toggle_disabled_bg_pressed: DARCULA_BACKGROUND,

            // Prompts/Dialogs
            prompt_info_bg: DARCULA_UI_BACKGROUND,
            prompt_info_fg: DARCULA_FOREGROUND,
            prompt_success_bg: DARCULA_STRING_GREEN,
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
            dialog_button_primary_info_fg: DARCULA_FOREGROUND,
            dialog_button_primary_info_bg: DARCULA_UI_BACKGROUND,
            dialog_button_primary_success_fg: DARCULA_BACKGROUND,
            dialog_button_primary_success_bg: DARCULA_STRING_GREEN,
            dialog_button_primary_warning_fg: DARCULA_BACKGROUND,
            dialog_button_primary_warning_bg: DARCULA_ORANGE,
            dialog_button_primary_danger_fg: DARCULA_FOREGROUND,
            dialog_button_primary_danger_bg: Color::Rgb {
                r: 255,
                g: 75,
                b: 75,
            },
            dialog_button_secondary_fg: DARCULA_FOREGROUND,
            dialog_button_secondary_bg: DARCULA_UI_BACKGROUND,

            // Config window - gray with orange border accent
            config_title_bg: DARCULA_UI_BACKGROUND,
            config_title_fg: DARCULA_FOREGROUND,
            config_border: DARCULA_ORANGE, // Orange accent border
            config_content_bg: DARCULA_BACKGROUND,
            config_content_fg: DARCULA_FOREGROUND,
            config_instructions_fg: DARCULA_COMMENT,
            config_toggle_on_color: DARCULA_ORANGE, // Orange for on state
            config_toggle_off_color: DARCULA_COMMENT,

            // Calendar
            calendar_bg: DARCULA_BACKGROUND,
            calendar_fg: DARCULA_FOREGROUND,
            calendar_title_color: DARCULA_FOREGROUND,
            calendar_today_bg: DARCULA_ORANGE, // Orange accent for today
            calendar_today_fg: DARCULA_BACKGROUND,

            // Scrollbar
            scrollbar_track_fg: DARCULA_COMMENT,
            scrollbar_thumb_fg: DARCULA_ORANGE, // Orange accent

            // Context menu - gray with orange selection
            menu_bg: DARCULA_BACKGROUND,
            menu_fg: DARCULA_FOREGROUND,
            menu_border: DARCULA_UI_BACKGROUND,
            menu_selected_bg: DARCULA_ORANGE, // Orange accent for selection
            menu_selected_fg: DARCULA_BACKGROUND,
            menu_shadow_fg: PURE_BLACK,

            // Snap preview
            snap_preview_border: DARCULA_ORANGE, // Orange accent
            snap_preview_bg: DARCULA_BACKGROUND,

            // Splash screen - gray with orange accent
            splash_border: DARCULA_UI_BACKGROUND,
            splash_bg: DARCULA_BACKGROUND,
            splash_fg: DARCULA_ORANGE, // Orange accent text

            // Slight input popup - gray base
            slight_bg: DARCULA_BACKGROUND,
            slight_fg: DARCULA_FOREGROUND,
            slight_border: DARCULA_UI_BACKGROUND,
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
            button_maximize_color: MID_AMBER,  // Mid amber for maximize
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
            // Desktop - Norton Desktop blue background
            desktop_bg: NDD_DARK_GRAY,
            desktop_fg: Color::Black,

            // Top bar - White/grey menu bar like Norton Desktop
            topbar_bg_focused: Color::White,
            topbar_bg_unfocused: NDD_LIGHT_PURPLE,
            topbar_fg_focused: Color::Black,
            topbar_fg_unfocused: Color::White,
            clock_bg: Color::Grey,
            clock_fg: Color::Black,

            // Windows - Title bar (Light purple like Norton Desktop)
            window_title_unfocused_fg: Color::Black,
            window_title_unfocused_bg: NDD_LIGHT_GRAY,
            window_title_focused_fg: Color::Black,
            window_title_focused_bg: Color::White,
            // Windows - Border
            window_border_unfocused_fg: Color::Black,
            window_border_unfocused_bg: NDD_LIGHT_GRAY,
            window_border_focused_fg: Color::Black,
            window_border_focused_bg: Color::White,
            // Windows - Content (Light purple RGB 87,87,246)
            window_content_bg: NDD_LIGHT_PURPLE,
            window_content_fg: Color::White,
            window_shadow_color: Color::Black,

            // Window controls
            button_close_color: Color::White,
            button_maximize_color: Color::White,
            button_minimize_color: Color::White,
            button_bg: Color::Black,
            resize_handle_normal_fg: Color::White,
            resize_handle_normal_bg: NDD_LIGHT_PURPLE,
            resize_handle_active_fg: Color::Black,
            resize_handle_active_bg: Color::White,

            // UI Buttons
            button_normal_fg: Color::Black,
            button_normal_bg: Color::White,
            button_hovered_fg: Color::Black,
            button_hovered_bg: NDD_LIGHT_GRAY,
            button_pressed_fg: Color::White,
            button_pressed_bg: NDD_LIGHT_PURPLE,

            // Bottom bar - Light purple like Norton Desktop
            bottombar_bg: NDD_LIGHT_PURPLE,
            bottombar_fg: Color::White,
            bottombar_button_normal_fg: Color::Black,
            bottombar_button_normal_bg: Color::White,
            bottombar_button_focused_fg: Color::Black,
            bottombar_button_focused_bg: Color::White,
            bottombar_button_minimized_fg: Color::Black,
            bottombar_button_minimized_bg: NDD_LIGHT_GRAY,

            // Toggle button
            toggle_enabled_fg: Color::Black,
            toggle_enabled_bg_normal: Color::White,
            toggle_enabled_bg_hovered: NDD_LIGHT_GRAY,
            toggle_enabled_bg_pressed: NDD_LIGHT_PURPLE,
            toggle_disabled_fg: Color::Black,
            toggle_disabled_bg_normal: Color::White,
            toggle_disabled_bg_hovered: NDD_LIGHT_GRAY,
            toggle_disabled_bg_pressed: NDD_LIGHT_PURPLE,

            // Prompts/Dialogs
            prompt_info_bg: NDD_LIGHT_PURPLE,
            prompt_info_fg: Color::Cyan,
            prompt_success_bg: Color::Green,
            prompt_success_fg: Color::Black,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: Color::Black,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::Black,
            dialog_button_primary_info_bg: NDD_LIGHT_PURPLE,
            dialog_button_primary_success_fg: Color::Black,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: Color::Black,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: Color::White,
            dialog_button_secondary_bg: Color::Blue,

            // Config window
            config_title_bg: NDD_LIGHT_PURPLE,
            config_title_fg: Color::Black,
            config_border: Color::Cyan,
            config_content_bg: NDD_LIGHT_PURPLE,
            config_content_fg: Color::White,
            config_instructions_fg: Color::Cyan,
            config_toggle_on_color: Color::Green,
            config_toggle_off_color: Color::Grey,

            // Calendar
            calendar_bg: NDD_LIGHT_PURPLE,
            calendar_fg: Color::White,
            calendar_title_color: Color::Cyan,
            calendar_today_bg: Color::Cyan,
            calendar_today_fg: Color::Black,

            // Scrollbar
            scrollbar_track_fg: Color::DarkCyan,
            scrollbar_thumb_fg: Color::Cyan,

            // Context menu - Blue with cyan/white text
            menu_bg: NDD_LIGHT_PURPLE,
            menu_fg: Color::White,
            menu_border: Color::Cyan,
            menu_selected_bg: Color::Cyan,
            menu_selected_fg: Color::Black,
            menu_shadow_fg: Color::Black,

            // Snap preview
            snap_preview_border: Color::Cyan,
            snap_preview_bg: Color::Blue,

            // Splash screen
            splash_border: Color::White,
            splash_bg: NDD_LIGHT_PURPLE,
            splash_fg: Color::White,

            // Slight input popup
            slight_bg: NDD_LIGHT_PURPLE,
            slight_fg: Color::White,
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

    /// QBasic theme - authentic MS-DOS QBasic (QBASIC.EXE) color scheme
    /// Royal blue background, light grey dialogs, cyan status line
    pub fn qbasic() -> Self {
        Self {
            // Desktop - royal blue main area (#0000AA)
            desktop_bg: QBASIC_ROYAL_BLUE,
            desktop_fg: Color::White,

            // Top bar - pale grey menu bar with black text (#E0E0E0)
            topbar_bg_focused: QBASIC_PALE_GREY,
            topbar_bg_unfocused: QBASIC_PALE_GREY,
            topbar_fg_focused: PURE_BLACK,
            topbar_fg_unfocused: PURE_BLACK,
            clock_bg: QBASIC_DARK_GREY,
            clock_fg: PURE_BLACK,

            // Windows - Title bar (light grey background, black text)
            window_title_unfocused_fg: PURE_BLACK,
            window_title_unfocused_bg: QBASIC_DARK_GREY,
            window_title_focused_fg: PURE_BLACK,
            window_title_focused_bg: QBASIC_LIGHT_GREY,
            // Windows - Border (darker grey #808080)
            window_border_unfocused_fg: QBASIC_DARK_GREY,
            window_border_unfocused_bg: QBASIC_DARK_GREY,
            window_border_focused_fg: QBASIC_DARK_GREY,
            window_border_focused_bg: QBASIC_LIGHT_GREY,
            // Windows - Content (royal blue like main editing area)
            window_content_bg: QBASIC_ROYAL_BLUE,
            window_content_fg: Color::White,
            window_shadow_color: PURE_BLACK,

            // Window controls
            button_close_color: Color::Red,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Yellow,
            button_bg: QBASIC_LIGHT_GREY,
            resize_handle_normal_fg: Color::White,
            resize_handle_normal_bg: QBASIC_ROYAL_BLUE,
            resize_handle_active_fg: PURE_BLACK,
            resize_handle_active_bg: QBASIC_PALE_GREY,

            // UI Buttons - black text on light grey
            button_normal_fg: PURE_BLACK,
            button_normal_bg: QBASIC_LIGHT_GREY,
            button_hovered_fg: PURE_BLACK,
            button_hovered_bg: QBASIC_PALE_GREY,
            button_pressed_fg: Color::White,
            button_pressed_bg: QBASIC_DARK_GREY,

            // Bottom bar - cyan/turquoise status line (#00FFFF) with black text
            bottombar_bg: QBASIC_CYAN,
            bottombar_fg: PURE_BLACK,
            bottombar_button_normal_fg: PURE_BLACK,
            bottombar_button_normal_bg: QBASIC_CYAN,
            bottombar_button_focused_fg: PURE_BLACK,
            bottombar_button_focused_bg: QBASIC_PALE_GREY,
            bottombar_button_minimized_fg: QBASIC_DARK_GREY,
            bottombar_button_minimized_bg: QBASIC_CYAN,

            // Toggle button
            toggle_enabled_fg: Color::Green,
            toggle_enabled_bg_normal: QBASIC_LIGHT_GREY,
            toggle_enabled_bg_hovered: QBASIC_PALE_GREY,
            toggle_enabled_bg_pressed: QBASIC_DARK_GREY,
            toggle_disabled_fg: QBASIC_DARK_GREY,
            toggle_disabled_bg_normal: QBASIC_LIGHT_GREY,
            toggle_disabled_bg_hovered: QBASIC_PALE_GREY,
            toggle_disabled_bg_pressed: QBASIC_DARK_GREY,

            // Prompts/Dialogs - light grey background with black text
            prompt_info_bg: QBASIC_LIGHT_GREY,
            prompt_info_fg: PURE_BLACK,
            prompt_success_bg: Color::Green,
            prompt_success_fg: PURE_BLACK,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: PURE_BLACK,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: PURE_BLACK,
            dialog_button_primary_info_bg: QBASIC_PALE_GREY,
            dialog_button_primary_success_fg: PURE_BLACK,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: PURE_BLACK,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: PURE_BLACK,
            dialog_button_secondary_bg: QBASIC_LIGHT_GREY,

            // Config window - light grey like QBasic dialogs
            config_title_bg: QBASIC_LIGHT_GREY,
            config_title_fg: PURE_BLACK,
            config_border: QBASIC_DARK_GREY,
            config_content_bg: QBASIC_LIGHT_GREY,
            config_content_fg: PURE_BLACK,
            config_instructions_fg: QBASIC_DARK_GREY,
            config_toggle_on_color: Color::Green,
            config_toggle_off_color: QBASIC_DARK_GREY,

            // Calendar - royal blue background like main area
            calendar_bg: QBASIC_ROYAL_BLUE,
            calendar_fg: Color::White,
            calendar_title_color: Color::White,
            calendar_today_bg: QBASIC_CYAN,
            calendar_today_fg: PURE_BLACK,

            // Scrollbar
            scrollbar_track_fg: QBASIC_DARK_GREY,
            scrollbar_thumb_fg: QBASIC_LIGHT_GREY,

            // Context menu - light grey like dialogs
            menu_bg: QBASIC_LIGHT_GREY,
            menu_fg: PURE_BLACK,
            menu_border: QBASIC_DARK_GREY,
            menu_selected_bg: QBASIC_ROYAL_BLUE,
            menu_selected_fg: Color::White,
            menu_shadow_fg: PURE_BLACK,

            // Snap preview
            snap_preview_border: QBASIC_CYAN,
            snap_preview_bg: QBASIC_ROYAL_BLUE,

            // Splash screen - royal blue with white text
            splash_border: QBASIC_LIGHT_GREY,
            splash_bg: QBASIC_ROYAL_BLUE,
            splash_fg: Color::White,

            // Slight input popup - light grey dialog style
            slight_bg: QBASIC_LIGHT_GREY,
            slight_fg: PURE_BLACK,
            slight_border: QBASIC_DARK_GREY,
            slight_input_bg: QBASIC_PALE_GREY,
            slight_input_fg: PURE_BLACK,
            slight_suggestion_fg: QBASIC_DARK_GREY,
            slight_dropdown_bg: QBASIC_LIGHT_GREY,
            slight_dropdown_fg: PURE_BLACK,
            slight_dropdown_selected_bg: QBASIC_ROYAL_BLUE,
            slight_dropdown_selected_fg: Color::White,
        }
    }

    /// TurboP theme - authentic Borland Turbo Pascal 7.0 color scheme
    /// Dark navy blue desktop, blue-purple title bars, teal help windows, beige status bar
    pub fn turbo() -> Self {
        Self {
            // Desktop - dark navy blue (#00007B)
            desktop_bg: TURBO_DARK_BLUE,
            desktop_fg: Color::White,

            // Top bar - light grey menu bar (#B5B1BD) with black text
            topbar_bg_focused: TURBO_LIGHT_GREY,
            topbar_bg_unfocused: TURBO_LIGHT_GREY,
            topbar_fg_focused: PURE_BLACK,
            topbar_fg_unfocused: PURE_BLACK,
            clock_bg: TURBO_DARK_TEAL,
            clock_fg: Color::White,

            // Windows - Title bar (blue-purple #3E3B95 with white text)
            window_title_unfocused_fg: Color::White,
            window_title_unfocused_bg: TURBO_DARK_TEAL,
            window_title_focused_fg: Color::White,
            window_title_focused_bg: TURBO_BLUE_PURPLE,
            // Windows - Border
            window_border_unfocused_fg: TURBO_DARK_TEAL,
            window_border_unfocused_bg: TURBO_DARK_TEAL,
            window_border_focused_fg: TURBO_TEAL,
            window_border_focused_bg: TURBO_TEAL,
            // Windows - Content (dark blue editor area)
            window_content_bg: TURBO_DARK_BLUE,
            window_content_fg: Color::White,
            window_shadow_color: PURE_BLACK,

            // Window controls
            button_close_color: Color::Red,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Cyan,
            button_bg: TURBO_BLUE_PURPLE,
            resize_handle_normal_fg: Color::White,
            resize_handle_normal_bg: TURBO_DARK_BLUE,
            resize_handle_active_fg: Color::Yellow,
            resize_handle_active_bg: TURBO_TEAL,

            // UI Buttons - light grey style
            button_normal_fg: PURE_BLACK,
            button_normal_bg: TURBO_LIGHT_GREY,
            button_hovered_fg: PURE_BLACK,
            button_hovered_bg: TURBO_TEAL,
            button_pressed_fg: Color::White,
            button_pressed_bg: TURBO_DARK_BLUE,

            // Bottom bar - beige status bar (#E7E7CE) with black text
            bottombar_bg: TURBO_BEIGE,
            bottombar_fg: PURE_BLACK,
            bottombar_button_normal_fg: PURE_BLACK,
            bottombar_button_normal_bg: TURBO_BEIGE,
            bottombar_button_focused_fg: Color::White,
            bottombar_button_focused_bg: TURBO_BLUE_PURPLE,
            bottombar_button_minimized_fg: TURBO_DARK_TEAL,
            bottombar_button_minimized_bg: TURBO_BEIGE,

            // Toggle button
            toggle_enabled_fg: Color::Green,
            toggle_enabled_bg_normal: TURBO_LIGHT_GREY,
            toggle_enabled_bg_hovered: TURBO_TEAL,
            toggle_enabled_bg_pressed: TURBO_DARK_BLUE,
            toggle_disabled_fg: TURBO_DARK_TEAL,
            toggle_disabled_bg_normal: TURBO_LIGHT_GREY,
            toggle_disabled_bg_hovered: TURBO_TEAL,
            toggle_disabled_bg_pressed: TURBO_DARK_BLUE,

            // Prompts/Dialogs - teal help window style (#008484)
            prompt_info_bg: TURBO_TEAL,
            prompt_info_fg: PURE_BLACK,
            prompt_success_bg: Color::Green,
            prompt_success_fg: PURE_BLACK,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: PURE_BLACK,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::White,
            dialog_button_primary_info_bg: TURBO_BLUE_PURPLE,
            dialog_button_primary_success_fg: PURE_BLACK,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: PURE_BLACK,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: PURE_BLACK,
            dialog_button_secondary_bg: TURBO_LIGHT_GREY,

            // Config window - teal like help windows
            config_title_bg: TURBO_DARK_TEAL,
            config_title_fg: Color::White,
            config_border: TURBO_TEAL,
            config_content_bg: TURBO_TEAL,
            config_content_fg: PURE_BLACK,
            config_instructions_fg: TURBO_DARK_TEAL,
            config_toggle_on_color: Color::Green,
            config_toggle_off_color: TURBO_DARK_TEAL,

            // Calendar - dark blue like editor
            calendar_bg: TURBO_DARK_BLUE,
            calendar_fg: Color::White,
            calendar_title_color: Color::Yellow,
            calendar_today_bg: TURBO_TEAL,
            calendar_today_fg: PURE_BLACK,

            // Scrollbar
            scrollbar_track_fg: TURBO_DARK_TEAL,
            scrollbar_thumb_fg: TURBO_TEAL,

            // Context menu - light grey like menu bar
            menu_bg: TURBO_LIGHT_GREY,
            menu_fg: PURE_BLACK,
            menu_border: TURBO_DARK_TEAL,
            menu_selected_bg: TURBO_BLUE_PURPLE,
            menu_selected_fg: Color::White,
            menu_shadow_fg: PURE_BLACK,

            // Snap preview
            snap_preview_border: TURBO_TEAL,
            snap_preview_bg: TURBO_DARK_BLUE,

            // Splash screen - dark blue with white text
            splash_border: TURBO_TEAL,
            splash_bg: TURBO_DARK_BLUE,
            splash_fg: Color::White,

            // Slight input popup - teal dialog style
            slight_bg: TURBO_TEAL,
            slight_fg: PURE_BLACK,
            slight_border: TURBO_DARK_TEAL,
            slight_input_bg: TURBO_LIGHT_GREY,
            slight_input_fg: PURE_BLACK,
            slight_suggestion_fg: TURBO_DARK_TEAL,
            slight_dropdown_bg: TURBO_TEAL,
            slight_dropdown_fg: PURE_BLACK,
            slight_dropdown_selected_bg: TURBO_BLUE_PURPLE,
            slight_dropdown_selected_fg: Color::White,
        }
    }

    /// NCC theme - authentic Norton Commander 5.0 color scheme
    /// Blue panels with cyan text, teal headers, grey menu bar, yellow/orange-red selection
    pub fn norton_commander() -> Self {
        Self {
            // Desktop - blue panel background (#0000AF) with cyan text (#50FFFF)
            desktop_bg: NC_BLUE,
            desktop_fg: NC_CYAN,

            // Top bar - grey menu bar (#AFA8AF) with black text
            topbar_bg_focused: NC_GREY,
            topbar_bg_unfocused: NC_GREY,
            topbar_fg_focused: PURE_BLACK,
            topbar_fg_unfocused: PURE_BLACK,
            clock_bg: NC_TEAL,
            clock_fg: Color::White,

            // Windows - Title bar (teal headers #00A8AF with black text)
            window_title_unfocused_fg: Color::White,
            window_title_unfocused_bg: NC_BLUE,
            window_title_focused_fg: PURE_BLACK,
            window_title_focused_bg: NC_TEAL,
            // Windows - Border (white borders)
            window_border_unfocused_fg: NC_CYAN,
            window_border_unfocused_bg: NC_BLUE,
            window_border_focused_fg: Color::White,
            window_border_focused_bg: NC_BLUE,
            // Windows - Content (blue panels with cyan text)
            window_content_bg: NC_BLUE,
            window_content_fg: NC_CYAN,
            window_shadow_color: PURE_BLACK,

            // Window controls
            button_close_color: NC_ORANGE_RED,
            button_maximize_color: Color::Green,
            button_minimize_color: NC_YELLOW,
            button_bg: NC_TEAL,
            resize_handle_normal_fg: NC_CYAN,
            resize_handle_normal_bg: NC_BLUE,
            resize_handle_active_fg: NC_YELLOW,
            resize_handle_active_bg: NC_TEAL,

            // UI Buttons - teal background with white text
            button_normal_fg: Color::White,
            button_normal_bg: NC_TEAL,
            button_hovered_fg: PURE_BLACK,
            button_hovered_bg: NC_YELLOW,
            button_pressed_fg: Color::White,
            button_pressed_bg: NC_BLUE,

            // Bottom bar - black status bar with yellow/cyan F-key text
            bottombar_bg: PURE_BLACK,
            bottombar_fg: NC_CYAN,
            bottombar_button_normal_fg: NC_YELLOW,
            bottombar_button_normal_bg: PURE_BLACK,
            bottombar_button_focused_fg: Color::White,
            bottombar_button_focused_bg: NC_ORANGE_RED,
            bottombar_button_minimized_fg: NC_TEAL,
            bottombar_button_minimized_bg: PURE_BLACK,

            // Toggle button
            toggle_enabled_fg: NC_YELLOW,
            toggle_enabled_bg_normal: NC_BLUE,
            toggle_enabled_bg_hovered: NC_TEAL,
            toggle_enabled_bg_pressed: PURE_BLACK,
            toggle_disabled_fg: NC_TEAL,
            toggle_disabled_bg_normal: NC_BLUE,
            toggle_disabled_bg_hovered: NC_TEAL,
            toggle_disabled_bg_pressed: PURE_BLACK,

            // Prompts/Dialogs - teal background (#00A8AF) with white text
            prompt_info_bg: NC_TEAL,
            prompt_info_fg: Color::White,
            prompt_success_bg: Color::Green,
            prompt_success_fg: PURE_BLACK,
            prompt_warning_bg: NC_YELLOW,
            prompt_warning_fg: PURE_BLACK,
            prompt_danger_bg: NC_ORANGE_RED,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: Color::White,
            dialog_button_primary_info_bg: NC_TEAL,
            dialog_button_primary_success_fg: PURE_BLACK,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: PURE_BLACK,
            dialog_button_primary_warning_bg: NC_YELLOW,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: NC_ORANGE_RED,
            dialog_button_secondary_fg: NC_CYAN,
            dialog_button_secondary_bg: NC_BLUE,

            // Config window - teal like NC dialogs
            config_title_bg: NC_TEAL,
            config_title_fg: PURE_BLACK,
            config_border: Color::White,
            config_content_bg: NC_BLUE,
            config_content_fg: NC_CYAN,
            config_instructions_fg: NC_TEAL,
            config_toggle_on_color: NC_YELLOW,
            config_toggle_off_color: NC_TEAL,

            // Calendar - blue panels with cyan text
            calendar_bg: NC_BLUE,
            calendar_fg: NC_CYAN,
            calendar_title_color: Color::White,
            calendar_today_bg: NC_ORANGE_RED,
            calendar_today_fg: Color::White,

            // Scrollbar
            scrollbar_track_fg: NC_TEAL,
            scrollbar_thumb_fg: NC_CYAN,

            // Context menu - teal drop-down (#00A8AF) with white text, yellow selection
            menu_bg: NC_TEAL,
            menu_fg: Color::White,
            menu_border: Color::White,
            menu_selected_bg: NC_YELLOW,
            menu_selected_fg: PURE_BLACK,
            menu_shadow_fg: PURE_BLACK,

            // Snap preview
            snap_preview_border: NC_YELLOW,
            snap_preview_bg: NC_BLUE,

            // Splash screen - blue with cyan text
            splash_border: Color::White,
            splash_bg: NC_BLUE,
            splash_fg: NC_CYAN,

            // Slight input popup - teal dialog style
            slight_bg: NC_TEAL,
            slight_fg: Color::White,
            slight_border: Color::White,
            slight_input_bg: NC_BLUE,
            slight_input_fg: NC_CYAN,
            slight_suggestion_fg: NC_YELLOW,
            slight_dropdown_bg: NC_TEAL,
            slight_dropdown_fg: Color::White,
            slight_dropdown_selected_bg: NC_YELLOW,
            slight_dropdown_selected_fg: PURE_BLACK,
        }
    }

    /// XT theme - authentic XTree Gold color scheme
    /// Dark blue background, cyan file text, yellow borders/selection, orange accents
    pub fn xtree() -> Self {
        Self {
            // Desktop - dark blue background (#00007B) with white text
            desktop_bg: XT_DARK_BLUE,
            desktop_fg: Color::White,

            // Top bar - dark blue with white path text
            topbar_bg_focused: XT_DARK_BLUE,
            topbar_bg_unfocused: XT_DARK_BLUE,
            topbar_fg_focused: Color::White,
            topbar_fg_unfocused: Color::White,
            clock_bg: XT_DARK_BLUE,
            clock_fg: XT_YELLOW,

            // Windows - Title bar (dark blue with white text)
            window_title_unfocused_fg: XT_CYAN,
            window_title_unfocused_bg: XT_DARK_BLUE,
            window_title_focused_fg: Color::White,
            window_title_focused_bg: XT_DARK_BLUE,
            // Windows - Border (yellow borders on blue)
            window_border_unfocused_fg: XT_LIGHT_PURPLE,
            window_border_unfocused_bg: XT_DARK_BLUE,
            window_border_focused_fg: XT_YELLOW,
            window_border_focused_bg: XT_DARK_BLUE,
            // Windows - Content (dark blue with cyan file text)
            window_content_bg: XT_DARK_BLUE,
            window_content_fg: XT_CYAN,
            window_shadow_color: PURE_BLACK,

            // Window controls
            button_close_color: XT_ORANGE,
            button_maximize_color: Color::Green,
            button_minimize_color: XT_CYAN,
            button_bg: XT_DARK_BLUE,
            resize_handle_normal_fg: XT_YELLOW,
            resize_handle_normal_bg: XT_DARK_BLUE,
            resize_handle_active_fg: Color::White,
            resize_handle_active_bg: XT_YELLOW,

            // UI Buttons - yellow selection style
            button_normal_fg: PURE_BLACK,
            button_normal_bg: XT_YELLOW,
            button_hovered_fg: PURE_BLACK,
            button_hovered_bg: Color::White,
            button_pressed_fg: XT_YELLOW,
            button_pressed_bg: XT_DARK_BLUE,

            // Bottom bar - dark blue with yellow/cyan F-key text
            bottombar_bg: XT_DARK_BLUE,
            bottombar_fg: XT_CYAN,
            bottombar_button_normal_fg: XT_YELLOW,
            bottombar_button_normal_bg: XT_DARK_BLUE,
            bottombar_button_focused_fg: PURE_BLACK,
            bottombar_button_focused_bg: XT_YELLOW,
            bottombar_button_minimized_fg: XT_LIGHT_PURPLE,
            bottombar_button_minimized_bg: XT_DARK_BLUE,

            // Toggle button
            toggle_enabled_fg: XT_YELLOW,
            toggle_enabled_bg_normal: XT_DARK_BLUE,
            toggle_enabled_bg_hovered: XT_LIGHT_PURPLE,
            toggle_enabled_bg_pressed: PURE_BLACK,
            toggle_disabled_fg: XT_LIGHT_PURPLE,
            toggle_disabled_bg_normal: XT_DARK_BLUE,
            toggle_disabled_bg_hovered: XT_LIGHT_PURPLE,
            toggle_disabled_bg_pressed: PURE_BLACK,

            // Prompts/Dialogs - yellow highlight bar style
            prompt_info_bg: XT_YELLOW,
            prompt_info_fg: PURE_BLACK,
            prompt_success_bg: Color::Green,
            prompt_success_fg: PURE_BLACK,
            prompt_warning_bg: XT_ORANGE,
            prompt_warning_fg: PURE_BLACK,
            prompt_danger_bg: Color::Red,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: PURE_BLACK,
            dialog_button_primary_info_bg: XT_YELLOW,
            dialog_button_primary_success_fg: PURE_BLACK,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: PURE_BLACK,
            dialog_button_primary_warning_bg: XT_ORANGE,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: Color::Red,
            dialog_button_secondary_fg: XT_CYAN,
            dialog_button_secondary_bg: XT_DARK_BLUE,

            // Config window - dark blue with yellow borders
            config_title_bg: XT_DARK_BLUE,
            config_title_fg: Color::White,
            config_border: XT_YELLOW,
            config_content_bg: XT_DARK_BLUE,
            config_content_fg: XT_CYAN,
            config_instructions_fg: XT_LIGHT_PURPLE,
            config_toggle_on_color: XT_YELLOW,
            config_toggle_off_color: XT_LIGHT_PURPLE,

            // Calendar - dark blue with cyan/yellow
            calendar_bg: XT_DARK_BLUE,
            calendar_fg: XT_CYAN,
            calendar_title_color: Color::White,
            calendar_today_bg: XT_YELLOW,
            calendar_today_fg: PURE_BLACK,

            // Scrollbar
            scrollbar_track_fg: XT_LIGHT_PURPLE,
            scrollbar_thumb_fg: XT_YELLOW,

            // Context menu - dark blue with yellow selection
            menu_bg: XT_DARK_BLUE,
            menu_fg: XT_CYAN,
            menu_border: XT_YELLOW,
            menu_selected_bg: XT_YELLOW,
            menu_selected_fg: PURE_BLACK,
            menu_shadow_fg: PURE_BLACK,

            // Snap preview
            snap_preview_border: XT_YELLOW,
            snap_preview_bg: XT_DARK_BLUE,

            // Splash screen - dark blue with cyan text
            splash_border: XT_YELLOW,
            splash_bg: XT_DARK_BLUE,
            splash_fg: XT_CYAN,

            // Slight input popup - dark blue with yellow accents
            slight_bg: XT_DARK_BLUE,
            slight_fg: XT_CYAN,
            slight_border: XT_YELLOW,
            slight_input_bg: XT_DARK_BLUE,
            slight_input_fg: Color::White,
            slight_suggestion_fg: XT_LIGHT_PURPLE,
            slight_dropdown_bg: XT_DARK_BLUE,
            slight_dropdown_fg: XT_CYAN,
            slight_dropdown_selected_bg: XT_YELLOW,
            slight_dropdown_selected_fg: PURE_BLACK,
        }
    }

    /// WP theme - authentic WordPerfect 5.1 VGA color scheme
    /// Royal blue background, light grey menus, red highlight, cyan selections
    pub fn wordperfect() -> Self {
        Self {
            // Desktop - royal blue background (#0000AA) with white text
            desktop_bg: WP_BLUE,
            desktop_fg: Color::White,

            // Top bar - light grey menu bar (#C0C0C0) with black text
            topbar_bg_focused: WP_LIGHT_GREY,
            topbar_bg_unfocused: WP_LIGHT_GREY,
            topbar_fg_focused: PURE_BLACK,
            topbar_fg_unfocused: PURE_BLACK,
            clock_bg: WP_BLUE,
            clock_fg: Color::White,

            // Windows - Title bar (light grey with black text)
            window_title_unfocused_fg: PURE_BLACK,
            window_title_unfocused_bg: WP_CYAN,
            window_title_focused_fg: PURE_BLACK,
            window_title_focused_bg: WP_LIGHT_GREY,
            // Windows - Border (blue borders #0000AA)
            window_border_unfocused_fg: WP_CYAN,
            window_border_unfocused_bg: WP_CYAN,
            window_border_focused_fg: WP_BLUE,
            window_border_focused_bg: WP_LIGHT_GREY,
            // Windows - Content (royal blue with white text)
            window_content_bg: WP_BLUE,
            window_content_fg: Color::White,
            window_shadow_color: PURE_BLACK,

            // Window controls
            button_close_color: WP_RED,
            button_maximize_color: Color::Green,
            button_minimize_color: Color::Yellow,
            button_bg: WP_LIGHT_GREY,
            resize_handle_normal_fg: Color::White,
            resize_handle_normal_bg: WP_BLUE,
            resize_handle_active_fg: Color::White,
            resize_handle_active_bg: WP_RED,

            // UI Buttons - cyan selection boxes (#00AAAA) with black text
            button_normal_fg: PURE_BLACK,
            button_normal_bg: WP_CYAN,
            button_hovered_fg: Color::White,
            button_hovered_bg: WP_RED,
            button_pressed_fg: WP_BLUE,
            button_pressed_bg: Color::White,

            // Bottom bar - dark blue status bar with white text
            bottombar_bg: WP_BLUE,
            bottombar_fg: Color::White,
            bottombar_button_normal_fg: Color::White,
            bottombar_button_normal_bg: WP_BLUE,
            bottombar_button_focused_fg: Color::White,
            bottombar_button_focused_bg: WP_RED,
            bottombar_button_minimized_fg: WP_BRIGHT_BLUE,
            bottombar_button_minimized_bg: WP_BLUE,

            // Toggle button
            toggle_enabled_fg: Color::White,
            toggle_enabled_bg_normal: WP_CYAN,
            toggle_enabled_bg_hovered: WP_RED,
            toggle_enabled_bg_pressed: WP_BLUE,
            toggle_disabled_fg: WP_BRIGHT_BLUE,
            toggle_disabled_bg_normal: WP_LIGHT_GREY,
            toggle_disabled_bg_hovered: WP_CYAN,
            toggle_disabled_bg_pressed: WP_BLUE,

            // Prompts/Dialogs - light grey pop-up panels (#C0C0C0)
            prompt_info_bg: WP_LIGHT_GREY,
            prompt_info_fg: PURE_BLACK,
            prompt_success_bg: Color::Green,
            prompt_success_fg: PURE_BLACK,
            prompt_warning_bg: Color::Yellow,
            prompt_warning_fg: PURE_BLACK,
            prompt_danger_bg: WP_RED,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: PURE_BLACK,
            dialog_button_primary_info_bg: WP_CYAN,
            dialog_button_primary_success_fg: PURE_BLACK,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: PURE_BLACK,
            dialog_button_primary_warning_bg: Color::Yellow,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: WP_RED,
            dialog_button_secondary_fg: Color::White,
            dialog_button_secondary_bg: WP_BLUE,

            // Config window - light grey like WP dialogs
            config_title_bg: WP_LIGHT_GREY,
            config_title_fg: PURE_BLACK,
            config_border: WP_BLUE,
            config_content_bg: WP_LIGHT_GREY,
            config_content_fg: PURE_BLACK,
            config_instructions_fg: WP_BLUE,
            config_toggle_on_color: Color::Green,
            config_toggle_off_color: WP_RED,

            // Calendar - royal blue with white text
            calendar_bg: WP_BLUE,
            calendar_fg: Color::White,
            calendar_title_color: WP_BRIGHT_CYAN,
            calendar_today_bg: WP_RED,
            calendar_today_fg: Color::White,

            // Scrollbar
            scrollbar_track_fg: WP_BRIGHT_BLUE,
            scrollbar_thumb_fg: WP_CYAN,

            // Context menu - light grey (#C0C0C0) with red highlight (#AA0000)
            menu_bg: WP_LIGHT_GREY,
            menu_fg: PURE_BLACK,
            menu_border: WP_BLUE,
            menu_selected_bg: WP_RED,
            menu_selected_fg: Color::White,
            menu_shadow_fg: PURE_BLACK,

            // Snap preview
            snap_preview_border: WP_BRIGHT_CYAN,
            snap_preview_bg: WP_BLUE,

            // Splash screen - royal blue with white text
            splash_border: WP_LIGHT_GREY,
            splash_bg: WP_BLUE,
            splash_fg: Color::White,

            // Slight input popup - light grey dialog style
            slight_bg: WP_LIGHT_GREY,
            slight_fg: PURE_BLACK,
            slight_border: WP_BLUE,
            slight_input_bg: Color::White,
            slight_input_fg: WP_BLUE,
            slight_suggestion_fg: WP_CYAN,
            slight_dropdown_bg: WP_LIGHT_GREY,
            slight_dropdown_fg: PURE_BLACK,
            slight_dropdown_selected_bg: WP_RED,
            slight_dropdown_selected_fg: Color::White,
        }
    }

    /// dB theme - authentic Borland dBASE IV 2.0 color scheme
    /// Blue background, grey dialogs, yellow/red F-key highlights
    pub fn dbase() -> Self {
        Self {
            // Desktop - dark patterned blue (#0000AA) with white text
            desktop_bg: Color::Black,
            desktop_fg: DB_BLUE,

            // Top bar - light grey (#C0C0C0) with black text
            topbar_bg_focused: DB_GREY,
            topbar_bg_unfocused: DB_GREY,
            topbar_fg_focused: PURE_BLACK,
            topbar_fg_unfocused: PURE_BLACK,
            clock_bg: DB_BLUE,
            clock_fg: DB_YELLOW,

            // Windows - Title bar (light grey #C0C0C0 with black text)
            window_title_unfocused_fg: PURE_BLACK,
            window_title_unfocused_bg: DB_LIGHT_GREY,
            window_title_focused_fg: PURE_BLACK,
            window_title_focused_bg: DB_GREY,
            // Windows - Border (white #FFFFFF and darker grey #AAAAAA - 3D effect)
            window_border_unfocused_fg: DB_LIGHT_GREY,
            window_border_unfocused_bg: DB_LIGHT_GREY,
            window_border_focused_fg: Color::White,
            window_border_focused_bg: DB_GREY,
            // Windows - Content (blue description panel #0000AA with white text)
            window_content_bg: DB_LIGHT_GREY,
            window_content_fg: Color::White,
            window_shadow_color: DB_BLUE,

            // Window controls
            button_close_color: DB_BRIGHT_RED,
            button_maximize_color: Color::Green,
            button_minimize_color: DB_YELLOW,
            button_bg: DB_GREY,
            resize_handle_normal_fg: Color::White,
            resize_handle_normal_bg: DB_BLUE,
            resize_handle_active_fg: DB_YELLOW,
            resize_handle_active_bg: DB_GREY,

            // UI Buttons - grey dialog style
            button_normal_fg: PURE_BLACK,
            button_normal_bg: DB_GREY,
            button_hovered_fg: Color::White,
            button_hovered_bg: DB_BLUE,
            button_pressed_fg: Color::White,
            button_pressed_bg: PURE_BLACK,

            // Bottom bar - light grey key bar (#C0C0C0) with red/yellow/black text
            bottombar_bg: DB_GREY,
            bottombar_fg: PURE_BLACK,
            bottombar_button_normal_fg: DB_BRIGHT_RED,
            bottombar_button_normal_bg: DB_GREY,
            bottombar_button_focused_fg: PURE_BLACK,
            bottombar_button_focused_bg: DB_YELLOW,
            bottombar_button_minimized_fg: DB_LIGHT_GREY,
            bottombar_button_minimized_bg: DB_GREY,

            // Toggle button
            toggle_enabled_fg: DB_YELLOW,
            toggle_enabled_bg_normal: DB_BLUE,
            toggle_enabled_bg_hovered: DB_GREY,
            toggle_enabled_bg_pressed: PURE_BLACK,
            toggle_disabled_fg: DB_LIGHT_GREY,
            toggle_disabled_bg_normal: DB_GREY,
            toggle_disabled_bg_hovered: DB_LIGHT_GREY,
            toggle_disabled_bg_pressed: PURE_BLACK,

            // Prompts/Dialogs - light grey dialog (#C0C0C0) with black text
            prompt_info_bg: DB_GREY,
            prompt_info_fg: PURE_BLACK,
            prompt_success_bg: Color::Green,
            prompt_success_fg: PURE_BLACK,
            prompt_warning_bg: DB_YELLOW,
            prompt_warning_fg: PURE_BLACK,
            prompt_danger_bg: DB_BRIGHT_RED,
            prompt_danger_fg: Color::White,

            // Dialog buttons
            dialog_button_primary_info_fg: PURE_BLACK,
            dialog_button_primary_info_bg: DB_GREY,
            dialog_button_primary_success_fg: PURE_BLACK,
            dialog_button_primary_success_bg: Color::Green,
            dialog_button_primary_warning_fg: PURE_BLACK,
            dialog_button_primary_warning_bg: DB_YELLOW,
            dialog_button_primary_danger_fg: Color::White,
            dialog_button_primary_danger_bg: DB_BRIGHT_RED,
            dialog_button_secondary_fg: Color::White,
            dialog_button_secondary_bg: DB_BLUE,

            // Config window - grey dialog with blue description panel
            config_title_bg: DB_GREY,
            config_title_fg: PURE_BLACK,
            config_border: Color::White,
            config_content_bg: DB_GREY,
            config_content_fg: PURE_BLACK,
            config_instructions_fg: DB_BLUE,
            config_toggle_on_color: DB_YELLOW,
            config_toggle_off_color: DB_LIGHT_GREY,

            // Calendar - blue with yellow header
            calendar_bg: DB_BLUE,
            calendar_fg: Color::White,
            calendar_title_color: DB_YELLOW,
            calendar_today_bg: DB_YELLOW,
            calendar_today_fg: PURE_BLACK,

            // Scrollbar
            scrollbar_track_fg: DB_LIGHT_GREY,
            scrollbar_thumb_fg: Color::White,

            // Context menu - grey with yellow selection
            menu_bg: DB_GREY,
            menu_fg: PURE_BLACK,
            menu_border: Color::White,
            menu_selected_bg: DB_BLUE,
            menu_selected_fg: Color::White,
            menu_shadow_fg: PURE_BLACK,

            // Snap preview
            snap_preview_border: DB_YELLOW,
            snap_preview_bg: DB_BLUE,

            // Splash screen - blue with yellow text
            splash_border: DB_GREY,
            splash_bg: DB_BLUE,
            splash_fg: DB_YELLOW,

            // Slight input popup - grey dialog style with black input field
            slight_bg: DB_GREY,
            slight_fg: PURE_BLACK,
            slight_border: Color::White,
            slight_input_bg: PURE_BLACK,
            slight_input_fg: Color::White,
            slight_suggestion_fg: DB_LIGHT_GREY,
            slight_dropdown_bg: DB_GREY,
            slight_dropdown_fg: PURE_BLACK,
            slight_dropdown_selected_bg: DB_BLUE,
            slight_dropdown_selected_fg: Color::White,
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
