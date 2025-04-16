#!/bin/bash
# Convert scissors.png to Windows ICO format

mkdir -p resources/icons/windows

# Create temporary directory for multi-size icons
mkdir -p resources/icons/tmp

# Generate different sizes
for size in 16 32 48 64 128 256; do
  convert resources/icons/png/scissors.png -resize ${size}x${size} resources/icons/tmp/scissors-${size}.png
done

# Create Windows ICO with multiple sizes
convert resources/icons/tmp/scissors-16.png resources/icons/tmp/scissors-32.png \
  resources/icons/tmp/scissors-48.png resources/icons/tmp/scissors-64.png \
  resources/icons/tmp/scissors-128.png resources/icons/tmp/scissors-256.png \
  resources/icons/windows/apisnip.ico

# Clean up temporary files
rm -rf resources/icons/tmp

echo "Windows icon conversion complete" 