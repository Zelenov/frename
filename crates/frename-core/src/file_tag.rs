//! File tag: simple tag value as it appears in or is saved to a file.
//! Stored tags (Tag in tags.rs) are the enriched set we keep in the system; FileTag is the plain value for parse/save.

/// A tag value as read from or written to a file (no extra metadata).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTag {
    value: String,
}

impl FileTag {
    /// Create a file tag with the given value.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Get the tag value.
    pub fn value(&self) -> &str {
        &self.value
    }
}
