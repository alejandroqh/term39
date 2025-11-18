use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Tracks command usage frequency for intelligent suggestions
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandHistory {
    /// Command -> usage count
    frequency: HashMap<String, u32>,
    /// Path to history file
    #[serde(skip)]
    history_file: PathBuf,
}

impl Clone for CommandHistory {
    fn clone(&self) -> Self {
        CommandHistory {
            frequency: self.frequency.clone(),
            history_file: Self::get_history_path(),
        }
    }
}

impl CommandHistory {
    /// Creates a new CommandHistory and loads from disk
    pub fn new() -> Self {
        let history_file = Self::get_history_path();
        let mut history = CommandHistory {
            frequency: HashMap::new(),
            history_file: history_file.clone(),
        };

        // Try to load existing history
        if history_file.exists() {
            if let Ok(contents) = fs::read_to_string(&history_file) {
                if let Ok(loaded) = serde_json::from_str::<CommandHistory>(&contents) {
                    history.frequency = loaded.frequency;
                }
            }
        }

        history
    }

    /// Returns the path to the history file
    fn get_history_path() -> PathBuf {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".term39");

        // Create directory if it doesn't exist
        let _ = fs::create_dir_all(&path);

        path.push("command_history.json");
        path
    }

    /// Records a command execution
    pub fn record_command(&mut self, command: &str) {
        // Only record the command name, not arguments
        let cmd_name = command.split_whitespace().next().unwrap_or(command);
        if !cmd_name.is_empty() {
            *self.frequency.entry(cmd_name.to_string()).or_insert(0) += 1;
            // Save after each update
            let _ = self.save();
        }
    }

    /// Gets the usage frequency for a command (0 if never used)
    pub fn get_frequency(&self, command: &str) -> u32 {
        *self.frequency.get(command).unwrap_or(&0)
    }

    /// Returns all commands sorted by frequency (most frequent first)
    pub fn get_frequent_commands(&self) -> Vec<(String, u32)> {
        let mut commands: Vec<_> = self
            .frequency
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        commands.sort_by(|a, b| b.1.cmp(&a.1)); // Sort descending by frequency
        commands
    }

    /// Saves the history to disk
    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&self.history_file, json)?;
        Ok(())
    }

    /// Clears all history
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.frequency.clear();
        let _ = self.save();
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}
