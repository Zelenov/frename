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

    /// Whether `tag_name` is a stored tag (has an entry here), as opposed to a stray word from a
    /// file name that was never added. Unlike [`Self::color_index_for`], never confused by a
    /// stored tag that happens to land on color index 0.
    pub fn contains(&self, tag_name: &str) -> bool {
        self.name_to_index.contains_key(tag_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_tells_a_stored_tag_apart_from_an_unmapped_one_even_at_index_zero() {
        let mapping = TagColorMapping::from_entries([("pick".to_string(), 0)]);
        assert!(
            mapping.contains("pick"),
            "stored, even though its index is 0"
        );
        assert!(!mapping.contains("aurora"), "never stored");
        assert_eq!(
            mapping.color_index_for("aurora"),
            0,
            "defaults to 0 either way"
        );
    }
}
