use crate::kitty;
use crate::render::Renderer;
use crossterm::{
    cursor,
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};
use std::path::PathBuf;

pub struct DisplayState {
    pub renderer: Renderer,
    pixel_w: u32,
    pixel_h: u32,
    cell_rows: u32,
    header_rows: u16,
    pub text_color: (u8, u8, u8),
}

impl DisplayState {
    pub fn new() -> Self {
        let (pw, ph, _cols, rows) = kitty::get_pixel_size();
        let color = get_system_theme_color();
        DisplayState {
            renderer: Renderer::new(),
            pixel_w: pw,
            pixel_h: ph,
            cell_rows: rows,
            header_rows: 2,
            text_color: color,
        }
    }

    pub fn refresh_size(&mut self) {
        let (pw, ph, _cols, rows) = kitty::get_pixel_size();
        self.pixel_w = pw;
        self.pixel_h = ph;
        self.cell_rows = rows;
    }

    pub fn refresh_theme(&mut self) {
        self.text_color = get_system_theme_color();
    }

    /// Draw the artist/title header and clear the word area.
    pub fn render_header(&mut self, artist: &str, title: &str) {
        self.refresh_size();
        self.refresh_theme();
        kitty::clear_images();
        let mut stdout = io::stdout();
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::DarkGrey),
            SetAttribute(Attribute::Italic),
            Print(format!("  ♪  {} — {}", artist, title)),
            SetAttribute(Attribute::Reset),
            ResetColor,
        )
        .ok();
    }

    /// Display one big word via Kitty graphics protocol.
    pub fn show_word(&mut self, word: &str) {
        self.refresh_size();
        let cell_h = if self.cell_rows > 0 {
            self.pixel_h / self.cell_rows
        } else {
            20
        };
        let img_y_px = self.header_rows as u32 * cell_h;
        let img_h = self.pixel_h.saturating_sub(img_y_px);
        if img_h == 0 || self.pixel_w == 0 {
            return;
        }

        let pixels = self.renderer.render_word(word, self.pixel_w, img_h, self.text_color);

        kitty::clear_images();
        let mut stdout = io::stdout();
        execute!(stdout, cursor::MoveTo(0, self.header_rows)).ok();
        stdout.flush().ok();
        kitty::display_rgba(&pixels, self.pixel_w, img_h);
    }

    /// Show a centered status message.
    pub fn show_status(&mut self, msg: &str) {
        kitty::clear_images();
        let (_, rows) = terminal::size().unwrap_or((80, 24));
        let mut stdout = io::stdout();
        execute!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, rows / 2),
            SetForegroundColor(Color::DarkGrey),
            SetAttribute(Attribute::Italic),
            Print(format!("  {}", msg)),
            SetAttribute(Attribute::Reset),
            ResetColor,
        )
        .ok();
    }
}

impl Drop for DisplayState {
    fn drop(&mut self) {
        kitty::clear_images();
        execute!(io::stdout(), cursor::Show).ok();
    }
}

/// Tries to find a theme color from Matugen or fallbacks to Black/White based on dark/light mode.
fn get_system_theme_color() -> (u8, u8, u8) {
    // 1. Try Matugen (we check for common files)
    if let Some(c) = read_matugen_color() {
        return c;
    }

    // 2. Fallback: Auto-detect terminal background
    // Most users use dark themes. We can check for a few hints.
    let is_dark = std::env::var("COLORFGBG")
        .map(|s| {
            if let Some(pos) = s.find(';') {
                let bg = &s[pos + 1..];
                bg.parse::<u8>().unwrap_or(0) < 8 // 0-7 are usually dark colors
            } else {
                true
            }
        })
        .unwrap_or(true);

    if is_dark {
        (255, 255, 255) // White text for dark terminal
    } else {
        (0, 0, 0) // Black text for light terminal
    }
}

fn read_matugen_color() -> Option<(u8, u8, u8)> {
    let home = std::env::var("HOME").ok()?;
    
    // We'll check for a few possible generated files
    let possible_files = [
        format!("{}/.local/state/quickshell/user/generated/colors.json", home),
        format!("{}/.cache/matugen/colors.json", home),
    ];

    for path in possible_files {
        let p = PathBuf::from(path);
        if !p.exists() { continue; }
        
        if let Ok(content) = std::fs::read_to_string(p) {
            // Check for "primary" or "on_surface"
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // Matugen template files might have {{...}}, so we need to be careful.
                // If it's a real JSON with hex values:
                if let Some(hex) = json.get("primary").and_then(|v| v.as_str()) {
                    if let Some(rgb) = hex_to_rgb(hex) {
                        return Some(rgb);
                    }
                }
                if let Some(hex) = json.get("on_surface").and_then(|v| v.as_str()) {
                    if let Some(rgb) = hex_to_rgb(hex) {
                        return Some(rgb);
                    }
                }
            }
        }
    }
    None
}

fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}
