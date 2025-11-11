# TERM39

[![CI](https://github.com/alejandroqh/term39/actions/workflows/ci.yml/badge.svg)](https://github.com/alejandroqh/term39/actions/workflows/ci.yml)
[![Release](https://github.com/alejandroqh/term39/actions/workflows/release.yml/badge.svg)](https://github.com/alejandroqh/term39/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

A modern, retro-styled terminal multiplexer inspired by Norton Disk Doctor (MS-DOS), built with Rust. TERM39 brings the classic DOS aesthetic to your terminal with a full-screen text-based interface, window management, and terminal emulation.

```
 ███████████ ██████████ ███████████   ██████   ██████  ████████   ████████
░█░░░███░░░█░░███░░░░░█░░███░░░░░███ ░░██████ ██████  ███░░░░███ ███░░░░███
░   ░███  ░  ░███  █ ░  ░███    ░███  ░███░█████░███ ░░░    ░███░███   ░███
    ░███     ░██████    ░██████████   ░███░░███ ░███    ██████░ ░░█████████
    ░███     ░███░░█    ░███░░░░░███  ░███ ░░░  ░███   ░░░░░░███ ░░░░░░░███
    ░███     ░███ ░   █ ░███    ░███  ░███      ░███  ███   ░███ ███   ░███
    █████    ██████████ █████   █████ █████     █████░░████████ ░░████████
   ░░░░░    ░░░░░░░░░░ ░░░░░   ░░░░░ ░░░░░     ░░░░░  ░░░░░░░░   ░░░░░░░░
```

## Screenshot

![TERM39 in action](assets/screenshot.png)

## Features

- **Retro DOS Aesthetic**: Classic blue-and-white color scheme with box-drawing characters
- **Multiple Terminal Windows**: Create, manage, and switch between multiple terminal sessions
- **Window Management**: Drag, resize, minimize, and maximize windows with mouse or keyboard
- **Configuration System**: Persistent settings with auto-tiling and UI customization options
- **Double-Buffered Rendering**: Smooth, flicker-free display at ~60fps
- **Cross-Platform**: Works on Linux, macOS, and Windows
- **ASCII Compatibility Mode**: Optional `--ascii` flag for maximum terminal compatibility
- **Full Terminal Emulation**: Complete VT100/ANSI escape sequence support
- **Mouse & Keyboard Support**: Intuitive interface with both input methods

## Installation

### From crates.io (Recommended)

The easiest way to install TERM39 is via cargo:

```bash
cargo install term39
```

Requires Rust to be installed. [Install Rust](https://rustup.rs/)

### Linux Packages

#### Debian/Ubuntu (and derivatives)

Download and install the `.deb` package from the [Releases](https://github.com/alejandroqh/term39/releases/latest) page:

```bash
# x86_64 (amd64)
sudo dpkg -i term39_*_amd64.deb
# or
sudo apt install ./term39_*_amd64.deb

# ARM64 (aarch64)
sudo dpkg -i term39_*_arm64.deb
# or
sudo apt install ./term39_*_arm64.deb
```

#### Fedora/RHEL/CentOS/openSUSE

Download and install the `.rpm` package from the [Releases](https://github.com/alejandroqh/term39/releases/latest) page:

```bash
# x86_64
sudo rpm -i term39-*.x86_64.rpm
# or
sudo dnf install term39-*.x86_64.rpm

# ARM64 (aarch64)
sudo rpm -i term39-*.aarch64.rpm
# or
sudo dnf install term39-*.aarch64.rpm
```

#### Arch Linux (AUR)

Using an AUR helper (yay, paru, etc.):

```bash
# Binary package (recommended - faster)
yay -S term39-bin

# Source package (builds from source)
yay -S term39
```

Manual installation:

```bash
# Binary package
git clone https://aur.archlinux.org/term39-bin.git
cd term39-bin
makepkg -si

# Source package
git clone https://aur.archlinux.org/term39.git
cd term39
makepkg -si
```

### macOS

Download the binary for your architecture from the [Releases](https://github.com/alejandroqh/term39/releases/latest) page:

```bash
# Extract and install
tar xzf term39-*.tar.gz
sudo mv term39 /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/alejandroqh/term39.git
cd term39
cargo build --release
./target/release/term39
```

Requires Rust 1.70 or later. [Install Rust](https://rustup.rs/)

## Usage

### Basic Usage

```bash
# Run with Unicode characters (recommended)
./term39

# Run with ASCII-only characters for compatibility
./term39 --ascii
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `t` | Create new terminal window (from desktop) |
| `T` | Create new maximized terminal window (from desktop) |
| `q` or `ESC` | Exit application (from desktop) |
| `h` | Show help screen |
| `s` | Show settings/configuration window |
| `l` | Show license and about information |
| `c` | Show calendar |
| `ALT+TAB` | Switch between windows |

### Dialog Controls

| Key | Action |
|-----|--------|
| `TAB` or `LEFT`/`RIGHT` | Navigate between dialog buttons |
| `ENTER` | Activate selected button |
| `ESC` | Close dialog |

### Calendar Navigation (when calendar is open)

| Key | Action |
|-----|--------|
| `LEFT`/`RIGHT` or `<`/`>` or `,`/`.` | Navigate months |
| `UP`/`DOWN` | Navigate years |
| `t` or `HOME` | Jump to today |
| `ESC` | Close calendar |

### Mouse Controls

- **Click title bar** - Drag window to move
- **Click [X]** - Close window
- **Drag ╬ handle** - Resize window (bottom-right corner)
- **Click window** - Focus window
- **Click bottom bar** - Switch between windows

### Window Controls

Each window has three buttons in the title bar:
- **[X]** (red) - Close window
- **[+]** (green) - Maximize/restore window
- **[_]** (yellow) - Minimize window

## Architecture

### Core Components

- **Video Buffer System**: Double-buffered rendering with dirty region tracking
- **Window Manager**: Z-order management with focus handling
- **Terminal Emulator**: VT100/ANSI escape sequence parser using VTE
- **Charset Configuration**: Switchable Unicode/ASCII rendering modes
- **PTY Integration**: Real shell integration via portable-pty

### Rendering System

TERM39 uses a sophisticated double-buffer system:
- Front/back buffers for flicker-free rendering
- Per-cell dirty tracking (only updates changed cells)
- Save/restore regions for efficient window management
- Shadow system for 3D depth effect

## Building and Development

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Build Commands

```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# Run directly
cargo run

# Run with ASCII mode
cargo run -- --ascii

# Run tests
cargo test

# Check code quality
cargo clippy

# Format code
cargo fmt
```

### Project Structure

```
term39/
├── src/
│   ├── main.rs              # Entry point and event loop
│   ├── charset.rs           # Unicode/ASCII character sets
│   ├── video_buffer.rs      # Double-buffered rendering
│   ├── window.rs            # Window rendering and UI
│   ├── window_manager.rs    # Multi-window management
│   ├── terminal_emulator.rs # VT100/ANSI parser
│   ├── terminal_window.rs   # Terminal integration
│   ├── term_grid.rs         # Terminal cell grid
│   ├── ansi_handler.rs      # ANSI escape handling
│   ├── button.rs            # UI button component
│   └── prompt.rs            # Dialog/prompt system
├── Cargo.toml
├── LICENSE
└── README.md
```

## Contributing

Contributions are welcome! Whether you're fixing bugs, adding features, or improving documentation, your help is appreciated.

### How to Contribute

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes**: Follow the existing code style
4. **Test your changes**: `cargo test && cargo clippy`
5. **Commit your changes**: `git commit -m 'Add amazing feature'`
6. **Push to the branch**: `git push origin feature/amazing-feature`
7. **Open a Pull Request**

### Development Guidelines

- Follow Rust best practices and idioms
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Add tests for new functionality
- Update documentation as needed
- Keep commits atomic and well-described

### Areas for Contribution

- **Terminal emulation**: Improve ANSI/VT escape sequence coverage
- **Performance**: Optimize rendering and buffer management
- **Features**: Tab completion, session saving, themes, etc.
- **Platform support**: Testing and fixes for Windows/Linux/macOS
- **Documentation**: Tutorials, examples, code comments
- **Testing**: Unit tests, integration tests, edge cases

## Roadmap

- [x] Configuration file support (colors, keybindings)
- [ ] Custom themes
- [ ] Session persistence (save/restore windows)
- [ ] Tab completion
- [ ] Split panes within windows
- [x] Scrollback buffer
- [ ] Search functionality
- [ ] Copy/paste support

## Dependencies

- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [chrono](https://github.com/chronotope/chrono) - Date and time library
- [portable-pty](https://github.com/wez/wezterm/tree/main/pty) - Cross-platform PTY support
- [vte](https://github.com/alacritty/vte) - ANSI/VT parser

## Similar Projects

If you're interested in terminal multiplexers, check out:
- [tmux](https://github.com/tmux/tmux) - Terminal multiplexer
- [GNU Screen](https://www.gnu.org/software/screen/) - Terminal multiplexer
- [byobu](https://www.byobu.org/) - Text-based window manager
- [zellij](https://github.com/zellij-org/zellij) - Modern terminal workspace

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2025 Alejandro Quintanar

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.
```

## Acknowledgments

- Inspired by **Norton Disk Doctor** and classic DOS applications
- Built with the amazing [Rust](https://www.rust-lang.org/) programming language
- Thanks to the open source community for the excellent libraries

## Support

- **Issues**: [GitHub Issues](https://github.com/alejandroqh/term39/issues)
- **Discussions**: [GitHub Discussions](https://github.com/alejandroqh/term39/discussions)
- **Documentation**: Check the [Wiki](https://github.com/alejandroqh/term39/wiki)


