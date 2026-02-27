use crate::display::DisplayState;
use crate::lrc::LrcLine;
use crate::player;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const MIN_WORD_MS: f64 = 100.0;
const MAX_WORD_MS: f64 = 3000.0;

struct WordEvent {
    word: String,
    show_at: f64, // seconds into song
}

pub fn run_lyrics(
    lines: Vec<LrcLine>,
    song_length_secs: f64,
    start_position: f64,
    stop_signal: Arc<AtomicBool>,
    global_quit: Arc<AtomicBool>,
    display: &mut DisplayState,
) {
    if lines.is_empty() {
        return;
    }

    // Pre-compute every word's show time across the whole song
    let mut events: Vec<WordEvent> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let next_ts = if i + 1 < lines.len() {
            lines[i + 1].timestamp_secs
        } else {
            song_length_secs.max(line.timestamp_secs + 5.0)
        };
        let line_dur = (next_ts - line.timestamp_secs).max(0.3);
        let words: Vec<&str> = line.text.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        let word_delay =
            (line_dur * 0.85 / words.len() as f64).clamp(MIN_WORD_MS / 1000.0, MAX_WORD_MS / 1000.0);

        for (j, word) in words.iter().enumerate() {
            // Clean the word: only keep alphanumeric characters
            let cleaned_word: String = word.chars().filter(|c| c.is_alphanumeric() || *c == '\'').collect();
            if cleaned_word.is_empty() {
                continue;
            }

            events.push(WordEvent {
                word: cleaned_word,
                show_at: line.timestamp_secs + j as f64 * word_delay,
            });
        }
    }

    // Find first event we still need to show
    let start_idx = events
        .iter()
        .position(|e| e.show_at > start_position.max(0.0))
        .unwrap_or(events.len());

    // Calibrate epoch: real-clock time corresponding to position=0 in the song
    let now_pos = player::get_position();
    let epoch = Instant::now() - Duration::from_secs_f64(now_pos.max(0.0));

    for event in &events[start_idx..] {
        if stop_signal.load(Ordering::Relaxed) || global_quit.load(Ordering::Relaxed) {
            return;
        }

        // Wait until this word's cue time
        let due = epoch + Duration::from_secs_f64(event.show_at);
        let now = Instant::now();
        if due > now {
            let mut remaining = due - now;
            while remaining > Duration::from_millis(10) {
                if stop_signal.load(Ordering::Relaxed) || global_quit.load(Ordering::Relaxed) {
                    return;
                }
                thread::sleep(remaining.min(Duration::from_millis(40)));
                let now2 = Instant::now();
                if due <= now2 {
                    break;
                }
                remaining = due - now2;
            }
        }

        if stop_signal.load(Ordering::Relaxed) || global_quit.load(Ordering::Relaxed) {
            return;
        }

        display.show_word(&event.word);
    }
}
