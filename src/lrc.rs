#[derive(Debug, Clone)]
pub struct LrcLine {
    pub timestamp_secs: f64,
    pub text: String,
}

/// Parse an LRC-format string into a sorted list of timestamped lines.
/// Handles both `[MM:SS.xx]` (centiseconds) and `[MM:SS.xxx]` (milliseconds).
/// Filters out empty/instrumental lines.
pub fn parse_lrc(lrc: &str) -> Vec<LrcLine> {
    let mut lines: Vec<LrcLine> = Vec::new();

    for raw in lrc.lines() {
        let trimmed = raw.trim();
        // Must start with '['
        if !trimmed.starts_with('[') {
            continue;
        }
        // Find closing ']'
        let close = match trimmed.find(']') {
            Some(i) => i,
            None => continue,
        };
        let timestamp_str = &trimmed[1..close];
        let text = trimmed[close + 1..].trim().to_string();

        // Skip empty lines
        if text.is_empty() {
            continue;
        }

        if let Some(ts) = parse_timestamp(timestamp_str) {
            lines.push(LrcLine {
                timestamp_secs: ts,
                text,
            });
        }
    }

    lines.sort_by(|a, b| a.timestamp_secs.partial_cmp(&b.timestamp_secs).unwrap());
    lines
}

/// Parse `MM:SS.xx` or `MM:SS.xxx` → seconds as f64
fn parse_timestamp(s: &str) -> Option<f64> {
    // Split on ':'
    let colon = s.find(':')?;
    let min_str = &s[..colon];
    let rest = &s[colon + 1..];

    // Split on '.'
    let dot = rest.find('.')?;
    let sec_str = &rest[..dot];
    let frac_str = &rest[dot + 1..];

    let minutes: f64 = min_str.parse().ok()?;
    let seconds: f64 = sec_str.parse().ok()?;
    let frac: f64 = if frac_str.len() == 3 {
        frac_str.parse::<f64>().ok()? / 1000.0
    } else {
        frac_str.parse::<f64>().ok()? / 100.0
    };

    Some(minutes * 60.0 + seconds + frac)
}

/// Convert plain lyrics to fake LRC lines spaced evenly over the song duration.
pub fn plain_to_lrc(plain: &str, duration_secs: f64) -> Vec<LrcLine> {
    let text_lines: Vec<&str> = plain
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if text_lines.is_empty() {
        return vec![];
    }

    let total = text_lines.len() as f64;
    let step = if duration_secs > 0.0 {
        duration_secs / total
    } else {
        3.5
    };

    text_lines
        .into_iter()
        .enumerate()
        .map(|(i, text)| LrcLine {
            timestamp_secs: i as f64 * step,
            text: text.to_string(),
        })
        .collect()
}
