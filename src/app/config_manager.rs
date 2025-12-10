use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Authentication mode for lockscreen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LockscreenAuthMode {
    OsAuth, // PAM/macOS/Windows native auth
    #[default]
    Pin, // Alphanumeric PIN with local hash (default - always available)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_auto_tiling_on_startup")]
    pub auto_tiling_on_startup: bool,
    #[serde(default = "default_tiling_gaps")]
    pub tiling_gaps: bool,
    #[serde(default = "default_show_date_in_clock")]
    pub show_date_in_clock: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_background_char_index")]
    pub background_char_index: usize,
    #[serde(default = "default_tint_terminal")]
    pub tint_terminal: bool,
    #[serde(default = "default_auto_save")]
    pub auto_save: bool,
    #[serde(default = "default_lockscreen_enabled")]
    pub lockscreen_enabled: bool,
    #[serde(default)]
    pub lockscreen_auth_mode: LockscreenAuthMode,
    #[serde(default)]
    pub lockscreen_pin_hash: Option<String>,
    #[serde(default)]
    pub lockscreen_salt: Option<String>,
}

fn default_auto_tiling_on_startup() -> bool {
    false // Default to false (disabled at startup)
}

fn default_tiling_gaps() -> bool {
    true // Default to true (gaps between tiled windows for better visual separation)
}

fn default_show_date_in_clock() -> bool {
    true // Default to true (show date in clock)
}

fn default_theme() -> String {
    "classic".to_string()
}

fn default_background_char_index() -> usize {
    0 // Default to first option (light shade)
}

fn default_tint_terminal() -> bool {
    false // Default to false (preserve native ANSI colors)
}

fn default_auto_save() -> bool {
    true // Default to true (auto-save session on exit)
}

fn default_lockscreen_enabled() -> bool {
    true // Default to true (maintains existing behavior)
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_tiling_on_startup: false,
            tiling_gaps: true,
            show_date_in_clock: true,
            theme: default_theme(),
            background_char_index: default_background_char_index(),
            tint_terminal: default_tint_terminal(),
            auto_save: default_auto_save(),
            lockscreen_enabled: default_lockscreen_enabled(),
            lockscreen_auth_mode: LockscreenAuthMode::default(),
            lockscreen_pin_hash: None,
            lockscreen_salt: None,
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

    /// Toggle tiling gaps setting and save
    pub fn toggle_tiling_gaps(&mut self) {
        self.tiling_gaps = !self.tiling_gaps;
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
        '█', // 4: Full block (100% solid)
    ];

    /// Available background character names for display
    pub const BACKGROUND_CHAR_NAMES: [&'static str; 5] = [
        "Light Shade",
        "Empty",
        "Medium Shade",
        "Dark Shade",
        "Full Block",
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

    /// Cycle to the previous background character and save
    pub fn cycle_background_char_backward(&mut self) {
        if self.background_char_index == 0 {
            self.background_char_index = Self::BACKGROUND_CHARS.len() - 1;
        } else {
            self.background_char_index -= 1;
        }
        let _ = self.save();
    }

    /// Toggle terminal tinting setting and save
    pub fn toggle_tint_terminal(&mut self) {
        self.tint_terminal = !self.tint_terminal;
        let _ = self.save();
    }

    /// Toggle auto-save setting and save
    pub fn toggle_auto_save(&mut self) {
        self.auto_save = !self.auto_save;
        let _ = self.save();
    }

    /// Toggle lockscreen enabled state
    pub fn toggle_lockscreen_enabled(&mut self) {
        self.lockscreen_enabled = !self.lockscreen_enabled;
        if !self.lockscreen_enabled {
            // Clear PIN when disabling lockscreen
            self.lockscreen_pin_hash = None;
            self.lockscreen_salt = None;
        }
        let _ = self.save();
    }

    /// Cycle lockscreen auth mode (only switches if OS auth is available)
    pub fn cycle_lockscreen_auth_mode(&mut self, os_auth_available: bool) {
        self.lockscreen_auth_mode = match self.lockscreen_auth_mode {
            LockscreenAuthMode::OsAuth => LockscreenAuthMode::Pin,
            LockscreenAuthMode::Pin => {
                if os_auth_available {
                    LockscreenAuthMode::OsAuth
                } else {
                    LockscreenAuthMode::Pin // Stay on PIN if OS auth unavailable
                }
            }
        };
        let _ = self.save();
    }

    /// Check if PIN is configured
    pub fn has_pin_configured(&self) -> bool {
        self.lockscreen_pin_hash.is_some() && self.lockscreen_salt.is_some()
    }

    /// Set PIN hash and salt
    pub fn set_pin(&mut self, hash: String, salt: String) {
        self.lockscreen_pin_hash = Some(hash);
        self.lockscreen_salt = Some(salt);
        let _ = self.save();
    }

    /// Clear PIN
    #[allow(dead_code)]
    pub fn clear_pin(&mut self) {
        self.lockscreen_pin_hash = None;
        self.lockscreen_salt = None;
        let _ = self.save();
    }

    /// Get or create salt for PIN hashing
    pub fn get_or_create_salt(&mut self) -> String {
        if let Some(ref salt) = self.lockscreen_salt {
            return salt.clone();
        }

        // Try to read machine-id (Linux)
        #[cfg(target_os = "linux")]
        if let Ok(machine_id) = std::fs::read_to_string("/etc/machine-id") {
            let salt = machine_id.trim().to_string();
            self.lockscreen_salt = Some(salt.clone());
            let _ = self.save();
            return salt;
        }

        // Try to read hostid (FreeBSD/NetBSD)
        #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
        {
            // First try /etc/hostid
            if let Ok(hostid) = std::fs::read_to_string("/etc/hostid") {
                let salt = hostid.trim().to_string();
                if !salt.is_empty() {
                    self.lockscreen_salt = Some(salt.clone());
                    let _ = self.save();
                    return salt;
                }
            }

            // FreeBSD: Try kern.hostuuid sysctl
            #[cfg(target_os = "freebsd")]
            {
                use std::process::Command;
                if let Ok(output) = Command::new("sysctl")
                    .args(["-n", "kern.hostuuid"])
                    .output()
                {
                    if output.status.success() {
                        let uuid = String::from_utf8_lossy(&output.stdout);
                        let salt = uuid.trim().to_string();
                        if !salt.is_empty() {
                            self.lockscreen_salt = Some(salt.clone());
                            let _ = self.save();
                            return salt;
                        }
                    }
                }
            }
        }

        // OpenBSD: No standard machine-id, fall through to random salt

        // Try to read machine UUID (macOS)
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("ioreg")
                .args(["-rd1", "-c", "IOPlatformExpertDevice"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("IOPlatformUUID") {
                        if let Some(uuid) = line.split('"').nth(3) {
                            self.lockscreen_salt = Some(uuid.to_string());
                            let _ = self.save();
                            return uuid.to_string();
                        }
                    }
                }
            }
        }

        // Fallback: generate random salt
        let random_salt = Self::generate_random_salt();
        self.lockscreen_salt = Some(random_salt.clone());
        let _ = self.save();
        random_salt
    }

    fn generate_random_salt() -> String {
        use sha2::{Digest, Sha256};
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut hasher = Sha256::new();

        // Time-based entropy
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            hasher.update(duration.as_nanos().to_le_bytes());
        }

        // Process ID entropy
        hasher.update(std::process::id().to_le_bytes());

        // Additional entropy from environment if available
        if let Ok(home) = std::env::var("HOME") {
            hasher.update(home.as_bytes());
        }
        if let Ok(user) = std::env::var("USER") {
            hasher.update(user.as_bytes());
        }

        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
