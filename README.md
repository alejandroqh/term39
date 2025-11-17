# TERM39

[![CI](https://github.com/alejandroqh/term39/actions/workflows/ci.yml/badge.svg)](https://github.com/alejandroqh/term39/actions/workflows/ci.yml)
[![Release](https://github.com/alejandroqh/term39/actions/workflows/release.yml/badge.svg)](https://github.com/alejandroqh/term39/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

A modern terminal multiplexer with classic MS-DOS aesthetic, built with Rust. Full-screen interface with window management and complete terminal emulation.

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

## Screenshots

<div align="center">
  <a href="assets/screenshot1.png">
    <img src="assets/screenshot1.png" width="700" alt="TERM39 Main Interface"/>
  </a>

  <br><br>

  <table>
    <tr>
      <td align="center">
        <a href="assets/screenshot2.png">
          <img src="assets/screenshot2.png" width="200" alt="TERM39 Screenshot 2"/>
        </a>
      </td>
      <td align="center">
        <a href="assets/screenshot3.png">
          <img src="assets/screenshot3.png" width="200" alt="TERM39 Screenshot 3"/>
        </a>
      </td>
    </tr>
    <tr>
      <td align="center">
        <a href="assets/screenshot4.png">
          <img src="assets/screenshot4.png" width="200" alt="TERM39 Screenshot 4"/>
        </a>
      </td>
      <td align="center">
        <a href="assets/screenshot5.png">
          <img src="assets/screenshot5.png" width="200" alt="TERM39 Screenshot 5"/>
        </a>
      </td>
    </tr>
  </table>

  <p><em>Click on any image to view full size</em></p>
</div>

## Features

- **Retro DOS Aesthetic**: Classic blue-and-white color scheme with box-drawing characters, ~60fps rendering
- **Multiple Terminal Windows**: Create, drag, resize, minimize, and maximize windows with mouse or keyboard
- **Window Management**: Automatic tiling, snap to corners, focus management with ALT+TAB
- **Clipboard Support**: System clipboard integration with drag-to-select, Ctrl+Shift+C/V, right-click menu
- **Customizable Themes**: Classic, Dark, Monochrome, Green Phosphor, Amber (via `--theme` flag)
- **Cross-Platform**: Linux, macOS, Windows with full VT100/ANSI support and true color
- **Linux Framebuffer Mode**: Direct `/dev/fb0` rendering with DOS text modes (40x25, 80x25, 80x43, 80x50), requires `--features framebuffer-backend`
- **ASCII Compatibility**: `--ascii` flag for maximum terminal compatibility

## Installation

### From crates.io
```bash
# Standard installation
cargo install term39

# Linux with framebuffer support
cargo install term39 --features framebuffer-backend
```
Requires Rust ([Install](https://rustup.rs/))

### Linux Packages
Download from [Releases](https://github.com/alejandroqh/term39/releases/latest):
- **Fedora/RHEL**:
  ```bash
  sudo rpm -i term39-*-1.x86_64.rpm
  # or: sudo dnf install term39-*-1.x86_64.rpm
  ```
- **Arch (AUR)**: `yay -S term39-bin` or `yay -S term39`
- **Tarball**:
  ```bash
  tar xzf term39-v*-linux-x86_64.tar.gz
  sudo mv term39 /usr/local/bin/
  ```

### macOS
```bash
# Intel (x86_64)
tar xzf term39-v*-macos-x86_64.tar.gz
sudo mv term39 /usr/local/bin/

# Apple Silicon (ARM64)
tar xzf term39-v*-macos-arm64.tar.gz
sudo mv term39 /usr/local/bin/
```

### Windows
Download from [Releases](https://github.com/alejandroqh/term39/releases/latest):
```powershell
# Option 1: Run the installer (x86_64)
.\term39-v*-windows-x86_64-pc-windows-msvc-installer.exe

# Option 2: Portable - Extract ZIP
Expand-Archive term39-v*-windows-x86_64.zip
# Add to PATH (optional)
$env:Path += ";$PWD\term39-v*-windows-x86_64"
```

### From Source
```bash
git clone https://github.com/alejandroqh/term39.git
cd term39
# Add --features framebuffer-backend for Linux framebuffer
cargo build --release
./target/release/term39
```

## Usage

```bash
./term39                 # Run with Unicode (recommended)
./term39 --ascii         # ASCII mode for compatibility
./term39 --theme dark    # Themes: classic, dark, monochrome,
                         #         green_phosphor, amber
```

### Keyboard Shortcuts

**General**
| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `t` / `T` | New window / Maximized window | `q` / `ESC` | Exit (desktop) |
| `F1` / `h` | Show help | `s` | Settings |
| `l` | License | `c` | Calendar |

**Window & Session**
| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `F2` / `ALT+TAB` | Switch windows | `F3` | Save session |
| `F4` / `Ctrl+L` | Clear terminal | | |

**Copy & Paste**
| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `F6` / `Ctrl+Shift+C` | Copy selection | `F7` / `Ctrl+Shift+V` | Paste |
| `Cmd+C` (macOS) | Copy selection | `Cmd+V` (macOS) | Paste |

**Dialog Controls**
| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `TAB` / Arrow keys | Navigate buttons | `ENTER` | Activate button |
| `ESC` | Close dialog | | |

### Mouse Controls
- **Title bar**: Drag to move | **[X]/[+]/[_]**: Close/Maximize/Minimize | **╬ handle**: Resize
- **Click window**: Focus | **Bottom bar**: Switch | **Drag/Right-click**: Select/Context menu

## Architecture

**Core**: Double-buffered video system, window manager (Z-order/focus), VT100/ANSI parser (VTE), PTY integration (portable-pty)
**Rendering**: Front/back buffers with dirty tracking, save/restore regions, shadow system for depth

## Development

**Prerequisites**: Rust 1.70+ ([Install](https://rustup.rs/))

```bash
cargo build --release                     # Build optimized binary
cargo run -- --ascii                      # Run in ASCII mode
cargo test && cargo clippy && cargo fmt   # Test, lint, format
```

## Contributing

Fork, create branch, test with `cargo test && cargo clippy`, commit, push, open PR. Follow Rust best practices, run `cargo fmt`, keep commits atomic.

## Dependencies

[crossterm](https://github.com/crossterm-rs/crossterm), [chrono](https://github.com/chronotope/chrono), [portable-pty](https://github.com/wez/wezterm/tree/main/pty), [vte](https://github.com/alacritty/vte), [arboard](https://github.com/1Password/arboard)

## License

MIT License - see [LICENSE](LICENSE) file. Copyright (c) 2025 Alejandro Quintanar

## Support

[Issues](https://github.com/alejandroqh/term39/issues) | [Discussions](https://github.com/alejandroqh/term39/discussions) | [Wiki](https://github.com/alejandroqh/term39/wiki)


