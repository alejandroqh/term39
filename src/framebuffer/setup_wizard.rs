//! Framebuffer setup wizard.
//!
//! This module provides an interactive setup wizard for configuring
//! framebuffer mode settings like text mode, font, and mouse options.

use crate::framebuffer::fb_setup_window::{FbSetupAction, FbSetupWindow};
use crate::framebuffer::font_manager::FontManager;
use crate::rendering::{Charset, RenderBackend, Theme, VideoBuffer};
use crossterm::event::{self, Event, KeyEventKind, MouseButton, MouseEventKind};
use crossterm::terminal;
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::time::Duration;

/// Run the framebuffer setup wizard.
///
/// This function displays an interactive UI for configuring framebuffer
/// settings and optionally launches the application in framebuffer mode.
///
/// # Returns
/// - `Ok(())` if the wizard completes successfully
/// - `Err(_)` if terminal setup or launch fails
pub fn run_setup_wizard() -> io::Result<()> {
    // Set up terminal for setup wizard
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    // Get terminal size and create video buffer
    let (cols, rows) = terminal::size()?;
    let mut video_buffer = VideoBuffer::new(cols, rows);

    // Create setup window
    let mut setup_window = FbSetupWindow::new(cols, rows);

    // Load available fonts
    let fonts = FontManager::list_available_fonts();
    setup_window.set_fonts(fonts);

    // Create charset and theme for rendering
    let charset = Charset::unicode();
    let theme = Theme::from_name("classic");

    // Create terminal backend for rendering
    let mut term_backend = crate::rendering::TerminalBackend::new()?;

    // Setup wizard event loop
    let mut should_launch = false;
    loop {
        // Render setup window
        setup_window.render(&mut video_buffer, &charset, &theme);

        // Present to terminal
        term_backend.present(&mut video_buffer)?;

        // Poll for crossterm events
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) => {
                    // Only process key press and repeat events (ignore Release)
                    if key_event.kind == KeyEventKind::Release {
                        continue;
                    }
                    let action = setup_window.handle_key(key_event);
                    match action {
                        FbSetupAction::Close => break,
                        FbSetupAction::SaveAndLaunch => {
                            if let Err(e) = setup_window.save_config() {
                                eprintln!("Error saving config: {}", e);
                            }
                            should_launch = true;
                            break;
                        }
                        FbSetupAction::SaveOnly => {
                            if let Err(e) = setup_window.save_config() {
                                eprintln!("Error saving config: {}", e);
                            }
                            break;
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse_event) => {
                    // Only handle actual left-button clicks, not moves
                    if let MouseEventKind::Down(MouseButton::Left) = mouse_event.kind {
                        let action =
                            setup_window.handle_click(mouse_event.column, mouse_event.row);
                        match action {
                            FbSetupAction::Close => break,
                            FbSetupAction::SaveAndLaunch => {
                                if let Err(e) = setup_window.save_config() {
                                    eprintln!("Error saving config: {}", e);
                                }
                                should_launch = true;
                                break;
                            }
                            FbSetupAction::SaveOnly => {
                                if let Err(e) = setup_window.save_config() {
                                    eprintln!("Error saving config: {}", e);
                                }
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Event::Resize(new_cols, new_rows) => {
                    video_buffer = VideoBuffer::new(new_cols, new_rows);
                    setup_window = FbSetupWindow::new(new_cols, new_rows);
                    let fonts = FontManager::list_available_fonts();
                    setup_window.set_fonts(fonts);
                }
                _ => {}
            }
        }
    }

    // Cleanup terminal - reset colors properly to avoid color bleeding on TTY
    cleanup_terminal(&mut stdout)?;

    // If user chose to launch, actually launch the application
    if should_launch {
        launch_framebuffer_mode(&setup_window)?;
    } else {
        println!("Configuration saved to ~/.config/term39/fb.toml");
    }

    Ok(())
}

/// Clean up terminal state after setup wizard.
fn cleanup_terminal(stdout: &mut io::Stdout) -> io::Result<()> {
    crossterm::execute!(stdout, crossterm::event::DisableMouseCapture)?;
    crossterm::execute!(stdout, crossterm::style::ResetColor)?;
    crossterm::execute!(
        stdout,
        crossterm::style::SetAttribute(crossterm::style::Attribute::Reset),
        crossterm::style::SetForegroundColor(crossterm::style::Color::Reset),
        crossterm::style::SetBackgroundColor(crossterm::style::Color::Reset)
    )?;
    crossterm::execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    crossterm::execute!(
        stdout,
        crossterm::cursor::MoveTo(0, 0),
        crossterm::style::ResetColor
    )?;
    crossterm::execute!(
        stdout,
        crossterm::cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    crossterm::execute!(stdout, crossterm::style::ResetColor)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

/// Check permissions and launch framebuffer mode.
fn launch_framebuffer_mode(setup_window: &FbSetupWindow) -> io::Result<()> {
    let config = setup_window.get_config();

    // Check device permissions before launching
    let mut permission_errors: Vec<String> = Vec::new();
    let mut fix_hints: Vec<String> = Vec::new();

    // Check framebuffer device access
    let fb_device = "/dev/fb0";
    if std::fs::metadata(fb_device).is_err() {
        permission_errors.push(format!("Framebuffer device '{}' not found", fb_device));
        fix_hints.push(
            "Ensure you're on a Linux console (TTY), not a terminal emulator".to_string(),
        );
    } else if std::fs::File::open(fb_device).is_err() {
        permission_errors.push(format!("No permission to access '{}'", fb_device));
        fix_hints.push("Add user to video group: sudo usermod -aG video $USER".to_string());
    }

    // Check mouse device access
    let mouse_device = config.get_mouse_device();
    if !mouse_device.is_empty() {
        if std::fs::metadata(&mouse_device).is_err() {
            permission_errors.push(format!("Mouse device '{}' not found", mouse_device));
            fix_hints.push("Check if the mouse device path is correct".to_string());
        } else if std::fs::File::open(&mouse_device).is_err() {
            permission_errors.push(format!("No permission to access '{}'", mouse_device));
            fix_hints.push("Add user to input group: sudo usermod -aG input $USER".to_string());
        }
    }

    // If there are permission errors, show them and exit
    if !permission_errors.is_empty() {
        println!("Configuration saved to ~/.config/term39/fb.toml\n");
        println!("Cannot launch framebuffer mode due to permission issues:\n");
        for error in &permission_errors {
            println!("  - {}", error);
        }
        println!("\nTo fix:");
        for hint in &fix_hints {
            println!("  {}", hint);
        }
        println!("\nAfter adding groups, log out and back in for changes to take effect.");
        println!("\nAlternatively, run with sudo:");
        println!("  sudo term39 -f --fb-mode={}", config.display.mode);
        return Ok(());
    }

    println!("Configuration saved! Launching framebuffer mode...\n");

    // Get the current executable path
    let exe_path =
        std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("./term39"));

    // Build command arguments
    let mut args = vec![
        "-f".to_string(),
        format!("--fb-mode={}", config.display.mode),
        format!("--fb-font={}", config.font.name),
    ];

    if config.display.scale != "auto" {
        args.push(format!("--fb-scale={}", config.display.scale));
    }

    if config.mouse.invert_x {
        args.push("--invert-mouse-x".to_string());
    }

    if config.mouse.invert_y {
        args.push("--invert-mouse-y".to_string());
    }

    if config.mouse.swap_buttons {
        args.push("--swap-mouse-buttons".to_string());
    }

    // Launch directly (user has permissions)
    let mut cmd = std::process::Command::new(&exe_path);
    cmd.args(&args);

    // Use exec to replace current process
    let err = cmd.exec();
    // If we get here, exec failed
    eprintln!("Failed to launch: {}", err);
    Err(err)
}
