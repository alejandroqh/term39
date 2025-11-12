use arboard::Clipboard;

/// Manages clipboard operations with system clipboard integration
pub struct ClipboardManager {
    clipboard: Option<Clipboard>,
    last_copied: Option<String>,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Self {
        let clipboard = Clipboard::new().ok();
        if clipboard.is_none() {
            eprintln!("Warning: Could not initialize system clipboard");
        }
        Self {
            clipboard,
            last_copied: None,
        }
    }

    /// Copy text to clipboard
    pub fn copy(&mut self, text: String) -> Result<(), String> {
        if text.is_empty() {
            return Err("Cannot copy empty text".to_string());
        }

        // Store in internal buffer
        self.last_copied = Some(text.clone());

        // Try to copy to system clipboard
        if let Some(clipboard) = &mut self.clipboard {
            clipboard
                .set_text(text)
                .map_err(|e| format!("Failed to copy to system clipboard: {}", e))?;
        }

        Ok(())
    }

    /// Get text from clipboard (system or internal)
    pub fn paste(&mut self) -> Result<String, String> {
        // Try system clipboard first
        if let Some(clipboard) = &mut self.clipboard {
            if let Ok(text) = clipboard.get_text() {
                return Ok(text);
            }
        }

        // Fall back to internal buffer
        self.last_copied
            .clone()
            .ok_or_else(|| "Clipboard is empty".to_string())
    }

    /// Check if clipboard has content
    pub fn has_content(&self) -> bool {
        self.last_copied.is_some()
    }

    /// Clear clipboard
    pub fn clear(&mut self) {
        self.last_copied = None;
        if let Some(clipboard) = &mut self.clipboard {
            let _ = clipboard.clear();
        }
    }

    /// Get last copied text (internal buffer only)
    #[allow(dead_code)]
    pub fn last_copied(&self) -> Option<&str> {
        self.last_copied.as_deref()
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}
