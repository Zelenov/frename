//! Built-in tag set written into a folder that has no tag file of its own yet.
//!
//! Ids are fixed rather than generated, so a built-in tag keeps one identity across every
//! folder: `pick` in a Kenya folder and `pick` in a China folder are the same tag.
//! The set is deliberately generic (picks, shot type, light, subject); folder-specific tags
//! such as place names are added by the user afterwards.

use uuid::Uuid;

/// One entry of the built-in tag set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefaultTag {
    /// Stable id, identical in every folder.
    pub id: Uuid,
    /// Tag label.
    pub name: &'static str,
    /// Index into the 16-color palette.
    pub color_index: u8,
    /// Whether the tag starts out starred (pinned to the starred section).
    pub starred: bool,
}

impl DefaultTag {
    /// Builds an entry from the low bits of its id, so the table below stays readable.
    const fn new(id: u128, name: &'static str, color_index: u8, starred: bool) -> Self {
        Self {
            id: Uuid::from_u128(id),
            name,
            color_index,
            starred,
        }
    }
}

/// The built-in tag set, in display order. Array order is the tag order; there are no
/// sort keys here, because the order is whatever the folder's tag file says it is.
pub const DEFAULT_TAGS: &[DefaultTag] = &[
    // Picks
    DefaultTag::new(0x00, "pick", 9, true),
    DefaultTag::new(0x01, "skip", 7, true),
    DefaultTag::new(0x02, "review", 10, true),
    // Shot type
    DefaultTag::new(0x03, "wide", 3, false),
    DefaultTag::new(0x04, "medium", 3, false),
    DefaultTag::new(0x05, "close", 3, false),
    // Camera movement
    DefaultTag::new(0x06, "handheld", 5, false),
    DefaultTag::new(0x07, "gimbal", 5, false),
    DefaultTag::new(0x08, "tripod", 5, false),
    DefaultTag::new(0x09, "drone", 6, false),
    // Lighting
    DefaultTag::new(0x0a, "golden-hour", 12, false),
    DefaultTag::new(0x0b, "sunrise", 12, false),
    DefaultTag::new(0x0c, "sunset", 11, false),
    DefaultTag::new(0x0d, "night", 8, false),
    DefaultTag::new(0x0e, "overcast", 4, false),
    // Subject
    DefaultTag::new(0x0f, "people", 2, false),
    DefaultTag::new(0x10, "wildlife", 13, false),
    DefaultTag::new(0x11, "landscape", 14, false),
    DefaultTag::new(0x12, "city", 1, false),
    DefaultTag::new(0x13, "action", 15, false),
    // Usage
    DefaultTag::new(0x14, "b-roll", 4, false),
    DefaultTag::new(0x15, "interview", 2, false),
    DefaultTag::new(0x16, "timelapse", 6, false),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn default_tag_ids_are_unique() {
        let ids: HashSet<Uuid> = DEFAULT_TAGS.iter().map(|t| t.id).collect();
        assert_eq!(ids.len(), DEFAULT_TAGS.len());
    }

    #[test]
    fn default_tag_names_are_unique() {
        let names: HashSet<&str> = DEFAULT_TAGS.iter().map(|t| t.name).collect();
        assert_eq!(names.len(), DEFAULT_TAGS.len());
    }

    #[test]
    fn default_tag_colors_are_within_palette() {
        assert!(DEFAULT_TAGS.iter().all(|t| t.color_index < 16));
    }
}
