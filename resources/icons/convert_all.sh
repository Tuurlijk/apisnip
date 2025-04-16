#!/bin/bash
# Master script to convert the scissors.png to all required formats

echo "Converting icons for all platforms..."

# Run platform-specific conversion scripts
echo "Converting for Linux..."
./resources/icons/convert_for_linux.sh

echo "Converting for macOS..."
./resources/icons/convert_for_macos.sh

echo "Converting for Windows..."
./resources/icons/convert_for_windows.sh

echo "All icon conversions complete!" 