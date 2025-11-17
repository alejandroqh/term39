#!/bin/bash
set -e

# Usage: ./build-macos-dmg.sh VERSION TARGET [SIGNING_IDENTITY]
# Example: ./build-macos-dmg.sh 0.8.1 aarch64-apple-darwin "Developer ID Application: Your Name"

VERSION=$1
TARGET=$2
SIGNING_IDENTITY="${3:-}"

if [ -z "$VERSION" ] || [ -z "$TARGET" ]; then
    echo "Usage: $0 VERSION TARGET [SIGNING_IDENTITY]"
    echo "Example: $0 0.8.1 aarch64-apple-darwin 'Developer ID Application: Your Name'"
    exit 1
fi

BINARY_PATH="target/${TARGET}/release/term39"
ICON_SOURCE="assets/term39.ico"

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
OUTPUT_DMG="${OUTPUT_DIR}/term39-v${VERSION}-macos-${ARCH}.dmg"
DMG_DIR="dmg-contents"
TEMP_DMG="${OUTPUT_DIR}/temp-term39-v${VERSION}-macos-${ARCH}.dmg"

echo "Building DMG installer for term39 v${VERSION} (${ARCH})..."

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    exit 1
fi

# Convert icon from .ico to .icns format
echo "Converting icon to macOS format..."
ICONSET_DIR="term39.iconset"
ICNS_FILE="term39.icns"

rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

# Extract PNG from ICO and create multiple sizes for iconset
sips -s format png "$ICON_SOURCE" --out "$ICONSET_DIR/icon_512x512.png" -z 512 512 2>/dev/null
sips -s format png "$ICON_SOURCE" --out "$ICONSET_DIR/icon_512x512@2x.png" -z 1024 1024 2>/dev/null
sips -s format png "$ICON_SOURCE" --out "$ICONSET_DIR/icon_256x256.png" -z 256 256 2>/dev/null
sips -s format png "$ICON_SOURCE" --out "$ICONSET_DIR/icon_256x256@2x.png" -z 512 512 2>/dev/null
sips -s format png "$ICON_SOURCE" --out "$ICONSET_DIR/icon_128x128.png" -z 128 128 2>/dev/null
sips -s format png "$ICON_SOURCE" --out "$ICONSET_DIR/icon_128x128@2x.png" -z 256 256 2>/dev/null
sips -s format png "$ICON_SOURCE" --out "$ICONSET_DIR/icon_32x32.png" -z 32 32 2>/dev/null
sips -s format png "$ICON_SOURCE" --out "$ICONSET_DIR/icon_32x32@2x.png" -z 64 64 2>/dev/null
sips -s format png "$ICON_SOURCE" --out "$ICONSET_DIR/icon_16x16.png" -z 16 16 2>/dev/null
sips -s format png "$ICON_SOURCE" --out "$ICONSET_DIR/icon_16x16@2x.png" -z 32 32 2>/dev/null

# Create .icns file
iconutil -c icns "$ICONSET_DIR" -o "$ICNS_FILE"
rm -rf "$ICONSET_DIR"

echo "Icon converted to: $ICNS_FILE"

# Create DMG contents directory
echo "Creating DMG contents..."
rm -rf "$DMG_DIR"
mkdir -p "$DMG_DIR"

# Copy binary
cp "$BINARY_PATH" "$DMG_DIR/"

# Create a temporary resource directory to hold the icon
RSRC_DIR="$DMG_DIR/.rsrc"
mkdir -p "$RSRC_DIR"
cp "$ICNS_FILE" "$RSRC_DIR/term39.icns"

# Apply icon to the binary using SetFile and custom icon resource
# First, create an icon resource file
cat > "$RSRC_DIR/icon.r" << 'EOF'
read 'icns' (-16455) "term39.icns";
EOF

# Compile the resource and attach to binary
Rez -append "$RSRC_DIR/icon.r" -o "$DMG_DIR/term39" 2>/dev/null || {
    echo "Warning: Could not attach icon with Rez, trying alternative method..."
}

# Set custom icon attribute
SetFile -a C "$DMG_DIR/term39" 2>/dev/null || {
    echo "Warning: Could not set custom icon attribute"
}

# Create Applications symlink for drag & drop installation
ln -s /Applications "$DMG_DIR/Applications"

# Create a temporary writable DMG
echo "Creating temporary DMG..."
rm -f "$TEMP_DMG"
hdiutil create -volname "term39 v${VERSION}" -srcfolder "$DMG_DIR" -ov -format UDRW "$TEMP_DMG"

# Mount the DMG to customize it
echo "Mounting DMG to customize layout..."
MOUNT_DIR=$(hdiutil attach -readwrite -noverify -noautoopen "$TEMP_DMG" | grep '/Volumes/' | sed 's/.*\/Volumes/\/Volumes/')

if [ -z "$MOUNT_DIR" ]; then
    echo "Error: Failed to mount DMG"
    exit 1
fi

# Wait for mount to complete
sleep 2

# Set Finder view options using AppleScript
echo "Setting Finder view options..."
osascript <<EOD || echo "Warning: Could not set all Finder options"
tell application "Finder"
    tell disk "term39 v${VERSION}"
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set bounds of container window to {100, 100, 600, 400}
        set theViewOptions to the icon view options of container window
        set arrangement of theViewOptions to not arranged
        set icon size of theViewOptions to 128
        try
            set position of item "term39" of container window to {125, 150}
        end try
        try
            set position of item "Applications" of container window to {375, 150}
        end try
        update without registering applications
        delay 1
    end tell
end tell
EOD

# Unmount the DMG
echo "Unmounting DMG..."
hdiutil detach "$MOUNT_DIR" -quiet || {
    echo "Warning: Could not unmount cleanly, forcing..."
    hdiutil detach "$MOUNT_DIR" -force -quiet
}

# Convert to compressed read-only DMG
echo "Compressing DMG..."
rm -f "$OUTPUT_DMG"
hdiutil convert "$TEMP_DMG" -format UDZO -o "$OUTPUT_DMG"

# If signing is requested, sign the DMG
if [ -n "$SIGNING_IDENTITY" ]; then
    echo "Signing DMG with: $SIGNING_IDENTITY"
    codesign --sign "$SIGNING_IDENTITY" "$OUTPUT_DMG" || {
        echo "Warning: Signing failed, keeping unsigned DMG"
    }
fi

# Cleanup
echo "Cleaning up..."
rm -f "$TEMP_DMG"
rm -rf "$DMG_DIR"
rm -f "$ICNS_FILE"

echo "Successfully created: $OUTPUT_DMG"
ls -lh "$OUTPUT_DMG"
