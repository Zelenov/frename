//! File snapshot: tags, file name without extension, extension, initial file name,
//! and optional segment markers stored as plain seconds (f32).
//!
//! `file_name()` serialises segments to `in_HH_MM_SS` / `out_HH_MM_SS`.
//! `FileSnapshot::parse()` is the inverse: parses a raw file-name string back into a snapshot.

use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use crate::markers::Marker;

// ---------------------------------------------------------------------------
// Shared regex for segment markers
// ---------------------------------------------------------------------------

fn segment_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(in|out)_(\d{2})_(\d{2})_(\d{2})$").unwrap())
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
    /// The comment and in/out points are stored inside the video and are still loading: a
    /// folder scan defers that. Until they are loaded the snapshot does not know them, so
    /// saving it leaves them as they are in the file.
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

    /// Build the full file name: `[tags.]name[.in_HH_MM_SS][.out_HH_MM_SS].ext`, with a space
    /// after each tag's dot when [`space_after_tags`] is on.
    pub fn file_name(&self) -> String {
        self.file_name_with(space_after_tags())
    }

    /// [`Self::file_name`] with the tag spacing given explicitly.
    pub fn file_name_with(&self, space_after_tags: bool) -> String {
        let tag_separator = if space_after_tags { ". " } else { "." };
        let mut middle: Vec<String> = Vec::new();
        if !self.name_without_extension.is_empty() {
            middle.push(self.name_without_extension.clone());
        }
        if let Some(s) = self.segment_start {
            middle.push(Self::secs_to_marker("in", s));
        }
        if let Some(e) = self.segment_end {
            middle.push(Self::secs_to_marker("out", e));
        }

        let name_ext = if middle.is_empty() {
            self.extension.clone()
        } else {
            let base = middle.join(".");
            if self.extension.is_empty() {
                base
            } else {
                format!("{}{}", base, self.extension)
            }
        };

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

    fn secs_to_marker(prefix: &str, secs: f32) -> String {
        let total = secs as u32;
        format!(
            "{prefix}_{:02}_{:02}_{:02}",
            total / 3600,
            (total % 3600) / 60,
            total % 60
        )
    }

    // ------------------------------------------------------------------
    // Deserialise: file name string → snapshot
    // ------------------------------------------------------------------

    /// Parse a raw file-name string (not a full path) into a `FileSnapshot`.
    /// Segment markers (`in_HH_MM_SS` / `out_HH_MM_SS`) are extracted and stored as seconds;
    /// the remaining dot-parts are split into tags / name / extension as usual.
    pub fn parse(raw_name: &str) -> Self {
        let all_parts: Vec<&str> = raw_name
            .split('.')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();

        let mut segment_start: Option<f32> = None;
        let mut segment_end: Option<f32> = None;

        let parts: Vec<&str> = all_parts
            .iter()
            .copied()
            .filter(|part| match segment_re().captures(part) {
                Some(caps) => {
                    let h: u32 = caps[2].parse().unwrap_or(0);
                    let m: u32 = caps[3].parse().unwrap_or(0);
                    let s: u32 = caps[4].parse().unwrap_or(0);
                    if m < 60 && s < 60 {
                        let secs = h as f32 * 3600.0 + m as f32 * 60.0 + s as f32;
                        if &caps[1] == "in" {
                            segment_start = Some(secs);
                        } else {
                            segment_end = Some(secs);
                        }
                        false // remove from parts
                    } else {
                        true // invalid — treat as regular part
                    }
                }
                None => true,
            })
            .collect();

        let (tags, name, ext) = match parts.as_slice() {
            [] => (vec![], String::new(), String::new()),
            [only] => (vec![], only.to_string(), String::new()),
            [name, ext] => (vec![], name.to_string(), format!(".{ext}")),
            [tags @ .., name, ext] => (
                tags.iter().map(|s| s.to_string()).collect(),
                name.to_string(),
                format!(".{ext}"),
            ),
        };

        let mut snap = FileSnapshot::new(tags, name, ext, raw_name);
        snap.set_segment_start(segment_start);
        snap.set_segment_end(segment_end);
        snap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_space_after_each_tag_is_written_on_request_and_read_either_way() {
        let mut snapshot = FileSnapshot::parse("Food.Goat.clip.in_00_00_07.mp4");
        assert_eq!(
            snapshot.file_name_with(true),
            "Food. Goat. clip.in_00_00_07.mp4"
        );
        assert_eq!(
            snapshot.file_name_with(false),
            "Food.Goat.clip.in_00_00_07.mp4"
        );

        let spaced = FileSnapshot::parse("Food. Goat. clip.in_00_00_07.mp4");
        assert_eq!(spaced.tags(), ["Food", "Goat"]);
        assert_eq!(spaced.name_without_extension(), "clip");
        assert_eq!(spaced.segment_start(), Some(7.0));

        snapshot.set_tags(Vec::<String>::new());
        assert_eq!(
            snapshot.file_name_with(true),
            "clip.in_00_00_07.mp4",
            "no tags, no space"
        );
    }
}
