#!/bin/bash
# Comprehensive icon generation script for apisnip
# This script generates all icons needed for Linux, macOS, and Windows platforms

set -e  # Exit on error

echo "Starting icon generation for all platforms..."

# Source image - now relative to resources/
SOURCE="icons/png/apisnip.png"

if [ ! -f "$SOURCE" ]; then
    echo "Error: Source image $SOURCE not found!"
    exit 1
fi

# Make sure ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo "Error: ImageMagick is required but not installed."
    echo "Please install ImageMagick:"
    echo "  Ubuntu/Debian: sudo apt-get install imagemagick"
    echo "  macOS: brew install imagemagick"
    echo "  Windows: Install from https://imagemagick.org/script/download.php"
    exit 1
fi

# Make sure potrace is installed for SVG generation
if ! command -v potrace &> /dev/null; then
    echo "Warning: potrace is not installed. SVG icons will not be generated."
    echo "To install potrace:"
    echo "  Ubuntu/Debian: sudo apt-get install potrace"
    echo "  macOS: brew install potrace"
    HAS_POTRACE=false
else
    HAS_POTRACE=true
fi

# Create directories if they don't exist - all paths now relative to resources/
mkdir -p icons/linux/16x16
mkdir -p icons/linux/32x32
mkdir -p icons/linux/48x48
mkdir -p icons/linux/64x64
mkdir -p icons/linux/128x128
mkdir -p icons/linux/256x256

mkdir -p icons/macos
mkdir -p icons/windows

# Generate Linux icons
echo "Generating Linux icons..."
convert "$SOURCE" -resize 16x16 icons/linux/16x16/apisnip.png
convert "$SOURCE" -resize 32x32 icons/linux/32x32/apisnip.png
convert "$SOURCE" -resize 48x48 icons/linux/48x48/apisnip.png
convert "$SOURCE" -resize 64x64 icons/linux/64x64/apisnip.png
convert "$SOURCE" -resize 128x128 icons/linux/128x128/apisnip.png
convert "$SOURCE" -resize 256x256 icons/linux/256x256/apisnip.png

# Create SVG icon for Linux using potrace
if [ "$HAS_POTRACE" = true ]; then
    echo "Generating SVG icon for Linux using potrace..."
    # Create a high-quality PNG for tracing
    convert "$SOURCE" -resize 1024x1024 -flatten -negate icons/linux/apisnip_for_trace.pnm
    # Trace to SVG
    potrace icons/linux/apisnip_for_trace.pnm -s -o icons/linux/apisnip.svg
    # Clean up temp file
    rm icons/linux/apisnip_for_trace.pnm
else
    # For Linux SVG fallback - use the largest PNG
    echo "No potrace available, using PNG as fallback for scalable icon..."
    cp icons/linux/256x256/apisnip.png icons/linux/apisnip.png
fi

# Generate macOS icons
echo "Generating macOS icons..."
# Create temporary iconset directory
ICONSET="icons/macos/apisnip.iconset"
mkdir -p "$ICONSET"

# Generate different sizes
convert "$SOURCE" -resize 16x16 "$ICONSET/icon_16x16.png"
convert "$SOURCE" -resize 32x32 "$ICONSET/icon_16x16@2x.png"
convert "$SOURCE" -resize 32x32 "$ICONSET/icon_32x32.png"
convert "$SOURCE" -resize 64x64 "$ICONSET/icon_32x32@2x.png"
convert "$SOURCE" -resize 128x128 "$ICONSET/icon_128x128.png"
convert "$SOURCE" -resize 256x256 "$ICONSET/icon_128x128@2x.png"
convert "$SOURCE" -resize 256x256 "$ICONSET/icon_256x256.png"
convert "$SOURCE" -resize 512x512 "$ICONSET/icon_256x256@2x.png"
convert "$SOURCE" -resize 512x512 "$ICONSET/icon_512x512.png"
convert "$SOURCE" -resize 1024x1024 "$ICONSET/icon_512x512@2x.png"

# On macOS, create icns file
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Creating .icns file (macOS only)..."
    iconutil -c icns "$ICONSET" -o "icons/macos/apisnip.icns"
else
    echo "Skipping .icns generation (not on macOS)"
    # For non-macOS systems, just copy the largest icon
    cp "$ICONSET/icon_512x512@2x.png" "icons/macos/apisnip.png"
fi

# Generate Windows icons
echo "Generating Windows icons..."
convert "$SOURCE" -resize 16x16 icons/windows/apisnip-16.png
convert "$SOURCE" -resize 24x24 icons/windows/apisnip-24.png
convert "$SOURCE" -resize 32x32 icons/windows/apisnip-32.png
convert "$SOURCE" -resize 48x48 icons/windows/apisnip-48.png
convert "$SOURCE" -resize 64x64 icons/windows/apisnip-64.png
convert "$SOURCE" -resize 96x96 icons/windows/apisnip-96.png
convert "$SOURCE" -resize 128x128 icons/windows/apisnip-128.png
convert "$SOURCE" -resize 256x256 icons/windows/apisnip-256.png

# Create Windows .ico file with multiple sizes
echo "Creating .ico file for Windows..."
convert icons/windows/apisnip-16.png icons/windows/apisnip-24.png \
        icons/windows/apisnip-32.png icons/windows/apisnip-48.png \
        icons/windows/apisnip-64.png icons/windows/apisnip-96.png \
        icons/windows/apisnip-128.png icons/windows/apisnip-256.png \
        icons/windows/apisnip.ico

echo "Icon generation complete!"
echo "Icons are ready for all platforms in the icons directory."
echo "You should commit these generated icons to the repository." 