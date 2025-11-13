#!/bin/bash
# Build script for TERM39 with framebuffer backend on Debian/Ubuntu

set -e

echo "========================================"
echo "TERM39 Framebuffer Backend Build Script"
echo "========================================"
echo ""

# Check if running on Linux
if [[ "$(uname -s)" != "Linux" ]]; then
    echo "ERROR: This script must be run on a Linux system"
    echo "Current OS: $(uname -s)"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Rust/Cargo is not installed"
    echo ""
    echo "Install Rust with:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  source \$HOME/.cargo/env"
    exit 1
fi

echo "Rust version:"
rustc --version
cargo --version
echo ""

# Check build dependencies
echo "Checking build dependencies..."
MISSING_DEPS=()

if ! dpkg -l | grep -q build-essential; then
    MISSING_DEPS+=("build-essential")
fi

if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
    echo "WARNING: Missing dependencies: ${MISSING_DEPS[*]}"
    echo "Install with: sudo apt install ${MISSING_DEPS[*]}"
    echo ""
fi

# Build
echo "Building TERM39 with framebuffer backend..."
echo "Command: cargo build --release --features framebuffer-backend"
echo ""

cargo build --release --features framebuffer-backend

echo ""
echo "========================================"
echo "Build completed successfully!"
echo "========================================"
echo ""
echo "Binary location: ./target/release/term39"
echo "Size: $(ls -lh target/release/term39 | awk '{print $5}')"
echo ""

# Check /dev/fb0 access
echo "Checking framebuffer access..."
if [ -e /dev/fb0 ]; then
    if [ -r /dev/fb0 ] && [ -w /dev/fb0 ]; then
        echo "✓ You have access to /dev/fb0"
    else
        echo "⚠ /dev/fb0 exists but you don't have access"
        echo ""
        echo "To run without sudo, add yourself to the video group:"
        echo "  sudo usermod -a -G video \$USER"
        echo "  (then log out and log back in)"
        echo ""
        echo "Or run with: sudo ./target/release/term39 --fb-mode=80x25"
    fi
else
    echo "⚠ /dev/fb0 not found"
    echo "  The framebuffer backend will not work"
    echo "  Make sure you're on a Linux console (TTY), not a terminal emulator"
fi

echo ""
echo "To test on Linux console (TTY):"
echo "  1. Press Ctrl+Alt+F2 to switch to console"
echo "  2. Login with your credentials"
echo "  3. cd $(pwd)"
echo "  4. sudo ./target/release/term39 --fb-mode=80x25"
echo "  5. Press Ctrl+Alt+F1 to return to GUI"
echo ""
echo "Available modes: 40x25, 80x25 (default), 80x43, 80x50"
echo ""
echo "See BUILD_FRAMEBUFFER.md for detailed instructions"
