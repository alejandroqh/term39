use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

/// Scans the system PATH and indexes all available executable commands
#[derive(Clone)]
pub struct CommandIndexer {
    commands: Vec<String>,
}

impl CommandIndexer {
    /// Creates a new CommandIndexer and scans the PATH
    pub fn new() -> Self {
        let mut indexer = CommandIndexer {
            commands: Vec::new(),
        };
        indexer.scan_path();
        indexer
    }

    /// Scans all directories in PATH environment variable for executables
    fn scan_path(&mut self) {
        let mut seen = HashSet::new();

        if let Ok(path_var) = env::var("PATH") {
            for dir in path_var.split(':') {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        if let Ok(file_name) = entry.file_name().into_string() {
                            // Check if file is executable
                            if self.is_executable(&entry.path()) && !seen.contains(&file_name) {
                                seen.insert(file_name.clone());
                                self.commands.push(file_name);
                            }
                        }
                    }
                }
            }
        }

        // Sort for consistent ordering
        self.commands.sort();
    }

    /// Checks if a file is executable
    #[cfg(unix)]
    fn is_executable(&self, path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = fs::metadata(path) {
            let permissions = metadata.permissions();
            // Check if any execute bit is set (owner, group, or other)
            permissions.mode() & 0o111 != 0
        } else {
            false
        }
    }

    #[cfg(not(unix))]
    fn is_executable(&self, path: &Path) -> bool {
        // On Windows, check file extension
        if let Some(ext) = path.extension() {
            matches!(
                ext.to_str().unwrap_or("").to_lowercase().as_str(),
                "exe" | "bat" | "cmd" | "com"
            )
        } else {
            false
        }
    }

    /// Returns all indexed commands
    pub fn get_commands(&self) -> &[String] {
        &self.commands
    }

    /// Returns the number of indexed commands
    #[allow(dead_code)]
    pub fn count(&self) -> usize {
        self.commands.len()
    }
}

impl Default for CommandIndexer {
    fn default() -> Self {
        Self::new()
    }
}
