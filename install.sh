#!/bin/bash

# Vellum Installer

set -e

echo "🎵 Installing Vellum..."

# Check for requirements
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed. Please install it from https://rustup.rs"
    exit 1
fi

if ! command -v playerctl &> /dev/null; then
    echo "Warning: 'playerctl' not found. Vellum needs it to sync lyrics."
fi

# Build
echo "📦 Building project (this might take a minute)..."
cargo build --release

# Install locally
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

echo "🚚 Moving binary to $INSTALL_DIR/vellum"
cp target/release/vellum "$INSTALL_DIR/vellum"

echo ""
echo "✅ Done! Vellum has been installed to $INSTALL_DIR"
echo "🚀 If that folder is in your PATH, you can now just run: vellum"
echo "   (If not, you might need to add 'export PATH=\$PATH:\$HOME/.local/bin' to your .bashrc or .zshrc)"
