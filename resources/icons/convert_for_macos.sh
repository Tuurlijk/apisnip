#!/bin/bash
# Convert scissors.png to macOS ICNS format

mkdir -p resources/icons/macos
mkdir -p apisnip.iconset

# Generate sizes required for Mac ICNS
convert resources/icons/png/scissors.png -resize 16x16 apisnip.iconset/icon_16x16.png
convert resources/icons/png/scissors.png -resize 32x32 apisnip.iconset/icon_16x16@2x.png
convert resources/icons/png/scissors.png -resize 32x32 apisnip.iconset/icon_32x32.png
convert resources/icons/png/scissors.png -resize 64x64 apisnip.iconset/icon_32x32@2x.png
convert resources/icons/png/scissors.png -resize 128x128 apisnip.iconset/icon_128x128.png
convert resources/icons/png/scissors.png -resize 256x256 apisnip.iconset/icon_128x128@2x.png
convert resources/icons/png/scissors.png -resize 256x256 apisnip.iconset/icon_256x256.png
convert resources/icons/png/scissors.png -resize 512x512 apisnip.iconset/icon_256x256@2x.png
convert resources/icons/png/scissors.png -resize 512x512 apisnip.iconset/icon_512x512.png
convert resources/icons/png/scissors.png -resize 1024x1024 apisnip.iconset/icon_512x512@2x.png

# On macOS, use iconutil
if [[ "$OSTYPE" == "darwin"* ]]; then
  iconutil -c icns apisnip.iconset -o resources/icons/macos/apisnip.icns
else
  # On Linux, use ImageMagick to create ICNS (not as good as iconutil, but works)
  convert apisnip.iconset/icon_16x16.png apisnip.iconset/icon_32x32.png \
    apisnip.iconset/icon_128x128.png apisnip.iconset/icon_256x256.png \
    apisnip.iconset/icon_512x512.png resources/icons/macos/apisnip.icns
fi

# Clean up
rm -rf apisnip.iconset

echo "macOS icon conversion complete" 