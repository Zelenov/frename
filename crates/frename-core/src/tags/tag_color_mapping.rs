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
}
