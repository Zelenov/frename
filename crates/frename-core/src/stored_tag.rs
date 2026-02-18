//! Stored tag: a tag entry in the app store (database or fake). Has index and value; may gain more fields later.

/// A single stored tag in the store. Has an index (order) and value (label); structure may change later.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredTag {
    /// Display order (e.g. sort_order in DB).
    index: i64,
    /// Tag label (e.g. name in DB).
    value: String,
}

impl StoredTag {
    /// Create a stored tag with the given index and value.
    pub fn new(index: i64, value: impl Into<String>) -> Self {
        Self {
            index,
            value: value.into(),
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
}
