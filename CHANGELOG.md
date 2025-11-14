# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.1] - 2025-11-14

### Added
- **Linux Framebuffer Backend** (Experimental):
  - Direct Linux console rendering via `/dev/fb0` for DOS-like experience
  - Multiple text modes: 40x25, 80x25, 80x43, 80x50 (classic DOS modes)
  - PSF2 bitmap font support with Unifont for comprehensive Unicode coverage
  - Authentic VGA 16-color palette rendering
  - Pixel-perfect character rendering
  - Hardware cursor rendering with save/restore functionality
  - Requires compile-time flag: `--features framebuffer-backend`
  - Command-line option: `--fb-mode=<MODE>` to select text mode
  - Works on Linux console (TTY) only, not in terminal emulators
- **Enhanced Linux Mouse Support**:
  - Raw mouse input device support (`/dev/input/mice`, `/dev/input/event*`)
  - Hardware mouse cursor rendering in framebuffer mode
  - Pixel-accurate mouse tracking and cursor positioning
  - Button state tracking (left, right, middle buttons)
  - Mouse event queue for reliable button press/release handling
  - Automatic fallback to crossterm when mouse device unavailable
  - Mouse coordinate scaling for different text modes

### Changed
- **UI/UX Improvements**:
  - Enhanced theme system with improved color consistency
  - Updated popup/dialog rendering for better visual appeal
  - Refined shadow system for consistent depth effect across all UI elements
  - Better contrast and readability in all themes
- **Dependencies Updated**:
  - Updated to latest versions of core dependencies
  - Better compatibility with modern Rust toolchain
  - Security patches and performance improvements

### Fixed
- Clippy warning for `get_mouse_button_event()` dead code (framebuffer-only feature)
- Clippy warning for unused `mut` in `injected_event` variable
- Code quality improvements for stable Rust compatibility

### Technical Details
- New dependencies:
  - framebuffer 0.3 (optional, Linux only)
  - memmap2 0.9 (optional, for framebuffer)
  - flate2 1.0 (optional, for PSF2 font decompression)
- Zero clippy warnings with `-D warnings` flag
- Clean code formatting
- Improved build system with conditional compilation for framebuffer backend

## [0.6.6] - 2025-11-12

### Fixed
- **Theme loading on startup**: Fixed issue where selected theme from settings was saved but not loaded on application boot
  - CLI `--theme` argument is now optional (no default value)
  - Application now properly loads saved theme from config file on startup
  - CLI theme argument still takes precedence when explicitly provided

### Technical Details
- Zero clippy warnings
- All tests passing
- Clean code formatting

## [0.6.5] - 2025-11-12

### Fixed
- **Color theme setting persistence**: Fixed issue where color theme selection in settings window wasn't being properly saved to configuration
- **GPM (General Purpose Mouse) handling improvements**:
  - Simplified GPM integration and removed complex build-time detection
  - Improved GPM handler robustness and error handling
  - Better fallback behavior when GPM is unavailable

### Changed
- **AUR package deployment**: Updated .gitignore for better AUR package management

### Technical Details
- Zero clippy warnings with -D warnings flag
- All tests passing (7/7)
- Clean code formatting

## [0.6.0] - 2025-11-12

### Added
- **Clipboard support** with full copy/paste functionality:
  - System clipboard integration via arboard library
  - Text selection in terminal windows (mouse drag to select)
  - Copy selected text with Ctrl+Shift+C or right-click context menu
  - Paste from clipboard with Ctrl+Shift+V or right-click context menu
  - Visual selection highlighting with inverted colors
  - Top bar clipboard buttons (Copy/Paste/Clear) for easy access
  - Block selection mode for rectangular text regions
  - Internal clipboard fallback when system clipboard unavailable
- **Context menu** on right-click in terminal windows:
  - Copy selected text
  - Paste from clipboard
  - Select all (placeholder for future implementation)
  - Copy entire window content (placeholder for future implementation)
  - Close window
- **Additional themes** for enhanced visual customization:
  - Green Phosphor - Classic green monochrome terminal (IBM 5151, VT220 inspired)
  - Amber - Vintage amber monochrome terminal (DEC VT100, Wyse inspired)
  - Existing themes: Classic (DOS blue/cyan), Dark (Dracula), Monochrome (grayscale)
  - Command-line `--theme` option to select theme at startup
  - Theme selector in settings window
- **Tint terminal color improvements**:
  - Enhanced 256-color palette rendering
  - Better color accuracy for terminal applications
  - Improved true color (24-bit RGB) support
  - More accurate ANSI color rendering
- **GPM (General Purpose Mouse) support for Linux console**:
  - Native mouse support in Linux virtual consoles (TTY)
  - Automatic GPM detection and initialization
  - Fallback to crossterm when GPM unavailable
  - Build-time GPM library detection

### Changed
- Command-line help now shows all available themes
- Settings window includes theme selection dropdown
- Improved color rendering consistency across themes

### Fixed
- Clippy warnings resolved with `#![allow(clippy::collapsible_if)]` attribute
- Code quality improvements for stable Rust compatibility

### Technical Details
- New dependencies:
  - arboard 3.4 for clipboard operations
- Zero clippy warnings with -D warnings flag
- All tests passing (7/7)
- Clean release build

## [0.5.1] - 2025-11-11

### Added
- Distribution packaging support:
  - Debian/Ubuntu (.deb) packages for x86_64 and arm64
  - RPM packages for Fedora/RHEL/CentOS/openSUSE (x86_64 and aarch64)
  - AUR PKGBUILD files for Arch Linux (source and binary packages)
- GitHub Actions workflow for automated package building on release
- cargo-deb metadata configuration in Cargo.toml
- cargo-generate-rpm metadata configuration in Cargo.toml

### Changed
- Release workflow now automatically builds and publishes .deb and .rpm packages
- AUR packages available in `aur/` directory with installation instructions

## [0.5.0] - 2025-11-11

### Added
- InfoWindow component for consistent dialog rendering
  - Help and About dialogs now use same bordered window style as Settings
  - Title bar with borders and shadows
  - Consistent black content background across all info dialogs
  - Color code support for text formatting
- Shadow rendering to Config Window for visual consistency

### Fixed
- **Critical**: Window content foreground color now uses theme property instead of hardcoded white
  - All themes now correctly apply `window_content_fg` color
  - Removed hardcoded color fields from Window struct
- Dark theme visibility issues:
  - Shadow color changed from Black to DarkGrey (now visible on black backgrounds)
  - Button colors changed from Dark* variants to bright colors for better contrast
  - Window title bars now have proper color distinction
- Unfocused window title bars now use DarkGrey instead of Black across all themes
  - Prevents confusion with top bar when windows are open
  - Creates clear visual hierarchy between top bar, unfocused windows, and focused windows
- Clippy warning: Reduced function arguments in config_window.rs from 9 to 7 parameters
  - Functions now use theme colors directly instead of passing as parameters

### Changed
- **Dark theme redesigned with Dracula color scheme** (draculatheme.com):
  - Purple/Magenta accents for focused windows and interactive elements
  - Cyan accents for info dialogs and splash screen
  - Bright white foreground for better contrast and readability
  - Dracula semantic colors: Green (success), Red (danger), Yellow (warning)
  - Window borders now use Magenta (purple) instead of grey
  - Clock displays in Magenta (purple accent)
- Standardized shadow rendering across all window types:
  - Created shared `render_shadow()` helper function in video_buffer.rs
  - All dialogs (windows, prompts, config, help, about, splash) use consistent shadow rendering
  - All shadows now use `charset.shadow` character consistently
- Theme color consistency improvements:
  - Monochrome theme: Focused window title uses Grey instead of DarkGrey
  - Classic theme: Unfocused window title uses DarkGrey instead of Black
- Removed unused imports and dead code

### Technical Details
- Code quality improvements for release readiness
- Zero clippy warnings
- All tests passing (4/4)
- Clean release build (1.5 MB binary)

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

[0.7.1]: https://github.com/alejandroqh/term39/releases/tag/v0.7.1
[0.6.6]: https://github.com/alejandroqh/term39/releases/tag/v0.6.6
[0.6.5]: https://github.com/alejandroqh/term39/releases/tag/v0.6.5
[0.6.0]: https://github.com/alejandroqh/term39/releases/tag/v0.6.0
[0.5.1]: https://github.com/alejandroqh/term39/releases/tag/v0.5.1
[0.5.0]: https://github.com/alejandroqh/term39/releases/tag/v0.5.0
[0.4.0]: https://github.com/alejandroqh/term39/releases/tag/v0.4.0
[0.3.0]: https://github.com/alejandroqh/term39/releases/tag/v0.3.0
[0.2.1]: https://github.com/alejandroqh/term39/releases/tag/v0.2.1
[0.2.0]: https://github.com/alejandroqh/term39/releases/tag/v0.2.0
[0.1.0]: https://github.com/alejandroqh/term39/releases/tag/v0.1.0
