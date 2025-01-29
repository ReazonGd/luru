#!/bin/sh

set -e

# Deteksi OS dan arsitektur
OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" = "Linux" ]; then
  PLATFORM="linux"
# elif [ "$OS" = "Darwin" ]; then
#   PLATFORM="macos"
else
  echo "Unsupported OS: $OS"
  exit 1
fi

if [ "$ARCH" = "x86_64" ]; then
  ARCH="x86_64"
else
  echo "Unsupported architecture: $ARCH"
  exit 1
fi

# URL untuk biner
BINARY_URL="https://raw.githubusercontent.com/ReazonGd/luru/refs/heads/main/release/luru"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="luru"
TEMP_DIR="$(mktemp -d)/$BINARY_NAME"

# Unduh biner
echo "Downloading $BINARY_NAME from $BINARY_URL..."
curl -LsSf "$BINARY_URL" -o "$TEMP_DIR"

# Beri izin eksekusi
chmod +x "$TEMP_DIR"

# Pindahkan ke direktori instalasi
echo "Installing $BINARY_NAME to $INSTALL_DIR..."
sudo mv "$TEMP_DIR" "$INSTALL_DIR/$BINARY_NAME"

echo "Installation complete! You can now use $BINARY_NAME."
