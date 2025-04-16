#!/bin/sh
set -e

# Default settings
REPO="Tuurlijk/apisnip"
BINARY_NAME="apisnip"
LATEST_RELEASE="latest"

# Detect the architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    armv7l) ARCH="armv7" ;;
    armv6l) ARCH="arm" ;;
    i386|i686) ARCH="i386" ;;
    *) echo "Architecture $ARCH is not supported"; exit 1 ;;
esac

# Detect the OS
OS="$(uname -s)"
case "$OS" in
    Linux)     OS="linux" ;;
    Darwin)    OS="darwin" ;;
    MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
    *) echo "OS $OS is not supported"; exit 1 ;;
esac

# Do not continue if the OS is Windows
if [ "$OS" = "windows" ]; then
    echo "This script doesn't support Windows installation."
    echo "Please download the Windows binary from the releases page:"
    echo "https://github.com/$REPO/releases/latest"
    exit 1
fi

# Get the installation directory
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# Make sure the install directory is in PATH
if ! echo "$PATH" | tr ':' '\n' | grep -q "^$INSTALL_DIR$"; then
    echo "WARNING: '$INSTALL_DIR' is not in your PATH. The binary will be installed there anyway."
    if [ "$INSTALL_DIR" = "$HOME/.local/bin" ]; then
        echo "You can add it to your PATH by adding this line to your ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
fi

# Determine download URL
if [ "$OS" = "darwin" ] && [ "$ARCH" = "i386" ]; then
    echo "macOS doesn't support 32-bit binaries anymore. Aborting."
    exit 1
fi

# Create temp dir
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

# Get the latest release info
echo "Checking the latest version of $BINARY_NAME..."
DOWNLOAD_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/$LATEST_RELEASE" | 
               grep "browser_download_url.*$OS.*$ARCH" | 
               grep -v "sig\|sha" | 
               head -n 1 | 
               cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo "Could not find a release for your platform: $OS $ARCH"
    exit 1
fi

echo "Downloading from $DOWNLOAD_URL..."
FILENAME=$(basename "$DOWNLOAD_URL")
DOWNLOAD_PATH="$TMP_DIR/$FILENAME"

# Download the binary
curl -sL "$DOWNLOAD_URL" -o "$DOWNLOAD_PATH"

# Extract if it's an archive
if echo "$FILENAME" | grep -q "\.tar\.gz$"; then
    tar xzf "$DOWNLOAD_PATH" -C "$TMP_DIR"
    BINARY_PATH=$(find "$TMP_DIR" -type f -name "$BINARY_NAME" | head -n 1)
elif echo "$FILENAME" | grep -q "\.zip$"; then
    unzip -q "$DOWNLOAD_PATH" -d "$TMP_DIR"
    BINARY_PATH=$(find "$TMP_DIR" -type f -name "$BINARY_NAME" | head -n 1)
else
    BINARY_PATH="$DOWNLOAD_PATH"
fi

if [ ! -f "$BINARY_PATH" ]; then
    echo "Binary not found in the downloaded package"
    exit 1
fi

# Copy to destination
DEST_PATH="$INSTALL_DIR/$BINARY_NAME"
chmod +x "$BINARY_PATH"
cp "$BINARY_PATH" "$DEST_PATH"

echo "Installed $BINARY_NAME to $DEST_PATH"
echo "Run '$BINARY_NAME' to start using it" 