//! Tag color mapping: tag name -> color index. Stored in DB in a separate table from tags.
//! Colors are keyed by tag name, not by tag ID.

use std::collections::HashMap;

/// Maps tag name to palette color index. Loaded from the database (tag_color_mapping table).
#[derive(Clone, Debug, Default)]
pub struct TagColorMapping {
    name_to_index: HashMap<String, u8>,
}

impl TagColorMapping {
    /// Build mapping from name -> color_index entries (e.g. from the store).
    pub fn from_entries(entries: impl IntoIterator<Item = (String, u8)>) -> Self {
        Self {
            name_to_index: entries.into_iter().collect(),
        }
    }

    /// Returns the color index for the given tag name. Defaults to 0 if not found.
    pub fn color_index_for(&self, tag_name: &str) -> u8 {
        self.name_to_index.get(tag_name).copied().unwrap_or(0)
    }

    /// Whether `tag_name` is a stored tag: a color is recorded here only once a tag is saved
    /// (`save_tag`), so presence in the map is itself the "is this tag in the list" signal a
    /// caller with only a name (not a `Tag`) can use — see issue #53.
    pub fn is_stored(&self, tag_name: &str) -> bool {
        self.name_to_index.contains_key(tag_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_with_a_recorded_color_is_stored_one_without_is_not() {
        let mapping = TagColorMapping::from_entries([("Goat".to_string(), 0)]);
        assert!(mapping.is_stored("Goat"));
        assert!(!mapping.is_stored("Sheep"));
        // A recorded color of 0 must not be mistaken for "not found" by color_index_for either.
        assert_eq!(mapping.color_index_for("Goat"), 0);
        assert_eq!(mapping.color_index_for("Sheep"), 0);
    }
}
