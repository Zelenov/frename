//! File snapshot: tags, file name without extension, extension, initial file name,
//! and optional segment markers stored as plain seconds (f32).
//!
//! `file_name()` serialises segments to `in_HH_MM_SS` / `out_HH_MM_SS`.
//! `FileSnapshot::parse()` is the inverse: parses a raw file-name string back into a snapshot.

use regex::Regex;
use std::sync::OnceLock;

use super::screenshot::Screenshot;

// ---------------------------------------------------------------------------
// Shared regex for segment markers
// ---------------------------------------------------------------------------

fn segment_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(in|out)_(\d{2})_(\d{2})_(\d{2})$").unwrap())
}

#[derive(Debug, Clone)]
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
    /// Screenshot markers for this file (positions loaded from sidecar `.snap.*.jpg` files).
    screenshots: Vec<Screenshot>,
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
            screenshots: Vec::new(),
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

    pub fn screenshots(&self) -> &[Screenshot] {
        &self.screenshots
    }
    pub fn set_screenshots(&mut self, screenshots: Vec<Screenshot>) {
        self.screenshots = screenshots;
    }
    /// Whether the comment and in/out points are still loading.
    pub fn comment_loading(&self) -> bool {
        self.comment_loading
    }
    pub fn set_comment_loading(&mut self, pending: bool) {
        self.comment_loading = pending;
    }
    pub fn add_screenshot(&mut self, screenshot: Screenshot) {
        if !self.screenshots.contains(&screenshot) {
            self.screenshots.push(screenshot);
            self.screenshots.sort();
        }
    }

    // ------------------------------------------------------------------
    // Serialise: snapshot → file name string
    // ------------------------------------------------------------------

    /// Build the full file name: `[tags.]name[.in_HH_MM_SS][.out_HH_MM_SS].ext`.
    pub fn file_name(&self) -> String {
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
            self.tags.join(".")
        } else {
            format!("{}.{}", self.tags.join("."), name_ext)
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

impl Default for FileSnapshot {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            name_without_extension: String::new(),
            extension: String::new(),
            initial_file_name: String::new(),
            segment_start: None,
            segment_end: None,
            comment: String::new(),
            screenshots: Vec::new(),
            comment_loading: false,
        }
    }
}
