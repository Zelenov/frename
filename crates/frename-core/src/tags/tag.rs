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

    /// Create a new TagId (random UUID).
    #[allow(clippy::new_without_default)] // a random id is not a meaningful default
    pub fn new() -> Self {
        TagId(Uuid::new_v4())
    }

    /// The id of a tag that is not among the folder's tags (only in a clip's name, or just
    /// created): the same for the same name every time a tag list is built, so an undo step that
    /// holds it still finds the tag after the clip is opened again or a step rebuilt its list
    /// (#619). Saving the tag keeps this id. FNV-1a of the name, twice with different offsets for
    /// 128 bits: stable across runs and Rust versions, unlike `DefaultHasher`.
    pub fn for_unsaved_name(name: &str) -> Self {
        fn fnv1a(offset: u64, bytes: &[u8]) -> u64 {
            bytes.iter().fold(offset, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
            })
        }
        let bytes = name.as_bytes();
        TagId(Uuid::from_u64_pair(
            fnv1a(0xcbf2_9ce4_8422_2325, bytes),
            fnv1a(0x6c62_272e_07bb_0142, bytes),
        ))
    }
}

/// Whether two tag names are the same tag: equal when capitals are ignored, in any alphabet
/// ("Лес" and "лес", "Été" and "été", not only "Forest" and "forest"). Folded with
/// `str::to_lowercase`, the fold the tag filter's `match_rank` uses, so a name equal to a tag
/// apart from capitals is never offered for creation (#494). A whole-string fold, not one letter
/// at a time: a Greek capital sigma at the end of a word becomes `ς` ("ΔΡΟΜΟΣ" is "δρομος").
pub(crate) fn same_tag_name(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
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
    /// Whether this stored tag is starred (pinned to section 2). Always false for unstored tags.
    starred: bool,
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
            starred: false,
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

    /// Set the stored flag (e.g. after saving a snapshot-only tag to the store).
    pub fn set_stored(&mut self, stored: bool) {
        self.stored = stored;
    }

    /// Set the color index (e.g. after saving a new tag and assigning a random color).
    pub fn set_color_index(&mut self, color_index: u8) {
        self.color_index = color_index;
    }

    /// Whether this stored tag is starred (pinned above unstarred stored tags).
    pub fn is_starred(&self) -> bool {
        self.starred
    }

    /// Set the starred flag. Only meaningful for stored tags; unstored tags should always be false.
    pub fn set_starred(&mut self, starred: bool) {
        self.starred = starred;
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

#[cfg(test)]
mod tests {
    use super::{same_tag_name, TagId};

    /// #619: a tag that is not among the folder's tags gets the same id each time a list is built.
    #[test]
    fn an_unsaved_tag_has_the_same_id_for_the_same_name() {
        assert_eq!(
            TagId::for_unsaved_name("rough"),
            TagId::for_unsaved_name("rough")
        );
        assert_ne!(
            TagId::for_unsaved_name("rough"),
            TagId::for_unsaved_name("Rough")
        );
        assert_ne!(
            TagId::for_unsaved_name("rough"),
            TagId::for_unsaved_name("roug")
        );
    }

    /// #494: one tag whatever the capitals, in any alphabet, folded as the tag filter folds.
    #[test]
    fn tag_names_in_other_capitals_are_the_same_in_any_alphabet() {
        for (a, b) in [
            ("Лес", "лес"),
            ("Été", "été"),
            ("Forest", "FOREST"),
            ("ΔΡΟΜΟΣ", "δρομος"),
            ("κόσμος", "ΚΌΣΜΟΣ"),
        ] {
            assert!(same_tag_name(a, b), "{a} / {b}");
        }
        assert!(!same_tag_name("Лес", "Лесок"));
        assert!(!same_tag_name("Лес", "Лёс"));
    }
}
