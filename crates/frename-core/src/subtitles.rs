//! Read-only SubRip (`.srt`) and Advanced SubStation Alpha (`.ass` / `.ssa`) subtitles, for the
//! player's subtitle list.
//!
//! A video's subtitles live next to it under the same stem: `/dir/clip.mp4` →
//! `/dir/clip.srt`. This follows how transcription tools write them, unlike the
//! `{filename}.comment.txt` sidecars frename creates itself. Parsing and the file names are
//! clipscribe's (`clipscribe::srt`, `clipscribe::ass`), which describes clips from the same files.
//! A Premiere Pro transcript (`clip.premiere.json`, which Generate subtitles can write) is read
//! as subtitles too, by sonisub, laid out by the "short line" / "whole sentence" setting.
//! When a video has several, `.srt` wins, then `.ass`, then `.ssa`, then `.premiere.json`;
//! frename writes only `.srt` (and the transcript when asked).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

pub use clipscribe::srt::subtitle_path;

/// One subtitle cue: the text shown between `start` and `end`; multi-line cues keep their
/// line breaks.
pub type SubtitleCue = clipscribe::Cue;

/// All cues of one subtitle file, ordered by start time.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Subtitles {
    cues: Vec<SubtitleCue>,
}

impl Subtitles {
    /// Parse SRT text. Malformed blocks are skipped rather than failing the whole file,
    /// so a hand-edited file with one broken cue still shows everything else.
    pub fn parse(source: &str) -> Self {
        Self {
            cues: clipscribe::srt::parse(source),
        }
    }

    /// Parse ASS/SSA text (plain text of the `Dialogue:` lines, see `clipscribe::ass`).
    pub fn parse_ass(source: &str) -> Self {
        Self {
            cues: clipscribe::ass::parse(source),
        }
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

/// Language hints checked until the user changes them.
pub const DEFAULT_SUBTITLE_LANGUAGES: [&str; 2] = ["en", "ru"];

/// Path of the transcript saved next to a video when its subtitles were generated
/// (`clip.mp4` → `clip.soniox.json`). An empty one marks a video already found to have no
/// speech, so it is not paid for twice.
pub fn transcript_path(video_path: &Path) -> PathBuf {
    video_path.with_extension("soniox.json")
}

/// The extension of a Premiere Pro transcript written next to a video.
const PREMIERE_EXTENSION: &str = "premiere.json";

/// How long the lines of a transcript read back as subtitles may get, process-wide like the
/// space after tags: subtitles are loaded far from the setting.
static CUE_LENGTH: AtomicU8 = AtomicU8::new(0);

/// Choose the layout the subtitles read from a Premiere transcript get (a `.srt` is shown as
/// written). A clip already open keeps what it loaded.
pub fn set_cue_length(length: CueLength) {
    CUE_LENGTH.store(u8::from(length == CueLength::Sentence), Ordering::Relaxed);
}

/// The choice made with [`set_cue_length`]. Short lines by default.
pub fn cue_length() -> CueLength {
    if CUE_LENGTH.load(Ordering::Relaxed) == 1 {
        CueLength::Sentence
    } else {
        CueLength::Short
    }
}

/// The names a video's subtitles may have, in the order they win: `clip.srt`, `clip.ass`,
/// `clip.ssa`, `clip.premiere.json`.
pub fn subtitle_candidates(video_path: &Path) -> Vec<PathBuf> {
    std::iter::once(subtitle_path(video_path))
        .chain(
            clipscribe::ass::EXTENSIONS
                .iter()
                .map(|extension| video_path.with_extension(extension)),
        )
        .chain(std::iter::once(
            video_path.with_extension(PREMIERE_EXTENSION),
        ))
        .collect()
}

/// The subtitle file next to `video_path` that is shown: `.srt` if there is one, else `.ass`,
/// else `.ssa`, else the Premiere transcript.
pub fn existing_subtitle_path(video_path: &Path) -> Option<PathBuf> {
    subtitle_candidates(video_path)
        .into_iter()
        .find(|path| path.is_file())
}

/// Whether `path` (a subtitle file found by [`existing_subtitle_path`]) is ASS/SSA.
fn is_ass(path: &Path) -> bool {
    path.extension().is_some_and(|extension| {
        clipscribe::ass::EXTENSIONS
            .iter()
            .any(|ass| extension.eq_ignore_ascii_case(ass))
    })
}

/// Whether `path` is a Premiere Pro transcript (`clip.premiere.json`).
fn is_premiere(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(".premiere.json"))
}

/// The cues of the file at `path`, by its kind. A Premiere transcript is laid out by
/// `length`, with speaker names; one that is not a transcript (or has no words) gives none.
fn read_cues(path: &Path, length: CueLength) -> std::io::Result<Vec<SubtitleCue>> {
    let source = String::from_utf8_lossy(&std::fs::read(path)?).into_owned();
    Ok(if is_ass(path) {
        clipscribe::ass::parse(&source)
    } else if is_premiere(path) {
        let layout = match length {
            CueLength::Short => sonisub::srt::Layout::default(),
            CueLength::Sentence => sonisub::srt::Layout::unlimited(),
        };
        let layout = sonisub::srt::Layout {
            speaker_labels: true,
            ..layout
        };
        match sonisub::premiere::to_srt(&source, &layout) {
            Some((srt, _)) => clipscribe::srt::parse(&srt),
            None => {
                log::warn!("subtitles: {} is not a Premiere transcript", path.display());
                Vec::new()
            }
        }
    } else {
        clipscribe::srt::parse(&source)
    })
}

/// The cues of the subtitles next to `video_path` (`.srt` before `.ass` before `.ssa`); empty when
/// there are none. A file that exists but cannot be read is an error.
pub fn subtitle_cues(video_path: &Path) -> std::io::Result<Vec<SubtitleCue>> {
    match existing_subtitle_path(video_path) {
        None => Ok(Vec::new()),
        Some(path) => read_cues(&path, cue_length()),
    }
}

/// About how many bytes of subtitles `video_path` has: the size of its `.srt` / `.ass` / `.ssa`;
/// for a Premiere transcript, whose JSON is many times the text, what the same cues would take as
/// an `.srt`. 0 when there are none.
pub fn subtitle_bytes(video_path: &Path) -> usize {
    let Some(path) = existing_subtitle_path(video_path) else {
        return 0;
    };
    if is_premiere(&path) {
        // The text, a timing line and the numbering of each cue.
        return read_cues(&path, CueLength::Short)
            .map_or(0, |cues| cues.iter().map(|c| c.text.len() + 40).sum());
    }
    std::fs::metadata(path).map_or(0, |m| m.len() as usize)
}

/// Load the subtitles next to `video_path`. `None` when there is no file, it cannot be
/// read, or it holds no valid cue.
pub fn load_subtitles(video_path: &Path) -> Option<Subtitles> {
    let path = existing_subtitle_path(video_path)?;
    let cues = match read_cues(&path, cue_length()) {
        Ok(cues) => cues,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            log::warn!("subtitles: failed to read {}: {e}", path.display());
            return None;
        }
    };
    let subtitles = Subtitles { cues };
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
    for (old, new) in subtitle_candidates(old_video_path)
        .iter()
        .zip(subtitle_candidates(new_video_path).iter())
    {
        rename_companion(old, new);
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
        let dir = crate::test_support::fresh_dir("subs");
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
        let dir = crate::test_support::fresh_dir("marker");
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

    /// A fresh temp folder for one test.
    fn temp_dir(name: &str) -> PathBuf {
        crate::test_support::fresh_dir(name)
    }

    const ASS: &str = "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n\
        Dialogue: 0,0:00:01.00,0:00:02.50,Default,,0,0,0,,{\\an8}Hello\\Nthere\n";

    #[test]
    fn a_video_with_only_an_ass_shows_its_text() {
        let dir = temp_dir("ass-only");
        let video = dir.join("clip.mp4");
        assert!(load_subtitles(&video).is_none());
        std::fs::write(dir.join("clip.ass"), ASS).expect("write ass");

        let subs = load_subtitles(&video).expect("the .ass is read");
        assert_eq!(subs.cues().len(), 1);
        assert_eq!(subs.cues()[0].text, "Hello\nthere");
        assert_eq!(
            (subs.cues()[0].start, subs.cues()[0].end),
            (ms(1_000), ms(2_500))
        );
        assert_eq!(subtitle_cues(&video).expect("read").len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn srt_beats_ass_and_ass_beats_ssa() {
        let dir = temp_dir("order");
        let video = dir.join("clip.mp4");
        std::fs::write(dir.join("clip.ssa"), ASS.replace("Hello", "From ssa")).expect("ssa");
        assert_eq!(existing_subtitle_path(&video), Some(dir.join("clip.ssa")));
        std::fs::write(dir.join("clip.ass"), ASS.replace("Hello", "From ass")).expect("ass");
        assert_eq!(existing_subtitle_path(&video), Some(dir.join("clip.ass")));
        std::fs::write(subtitle_path(&video), SAMPLE).expect("srt");
        assert_eq!(existing_subtitle_path(&video), Some(subtitle_path(&video)));

        let cues = subtitle_cues(&video).expect("read");
        assert_eq!(cues, Subtitles::parse(SAMPLE).cues(), "the .srt is shown");
        let _ = std::fs::remove_file(subtitle_path(&video));
        assert!(load_subtitles(&video).expect("ass").cues()[0]
            .text
            .starts_with("From ass"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_moves_an_ass_and_an_ssa_with_the_video() {
        let dir = temp_dir("rename-ass");
        let old_video = dir.join("clip.MP4");
        let new_video = dir.join("tag.clip.MP4");
        std::fs::write(dir.join("clip.ass"), "a").expect("ass");
        std::fs::write(dir.join("clip.ssa"), "s").expect("ssa");

        rename_subtitle_file(&old_video, &new_video);
        assert!(!dir.join("clip.ass").exists() && !dir.join("clip.ssa").exists());
        assert_eq!(
            std::fs::read_to_string(dir.join("tag.clip.ass"))
                .ok()
                .as_deref(),
            Some("a")
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("tag.clip.ssa"))
                .ok()
                .as_deref(),
            Some("s")
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A Premiere transcript as sonisub writes it: two speakers, a long first sentence.
    fn transcript() -> String {
        let tok = |text: &str, start: u64, speaker: &str| {
            serde_json::json!({"text": text, "start_ms": start, "end_ms": start + 400, "confidence": 0.9,
                "language": "en", "speaker": speaker})
        };
        let mut tokens = Vec::new();
        for (i, word) in "Hello and welcome to the harbour where the boats come in every single evening before dark, \
and the gulls follow them home across the quiet water until the lights go out."
            .split(' ')
            .enumerate()
        {
            tokens.push(tok(&format!(" {word}"), 1_000 + i as u64 * 500, "1"));
        }
        tokens.push(tok(" Nice.", 20_000, "2"));
        let soniox = serde_json::json!({ "tokens": tokens });
        let layout = sonisub::srt::Layout::default();
        let names = sonisub::srt::Layout {
            speaker_names: vec!["Eugene".into(), "Sasha".into()],
            ..layout.clone()
        };
        serde_json::to_string_pretty(
            &sonisub::premiere::build(&soniox, &names, Some("en")).expect("a transcript"),
        )
        .expect("json")
    }

    fn premiere_file(name: &str) -> (PathBuf, PathBuf) {
        let dir = temp_dir(name);
        let video = dir.join("clip.mp4");
        std::fs::write(dir.join("clip.premiere.json"), transcript()).expect("write json");
        (dir, video)
    }

    #[test]
    fn a_video_with_only_a_premiere_transcript_shows_its_text_with_speaker_names() {
        let (dir, video) = premiere_file("premiere-only");
        assert_eq!(
            existing_subtitle_path(&video),
            Some(dir.join("clip.premiere.json"))
        );
        let path = dir.join("clip.premiere.json");
        let short = read_cues(&path, CueLength::Short).expect("read");
        assert!(short.len() > 2, "{short:?}");
        assert!(
            short[0].text.starts_with("Eugene: Hello and welcome"),
            "{short:?}"
        );
        assert!(short.iter().any(|c| c.text == "Sasha: Nice."), "{short:?}");
        assert_eq!(
            short
                .iter()
                .filter(|c| c.text.starts_with("Eugene:"))
                .count(),
            1,
            "a label only where the speaker changes"
        );
        assert!(
            short.iter().all(|c| c.text.chars().count() <= 110),
            "{short:?}"
        );

        let sentence = read_cues(&path, CueLength::Sentence).expect("read");
        assert_eq!(sentence.len(), 2, "one cue per sentence: {sentence:?}");
        assert!(sentence[0].text.ends_with("until the lights go out."));

        let subs = load_subtitles(&video).expect("shown");
        assert!(!subs.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_order_is_srt_then_ass_then_ssa_then_the_transcript() {
        let (dir, video) = premiere_file("order-all");
        std::fs::write(dir.join("clip.ssa"), ASS.replace("Hello", "From ssa")).expect("ssa");
        assert_eq!(existing_subtitle_path(&video), Some(dir.join("clip.ssa")));
        std::fs::write(dir.join("clip.ass"), ASS).expect("ass");
        assert_eq!(existing_subtitle_path(&video), Some(dir.join("clip.ass")));
        std::fs::write(subtitle_path(&video), SAMPLE).expect("srt");
        assert_eq!(existing_subtitle_path(&video), Some(subtitle_path(&video)));
        assert_eq!(
            subtitle_cues(&video).expect("read"),
            Subtitles::parse(SAMPLE).cues()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn something_that_is_not_a_transcript_is_ignored() {
        let dir = temp_dir("foreign-json");
        let video = dir.join("clip.mp4");
        for content in ["not json", "{\"tokens\": []}", "[1,2]", ""] {
            std::fs::write(dir.join("clip.premiere.json"), content).expect("write");
            assert!(load_subtitles(&video).is_none(), "{content}");
            assert!(subtitle_cues(&video).expect("no error").is_empty());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_transcript_is_renamed_with_its_video_and_sized_as_subtitles() {
        let (dir, video) = premiere_file("rename-premiere");
        let json_len = std::fs::metadata(dir.join("clip.premiere.json"))
            .expect("meta")
            .len() as usize;
        let bytes = subtitle_bytes(&video);
        assert!(
            bytes > 0 && bytes < json_len,
            "{bytes} vs the JSON's {json_len}"
        );

        rename_subtitle_file(&video, &dir.join("tag.clip.mp4"));
        assert!(!dir.join("clip.premiere.json").exists());
        assert!(dir.join("tag.clip.premiere.json").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
