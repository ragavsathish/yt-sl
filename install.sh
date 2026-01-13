#!/bin/bash

# YouTube Video Slide Extractor - Installation Script
set -e

REPO="ragavsathish/yt-sl"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="yt-sl-extractor"

echo "🚀 Installing YouTube Video Slide Extractor..."

# Determine OS and Architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Check dependencies and suggest OS-specific installation commands
check_dep() {
    if ! command -v "$1" &> /dev/null; then
        echo "❌ Error: $1 is not installed."
        if [ "$OS" = "darwin" ]; then
            echo "   Install with: brew install $1"
        elif [ "$OS" = "linux" ]; then
            case "$1" in
                yt-dlp) echo "   Install with: sudo wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp && sudo chmod a+rx /usr/local/bin/yt-dlp" ;;
                ffmpeg) echo "   Install with: sudo apt update && sudo apt install ffmpeg" ;;
                tesseract) echo "   Install with: sudo apt update && sudo apt install tesseract-ocr" ;;
            esac
        fi
        exit 1
    fi
}

check_dep "yt-dlp"
check_dep "ffmpeg"
check_dep "tesseract"

case "$OS" in
    linux)
        if [ "$ARCH" = "x86_64" ]; then
            ASSET_NAME="yt-sl-extractor-linux-x86_64"
        elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
            ASSET_NAME="yt-sl-extractor-linux-arm64"
        else
            echo "❌ Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    darwin)
        if [ "$ARCH" = "x86_64" ]; then
            ASSET_NAME="yt-sl-extractor-macos-x86_64"
        elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
            ASSET_NAME="yt-sl-extractor-macos-arm64"
        else
            echo "❌ Unsupported architecture: $ARCH"
            exit 1
        fi
        ;;
    *)
        echo "❌ Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Detected $OS-$ARCH, searching for latest release..."

# Get latest release tag
LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo "⚠️  No releases found on GitHub. Building from source..."
    if ! command -v cargo &> /dev/null; then
        echo "❌ Error: cargo is not installed and no release binary available."
        exit 1
    fi
    cargo build --release --package yt-sl-extractor
    cp target/release/yt-sl-extractor "$BINARY_NAME"
else
    echo "Downloading $LATEST_TAG ($ASSET_NAME)..."
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/$ASSET_NAME"

    if curl -sL --fail -o "$BINARY_NAME" "$DOWNLOAD_URL"; then
        chmod +x "$BINARY_NAME"
    else
        echo "⚠️  Failed to download binary from $DOWNLOAD_URL"
        echo "   Falling back to building from source..."
        if command -v cargo &> /dev/null; then
            cargo build --release --package yt-sl-extractor
            cp target/release/yt-sl-extractor "$BINARY_NAME"
            chmod +x "$BINARY_NAME"
        else
            echo "❌ Error: cargo is not installed and download failed."
            exit 1
        fi
    fi
fi

# Install to destination
echo "Moving $BINARY_NAME to $INSTALL_DIR (may require sudo)..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$BINARY_NAME" "$INSTALL_DIR/"
else
    sudo mv "$BINARY_NAME" "$INSTALL_DIR/"
fi

echo "✅ Installation complete! Run '$BINARY_NAME --help' to get started."
