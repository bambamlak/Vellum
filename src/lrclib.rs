use serde::Deserialize;
use std::error::Error;
use urlencoding::encode;

const LRCLIB_SEARCH: &str = "https://lrclib.net/api/search";

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Deserialize)]
struct LrclibSearchResponse {
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(rename = "trackName")]
    _track_name: String,
    #[serde(rename = "artistName")]
    _artist_name: String,
}

#[derive(Debug)]
pub enum LyricsResult {
    Synced(String),
    Plain(String),
}

/// Fetches lyrics from lrclib.net using the search API.
pub fn fetch_lyrics(
    artist: &str,
    title: &str,
    clean_artist: &str,
    clean_title: &str,
) -> Result<LyricsResult> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("vellum/0.1")
        .build()?;

    // Try multiple search queries if needed
    let queries = vec![
        format!("{} {}", artist, title),
        format!("{} {}", clean_artist, clean_title),
    ];

    for q in queries {
        let url = format!("{}?q={}", LRCLIB_SEARCH, encode(&q));
        
        let resp = client
            .get(&url)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !resp.status().is_success() {
            continue;
        }

        let results: Vec<LrclibSearchResponse> = resp
            .json()
            .map_err(|e| format!("Failed to parse lrclib search response: {}", e))?;

        // Find the first result with synced lyrics
        for res in results {
            if let Some(synced) = res.synced_lyrics {
                if !synced.trim().is_empty() {
                    return Ok(LyricsResult::Synced(synced));
                }
            }
        }
    }

    // If no synced lyrics found after all queries, try to just get the first plain one
    // (This is a fallback)
    for q in vec![format!("{} {}", artist, title), format!("{} {}", clean_artist, clean_title)] {
        let url = format!("{}?q={}", LRCLIB_SEARCH, encode(&q));
        if let Ok(resp) = client.get(&url).send() {
            if let Ok(results) = resp.json::<Vec<LrclibSearchResponse>>() {
                for res in results {
                    if let Some(plain) = res.plain_lyrics {
                        if !plain.trim().is_empty() {
                            return Ok(LyricsResult::Plain(plain));
                        }
                    }
                }
            }
        }
    }

    Err("No lyrics found in search results".into())
}
