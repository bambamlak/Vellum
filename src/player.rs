use std::error::Error;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub struct TrackInfo {
    pub artist: String,
    pub title: String,
    pub album: String,
    pub length_secs: f64,
}

impl TrackInfo {
    pub fn track_id(&self) -> String {
        format!("{}||{}", self.artist, self.title)
    }

    pub fn clean_title(&self) -> String {
        clean_string(&self.title)
    }

    pub fn clean_artist(&self) -> String {
        clean_string(&self.artist)
    }
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Fetch current track metadata using playerctl
pub fn get_current_track() -> Result<TrackInfo> {
    let artist = run_playerctl(&["metadata", "artist"])?;
    let title = run_playerctl(&["metadata", "title"])?;
    let album = run_playerctl(&["metadata", "album"]).unwrap_or_default();
    let length_str = run_playerctl(&["metadata", "mpris:length"]).unwrap_or_default();

    if artist.is_empty() || title.is_empty() {
        return Err("No track playing or metadata unavailable".into());
    }

    // mpris:length is in microseconds
    let length_secs = length_str
        .parse::<f64>()
        .map(|us| us / 1_000_000.0)
        .unwrap_or(0.0);

    Ok(TrackInfo {
        artist,
        title,
        album,
        length_secs,
    })
}

/// Get current playback position in seconds
pub fn get_position() -> f64 {
    run_playerctl(&["position"])
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// Check if a player is currently playing (not paused/stopped)
pub fn is_playing() -> bool {
    run_playerctl(&["status"])
        .map(|s| s == "Playing")
        .unwrap_or(false)
}

fn run_playerctl(args: &[&str]) -> Result<String> {
    let output = Command::new("playerctl")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run playerctl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("playerctl error: {}", stderr.trim()).into());
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 from playerctl: {}", e))?;

    Ok(stdout.trim().to_string())
}

/// Removes common noise from song/artist titles like (Remastered), [Explicit], feat. etc.
fn clean_string(s: &str) -> String {
    let mut cleaned = s.to_string();
    
    // Remove (feat. ...), (Explicit), [Remastered] etc.
    // Since we don't have regex, we'll do some basic split/strip logic
    let noise_indicators = ["(", "[", " - "];
    
    for indicator in noise_indicators {
        if let Some(pos) = cleaned.find(indicator) {
            let part = &cleaned[pos..].to_lowercase();
            if part.contains("feat") || 
               part.contains("explicit") || 
               part.contains("remaster") || 
               part.contains("live") ||
               part.contains("version") ||
               part.contains("digital") ||
               part.contains("deluxe") ||
               part.contains("20") { // e.g. 2024 Remaster
                cleaned = cleaned[..pos].trim().to_string();
            }
        }
    }
    
    cleaned.trim().to_string()
}
