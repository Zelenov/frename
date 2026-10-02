//! File snapshot: tags, file name without extension, extension, initial file name,
//! and optional in/out points stored as plain seconds (f32).
//!
//! `file_name()` builds `[tags.]name.ext`; `FileSnapshot::parse()` is the inverse. The in/out
//! points are not part of the name: they live in the video or in the comment (see
//! `crate::metadata`). Names written by older versions still carry `in_HH_MM_SS` /
//! `out_HH_MM_SS` after the name; those parts are read as part of the name (wherever they
//! are), never as tags.

use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use crate::markers::Marker;
use crate::metadata::Segment;

// ---------------------------------------------------------------------------
// In/out parts of names written by older versions
// ---------------------------------------------------------------------------

fn name_in_out_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(in|out)_(\d{2})_(\d{2})_(\d{2})$").expect("in/out name part regex")
    })
}

/// Whether a dot-part of a name is an in or out point as older versions wrote it
/// (`in_00_01_05`, `out_00_02_10`, a valid time).
fn is_name_in_out_part(part: &str) -> bool {
    let Some(caps) = name_in_out_re().captures(part) else {
        return false;
    };
    let minutes_and_seconds_valid = [&caps[3], &caps[4]]
        .iter()
        .all(|v| v.parse::<u32>().is_ok_and(|v| v < 60));
    caps[2].parse::<u32>().is_ok() && minutes_and_seconds_valid
}

// Whether saved names put a space after the dot that ends each tag, process-wide like the
// metadata storage: names are built deep inside the tagger, far from the setting.
static SPACE_AFTER_TAGS: AtomicBool = AtomicBool::new(false);

/// Choose whether file names put a space after each tag: `Food. Goat. clip.mp4` instead of
/// `Food.Goat.clip.mp4`. Names are read the same way either way; files take the new form
/// when they are next saved.
pub fn set_space_after_tags(space: bool) {
    SPACE_AFTER_TAGS.store(space, Ordering::Relaxed);
}

/// The choice made with [`set_space_after_tags`]. Off by default.
pub fn space_after_tags() -> bool {
    SPACE_AFTER_TAGS.load(Ordering::Relaxed)
}

#[derive(Debug, Clone, Default)]
pub struct FileSnapshot {
    tags: Vec<String>,
    name_without_extension: String,
    extension: String,
    initial_file_name: String,
    /// Segment start in seconds, if set.
    segment_start: Option<f32>,
    /// Segment end in seconds, if set.
    segment_end: Option<f32>,
    /// Comment text for this file (loaded from sidecar `.comment.txt`). Empty = no comment.
    comment: String,
    /// Clip markers kept in the video's XMP. `None` when they were not read (a folder scan and
    /// a reparse after a save do not read them) or the file cannot hold them: saving such a
    /// snapshot leaves the file's markers as they are.
    markers: Option<Vec<Marker>>,
    /// How many clip markers the file holds, as its XMP (or the tag file's file list) told
    /// when it was parsed. [`Self::marker_count`] prefers `markers` when they were read.
    marker_count: usize,
    /// The file's XMP (comment and in/out points stored inside the video, marker count) is
    /// still loading: a folder scan defers that. Until it is loaded the snapshot does not know
    /// them, so saving it leaves them as they are in the file.
    comment_loading: bool,
}

impl FileSnapshot {
    pub fn new(
        tags: Vec<String>,
        name_without_extension: impl Into<String>,
        extension: impl Into<String>,
        initial_file_name: impl Into<String>,
    ) -> Self {
        Self {
            tags,
            name_without_extension: name_without_extension.into(),
            extension: extension.into(),
            initial_file_name: initial_file_name.into(),
            segment_start: None,
            segment_end: None,
            comment: String::new(),
            markers: None,
            marker_count: 0,
            comment_loading: false,
        }
    }

    // ------------------------------------------------------------------
    // Getters / setters
    // ------------------------------------------------------------------

    pub fn set_tags(&mut self, tags: impl IntoIterator<Item = impl AsRef<str>>) {
        self.tags = tags.into_iter().map(|s| s.as_ref().to_string()).collect();
    }
    pub fn tags(&self) -> &[String] {
        &self.tags
    }
    /// Whether `self` and `other` are the same clip once their tags are left out: the same name
    /// and the same extension, its case aside. A file renamed outside frename that only changed
    /// its tags is found again this way; two files that merely share a name, such as
    /// `pick.clip.mkv` and `review.clip.mp4`, are not.
    pub fn same_clip_without_tags(&self, other: &FileSnapshot) -> bool {
        self.name_without_extension() == other.name_without_extension()
            && self.extension().eq_ignore_ascii_case(other.extension())
    }
    pub fn name_without_extension(&self) -> &str {
        &self.name_without_extension
    }
    pub fn extension(&self) -> &str {
        &self.extension
    }
    pub fn initial_file_name(&self) -> &str {
        &self.initial_file_name
    }
    pub fn segment_start(&self) -> Option<f32> {
        self.segment_start
    }
    pub fn segment_end(&self) -> Option<f32> {
        self.segment_end
    }
    pub fn set_segment_start(&mut self, v: Option<f32>) {
        self.segment_start = v;
    }
    pub fn set_segment_end(&mut self, v: Option<f32>) {
        self.segment_end = v;
    }
    /// Both in/out points.
    pub fn segment(&self) -> Segment {
        Segment {
            start: self.segment_start,
            end: self.segment_end,
        }
    }
    pub fn set_segment(&mut self, segment: Segment) {
        self.segment_start = segment.start;
        self.segment_end = segment.end;
    }
    pub fn has_tag(&self, value: &str) -> bool {
        self.tags.iter().any(|t| t == value)
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
    pub fn set_comment(&mut self, comment: String) {
        self.comment = comment;
    }

    /// The clip markers, when they were read; see the field.
    pub fn markers(&self) -> Option<&[Marker]> {
        self.markers.as_deref()
    }
    pub fn set_markers(&mut self, markers: Option<Vec<Marker>>) {
        self.markers = markers;
    }
    /// How many clip markers the file holds: the markers read, else the count parsed.
    pub fn marker_count(&self) -> usize {
        self.markers.as_ref().map_or(self.marker_count, Vec::len)
    }
    pub fn set_marker_count(&mut self, count: usize) {
        self.marker_count = count;
    }
    /// Whether the comment and in/out points are still loading.
    pub fn comment_loading(&self) -> bool {
        self.comment_loading
    }
    pub fn set_comment_loading(&mut self, pending: bool) {
        self.comment_loading = pending;
    }

    // ------------------------------------------------------------------
    // Serialise: snapshot → file name string
    // ------------------------------------------------------------------

    /// Build the full file name: `[tags.]name.ext`, with a space after each tag's dot when
    /// [`space_after_tags`] is on. The in/out points are not part of it.
    pub fn file_name(&self) -> String {
        self.file_name_with(space_after_tags())
    }

    /// [`Self::file_name`] with the tag spacing given explicitly.
    pub fn file_name_with(&self, space_after_tags: bool) -> String {
        let tag_separator = if space_after_tags { ". " } else { "." };
        let name_ext = format!("{}{}", self.name_without_extension, self.extension);

        if self.tags.is_empty() {
            name_ext
        } else if name_ext.is_empty() {
            self.tags.join(tag_separator)
        } else {
            format!(
                "{}{}{}",
                self.tags.join(tag_separator),
                tag_separator,
                name_ext
            )
        }
    }

    // ------------------------------------------------------------------
    // Deserialise: file name string → snapshot
    // ------------------------------------------------------------------

    /// Parse a raw file-name string (not a full path) into a `FileSnapshot`: the dot-parts
    /// are tags, then the name, then the extension. In/out parts older versions wrote after
    /// the name (`clip.in_00_00_07.out_00_00_12.mp4`) stay part of the name, so they are
    /// neither tags nor in/out points and the name round-trips unchanged. Such a part placed
    /// among the tags by hand is not a tag either: it joins the name, right after it, so that
    /// file is renamed the next time it is saved.
    pub fn parse(raw_name: &str) -> Self {
        let parts: Vec<&str> = raw_name
            .split('.')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        let is_in_out = |part: &&str| is_name_in_out_part(part);
        // The last part is the extension, unless it is an old in/out part: a name without an
        // extension (`clip.in_00_00_07`) keeps it as part of the name.
        let (before, ext) = match parts.split_last() {
            None => (&parts[..], String::new()),
            Some((last, rest)) if !rest.is_empty() && !is_in_out(last) => {
                (rest, format!(".{last}"))
            }
            Some(_) => (&parts[..], String::new()),
        };
        // The name is the last part that is not an old in/out part, with every in/out part:
        // those among the tags first, then those after it.
        let (tags, name) = match before.iter().rposition(|part| !is_in_out(part)) {
            None => (vec![], before.join(".")),
            Some(name_at) => {
                let (tags, stray): (Vec<&str>, Vec<&str>) =
                    before[..name_at].iter().partition(|part| !is_in_out(part));
                let name: Vec<&str> = std::iter::once(before[name_at])
                    .chain(stray)
                    .chain(before[name_at + 1..].iter().copied())
                    .collect();
                (
                    tags.into_iter().map(str::to_string).collect(),
                    name.join("."),
                )
            }
        };

        FileSnapshot::new(tags, name, ext, raw_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_space_after_each_tag_is_written_on_request_and_read_either_way() {
        let mut snapshot = FileSnapshot::parse("Food.Goat.clip.mp4");
        assert_eq!(snapshot.file_name_with(true), "Food. Goat. clip.mp4");
        assert_eq!(snapshot.file_name_with(false), "Food.Goat.clip.mp4");

        let spaced = FileSnapshot::parse("Food. Goat. clip.mp4");
        assert_eq!(spaced.tags(), ["Food", "Goat"]);
        assert_eq!(spaced.name_without_extension(), "clip");

        snapshot.set_tags(Vec::<String>::new());
        assert_eq!(
            snapshot.file_name_with(true),
            "clip.mp4",
            "no tags, no space"
        );
    }

    #[test]
    fn in_out_points_are_never_written_into_the_name() {
        let mut snapshot = FileSnapshot::parse("Food.clip.mp4");
        snapshot.set_segment_start(Some(7.0));
        snapshot.set_segment_end(Some(12.0));
        assert_eq!(snapshot.file_name_with(false), "Food.clip.mp4");
    }

    #[test]
    fn old_in_out_parts_are_words_of_the_name_not_tags_or_points() {
        for name in [
            "Food.Goat.clip.in_00_00_07.out_00_01_12.mp4",
            "Food. Goat. clip.in_00_00_07.mp4",
            "clip.out_00_00_12.mov",
            "in_00_00_07.mp4",
        ] {
            let snapshot = FileSnapshot::parse(name);
            assert_eq!(
                (snapshot.segment_start(), snapshot.segment_end()),
                (None, None),
                "{name}"
            );
            assert!(
                snapshot.tags().iter().all(|t| !t.starts_with("in_")),
                "{name}"
            );
            assert_eq!(snapshot.file_name_with(name.contains(". ")), name);
        }
        let snapshot = FileSnapshot::parse("Food.Goat.clip.in_00_00_07.out_00_01_12.mp4");
        assert_eq!(snapshot.tags(), ["Food", "Goat"]);
        assert_eq!(
            snapshot.name_without_extension(),
            "clip.in_00_00_07.out_00_01_12"
        );
        // A part that is no valid time is an ordinary part, as before.
        let invalid = FileSnapshot::parse("Food.in_00_75_00.clip.mp4");
        assert_eq!(invalid.tags(), ["Food", "in_00_75_00"]);
    }

    #[test]
    fn an_old_in_out_part_at_the_end_of_a_name_without_extension_is_no_extension() {
        let snapshot = FileSnapshot::parse("Food.clip.in_00_00_07");
        assert_eq!(snapshot.tags(), ["Food"]);
        assert_eq!(snapshot.name_without_extension(), "clip.in_00_00_07");
        assert_eq!(snapshot.extension(), "");
        assert_eq!(snapshot.file_name_with(false), "Food.clip.in_00_00_07");
        // Other names keep their extension and tags as before.
        let plain = FileSnapshot::parse("Food.clip.mp4");
        assert_eq!(
            (
                plain.tags(),
                plain.name_without_extension(),
                plain.extension()
            ),
            (&["Food".to_string()][..], "clip", ".mp4")
        );
        assert_eq!(FileSnapshot::parse("clip").name_without_extension(), "clip");
    }

    #[test]
    fn old_in_out_parts_among_the_tags_join_the_name() {
        let snapshot = FileSnapshot::parse("in_00_00_07.Food.out_00_00_09.Goat.clip.mp4");
        assert_eq!(snapshot.tags(), ["Food", "Goat"]);
        assert_eq!(
            snapshot.name_without_extension(),
            "clip.in_00_00_07.out_00_00_09"
        );
        assert_eq!(
            (snapshot.segment_start(), snapshot.segment_end()),
            (None, None)
        );
    }
}
