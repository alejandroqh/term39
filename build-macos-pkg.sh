#!/bin/bash
set -e

# Usage: ./build-macos-pkg.sh VERSION TARGET [SIGNING_IDENTITY]
# Example: ./build-macos-pkg.sh 0.8.1 aarch64-apple-darwin "Developer ID Installer: Your Name"

VERSION=$1
TARGET=$2
SIGNING_IDENTITY="${3:-}"

if [ -z "$VERSION" ] || [ -z "$TARGET" ]; then
    echo "Usage: $0 VERSION TARGET [SIGNING_IDENTITY]"
    echo "Example: $0 0.8.1 aarch64-apple-darwin 'Developer ID Installer: Your Name'"
    exit 1
fi

BINARY_PATH="target/${TARGET}/release/term39"

# Determine architecture name for output file
case "$TARGET" in
    aarch64-apple-darwin)
        ARCH="apple-silicon"
        ;;
    x86_64-apple-darwin)
        ARCH="intel"
        ;;
    *)
        echo "Unknown target: $TARGET"
        exit 1
        ;;
esac

OUTPUT_DIR="release"
OUTPUT_PKG="${OUTPUT_DIR}/term39-v${VERSION}-macos-${ARCH}.pkg"

echo "Building PKG installer for term39 v${VERSION} (${ARCH})..."

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    exit 1
fi

# Create payload structure
echo "Creating payload structure..."
mkdir -p pkg-payload/usr/local/bin
mkdir -p pkg-payload/usr/local/share/doc/term39

# Copy files
echo "Copying files..."
cp "$BINARY_PATH" pkg-payload/usr/local/bin/
cp README.md pkg-payload/usr/local/share/doc/term39/
cp LICENSE pkg-payload/usr/local/share/doc/term39/

# Set proper permissions
chmod 755 pkg-payload/usr/local/bin/term39
chmod 644 pkg-payload/usr/local/share/doc/term39/README.md
chmod 644 pkg-payload/usr/local/share/doc/term39/LICENSE

# Build package
echo "Building package..."
if [ -n "$SIGNING_IDENTITY" ]; then
    echo "Signing with: $SIGNING_IDENTITY"
    pkgbuild --root pkg-payload \
             --identifier com.alejandroqh.term39 \
             --version "$VERSION" \
             --install-location / \
             --sign "$SIGNING_IDENTITY" \
             "$OUTPUT_PKG"
else
    echo "Building unsigned package..."
    pkgbuild --root pkg-payload \
             --identifier com.alejandroqh.term39 \
             --version "$VERSION" \
             --install-location / \
             "$OUTPUT_PKG"
fi

# Cleanup
echo "Cleaning up..."
rm -rf pkg-payload

echo "Successfully created: $OUTPUT_PKG"
ls -lh "$OUTPUT_PKG"
