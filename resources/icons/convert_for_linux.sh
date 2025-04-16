#!/bin/bash
# Convert scissors.png to various Linux-friendly formats

mkdir -p resources/icons/linux

# Create XDG icons in various sizes
for size in 16 32 48 64 128 256; do
  mkdir -p resources/icons/linux/${size}x${size}
  convert resources/icons/png/scissors.png -resize ${size}x${size} resources/icons/linux/${size}x${size}/apisnip.png
done

# Create SVG for Linux
convert resources/icons/png/scissors.png resources/icons/linux/apisnip.svg

echo "Linux icon conversion complete" 