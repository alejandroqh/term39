//! CLI handlers for framebuffer-related command line options.
//!
//! This module provides handlers for `--fb-list-fonts` and similar CLI flags
//! that require framebuffer functionality.

use crate::framebuffer::font_manager::FontManager;

/// List all available console fonts and print them to stdout.
///
/// Fonts are grouped by dimensions and printed in a human-readable format.
/// This is used when `--fb-list-fonts` is passed on the command line.
pub fn list_fonts() {
    println!("Available console fonts:\n");
    let fonts = FontManager::list_available_fonts();

    if fonts.is_empty() {
        println!("No console fonts found in:");
        println!("  - /usr/share/consolefonts/");
        println!("  - /usr/share/kbd/consolefonts/");
        println!("\nInstall fonts with: sudo apt install kbd unifont");
    } else {
        // Group by dimensions
        let mut current_dim = (0, 0);
        for (name, width, height) in fonts {
            if (width, height) != current_dim {
                if current_dim != (0, 0) {
                    println!();
                }
                println!("{}×{} fonts:", width, height);
                current_dim = (width, height);
            }
            println!("  {}", name);
        }
        println!("\nUse with: term39 -f --fb-font=FONT_NAME");
    }
}
