mod display;
mod engine;
mod kitty;
mod lrc;
mod lrclib;
mod player;
mod render;

use crossterm::{
    cursor,
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode, KeyModifiers},
};
use display::DisplayState;
use lrc::{parse_lrc, plain_to_lrc};
use lrclib::LyricsResult;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

// No longer needed: ControlMsg enum

fn main() {
    let global_quit = Arc::new(AtomicBool::new(false));

    // Channel for Resize -> main
    let (resize_tx, resize_rx) = mpsc::channel::<()>();

    // Enter alternate screen
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide).ok();
    terminal::enable_raw_mode().ok();

    // Event-watcher thread (Key events + Resize)
    let quit_clone = global_quit.clone();
    thread::spawn(move || {
        loop {
            if let Ok(true) = event::poll(Duration::from_millis(100)) {
                match event::read() {
                    Ok(Event::Key(key)) => {
                        let should_quit = matches!(key.code, KeyCode::Char('q'))
                            || (key.code == KeyCode::Char('c')
                                && key.modifiers.contains(KeyModifiers::CONTROL));
                        if should_quit {
                            quit_clone.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                    Ok(Event::Resize(_, _)) => {
                        let _ = resize_tx.send(());
                    }
                    _ => {}
                }
            }
        }
    });

    let mut display = DisplayState::new();
    display.show_status("♪  Waiting for a player...");

    let mut last_track_id = String::new();
    let mut stop_signal = Arc::new(AtomicBool::new(false));

    'main: loop {
        if global_quit.load(Ordering::Relaxed) {
            break 'main;
        }

        // Check for resize signals
        while resize_rx.try_recv().is_ok() {
            display.refresh_size();
            // We don't force re-render here to avoid flickering, 
            // the next word will catch it.
        }

        match player::get_current_track() {
            Err(_) => {
                if last_track_id != "__none__" {
                    last_track_id = "__none__".to_string();
                    stop_signal.store(true, Ordering::Relaxed);
                    display.show_status("♪  Waiting for a player...");
                }
                interruptible_sleep(Duration::from_millis(1000), &global_quit);
                continue;
            }
            Ok(track) => {
                let track_id = track.track_id();

                if track_id == last_track_id {
                    interruptible_sleep(Duration::from_millis(500), &global_quit);
                    continue;
                }

                // New track — stop any running engine
                stop_signal.store(true, Ordering::Relaxed);
                thread::sleep(Duration::from_millis(50));

                last_track_id = track_id.clone();

                display.show_status(&format!(
                    "♪  Searching lyrics: {} — {}",
                    track.artist, track.title
                ));

                let lyrics_lines = match lrclib::fetch_lyrics(
                    &track.artist,
                    &track.title,
                    &track.clean_artist(),
                    &track.clean_title(),
                ) {
                    Err(e) => {
                        display.show_status(&format!("✗  {}", e));
                        interruptible_sleep(Duration::from_secs(3), &global_quit);
                        last_track_id.clear(); 
                        continue;
                    }
                    Ok(LyricsResult::Synced(raw)) => {
                        let lines = parse_lrc(&raw);
                        if lines.is_empty() {
                            display.show_status("✗  Failed to parse synced lyrics");
                            interruptible_sleep(Duration::from_secs(3), &global_quit);
                            last_track_id.clear();
                            continue;
                        }
                        lines
                    }
                    Ok(LyricsResult::Plain(plain)) => plain_to_lrc(&plain, track.length_secs),
                };

                if lyrics_lines.is_empty() {
                    display.show_status("✗  No lyrics available");
                    interruptible_sleep(Duration::from_secs(3), &global_quit);
                    last_track_id.clear();
                    continue;
                }

                if !player::is_playing() {
                    display.show_status("♪  Paused");
                    interruptible_sleep(Duration::from_millis(800), &global_quit);
                    last_track_id.clear(); // allow re-detect when played
                    continue;
                }

                let position = player::get_position();
                display.render_header(&track.artist, &track.title);

                let new_stop = Arc::new(AtomicBool::new(false));
                stop_signal = new_stop.clone();

                let watchdog_stop = new_stop.clone();
                let watchdog_quit = global_quit.clone();
                let saved_track_id = track_id.clone();

                thread::spawn(move || {
                    let mut last_check_time = std::time::Instant::now();
                    let mut last_check_pos = player::get_position();
                    
                    loop {
                        thread::sleep(Duration::from_millis(300)); // Faster check
                        if watchdog_stop.load(Ordering::Relaxed) || watchdog_quit.load(Ordering::Relaxed) {
                            break;
                        }
                        
                        let current_track = player::get_current_track();
                        let current_pos = player::get_position();
                        let now = std::time::Instant::now();
                        
                        // Check for track change
                        let track_changed = current_track
                            .map(|t| t.track_id() != saved_track_id)
                            .unwrap_or(true);
                            
                        // Check for play/pause state change
                        if !player::is_playing() {
                            watchdog_stop.store(true, Ordering::Relaxed);
                            break;
                        }
                            
                        // Check for seeking: if the gap between actual position 
                        // and expected position (last_pos + time_passed) is > 2.0s
                        let time_passed = now.duration_since(last_check_time).as_secs_f64();
                        let expected_pos = last_check_pos + time_passed;
                        let seek_detected = (current_pos - expected_pos).abs() > 2.0;

                        if track_changed || seek_detected {
                            watchdog_stop.store(true, Ordering::Relaxed);
                            break;
                        }
                        
                        last_check_time = now;
                        last_check_pos = current_pos;
                    }
                });

                engine::run_lyrics(
                    lyrics_lines, 
                    track.length_secs, 
                    position, 
                    new_stop, 
                    global_quit.clone(), 
                    &mut display
                );

                last_track_id.clear();
            }
        }
    }

    // Cleanup
    stop_signal.store(true, Ordering::Relaxed);
    terminal::disable_raw_mode().ok();
    execute!(io::stdout(), cursor::Show, LeaveAlternateScreen).ok();
}

fn interruptible_sleep(dur: Duration, quit: &Arc<AtomicBool>) {
    let steps = (dur.as_millis() / 50).max(1);
    for _ in 0..steps {
        thread::sleep(Duration::from_millis(50));
        if quit.load(Ordering::Relaxed) {
            break;
        }
    }
}
