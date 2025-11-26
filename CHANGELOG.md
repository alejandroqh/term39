# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.14.0] - 2025-11-26

### Added

- TTY/Framebuffer mouse wheel scroll support
- Custom TTY cursor rendering (removes GPM dependency)

### Changed

- Improved TTY exit handling for clean terminal restoration
- Enhanced mouse control in TTY/framebuffer modes

### Fixed

- Small screen new window positioning
- Alt menu handling issues

## [0.13.0] - 2025-11-25

### Added

- Exit confirmation prompts with Windows support
- Comprehensive keyboard system overhaul

### Changed

- Updated screenshots and theme assets
- Improved framebuffer font rendering
- Enhanced framebuffer and GPM handling

### Fixed

- Framebuffer font rendering issues
- GPM mouse handling improvements

## [0.12.0] - 2025-11-24

### Added

- Unicode width support for proper wide character handling

### Changed

- Improved color themes: Dbase, Tpascal, Qbasic, NDD, Dracula, Amber
- Enhanced window tinting for better visual appearance
- Updated battery indicator
- All CSI commands now handled properly
- Updated dependencies

### Fixed

- Tab completion problem with folder paths
- DMG build in CI by skipping Finder customization

## [0.11.3] - 2025-11-23

### Added

- Extra ANSI escape sequences for better terminal compatibility

### Changed

- Improved PTY handling for better terminal integration

### Fixed

- Issue with 'less' command not rendering correctly

## [0.11.0] - 2025-11-22

### Added

- Battery indicator displaying real-time battery status in top bar
- Dracula theme added to available color themes
- GPM (General Purpose Mouse) support for Linux console

### Changed

- Framebuffer backend is now enabled by default for Linux builds
- Framebuffer dependencies are now Linux-specific (automatically disabled on macOS/Windows)
- Updated dependencies to latest versions

### Fixed

- Duplicate input issue on Windows
- Framebuffer large font rendering
- GPM mouse coordinate handling
- Various framebuffer setup and initialization improvements

## [0.10.0] - 2025-11-20

### Added

- Corner resize support for windows
- Window titles now display running process name
- Code modularization with dedicated splash and UI modules

### Changed

- Better terminal integration with improved PTY handling
- UI optimizations for smoother rendering
- ANSI escape sequence optimization
- Rust code optimizations for better performance
- Security optimizations

### Fixed

- Window resizing bug

## [0.9.0] - 2025-11-19

### Added

- Command launcher with fuzzy search and autocomplete Ctrl + space
- Mouse axis inversion flags for framebuffer mode (`--invert-mouse-x`, `--invert-mouse-y`)
- macOS installer packages (PKG and DMG)
- Package deployment automation

### Changed

- Enhanced window border rendering system with improved resizing
- Updated help system with new features
- Improved README documentation

### Fixed

- Mouse dragging behavior in xterm-compliant terminals
- Clippy doc overindented list items warning

## [0.8.5] - 2025-11-17

### Added

- Android/Termux support via optional clipboard feature
- Internal clipboard buffer fallback for platforms without system clipboard support

### Changed

- Clipboard support (arboard) is now optional via feature flag
- Default features include clipboard support for desktop platforms
- Build with `--no-default-features` for Android/Termux compatibility

### Fixed

- Android/Termux compilation errors due to unsupported arboard dependency

## [0.8.1] - 2025-11-17

### Fixed

- Dead code warnings in framebuffer backend (fb_renderer.rs, mouse_input.rs, render_backend.rs)

## [0.8.0] - 2025-11-16

### Added

- Save / Restore Sessions
- Shortcuts optimization

## [0.7.6] - 2025-11-15

### Added

- 32-bit Linux (i686) support for older hardware
- Official Windows binary releases

## [0.7.5] - 2025-11-14

### Added

- Framebuffer backend completion with enhanced mouse support and hardware cursor rendering
- MUSL libc support for Alpine and static linking

### Changed

- Optimized framebuffer rendering pipeline and mouse event handling

### Fixed

- Framebuffer mode stability and mouse cursor synchronization

## [0.7.1] - 2025-11-14

### Added

- **Linux Framebuffer Backend**: Direct `/dev/fb0` rendering with DOS text modes (40x25, 80x25, 80x43, 80x50), PSF2 fonts, VGA palette (`--features framebuffer-backend`, `--fb-mode=<MODE>`)
- Raw Linux mouse support (`/dev/input/mice`, `/dev/input/event*`) with hardware cursor rendering
- Dependencies: framebuffer 0.3, memmap2 0.9, flate2 1.0 (optional)

### Changed

- Enhanced theme system with improved color consistency and popup rendering
- Updated dependencies for modern Rust toolchain

### Fixed

- Clippy warnings for framebuffer-only features

## [0.6.6] - 2025-11-12

### Fixed

- Theme loading on startup (saved theme now properly loaded from config file)

## [0.6.5] - 2025-11-12

### Fixed

- Color theme selection persistence in settings window
- GPM mouse handling and error recovery

## [0.6.0] - 2025-11-12

### Added

- **Clipboard support**: Copy/paste via arboard, text selection with mouse drag, Ctrl+Shift+C/V shortcuts, right-click context menu
- Right-click context menu in terminal windows (copy, paste, close)
- Additional themes: Green Phosphor, Amber (plus existing Classic, Dark, Monochrome)
- Theme selection via `--theme` CLI option and settings window
- GPM (General Purpose Mouse) support for Linux console
- Enhanced 256-color palette and true color rendering
- Dependencies: arboard 3.4

### Changed

- Command-line help shows all available themes

### Fixed

- Clippy collapsible_if warnings

## [0.5.1] - 2025-11-11

### Added

- Distribution packages: .deb (Debian/Ubuntu), .rpm (Fedora/RHEL/openSUSE), AUR (Arch Linux)
- GitHub Actions workflow for automated package building
- cargo-deb and cargo-generate-rpm metadata

## [0.5.0] - 2025-11-11

### Added

- InfoWindow component for consistent dialog rendering
- Shadow rendering to config window

### Changed

- **Dark theme redesigned** with Dracula color scheme (purple/magenta/cyan accents)
- Standardized shadow rendering with shared `render_shadow()` helper
- Unfocused window title bars use DarkGrey for better visual hierarchy

### Fixed

- **Critical**: Window content now uses `window_content_fg` theme property instead of hardcoded white
- Dark theme visibility (shadow color, button contrast)
- Clippy warnings in config_window.rs (reduced function arguments)

## [0.4.0] - 2025-11-11

### Added

- Settings window (press 's'): auto-tiling toggle, show date toggle, config persistence to `~/.config/term39/config.toml`
- Configuration manager for user preferences

### Changed

- Enhanced help screen and bottom bar with settings shortcut

## [0.3.0] - 2025-11-10

### Added

- Window snapping to corners with auto-snap during drag

### Changed

- Updated README with splash screen ASCII art

### Fixed

- Resize render glitch for smoother window resizing

## [0.2.1] - 2025-11-10

### Fixed

- Clippy collapsible_if warnings in window_manager.rs

## [0.2.0] - 2025-11-10

### Added

- Scrollbar support: mouse wheel (3 lines/notch), thumb dragging, click-to-jump
- Configuration file support
- Version display with `--version` flag
- Full screen terminal (T key) and calendar widget (c key)
- GitHub Actions CI/CD and crates.io metadata

### Changed

- Enhanced help system organization

### Fixed

- Clippy warnings, test failures, error handling

## [0.1.0] - 2025-11-10

### Added

- Initial release with DOS aesthetic terminal multiplexer
- Window management: drag, resize, minimize/maximize, close, focus switching
- Mouse and keyboard controls (t, q, ESC, h, ALT+TAB)
- Double-buffered rendering (~60fps) with dirty region tracking
- ASCII compatibility mode (`--ascii` flag)
- Cross-platform support: macOS (ARM64/x86_64), Linux (x86_64/ARM64)
- Top bar with clock, bottom bar with window list
- Interactive help dialog and confirmation dialogs
- Dependencies: crossterm 0.29, chrono 0.4, portable-pty 0.8, vte 0.13

[0.14.0]: https://github.com/alejandroqh/term39/releases/tag/v0.14.0
[0.13.0]: https://github.com/alejandroqh/term39/releases/tag/v0.13.0
[0.12.0]: https://github.com/alejandroqh/term39/releases/tag/v0.12.0
[0.11.3]: https://github.com/alejandroqh/term39/releases/tag/v0.11.3
[0.11.0]: https://github.com/alejandroqh/term39/releases/tag/v0.11.0
[0.10.0]: https://github.com/alejandroqh/term39/releases/tag/v0.10.0
[0.9.0]: https://github.com/alejandroqh/term39/releases/tag/v0.9.0
[0.8.5]: https://github.com/alejandroqh/term39/releases/tag/v0.8.5
[0.8.1]: https://github.com/alejandroqh/term39/releases/tag/v0.8.1
[0.8.0]: https://github.com/alejandroqh/term39/releases/tag/v0.8.0
[0.7.6]: https://github.com/alejandroqh/term39/releases/tag/v0.7.6
[0.7.5]: https://github.com/alejandroqh/term39/releases/tag/v0.7.5
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
