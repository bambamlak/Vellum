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

# Update PATH logic
echo "🛠️  Checking your PATH configuration..."
UPDATED=false

if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    # Handle Bash and Zsh
    for config in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [ -f "$config" ]; then
            if ! grep -q ".local/bin" "$config"; then
                echo "" >> "$config"
                echo "# Added by Vellum Installer" >> "$config"
                echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$config"
                echo "📝 Added PATH to $(basename $config)"
                UPDATED=true
            fi
        fi
    done

    # Handle Fish shell (Fish uses a different syntax)
    if command -v fish &> /dev/null; then
        if ! fish -c "echo \$fish_user_paths" | grep -q "$HOME/.local/bin"; then
            fish -c "fish_add_path $HOME/.local/bin"
            echo "📝 Added PATH to Fish configuration"
            UPDATED=true
        fi
    fi
fi

if [ "$UPDATED" = true ]; then
    echo "🚀 PATH updated! Please restart your terminal or run: source ~/.bashrc (or equivalent)"
else
    echo "🚀 Vellum is ready! You can now run: vellum"
fi
