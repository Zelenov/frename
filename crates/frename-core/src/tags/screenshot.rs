//! Screenshot marker: position inside a video where a frame was captured.
//! Image content is managed separately by `ProductionFileTagger`.

/// A single screenshot marker.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Screenshot {
    /// Position in the video, in milliseconds.
    pub position_ms: u64,
}

impl Screenshot {
    pub fn new(position_ms: u64) -> Self {
        Self { position_ms }
    }

    /// Position as seconds (for progress bar display).
    pub fn position_secs(&self) -> f32 {
        self.position_ms as f32 / 1000.0
    }

    /// Standard timecode string: `HH-MM-SS-mmm` (filename-safe, zero-padded).
    /// Example: 2440 ms → `00-00-02-440`.
    pub fn format_time(&self) -> String {
        let ms = self.position_ms;
        let ms_part = ms % 1000;
        let total_s = ms / 1000;
        let s_part = total_s % 60;
        let total_m = total_s / 60;
        let m_part = total_m % 60;
        let h_part = total_m / 60;
        format!("{:02}-{:02}-{:02}-{:03}", h_part, m_part, s_part, ms_part)
    }

    /// Parse a `HH-MM-SS-mmm` timecode string back to milliseconds.
    pub fn parse_time(s: &str) -> Option<u64> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 4 { return None; }
        let h: u64 = parts[0].parse().ok()?;
        let m: u64 = parts[1].parse().ok()?;
        let sec: u64 = parts[2].parse().ok()?;
        let ms: u64 = parts[3].parse().ok()?;
        Some((h * 3_600 + m * 60 + sec) * 1_000 + ms)
    }
}
