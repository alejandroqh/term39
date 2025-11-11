use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_tiling_on_startup: bool,
    pub show_date_in_clock: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_tiling_on_startup: true,
            show_date_in_clock: true,
        }
    }
}

impl AppConfig {
    /// Get the configuration file path
    /// Returns ~/.config/term39/config.toml on Unix
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
}
