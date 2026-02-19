//! Stored tag: a tag entry in the app store (database or fake). Identity is UUID + name; color is in TagColorMapping (by name).

use uuid::Uuid;

/// A single stored tag in the store. Has an id (UUID) and value (label). Color is stored separately in tag_color_mapping by name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredTag {
    /// Stable unique id (UUID).
    id: Uuid,
    /// Tag label (e.g. name in DB).
    value: String,
}

impl StoredTag {
    /// Create a stored tag with the given id and value.
    pub fn new(id: Uuid, value: impl Into<String>) -> Self {
        Self {
            id,
            value: value.into(),
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
}
