//! A single tag (id, text, checked state) for file classification.

use uuid::Uuid;

use crate::ordered::{OrderKey, OrderedThing};

/// Stable unique id for a tag (UUID).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TagId(pub Uuid);

impl TagId {
    /// For Iced widget identity: stable string per id.
    pub fn widget_id(&self) -> String {
        format!("tag-{}", self.0)
    }

    /// Create a new TagId (random UUID). Used for snapshot-only tags.
    pub fn new() -> Self {
        TagId(Uuid::new_v4())
    }
}

/// A single tag that can be applied to a file (stored tag with checked state).
#[derive(Debug, Clone)]
pub struct Tag {
    /// Stable id (from stored tag index, or from high range for snapshot-only tags).
    id: TagId,
    /// The tag text.
    tag: String,
    /// Whether this tag is currently checked.
    checked: bool,
    /// Index into the app's tag color palette (0-based).
    color_index: u8,
    /// Whether this tag comes from the stored tag store (false for tags only present in the file snapshot).
    stored: bool,
    /// Sort order (for [OrderedThing] and [crate::ordered::OrderedCollection]).
    order: OrderKey,
}

impl Tag {
    /// Create a new tag with the given id, text, color index, and stored flag (used when building from store or from file snapshot).
    /// Order is set to 0; use [Self::with_id_and_order] when sort order is known.
    pub fn with_id(id: TagId, tag: impl Into<String>, color_index: u8, stored: bool) -> Self {
        Self::with_id_and_order(id, tag, color_index, stored, 0)
    }

    /// Create a new tag with the given id, text, color index, stored flag, and sort order.
    pub fn with_id_and_order(
        id: TagId,
        tag: impl Into<String>,
        color_index: u8,
        stored: bool,
        order: OrderKey,
    ) -> Self {
        Self::with_id_order_checked(id, tag, color_index, stored, order, false)
    }

    /// Create a new tag with all fields set (id, text, color_index, stored, order, checked).
    pub fn with_id_order_checked(
        id: TagId,
        tag: impl Into<String>,
        color_index: u8,
        stored: bool,
        order: OrderKey,
        checked: bool,
    ) -> Self {
        Self {
            id,
            tag: tag.into(),
            checked,
            color_index,
            stored,
            order,
        }
    }

    /// Get the tag id.
    pub fn id(&self) -> TagId {
        self.id
    }

    /// Get the tag text.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// Whether this tag is checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Color palette index for this tag (for UI styling).
    pub fn color_index(&self) -> u8 {
        self.color_index
    }

    /// Whether this tag comes from the stored tag store (false for tags only present in the file snapshot).
    pub fn is_stored(&self) -> bool {
        self.stored
    }

    /// Toggle the checked state of this tag.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }
}

impl OrderedThing for Tag {
    type Id = TagId;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn order(&self) -> OrderKey {
        self.order
    }

    fn set_order(&mut self, order: OrderKey) {
        self.order = order;
    }
}
