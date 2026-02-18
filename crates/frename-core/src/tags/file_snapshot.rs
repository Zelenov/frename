//! File snapshot: tags, file name without extension, extension, and initial file name.
//! Built by parsing a file name (split by dots, trim; last = extension, second-to-last = name, rest = tags).

/// Holds tags, file name without extension, extension, and initial file name. Built by FileTagger::parse or from UI state.
#[derive(Debug, Clone)]
pub struct FileSnapshot {
    /// Tags parsed or set (everything before name and extension when split by dots).
    tags: Vec<String>,
    /// File name without extension (second-to-last part when split by dots).
    name_without_extension: String,
    /// File extension (last part when split by dots).
    extension: String,
    /// Initial file name (as first parsed from the path, or copied from base when building from UI).
    initial_file_name: String,
}

impl FileSnapshot {
    /// Create from parsed parts: tags, name without extension, extension, and initial file name.
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
        }
    }

    /// Set the tags (e.g. from UI checked state).
    pub fn set_tags(&mut self, tags: impl IntoIterator<Item = impl AsRef<str>>) {
        self.tags = tags.into_iter().map(|s| s.as_ref().to_string()).collect();
    }

    /// Tags on this list (for display and for saving).
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    /// File name without extension (stem before extension).
    pub fn name_without_extension(&self) -> &str {
        &self.name_without_extension
    }

    /// File extension.
    pub fn extension(&self) -> &str {
        &self.extension
    }

    /// Initial file name (as first parsed from the path, or from base when built from UI).
    pub fn initial_file_name(&self) -> &str {
        &self.initial_file_name
    }

    /// Whether this list contains a tag with the given value.
    pub fn has_tag(&self, value: &str) -> bool {
        self.tags.iter().any(|t| t.as_str() == value)
    }

    /// Build the full file name: tags (if any) + name without extension + extension.
    /// Extension may include a leading dot (e.g. ".mp4"); no extra dot is added in that case.
    pub fn file_name(&self) -> String {
        let name_ext = if self.extension.is_empty() {
            self.name_without_extension.to_string()
        } else {
            format!("{}{}", self.name_without_extension, self.extension)
        };
        if self.tags.is_empty() {
            name_ext
        } else if name_ext.is_empty() {
            self.tags.join(".")
        } else {
            format!("{}.{}", self.tags.join("."), name_ext)
        }
    }
}

impl Default for FileSnapshot {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            name_without_extension: String::new(),
            extension: String::new(),
            initial_file_name: String::new(),
        }
    }
}
