# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2025-11-11

### Added
- Settings/configuration window (press 's' key from desktop):
  - Configuration management system with persistence
  - Auto-tiling/auto-arrange windows toggle
  - Show date in top bar toggle
  - Settings saved to `~/.config/term39/config.toml`
- Configuration manager for loading and saving user preferences
- Visual settings window with interactive toggles

### Fixed
- Various clippy warnings for improved code quality

### Changed
- Enhanced help screen to include settings key ('s')
- Bottom bar now shows settings shortcut

## [0.3.0] - 2025-11-10

### Added
- Window snapping to corners and auto-snap feature
  - Automatically snaps windows to screen corners when dragging near edges
  - Improves window positioning and organization

### Fixed
- Resize render glitch that occurred during window resizing operations
  - Multiple improvements to rendering during resize events
  - Smoother window resizing experience

### Changed
- Updated README with splash screen ASCII art for better visual branding

## [0.2.1] - 2025-11-10

### Fixed
- Clippy warnings for collapsible_if in window_manager.rs

## [0.2.0] - 2025-11-10

### Added
- Scrollbar and scroll support for terminal windows:
  - Visual scrollbar in right border (charset-aware)
  - Mouse wheel scrolling (3 lines per notch)
  - Scrollbar thumb dragging for smooth navigation
  - Click track to jump to position
  - Fixed scroll offset to properly fetch from scrollback buffer
  - Inverted thumb position (bottom = current output)
- Configuration file support for user preferences
- Version display with `--version` flag
- Full screen terminal launcher with `T` key
- Calendar widget with `c` key
- Improved help dialog organization
- GitHub Actions CI/CD workflows
- Crates.io metadata for publishing

### Fixed
- Various clippy warnings and test failures
- Error handling improvements
- Code formatting issues

### Changed
- Enhanced help system with better organization
- Improved code quality and maintainability

## [0.1.0] - 2025-11-10

### Added
- Initial release of TERM39 terminal multiplexer
- Retro DOS aesthetic with blue-and-white color scheme
- Multiple terminal window support with full VT100/ANSI emulation
- Window management features:
  - Drag windows by title bar
  - Resize windows with bottom-right handle
  - Minimize/maximize windows
  - Close windows with [X] button
  - Focus management and window switching
- Mouse and keyboard controls:
  - Mouse support for all window operations
  - Keyboard shortcuts (t, q, ESC, h, ALT+TAB)
  - Full terminal input passthrough
- Double-buffered rendering system:
  - Flicker-free display at ~60fps
  - Dirty region tracking for efficiency
  - Save/restore regions for window operations
- ASCII compatibility mode (`--ascii` flag):
  - Fallback to ASCII characters for maximum compatibility
  - Automatic charset configuration
- Cross-platform support:
  - macOS (Apple Silicon ARM64 and Intel x86_64)
  - Linux (x86_64 and ARM64)
- MIT License
- Top bar with clock display and window controls
- Bottom bar with window list and help indicator
- Interactive help dialog (press 'h')
- Confirmation dialogs for exit with open windows
- Window shadows for depth effect

### Technical Details
- Built with Rust 2024 edition
- Dependencies:
  - crossterm 0.29.0 for terminal control
  - chrono 0.4.42 for clock display
  - portable-pty 0.8 for PTY support
  - vte 0.13 for ANSI/VT escape sequences

[0.4.0]: https://github.com/alejandroqh/term39/releases/tag/v0.4.0
[0.3.0]: https://github.com/alejandroqh/term39/releases/tag/v0.3.0
[0.2.1]: https://github.com/alejandroqh/term39/releases/tag/v0.2.1
[0.2.0]: https://github.com/alejandroqh/term39/releases/tag/v0.2.0
[0.1.0]: https://github.com/alejandroqh/term39/releases/tag/v0.1.0
