# TERM39

[![CI](https://github.com/alejandroqh/term39/actions/workflows/ci.yml/badge.svg)](https://github.com/alejandroqh/term39/actions/workflows/ci.yml)
[![Release](https://github.com/alejandroqh/term39/actions/workflows/release.yml/badge.svg)](https://github.com/alejandroqh/term39/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

A modern terminal multiplexer with classic MS-DOS aesthetic, built with Rust. Full-screen interface with window management and complete terminal emulation.

<div align="center">
  <img src="assets/ascii_logo.png" alt="TERM39 Logo"/>
</div>

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
- **Customizable Themes**: Classic, Dark, Monochrome, Green Phosphor, Amber, Dracu, NDD, QBasic, TurboP, NCC, XT, WP, dB (via `--theme` flag)
- **Battery Indicator**: Real-time battery status display in the top bar
- **Cross-Platform**: Linux, macOS, Windows with full VT100/ANSI support and true color
- **Linux Framebuffer Mode**: Direct `/dev/fb0` rendering with DOS text modes (40x25, 80x25, ... , 320x200),
- **GPM Mouse Support**: Native mouse support on Linux console via GPM (General Purpose Mouse)
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

**Homebrew** (Recommended):
```bash
brew tap alejandroqh/term39
brew install term39
```

Or download from [Releases](https://github.com/alejandroqh/term39/releases/latest):

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

**Homebrew** (Recommended):
```bash
brew tap alejandroqh/term39
brew install term39
```

Or download from [Releases](https://github.com/alejandroqh/term39/releases/latest):

**Option 1: PKG Installer**

- Intel: `term39-v*-macos-intel.pkg`
- Apple Silicon: `term39-v*-macos-apple-silicon.pkg`

**Right-click** the PKG file and select **"Open"** to install (macOS will show a security warning for unsigned apps). The binary will be automatically placed in `/usr/local/bin/`.

**Option 2: DMG Installer**

- Intel: `term39-v*-macos-intel.dmg`
- Apple Silicon: `term39-v*-macos-apple-silicon.dmg`

**Right-click** the DMG file and select **"Open"**, then drag the app into the Applications folder.

**Option 3: Manual Installation**

```bash
# Intel (x86_64)
tar xzf term39-v*-macos-64bit-x86-binary.tar.gz
sudo mv term39 /usr/local/bin/

# Apple Silicon (ARM64)
tar xzf term39-v*-macos-64bit-arm-binary.tar.gz
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

### Android/Termux

For Android/Termux, install or build without the clipboard feature:

```bash
# Install Rust in Termux
pkg install rust

# Option 1: Install from crates.io (disable clipboard for Android compatibility)
cargo install term39 --no-default-features

# Option 2: Build from source
git clone https://github.com/alejandroqh/term39.git
cd term39
cargo build --release --no-default-features
./target/release/term39
```

**Note**: The `--no-default-features` flag disables system clipboard integration (which is not supported on Android). Copy/paste will still work within the app using an internal buffer.

## Usage

```bash
./term39                 # Run with Unicode (recommended)
./term39 --ascii         # ASCII mode for compatibility
./term39 --theme dark    # Themes: classic, dark, monochrome,
                         #         green_phosphor, amber, dracu,
                         #         ndd, qbasic, turbo, nc, xt,
                         #         wp, db
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

## Themes

<div align="center">
  <table>
    <tr>
      <td align="center">
        <a href="assets/themes/NDD.png">
          <img src="assets/themes/NDD.png" width="150" alt="NDD Theme"/>
        </a><br><sub>NDD</sub>
      </td>
      <td align="center">
        <a href="assets/themes/Qbasic.png">
          <img src="assets/themes/Qbasic.png" width="150" alt="QBasic Theme"/>
        </a><br><sub>QBasic</sub>
      </td>
      <td align="center">
        <a href="assets/themes/TurboP.png">
          <img src="assets/themes/TurboP.png" width="150" alt="Turbo Pascal Theme"/>
        </a><br><sub>Turbo Pascal</sub>
      </td>
    </tr>
    <tr>
      <td align="center">
        <a href="assets/themes/NCC.png">
          <img src="assets/themes/NCC.png" width="150" alt="Norton Commander Theme"/>
        </a><br><sub>Norton Commander</sub>
      </td>
      <td align="center">
        <a href="assets/themes/Dracu.png">
          <img src="assets/themes/Dracu.png" width="150" alt="Dracula Theme"/>
        </a><br><sub>Dracula</sub>
      </td>
      <td align="center">
        <a href="assets/themes/Dark.png">
          <img src="assets/themes/Dark.png" width="150" alt="Dark Theme"/>
        </a><br><sub>Dark</sub>
      </td>
    </tr>
    <tr>
      <td align="center">
        <a href="assets/themes/XT.png">
          <img src="assets/themes/XT.png" width="150" alt="XT Theme"/>
        </a><br><sub>XT</sub>
      </td>
      <td align="center">
        <a href="assets/themes/WP.png">
          <img src="assets/themes/WP.png" width="150" alt="WordPerfect Theme"/>
        </a><br><sub>WordPerfect</sub>
      </td>
      <td align="center">
        <a href="assets/themes/dB.png">
          <img src="assets/themes/dB.png" width="150" alt="dBase Theme"/>
        </a><br><sub>dBase</sub>
      </td>
    </tr>
  </table>
  <p><em>Click on any thumbnail to view full size</em></p>
</div>

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

**Core**: [crossterm](https://github.com/crossterm-rs/crossterm) (terminal I/O), [chrono](https://github.com/chronotope/chrono) (clock), [portable-pty](https://github.com/wez/wezterm/tree/main/pty) (PTY), [vte](https://github.com/alacritty/vte) (ANSI parser), [clap](https://github.com/clap-rs/clap) (CLI args), [serde](https://github.com/serde-rs/serde)/[toml](https://github.com/toml-rs/toml) (config)

**Optional**: [arboard](https://github.com/1Password/arboard) (clipboard, default), [framebuffer](https://github.com/royaltm/rust-framebuffer) (Linux FB mode)

## Cargo Features

### `clipboard` (Default: **ON**)
System clipboard integration with Ctrl+Shift+C/V.
- **Enable**: Desktop usage, copy/paste between apps
- **Disable**: Android/Termux, headless servers → `--no-default-features`

### `framebuffer-backend` (Default: **ON** for Linux)
Direct Linux framebuffer rendering with DOS text modes (40x25, 80x25, ... , 320x200).
- **Modes**: 40x25, 80x25, 80x43, 80x50, 160x50, 160x100, 320x100, 320x200
- **Platform**: Linux only (automatically disabled on macOS/Windows)
- **Disable**: To opt-out on Linux → `--no-default-features --features clipboard`
- **Requires**: `/dev/fb0` access (root or 'video' group), physical console only

```bash
# Build/Install
# Standard (includes framebuffer on Linux)
cargo build --release

# Without framebuffer
cargo build --release --no-default-features --features clipboard

## Minimal (no clipboard, no framebuffer)
cargo build --release --no-default-features

# Standard (includes framebuffer on Linux)
cargo install term39

# Without framebuffer
cargo install term39 --no-default-features --features clipboard

# Run framebuffer
sudo ./target/release/term39 -f --fb-mode=80x25
```

## License

MIT License - see [LICENSE](LICENSE) file.

## Support

[Issues](https://github.com/alejandroqh/term39/issues) | [Discussions](https://github.com/alejandroqh/term39/discussions) | [Wiki](https://github.com/alejandroqh/term39/wiki)
