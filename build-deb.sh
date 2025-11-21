#!/bin/bash
set -e

# Build .deb packages for ARM64 and AMD64 using cargo-zigbuild

VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "Building term39 v$VERSION"

# Create release directory
mkdir -p release

# Build for x86_64 (AMD64)
echo "Building for x86_64-unknown-linux-gnu..."
cargo zigbuild --release --target x86_64-unknown-linux-gnu --features framebuffer-backend

# Build for aarch64 (ARM64)
echo "Building for aarch64-unknown-linux-gnu..."
cargo zigbuild --release --target aarch64-unknown-linux-gnu --features framebuffer-backend

# Create .deb for AMD64
echo "Creating .deb for amd64..."
cargo deb --no-build --target x86_64-unknown-linux-gnu -o "release/term39_${VERSION}_amd64.deb"

# Create .deb for ARM64
echo "Creating .deb for arm64..."
cargo deb --no-build --target aarch64-unknown-linux-gnu -o "release/term39_${VERSION}_arm64.deb"

echo ""
echo "Done! Packages created in release/:"
ls -lh release/*.deb
