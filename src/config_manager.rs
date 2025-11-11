use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_tiling_on_startup: bool,
    pub show_date_in_clock: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_background_char_index")]
    pub background_char_index: usize,
}

fn default_theme() -> String {
    "classic".to_string()
}

fn default_background_char_index() -> usize {
    0 // Default to first option (light shade)
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_tiling_on_startup: true,
            show_date_in_clock: true,
            theme: default_theme(),
            background_char_index: default_background_char_index(),
        }
    }
}

impl AppConfig {
    /// Get the configuration file path
    /// Returns ~/Library/Application Support/term39/config.toml on macOS
    /// Returns ~/.config/term39/config.toml on Linux
    /// Returns %APPDATA%\term39\config.toml on Windows
    fn config_path() -> Option<PathBuf> {
        let config_dir = dirs::config_dir()?;
        let app_config_dir = config_dir.join("term39");
        Some(app_config_dir.join("config.toml"))
    }

    /// Load configuration from file, creating default if it doesn't exist
    pub fn load() -> Self {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Self::default(),
        };

        // If config file doesn't exist, create default
        if !path.exists() {
            let default_config = Self::default();
            let _ = default_config.save();
            return default_config;
        }

        // Read and parse config file
        match fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path().ok_or("Could not determine config path")?;

        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize and write config
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;

        Ok(())
    }

    /// Toggle auto tiling on startup setting and save
    pub fn toggle_auto_tiling_on_startup(&mut self) {
        self.auto_tiling_on_startup = !self.auto_tiling_on_startup;
        let _ = self.save();
    }

    /// Toggle show date in clock setting and save
    pub fn toggle_show_date_in_clock(&mut self) {
        self.show_date_in_clock = !self.show_date_in_clock;
        let _ = self.save();
    }

    /// Set theme and save
    #[allow(dead_code)]
    pub fn set_theme(&mut self, theme_name: String) {
        self.theme = theme_name;
        let _ = self.save();
    }

    /// Available background characters (5 options)
    pub const BACKGROUND_CHARS: [char; 5] = [
        '░', // 0: Light shade (default)
        ' ', // 1: Empty/space (clean)
        '▒', // 2: Medium shade
        '▓', // 3: Dark shade
        '·', // 4: Middle dot (subtle)
    ];

    /// Available background character names for display
    pub const BACKGROUND_CHAR_NAMES: [&'static str; 5] = [
        "Light Shade",
        "Empty",
        "Medium Shade",
        "Dark Shade",
        "Dot Pattern",
    ];

    /// Get the current background character
    pub fn get_background_char(&self) -> char {
        Self::BACKGROUND_CHARS
            .get(self.background_char_index)
            .copied()
            .unwrap_or(Self::BACKGROUND_CHARS[0])
    }

    /// Get the current background character name
    pub fn get_background_char_name(&self) -> &'static str {
        Self::BACKGROUND_CHAR_NAMES
            .get(self.background_char_index)
            .unwrap_or(&Self::BACKGROUND_CHAR_NAMES[0])
    }

    /// Cycle to the next background character and save
    pub fn cycle_background_char(&mut self) {
        self.background_char_index =
            (self.background_char_index + 1) % Self::BACKGROUND_CHARS.len();
        let _ = self.save();
    }
}
