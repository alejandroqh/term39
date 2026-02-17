use crossterm::event::{KeyCode, KeyModifiers};

/// A single key binding: key code + required modifiers
#[derive(Clone, Debug)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Check if this binding matches the given key code and modifiers
    pub fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.code == code && modifiers.contains(self.modifiers)
    }
}

/// Helper: create a binding with no modifiers
fn key(code: KeyCode) -> KeyBinding {
    KeyBinding::new(code, KeyModifiers::NONE)
}

/// Helper: create a binding with ALT modifier
fn alt(code: KeyCode) -> KeyBinding {
    KeyBinding::new(code, KeyModifiers::ALT)
}

/// Helper: create a binding with ALT+SHIFT modifier
fn alt_shift(code: KeyCode) -> KeyBinding {
    KeyBinding::new(code, KeyModifiers::ALT.union(KeyModifiers::SHIFT))
}

/// Helper: create a binding with SHIFT modifier
fn shift(code: KeyCode) -> KeyBinding {
    KeyBinding::new(code, KeyModifiers::SHIFT)
}

/// Check if any binding in the list matches the given code and modifiers
pub fn matches_any(bindings: &[KeyBinding], code: KeyCode, modifiers: KeyModifiers) -> bool {
    bindings.iter().any(|b| b.matches(code, modifiers))
}

/// All available profile names
const PROFILE_NAMES: &[&str] = &["term39", "hyprland"];

/// Display names for profiles
const PROFILE_DISPLAY_NAMES: &[&str] = &["Term39", "Hyprland"];

/// A complete set of keybindings for all application actions
#[derive(Clone, Debug)]
pub struct KeybindingProfile {
    pub name: String,

    // -- Desktop actions --
    pub help: Vec<KeyBinding>,
    pub cycle_window: Vec<KeyBinding>,
    pub save_session: Vec<KeyBinding>,
    pub copy: Vec<KeyBinding>,
    pub paste: Vec<KeyBinding>,
    pub new_terminal: Vec<KeyBinding>,
    pub new_terminal_maximized: Vec<KeyBinding>,
    pub toggle_window_mode: Vec<KeyBinding>,
    pub exit: Vec<KeyBinding>,
    pub settings: Vec<KeyBinding>,
    pub calendar: Vec<KeyBinding>,
    pub about: Vec<KeyBinding>,
    #[allow(dead_code)]
    pub launcher: Vec<KeyBinding>,
    pub lock_screen: Vec<KeyBinding>,

    // -- Window Mode actions --
    pub wm_focus_left: Vec<KeyBinding>,
    pub wm_focus_down: Vec<KeyBinding>,
    pub wm_focus_up: Vec<KeyBinding>,
    pub wm_focus_right: Vec<KeyBinding>,
    pub wm_snap_left: Vec<KeyBinding>,
    pub wm_snap_down: Vec<KeyBinding>,
    pub wm_snap_up: Vec<KeyBinding>,
    pub wm_snap_right: Vec<KeyBinding>,
    pub wm_enter_move: Vec<KeyBinding>,
    pub wm_enter_resize: Vec<KeyBinding>,
    pub wm_close: Vec<KeyBinding>,
    pub wm_maximize: Vec<KeyBinding>,
    pub wm_minimize: Vec<KeyBinding>,
    pub wm_toggle_auto_tiling: Vec<KeyBinding>,

    // -- Direct-mode actions (Alt-modifier, work from any focus) --
    pub direct_close_window: Vec<KeyBinding>,
    pub direct_focus_left: Vec<KeyBinding>,
    pub direct_focus_down: Vec<KeyBinding>,
    pub direct_focus_up: Vec<KeyBinding>,
    pub direct_focus_right: Vec<KeyBinding>,
    pub direct_snap_left: Vec<KeyBinding>,
    pub direct_snap_down: Vec<KeyBinding>,
    pub direct_snap_up: Vec<KeyBinding>,
    pub direct_snap_right: Vec<KeyBinding>,
    pub direct_maximize: Vec<KeyBinding>,
    pub direct_toggle_auto_tiling: Vec<KeyBinding>,
    pub direct_new_terminal: Vec<KeyBinding>,
    pub direct_new_terminal_maximized: Vec<KeyBinding>,
    pub direct_settings: Vec<KeyBinding>,

    // -- Flags --
    #[allow(dead_code)]
    pub uses_window_mode: bool,
}

impl KeybindingProfile {
    /// Create the default term39 profile (maps current hardcoded keys exactly)
    pub fn term39() -> Self {
        Self {
            name: "term39".to_string(),

            // Desktop actions
            help: vec![
                key(KeyCode::F(1)),
                key(KeyCode::Char('?')),
                key(KeyCode::Char('h')),
            ],
            cycle_window: vec![key(KeyCode::F(2)), alt(KeyCode::Tab)],
            save_session: vec![key(KeyCode::F(3))],
            copy: vec![key(KeyCode::F(5))],
            paste: vec![key(KeyCode::F(6))],
            new_terminal: vec![key(KeyCode::F(7)), key(KeyCode::Char('t'))],
            new_terminal_maximized: vec![key(KeyCode::Char('T'))],
            toggle_window_mode: vec![key(KeyCode::Char('`')), key(KeyCode::F(8))],
            exit: vec![
                key(KeyCode::Esc),
                key(KeyCode::Char('q')),
                key(KeyCode::F(10)),
            ],
            settings: vec![key(KeyCode::Char('s'))],
            calendar: vec![key(KeyCode::Char('c'))],
            about: vec![key(KeyCode::Char('l'))],
            launcher: vec![], // Handled separately via Ctrl+Space
            lock_screen: vec![shift(KeyCode::Char('Q'))],

            // Window Mode actions
            wm_focus_left: vec![key(KeyCode::Char('h')), key(KeyCode::Left)],
            wm_focus_down: vec![key(KeyCode::Char('j')), key(KeyCode::Down)],
            wm_focus_up: vec![key(KeyCode::Char('k')), key(KeyCode::Up)],
            wm_focus_right: vec![key(KeyCode::Char('l')), key(KeyCode::Right)],
            wm_snap_left: vec![shift(KeyCode::Char('H')), shift(KeyCode::Left)],
            wm_snap_down: vec![shift(KeyCode::Char('J')), shift(KeyCode::Down)],
            wm_snap_up: vec![shift(KeyCode::Char('K')), shift(KeyCode::Up)],
            wm_snap_right: vec![shift(KeyCode::Char('L')), shift(KeyCode::Right)],
            wm_enter_move: vec![key(KeyCode::Char('m'))],
            wm_enter_resize: vec![key(KeyCode::Char('r'))],
            wm_close: vec![key(KeyCode::Char('x')), key(KeyCode::Char('q'))],
            wm_maximize: vec![
                key(KeyCode::Char('z')),
                key(KeyCode::Char('+')),
                key(KeyCode::Char(' ')),
            ],
            wm_minimize: vec![key(KeyCode::Char('-')), key(KeyCode::Char('_'))],
            wm_toggle_auto_tiling: vec![key(KeyCode::Char('a'))],

            // Direct-mode: empty for term39 (all through Window Mode)
            direct_close_window: vec![],
            direct_focus_left: vec![],
            direct_focus_down: vec![],
            direct_focus_up: vec![],
            direct_focus_right: vec![],
            direct_snap_left: vec![],
            direct_snap_down: vec![],
            direct_snap_up: vec![],
            direct_snap_right: vec![],
            direct_maximize: vec![],
            direct_toggle_auto_tiling: vec![],
            direct_new_terminal: vec![],
            direct_new_terminal_maximized: vec![],
            direct_settings: vec![],

            uses_window_mode: true,
        }
    }

    /// Create the Hyprland-style profile (Alt-modifier based)
    pub fn hyprland() -> Self {
        // macOS Option+letter character mappings
        let mut direct_focus_left = vec![alt(KeyCode::Char('h'))];
        let mut direct_focus_down = vec![alt(KeyCode::Char('j'))];
        let mut direct_focus_up = vec![alt(KeyCode::Char('k'))];
        let mut direct_focus_right = vec![alt(KeyCode::Char('l'))];
        let mut direct_snap_left = vec![alt_shift(KeyCode::Char('H'))];
        let mut direct_snap_down = vec![alt_shift(KeyCode::Char('J'))];
        let mut direct_snap_up = vec![alt_shift(KeyCode::Char('K'))];
        let mut direct_snap_right = vec![alt_shift(KeyCode::Char('L'))];
        let mut direct_close = vec![alt(KeyCode::Char('q'))];
        let mut direct_maximize = vec![alt(KeyCode::Char('f'))];
        let mut direct_auto_tiling = vec![alt(KeyCode::Char('v'))];
        let direct_new_term = vec![alt(KeyCode::Enter)];
        let direct_new_term_max = vec![alt_shift(KeyCode::Enter)];
        let mut direct_settings = vec![alt(KeyCode::Char('s'))];

        // On macOS, Option+letter produces special Unicode characters
        // Add those as additional bindings so they are recognized
        if cfg!(target_os = "macos") {
            // Option+H = '˙', Option+J = '∆', Option+K = '˚', Option+L = '¬'
            direct_focus_left.push(key(KeyCode::Char('˙')));
            direct_focus_down.push(key(KeyCode::Char('∆')));
            direct_focus_up.push(key(KeyCode::Char('˚')));
            direct_focus_right.push(key(KeyCode::Char('¬')));
            // Option+Q = 'œ', Option+F = 'ƒ', Option+V = '√', Option+S = 'ß'
            direct_close.push(key(KeyCode::Char('œ')));
            direct_maximize.push(key(KeyCode::Char('ƒ')));
            direct_auto_tiling.push(key(KeyCode::Char('√')));
            direct_settings.push(key(KeyCode::Char('ß')));
            // Shift+Option produces different chars
            // Shift+Option+H = 'Ó', Shift+Option+J = 'Ô', Shift+Option+K = '', Shift+Option+L = 'Ò'
            direct_snap_left.push(key(KeyCode::Char('Ó')));
            direct_snap_down.push(key(KeyCode::Char('Ô')));
            direct_snap_up.push(key(KeyCode::Char('\u{f8ff}'))); // Apple logo char
            direct_snap_right.push(key(KeyCode::Char('Ò')));
        }

        Self {
            name: "hyprland".to_string(),

            // Desktop actions (from desktop/topbar focus)
            help: vec![key(KeyCode::F(1)), key(KeyCode::Char('?'))],
            cycle_window: vec![alt(KeyCode::Tab), key(KeyCode::F(2))],
            save_session: vec![key(KeyCode::F(3))],
            copy: vec![key(KeyCode::F(5))],
            paste: vec![key(KeyCode::F(6))],
            new_terminal: vec![key(KeyCode::Char('t'))],
            new_terminal_maximized: vec![key(KeyCode::Char('T'))],
            toggle_window_mode: vec![key(KeyCode::F(8))], // No backtick (backtick goes to terminal)
            exit: vec![key(KeyCode::Esc), key(KeyCode::F(10))], // No bare 'q'
            settings: vec![key(KeyCode::Char('s'))],
            calendar: vec![key(KeyCode::Char('c'))],
            about: vec![key(KeyCode::Char('l'))],
            launcher: vec![alt(KeyCode::Char(' '))],
            lock_screen: vec![shift(KeyCode::Char('Q'))],

            // Window Mode actions (same as term39 since F8 still works)
            wm_focus_left: vec![key(KeyCode::Char('h')), key(KeyCode::Left)],
            wm_focus_down: vec![key(KeyCode::Char('j')), key(KeyCode::Down)],
            wm_focus_up: vec![key(KeyCode::Char('k')), key(KeyCode::Up)],
            wm_focus_right: vec![key(KeyCode::Char('l')), key(KeyCode::Right)],
            wm_snap_left: vec![shift(KeyCode::Char('H')), shift(KeyCode::Left)],
            wm_snap_down: vec![shift(KeyCode::Char('J')), shift(KeyCode::Down)],
            wm_snap_up: vec![shift(KeyCode::Char('K')), shift(KeyCode::Up)],
            wm_snap_right: vec![shift(KeyCode::Char('L')), shift(KeyCode::Right)],
            wm_enter_move: vec![key(KeyCode::Char('m'))],
            wm_enter_resize: vec![key(KeyCode::Char('r'))],
            wm_close: vec![key(KeyCode::Char('x')), key(KeyCode::Char('q'))],
            wm_maximize: vec![
                key(KeyCode::Char('z')),
                key(KeyCode::Char('+')),
                key(KeyCode::Char(' ')),
            ],
            wm_minimize: vec![key(KeyCode::Char('-')), key(KeyCode::Char('_'))],
            wm_toggle_auto_tiling: vec![key(KeyCode::Char('a'))],

            // Direct-mode actions (Alt-modifier, work from any focus)
            direct_close_window: direct_close,
            direct_focus_left,
            direct_focus_down,
            direct_focus_up,
            direct_focus_right,
            direct_snap_left,
            direct_snap_down,
            direct_snap_up,
            direct_snap_right,
            direct_maximize,
            direct_toggle_auto_tiling: direct_auto_tiling,
            direct_new_terminal: direct_new_term,
            direct_new_terminal_maximized: direct_new_term_max,
            direct_settings,

            uses_window_mode: false,
        }
    }

    /// Create a profile from a name string
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "hyprland" => Self::hyprland(),
            _ => Self::term39(),
        }
    }

    /// Get all available profile names
    #[allow(dead_code)]
    pub fn all_names() -> &'static [&'static str] {
        PROFILE_NAMES
    }

    /// Get the next profile name (cycling)
    pub fn next_name(current: &str) -> &'static str {
        let idx = PROFILE_NAMES
            .iter()
            .position(|&n| n == current)
            .unwrap_or(0);
        PROFILE_NAMES[(idx + 1) % PROFILE_NAMES.len()]
    }

    /// Get the previous profile name (cycling backward)
    pub fn prev_name(current: &str) -> &'static str {
        let idx = PROFILE_NAMES
            .iter()
            .position(|&n| n == current)
            .unwrap_or(0);
        if idx == 0 {
            PROFILE_NAMES[PROFILE_NAMES.len() - 1]
        } else {
            PROFILE_NAMES[idx - 1]
        }
    }

    /// Get the display name for a profile name
    pub fn display_name(name: &str) -> &'static str {
        let idx = PROFILE_NAMES.iter().position(|&n| n == name).unwrap_or(0);
        PROFILE_DISPLAY_NAMES[idx]
    }

    /// Check if this profile has any direct-mode bindings
    pub fn has_direct_bindings(&self) -> bool {
        !self.direct_close_window.is_empty()
            || !self.direct_focus_left.is_empty()
            || !self.direct_new_terminal.is_empty()
    }
}
