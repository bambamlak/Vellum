# Vellum 🎵

Vellum is a cinematic, word-by-word karaoke lyrics display for the **Kitty** terminal. It leverages the Kitty Graphics Protocol to render huge, beautifully typeset lyrics that synchronize perfectly with your music player.

## Features
- **Cinematic Rendering:** Renders lyrics as high-quality images using the Kitty terminal's graphics protocol.
- **Perfect Sync:** Automatically detects track changes, seeking (skipping forward/back), and play/pause states.
- **Smart Theming:** Integrates with **Matugen** system themes or falls back to high-contrast modes.
- **Clean Aesthetic:** Automatically strips symbols and noise, focusing purely on the words.
- **Dynamic Resizing:** Lyrics automatically scale to fit your terminal window, even if resized during playback.

## Requirements
- **Terminal:** [Kitty Terminal](https://sw.kovidgoyal.net/kitty/) (Required for graphics rendering).
- **Music Control:** `playerctl` (Used to sync with Spotify, MPV, VLC, etc.).
- **OS:** Linux.

## Installation

### The Quick Way (Recommended)
If you have Rust installed, just clone and run the installer:
```bash
git clone https://github.com/YOUR_USERNAME/vellum.git
cd vellum
sh install.sh
```

### The Rust Way
Install directly using cargo:
```bash
cargo install --path .
```
This will place the `vellum` binary in your `~/.cargo/bin/` directory.

## Usage
Simply play music in your favorite player and start Vellum. 
- Press `q` or `Ctrl+C` to quit.

## Credits
- Lyrics provided by [LRCLIB](https://lrclib.net/).
- Developed with Rust 🦀.
