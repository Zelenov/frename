//! Stored tag: a tag entry in the app store (database or fake). Identity is UUID + name; color is in TagColorMapping (by name).

use uuid::Uuid;

/// A single stored tag in the store. Has an id (UUID), value (label), and sort order. Color is stored separately in tag_color_mapping by name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredTag {
    /// Stable unique id (UUID).
    id: Uuid,
    /// Tag label (e.g. name in DB).
    value: String,
    /// Display/sort order (persisted as INTEGER in DB).
    sort_order: i64,
}

impl StoredTag {
    /// Create a stored tag with the given id and value. Sort order is 0 (append when inserting).
    pub fn new(id: Uuid, value: impl Into<String>) -> Self {
        Self::with_sort_order(id, value, 0)
    }

    /// Create a stored tag with the given id, value, and sort order.
    pub fn with_sort_order(id: Uuid, value: impl Into<String>, sort_order: i64) -> Self {
        Self {
            id,
            value: value.into(),
            sort_order,
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
}
