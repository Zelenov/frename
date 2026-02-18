//! Stored tag: a tag entry in the app store (database or fake). Has index, value, and color index.

/// A single stored tag in the store. Has an index (order), value (label), and color index into the UI palette.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredTag {
    /// Display order (e.g. sort_order in DB).
    index: i64,
    /// Tag label (e.g. name in DB).
    value: String,
    /// Index into the app's tag color palette (0-based). Wrapped by modulo in the UI.
    color_index: u8,
}

impl StoredTag {
    /// Create a stored tag with the given index, value, and color index.
    pub fn new(index: i64, value: impl Into<String>, color_index: u8) -> Self {
        Self {
            index,
            value: value.into(),
            color_index,
        }
    }

    /// Order index for this tag.
    pub fn index(&self) -> i64 {
        self.index
    }

    /// Tag label.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Color palette index for this tag.
    pub fn color_index(&self) -> u8 {
        self.color_index
    }
}
