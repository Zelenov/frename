//! Read-only SubRip (`.srt`) subtitles.
//!
//! A video's subtitles live next to it under the same stem: `/dir/clip.mp4` →
//! `/dir/clip.srt`. This follows how transcription tools write them, unlike the
//! `{filename}.comment.txt` sidecars frename creates itself.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// One subtitle cue: the text shown between `start` and `end`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleCue {
    pub start: Duration,
    pub end: Duration,
    /// Cue text; multi-line cues keep their line breaks.
    pub text: String,
}

/// All cues of one subtitle file, ordered by start time.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Subtitles {
    cues: Vec<SubtitleCue>,
}

impl Subtitles {
    /// Parse SRT text. Malformed blocks are skipped rather than failing the whole file,
    /// so a hand-edited file with one broken cue still shows everything else.
    pub fn parse(source: &str) -> Self {
        let source = source.strip_prefix('\u{feff}').unwrap_or(source);
        let normalized = source.replace("\r\n", "\n");
        let mut cues: Vec<SubtitleCue> = normalized.split("\n\n").filter_map(parse_block).collect();
        cues.sort_by_key(|cue| cue.start);
        Self { cues }
    }

    pub fn cues(&self) -> &[SubtitleCue] {
        &self.cues
    }

    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }

    /// Index of the cue shown at `position`, if any. Cue ends are exclusive.
    pub fn cue_index_at(&self, position: Duration) -> Option<usize> {
        // Cues are sorted by start, so the candidate is the last one starting at or before
        // `position`; an earlier cue can only still be showing if cues overlap, which SRT
        // files from transcription tools do not do.
        let index = self.last_started_index(position)?;
        (position < self.cues[index].end).then_some(index)
    }

    /// Index of the last cue that started at or before `position`, whether or not it is
    /// still on screen. Keeps the list's place through the gaps between cues.
    pub fn last_started_index(&self, position: Duration) -> Option<usize> {
        self.cues
            .partition_point(|cue| cue.start <= position)
            .checked_sub(1)
    }
}

/// How long a generated subtitle cue may get.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CueLength {
    /// One line of up to 100 characters and at most 8 s per cue.
    #[default]
    Short,
    /// One sentence per cue, however long.
    Sentence,
}

impl CueLength {
    /// Stable name for persisting the setting.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::Sentence => "sentence",
        }
    }

    /// Parse a persisted name; unknown names fall back to the default.
    pub fn from_name(name: &str) -> Self {
        match name {
            "sentence" => Self::Sentence,
            _ => Self::Short,
        }
    }
}

/// Languages offered as hints for generating subtitles: code sent to the service, and name.
pub const SUBTITLE_LANGUAGES: [(&str, &str); 6] = [
    ("en", "English"),
    ("ru", "Russian"),
    ("uk", "Ukrainian"),
    ("de", "German"),
    ("es", "Spanish"),
    ("fr", "French"),
];

/// Language hints checked until the user changes them.
pub const DEFAULT_SUBTITLE_LANGUAGES: [&str; 2] = ["en", "ru"];

/// Path of the subtitle file for a video: same directory and stem, `.srt` extension.
pub fn subtitle_path(video_path: &Path) -> PathBuf {
    video_path.with_extension("srt")
}

/// Path of the transcript saved next to a video when its subtitles were generated
/// (`clip.mp4` → `clip.soniox.json`). An empty one marks a video already found to have no
/// speech, so it is not paid for twice.
pub fn transcript_path(video_path: &Path) -> PathBuf {
    video_path.with_extension("soniox.json")
}

/// Load the subtitles next to `video_path`. `None` when there is no file, it cannot be
/// read, or it holds no valid cue.
pub fn load_subtitles(video_path: &Path) -> Option<Subtitles> {
    let path = subtitle_path(video_path);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            log::warn!("subtitles: failed to read {}: {e}", path.display());
            return None;
        }
    };
    let subtitles = Subtitles::parse(&String::from_utf8_lossy(&bytes));
    if subtitles.is_empty() {
        log::warn!("subtitles: no valid cues in {}", path.display());
        return None;
    }
    log::info!(
        "subtitles: loaded {} cues from {}",
        subtitles.cues.len(),
        path.display()
    );
    Some(subtitles)
}

/// Move the subtitle file and the saved transcript along when their video is renamed, so they
/// keep matching. Does nothing for a file that is not there; never overwrites an existing one.
pub fn rename_subtitle_file(old_video_path: &Path, new_video_path: &Path) {
    rename_companion(
        &subtitle_path(old_video_path),
        &subtitle_path(new_video_path),
    );
    rename_companion(
        &transcript_path(old_video_path),
        &transcript_path(new_video_path),
    );
}

fn rename_companion(old_path: &Path, new_path: &Path) {
    if old_path == new_path || !old_path.is_file() {
        return;
    }
    if new_path.exists() {
        log::warn!(
            "subtitles: not renaming {} — {} already exists",
            old_path.display(),
            new_path.display()
        );
        return;
    }
    match std::fs::rename(old_path, new_path) {
        Ok(()) => log::info!(
            "subtitles: renamed {} → {}",
            old_path.display(),
            new_path.display()
        ),
        Err(e) => log::warn!(
            "subtitles: failed to rename {} → {}: {e}",
            old_path.display(),
            new_path.display()
        ),
    }
}

/// Parse one blank-line-separated block: optional index line, timing line, text lines.
fn parse_block(block: &str) -> Option<SubtitleCue> {
    let mut lines = block
        .lines()
        .map(str::trim_end)
        .skip_while(|l| l.trim().is_empty());
    let mut timing = lines.next()?;
    if !timing.contains("-->") {
        // The first line was the cue number.
        timing = lines.next()?;
    }
    let (start, end) = timing.split_once("-->")?;
    let start = parse_timestamp(start)?;
    // Some files append positioning after the end time: `00:00:02,000 X1:...`.
    let end = parse_timestamp(end.split_whitespace().next()?)?;
    let text = lines.collect::<Vec<_>>().join("\n").trim().to_string();
    if text.is_empty() || end <= start {
        return None;
    }
    Some(SubtitleCue { start, end, text })
}

/// Parse `HH:MM:SS,mmm` (a `.` separator is accepted too).
fn parse_timestamp(value: &str) -> Option<Duration> {
    let value = value.trim();
    let (clock, millis) = value.split_once([',', '.']).unwrap_or((value, "0"));
    let mut parts = clock.split(':').map(|p| p.trim().parse::<u64>().ok());
    let hours = parts.next()??;
    let minutes = parts.next()??;
    let seconds = parts.next()??;
    if parts.next().is_some() {
        return None;
    }
    let millis: u64 = millis.trim().parse().ok()?;
    Some(Duration::from_millis(
        ((hours * 60 + minutes) * 60 + seconds) * 1000 + millis,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "1\r\n00:00:08,850 --> 00:00:11,730\r\nFirst line\r\nsecond line\r\n\r\n\
                          2\r\n00:00:12,810 --> 00:00:17,310\r\nSecond cue\r\n";

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[test]
    fn parses_crlf_file_with_multiline_cue() {
        let subs = Subtitles::parse(SAMPLE);
        assert_eq!(subs.cues().len(), 2);
        assert_eq!(subs.cues()[0].start, ms(8_850));
        assert_eq!(subs.cues()[0].end, ms(11_730));
        assert_eq!(subs.cues()[0].text, "First line\nsecond line");
        assert_eq!(subs.cues()[1].text, "Second cue");
    }

    #[test]
    fn strips_byte_order_mark() {
        let subs = Subtitles::parse(&format!("\u{feff}{SAMPLE}"));
        assert_eq!(subs.cues().len(), 2);
    }

    #[test]
    fn skips_malformed_block_and_keeps_the_rest() {
        let subs =
            Subtitles::parse("1\nnot a timing\nText\n\n2\n00:00:01,000 --> 00:00:02,000\nOk\n");
        assert_eq!(subs.cues().len(), 1);
        assert_eq!(subs.cues()[0].text, "Ok");
    }

    #[test]
    fn accepts_block_without_index_and_extra_blank_lines() {
        let subs = Subtitles::parse("\n\n00:00:01.500 --> 00:00:02.000\nNo index\n\n\n");
        assert_eq!(subs.cues().len(), 1);
        assert_eq!(subs.cues()[0].start, ms(1_500));
    }

    #[test]
    fn cue_index_at_finds_active_cue_and_gaps() {
        let subs = Subtitles::parse(SAMPLE);
        assert_eq!(subs.cue_index_at(ms(0)), None);
        assert_eq!(subs.cue_index_at(ms(8_850)), Some(0));
        assert_eq!(subs.cue_index_at(ms(11_730)), None);
        assert_eq!(subs.cue_index_at(ms(15_000)), Some(1));
        assert_eq!(subs.cue_index_at(ms(99_000)), None);
    }

    #[test]
    fn last_started_index_holds_through_gaps() {
        let subs = Subtitles::parse(SAMPLE);
        assert_eq!(subs.last_started_index(ms(0)), None);
        assert_eq!(subs.last_started_index(ms(9_000)), Some(0));
        assert_eq!(subs.last_started_index(ms(12_000)), Some(0));
        assert_eq!(subs.last_started_index(ms(99_000)), Some(1));
    }

    #[test]
    fn rename_moves_subtitles_with_the_video_and_never_overwrites() {
        let dir = std::env::temp_dir().join(format!("frename-subs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let old_video = dir.join("clip.MP4");
        let new_video = dir.join("tag.clip.MP4");
        std::fs::write(subtitle_path(&old_video), "old").expect("write srt");

        rename_subtitle_file(&old_video, &new_video);
        assert!(!subtitle_path(&old_video).exists());
        assert_eq!(
            std::fs::read_to_string(subtitle_path(&new_video))
                .ok()
                .as_deref(),
            Some("old")
        );

        // A subtitle file already at the destination is left alone.
        std::fs::write(subtitle_path(&old_video), "other").expect("write srt");
        rename_subtitle_file(&old_video, &new_video);
        assert_eq!(
            std::fs::read_to_string(subtitle_path(&new_video))
                .ok()
                .as_deref(),
            Some("old")
        );
        assert!(subtitle_path(&old_video).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_moves_the_saved_transcript_with_the_video() {
        let dir = std::env::temp_dir().join(format!("frename-marker-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let old_video = dir.join("clip.MP4");
        let new_video = dir.join("tag.clip.MP4");
        std::fs::write(transcript_path(&old_video), "{}").expect("write marker");

        rename_subtitle_file(&old_video, &new_video);
        assert!(!transcript_path(&old_video).exists());
        assert_eq!(
            transcript_path(&new_video),
            dir.join("tag.clip.soniox.json")
        );
        assert!(transcript_path(&new_video).is_file());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn subtitle_path_replaces_extension() {
        assert_eq!(
            subtitle_path(Path::new(r"C:\shoots\Villa.Story.DJI_0234_D.MP4")),
            PathBuf::from(r"C:\shoots\Villa.Story.DJI_0234_D.srt")
        );
    }
}
