# Building TERM39 with Framebuffer Backend for Debian

This guide explains how to build TERM39 with the experimental Linux framebuffer backend on Debian/Ubuntu systems.

## Prerequisites

On your Debian/Ubuntu system, install the required dependencies:

```bash
# Update package list
sudo apt update

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install build dependencies
sudo apt install -y \
    build-essential \
    pkg-config \
    libfontconfig1-dev \
    libfreetype6-dev \
    libx11-dev \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev
```

## Building from Source

### 1. Clone or Copy the Repository

```bash
cd /path/to/term39
```

### 2. Build Release Binary with Framebuffer Support

```bash
# Build with framebuffer backend feature
cargo build --release --features framebuffer-backend

# The binary will be at: ./target/release/term39
```

### 3. Build Debian Package (Optional)

```bash
# Install cargo-deb if not already installed
cargo install cargo-deb

# Build the .deb package with framebuffer support
cargo deb --features framebuffer-backend

# The .deb file will be in: ./target/debian/
# Install it with:
# sudo dpkg -i target/debian/term39_*.deb
```

## Running on Linux Console

The framebuffer backend requires:
- Running on a physical Linux console (TTY1-6)
- Access to `/dev/fb0` framebuffer device
- PSF2 fonts installed (usually in `/usr/share/kbd/consolefonts/`)

### Switch to Console

Press `Ctrl+Alt+F2` (or F3, F4, etc.) to switch to a text console (TTY).
Login with your username and password.

### Grant Framebuffer Access

You need permissions to access `/dev/fb0`:

**Option A: Run as root (quick test)**
```bash
sudo ./target/release/term39 --fb-mode=80x25
```

**Option B: Add user to video group (recommended)**
```bash
# Add your user to the video group
sudo usermod -a -G video $USER

# Log out and log back in for the group change to take effect
# Or use: newgrp video

# Now you can run without sudo:
./target/release/term39 --fb-mode=80x25
```

**Option C: Set permissions (temporary)**
```bash
sudo chmod 666 /dev/fb0  # Temporary, resets on reboot
./target/release/term39 --fb-mode=80x25
```

### Available Text Modes

```bash
# Standard DOS mode (default) - 80 columns x 25 rows
./target/release/term39 --fb-mode=80x25

# High density mode - 80 columns x 50 rows
./target/release/term39 --fb-mode=80x50

# Medium density mode - 80 columns x 43 rows
./target/release/term39 --fb-mode=80x43

# Wide character mode - 40 columns x 25 rows
./target/release/term39 --fb-mode=40x25
```

### Return to Graphical Environment

Press `Ctrl+Alt+F1` (or F7) to return to your graphical desktop.

## Troubleshooting

### "Failed to open /dev/fb0"

- Make sure you're running on a real console (TTY), not in a terminal emulator
- Check permissions: `ls -l /dev/fb0`
- Try running with sudo: `sudo ./target/release/term39 --fb-mode=80x25`

### "No suitable font found"

- Install console fonts: `sudo apt install kbd console-setup`
- Check fonts exist: `ls /usr/share/kbd/consolefonts/`

### Application automatically falls back to terminal mode

This is expected behavior if:
- Framebuffer cannot be initialized
- You're not on a Linux console (TTY)
- `/dev/fb0` is not accessible

The application will display an error message and fall back to the standard terminal backend.

## Testing the Framebuffer Backend

1. Build the binary with framebuffer support (see above)
2. Switch to console: `Ctrl+Alt+F2`
3. Login to your account
4. Run: `sudo ./target/release/term39 --fb-mode=80x25`
5. You should see pixel-perfect DOS-style rendering
6. Press 't' to create terminal windows
7. Press 'q' or ESC (from desktop) to exit
8. Return to GUI: `Ctrl+Alt+F1` or `Ctrl+Alt+F7`

## Notes

- The framebuffer backend is **experimental**
- Only works on Linux console (TTY) - no SSH, no terminal emulators
- Requires `/dev/fb0` access
- PSF2 fonts must be available
- Mouse support via GPM if installed and running
- Keyboard works the same as terminal mode

## Comparison: Terminal vs Framebuffer

| Feature | Terminal Backend | Framebuffer Backend |
|---------|-----------------|---------------------|
| Platform | Linux/macOS/Windows | Linux only |
| Access | Terminal emulators, SSH | Console (TTY) only |
| Rendering | Character-based | Pixel-based |
| Fonts | Terminal's font | PSF2 bitmap fonts |
| Resolution | Terminal size | Fixed text modes |
| Performance | Good | Excellent |
| Remote access | Yes | No |
| Permissions | User | Root or video group |

## Building Without Framebuffer Support

If you just want the standard cross-platform version:

```bash
# Build without framebuffer (default)
cargo build --release

# This binary works everywhere (terminal emulators, SSH, etc.)
./target/release/term39
```
