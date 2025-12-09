//! Panic handler for terminal state restoration.
//!
//! This module sets up a panic hook that restores the terminal to a usable state
//! when the application crashes. This prevents the terminal from being left in
//! raw mode with mouse capture enabled.

use std::io;
use std::panic;

/// Set up a panic hook that restores the terminal state on panic.
///
/// This should be called early in the application's initialization.
/// The hook will:
/// 1. Disable raw mode (most important - has the most side effects)
/// 2. Leave alternate screen
/// 3. Show the cursor
/// 4. Disable mouse capture
/// 5. Reset colors
/// 6. Call the default panic handler to print the panic message
pub fn setup_panic_hook() {
    let default_panic = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal state
        let mut stdout = io::stdout();

        // Best-effort cleanup - ignore errors since we're already panicking
        // Following ratatui's pattern: disable raw mode FIRST (most side effects)
        let _ = crossterm::terminal::disable_raw_mode();

        // Leave alternate screen and show cursor
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );

        // Disable mouse capture
        let _ = crossterm::execute!(stdout, crossterm::event::DisableMouseCapture);

        // Final color reset
        let _ = crossterm::execute!(stdout, crossterm::style::ResetColor);

        // Call the default panic handler to print the panic message
        default_panic(panic_info);
    }));
}
