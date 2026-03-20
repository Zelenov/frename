//! Stored tag: a tag entry in the app store (database or fake). Identity is UUID + name; color is in TagColorMapping (by name).

use uuid::Uuid;

/// A single stored tag in the store. Has an id (UUID), value (label), sort order, and starred flag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredTag {
    /// Stable unique id (UUID).
    id: Uuid,
    /// Tag label (e.g. name in DB).
    value: String,
    /// Display/sort order (persisted as INTEGER in DB).
    sort_order: i64,
    /// Whether this tag is starred (pinned to the starred section). Persisted as INTEGER (0/1) in DB.
    starred: bool,
}

impl StoredTag {
    /// Create a stored tag with the given id and value. Sort order is 0, starred is false.
    pub fn new(id: Uuid, value: impl Into<String>) -> Self {
        Self::with_all(id, value, 0, false)
    }

    /// Create a stored tag with all fields.
    pub fn with_all(id: Uuid, value: impl Into<String>, sort_order: i64, starred: bool) -> Self {
        Self {
            id,
            value: value.into(),
            sort_order,
            starred,
        }
    }

    /// Tag id (UUID).
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Tag label.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sort order for display (persisted in DB).
    pub fn sort_order(&self) -> i64 {
        self.sort_order
    }

    /// Whether this tag is starred.
    pub fn starred(&self) -> bool {
        self.starred
    }
}
