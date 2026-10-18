//! Read-only SubRip (`.srt`) and Advanced SubStation Alpha (`.ass` / `.ssa`) subtitles, for the
//! player's subtitle list.
//!
//! A video's subtitles live next to it under the same stem: `/dir/clip.mp4` →
//! `/dir/clip.srt`. This follows how transcription tools write them, unlike the
//! `{filename}.comment.txt` sidecars frename creates itself. Parsing and the file names are
//! clipscribe's (`clipscribe::srt`, `clipscribe::ass`), which describes clips from the same files.
//! A Premiere Pro transcript (`clip.premiere.json`, which Generate subtitles can write) is read
//! as subtitles too, by sonisub, laid out by the "short line" / "whole sentence" setting.
//! When a video has several, `.ass` wins, then `.ssa`, then `.premiere.json`, then `.srt`, unless
//! the editor picked one for that clip ([`SubtitleFormat`], kept by the app database); a file
//! that is empty or cannot be read gives way to the next. frename writes only `.srt` (and the
//! transcript when asked).

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

/// How large subtitles are drawn over and under the video.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SubtitleSize {
    Small,
    #[default]
    Medium,
    Large,
    ExtraLarge,
}

impl SubtitleSize {
    /// Every size, smallest first.
    pub const ALL: [Self; 4] = [Self::Small, Self::Medium, Self::Large, Self::ExtraLarge];

    /// Stable name for persisting the setting.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::ExtraLarge => "extra_large",
        }
    }

    /// Parse a persisted name; unknown names fall back to the default.
    pub fn from_name(name: &str) -> Self {
        match name {
            "small" => Self::Small,
            "large" => Self::Large,
            "extra_large" => Self::ExtraLarge,
            _ => Self::Medium,
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

/// The kinds of subtitle file a video may have next to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubtitleFormat {
    Ass,
    Ssa,
    /// A Premiere Pro transcript, `clip.premiere.json`.
    Premiere,
    Srt,
}

impl SubtitleFormat {
    /// Every format in the order they win when the editor has not picked one.
    pub const DEFAULT_ORDER: [Self; 4] = [Self::Ass, Self::Ssa, Self::Premiere, Self::Srt];

    /// The extension of the file (after the video's stem); also the name it is persisted under.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Ass => "ass",
            Self::Ssa => "ssa",
            Self::Premiere => PREMIERE_EXTENSION,
            Self::Srt => "srt",
        }
    }

    /// Parse a persisted name; `None` for one this version does not know.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::DEFAULT_ORDER
            .into_iter()
            .find(|format| format.extension() == name)
    }

    /// The file of this format next to `video_path`.
    pub fn path(self, video_path: &Path) -> PathBuf {
        video_path.with_extension(self.extension())
    }
}

/// The names a video's subtitles may have, in the order they win: `clip.ass`, `clip.ssa`,
/// `clip.premiere.json`, `clip.srt`.
pub fn subtitle_candidates(video_path: &Path) -> Vec<PathBuf> {
    SubtitleFormat::DEFAULT_ORDER
        .iter()
        .map(|format| format.path(video_path))
        .collect()
}

/// The first subtitle file next to `video_path` in the default order (`.ass`, `.ssa`, the
/// Premiere transcript, `.srt`), whether or not it can be read. `None` when there is none.
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

/// One subtitle file next to a video.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleFile {
    pub format: SubtitleFormat,
    pub path: PathBuf,
    /// False when the file is empty, holds no valid cue, or cannot be read.
    pub readable: bool,
}

/// What a clip shows: the subtitle files it has, which of them is shown, and its cues.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubtitleView {
    /// Every file that exists, in the default order.
    pub files: Vec<SubtitleFile>,
    /// The format shown; `None` when no file can be read.
    pub shown: Option<SubtitleFormat>,
    pub subtitles: Option<Subtitles>,
}

/// The formats to try, the editor's pick (when there is one) first, then the default order.
fn tried_in_order(chosen: Option<SubtitleFormat>) -> impl Iterator<Item = SubtitleFormat> {
    chosen.into_iter().chain(
        SubtitleFormat::DEFAULT_ORDER
            .into_iter()
            .filter(move |format| Some(*format) != chosen),
    )
}

/// The first file that can be read, in the order of [`tried_in_order`], with its cues laid out
/// by `length`. A file that is there but empty or unreadable is logged and passed over.
fn first_readable(
    video_path: &Path,
    chosen: Option<SubtitleFormat>,
    length: CueLength,
) -> Option<(SubtitleFormat, PathBuf, Vec<SubtitleCue>)> {
    tried_in_order(chosen).find_map(|format| {
        let path = format.path(video_path);
        if !path.is_file() {
            return None;
        }
        match read_cues(&path, length) {
            Ok(cues) if !cues.is_empty() => Some((format, path, cues)),
            Ok(_) => {
                log::warn!("subtitles: no valid cues in {}", path.display());
                None
            }
            Err(e) => {
                log::warn!("subtitles: failed to read {}: {e}", path.display());
                None
            }
        }
    })
}

/// Open the subtitles next to `video_path`: the file the editor `chosen` for it, or the first in
/// the default order; if that cannot be read, the next one.
pub fn open_subtitles(video_path: &Path, chosen: Option<SubtitleFormat>) -> SubtitleView {
    let shown = first_readable(video_path, chosen, cue_length());
    let files = SubtitleFormat::DEFAULT_ORDER
        .into_iter()
        .filter_map(|format| {
            let path = format.path(video_path);
            if !path.is_file() {
                return None;
            }
            let readable = match &shown {
                Some((shown_format, _, _)) if *shown_format == format => true,
                _ => read_cues(&path, cue_length()).is_ok_and(|cues| !cues.is_empty()),
            };
            Some(SubtitleFile {
                format,
                path,
                readable,
            })
        })
        .collect();
    let Some((format, path, cues)) = shown else {
        return SubtitleView {
            files,
            ..SubtitleView::default()
        };
    };
    log::info!(
        "subtitles: loaded {} cues from {}",
        cues.len(),
        path.display()
    );
    SubtitleView {
        files,
        shown: Some(format),
        subtitles: Some(Subtitles { cues }),
    }
}

/// The cues of the file `open_subtitles` shows for `video_path`; empty when there are none.
pub fn subtitle_cues(video_path: &Path, chosen: Option<SubtitleFormat>) -> Vec<SubtitleCue> {
    first_readable(video_path, chosen, cue_length()).map_or_else(Vec::new, |(_, _, cues)| cues)
}

/// About how many bytes of subtitles `video_path` has: the size of the `.srt` / `.ass` / `.ssa`
/// shown; for a Premiere transcript, whose JSON is many times the text, what the same cues would
/// take as an `.srt`. 0 when there are none.
pub fn subtitle_bytes(video_path: &Path, chosen: Option<SubtitleFormat>) -> usize {
    let Some((format, path, cues)) = first_readable(video_path, chosen, CueLength::Short) else {
        return 0;
    };
    if format == SubtitleFormat::Premiere {
        // The text, a timing line and the numbering of each cue.
        return cues.iter().map(|c| c.text.len() + 40).sum();
    }
    std::fs::metadata(path).map_or(0, |m| m.len() as usize)
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

    /// What the default order shows for `video`.
    fn load_subtitles(video: &Path) -> Option<Subtitles> {
        open_subtitles(video, None).subtitles
    }

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
        assert_eq!(subtitle_cues(&video, None).len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ass_beats_ssa_and_ssa_beats_srt() {
        let dir = temp_dir("order");
        let video = dir.join("clip.mp4");
        std::fs::write(subtitle_path(&video), SAMPLE).expect("srt");
        assert_eq!(existing_subtitle_path(&video), Some(subtitle_path(&video)));
        std::fs::write(dir.join("clip.ssa"), ASS.replace("Hello", "From ssa")).expect("ssa");
        assert_eq!(existing_subtitle_path(&video), Some(dir.join("clip.ssa")));
        std::fs::write(dir.join("clip.ass"), ASS.replace("Hello", "From ass")).expect("ass");
        assert_eq!(existing_subtitle_path(&video), Some(dir.join("clip.ass")));

        assert!(load_subtitles(&video).expect("ass").cues()[0]
            .text
            .starts_with("From ass"));
        let _ = std::fs::remove_file(dir.join("clip.ass"));
        assert!(load_subtitles(&video).expect("ssa").cues()[0]
            .text
            .starts_with("From ssa"));
        let _ = std::fs::remove_file(dir.join("clip.ssa"));
        assert_eq!(
            subtitle_cues(&video, None),
            Subtitles::parse(SAMPLE).cues(),
            "the .srt is shown when it is alone"
        );

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
    fn the_order_is_ass_then_ssa_then_the_transcript_then_srt() {
        let (dir, video) = premiere_file("order-all");
        std::fs::write(subtitle_path(&video), SAMPLE).expect("srt");
        let shown = |video: &Path| open_subtitles(video, None).shown;
        assert_eq!(shown(&video), Some(SubtitleFormat::Premiere));
        std::fs::write(dir.join("clip.ssa"), ASS.replace("Hello", "From ssa")).expect("ssa");
        assert_eq!(shown(&video), Some(SubtitleFormat::Ssa));
        std::fs::write(dir.join("clip.ass"), ASS).expect("ass");
        assert_eq!(shown(&video), Some(SubtitleFormat::Ass));
        let formats: Vec<_> = open_subtitles(&video, None)
            .files
            .iter()
            .map(|file| file.format)
            .collect();
        assert_eq!(formats, SubtitleFormat::DEFAULT_ORDER);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_picked_file_is_shown_instead_of_the_default() {
        let (dir, video) = premiere_file("picked");
        std::fs::write(subtitle_path(&video), SAMPLE).expect("srt");
        std::fs::write(dir.join("clip.ass"), ASS).expect("ass");

        let view = open_subtitles(&video, Some(SubtitleFormat::Srt));
        assert_eq!(view.shown, Some(SubtitleFormat::Srt));
        assert_eq!(view.files.len(), 3, "all three are offered");
        assert_eq!(view.subtitles, Some(Subtitles::parse(SAMPLE)));
        assert_eq!(
            subtitle_cues(&video, Some(SubtitleFormat::Srt)),
            Subtitles::parse(SAMPLE).cues(),
            "Describe with AI reads the picked file"
        );
        assert!(subtitle_bytes(&video, Some(SubtitleFormat::Srt)) > 0);
        assert_ne!(
            subtitle_bytes(&video, Some(SubtitleFormat::Srt)),
            subtitle_bytes(&video, None),
            "the default (.ass) is another file"
        );

        // A pick that is no longer there gives way to the default order.
        let _ = std::fs::remove_file(subtitle_path(&video));
        let view = open_subtitles(&video, Some(SubtitleFormat::Srt));
        assert_eq!(view.shown, Some(SubtitleFormat::Ass));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_file_that_is_empty_or_unreadable_gives_way_to_the_next_and_is_marked() {
        let dir = temp_dir("fallback");
        let video = dir.join("clip.mp4");
        std::fs::write(dir.join("clip.ass"), "").expect("empty ass");
        std::fs::write(dir.join("clip.ssa"), vec![0xff_u8; 8]).expect("garbage ssa");
        std::fs::write(subtitle_path(&video), SAMPLE).expect("srt");

        let view = open_subtitles(&video, None);
        assert_eq!(view.shown, Some(SubtitleFormat::Srt));
        let readable: Vec<_> = view
            .files
            .iter()
            .map(|file| (file.format, file.readable))
            .collect();
        assert_eq!(
            readable,
            [
                (SubtitleFormat::Ass, false),
                (SubtitleFormat::Ssa, false),
                (SubtitleFormat::Srt, true)
            ]
        );
        // The pick is empty too: the next in the default order is shown.
        let view = open_subtitles(&video, Some(SubtitleFormat::Ssa));
        assert_eq!(view.shown, Some(SubtitleFormat::Srt));
        assert_eq!(subtitle_cues(&video, None), Subtitles::parse(SAMPLE).cues());

        // Nothing readable: nothing is shown, and every file is marked.
        let _ = std::fs::remove_file(subtitle_path(&video));
        let view = open_subtitles(&video, None);
        assert_eq!((view.shown, view.subtitles), (None, None));
        assert!(view.files.iter().all(|file| !file.readable));
        assert_eq!(subtitle_bytes(&video, None), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_clip_with_one_file_or_none_offers_what_it_has() {
        let dir = temp_dir("offered");
        let video = dir.join("clip.mp4");
        assert!(open_subtitles(&video, None).files.is_empty());
        std::fs::write(subtitle_path(&video), SAMPLE).expect("srt");
        assert_eq!(open_subtitles(&video, None).files.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_format_round_trips_through_its_persisted_name() {
        for format in SubtitleFormat::DEFAULT_ORDER {
            assert_eq!(SubtitleFormat::from_name(format.extension()), Some(format));
        }
        assert_eq!(SubtitleFormat::from_name("vtt"), None);
    }

    #[test]
    fn something_that_is_not_a_transcript_is_ignored() {
        let dir = temp_dir("foreign-json");
        let video = dir.join("clip.mp4");
        for content in ["not json", "{\"tokens\": []}", "[1,2]", ""] {
            std::fs::write(dir.join("clip.premiere.json"), content).expect("write");
            assert!(load_subtitles(&video).is_none(), "{content}");
            assert!(subtitle_cues(&video, None).is_empty());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_transcript_is_renamed_with_its_video_and_sized_as_subtitles() {
        let (dir, video) = premiere_file("rename-premiere");
        let json_len = std::fs::metadata(dir.join("clip.premiere.json"))
            .expect("meta")
            .len() as usize;
        let bytes = subtitle_bytes(&video, None);
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
