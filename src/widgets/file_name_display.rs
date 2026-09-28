//! A file name as a file list row shows it (`docs/design/design-system.md` §13.4.2): each tag as a
//! mini chip, a `·` where the name has a dot, then the rest of the name with its extension in
//! mono. Display-only. In/out points are not part of the file name, so they are not shown here.

use iced::widget::{row, Row};
use iced::{Alignment, Element};

use frename_core::{FileSnapshot, TagColorMapping};

use crate::ui::palette::TagPalette;
use crate::ui::text;
use crate::ui::tokens::*;
use crate::widgets::tag_chip;

/// Where the name has a dot between its parts.
const DOT: &str = "·";

/// Renders the file name as mini chips + name.extension (no outer container), on one line.
///
/// Borrows the snapshot: the folder list renders one of these per row on every redraw, so taking
/// it by value cost a full `FileSnapshot` clone (plus a `Vec<String>` and a `String` per tag) per
/// row per frame. `color_mapping` is only read for colour lookups, so its borrow does not escape.
pub fn view<'a, Message: 'a>(
    snapshot: &'a FileSnapshot,
    color_mapping: &TagColorMapping,
    tag_palette: TagPalette,
) -> Row<'a, Message> {
    let tags = snapshot.tags();
    let name_ext = name_ext_from_parts(snapshot.name_without_extension(), snapshot.extension());

    let mut parts: Vec<Element<'a, Message>> = Vec::with_capacity(2 * tags.len() + 1);
    for (i, tag_name) in tags.iter().enumerate() {
        if i > 0 {
            parts.push(dot());
        }
        let tag_color = tag_palette.color(color_mapping.color_index_for(tag_name));
        parts.push(tag_chip::mini(tag_name, tag_color));
    }
    if !name_ext.is_empty() {
        if !tags.is_empty() {
            parts.push(dot());
        }
        parts.push(
            text::mono(name_ext)
                .color(TEXT)
                .wrapping(iced::widget::text::Wrapping::None)
                .into(),
        );
    }
    row(parts).spacing(SPACE_XS).align_y(Alignment::Center)
}

fn dot<'a, Message: 'a>() -> Element<'a, Message> {
    text::caption(DOT).into()
}

/// `name` and `ext` joined as the file name shows them: `name.ext`, with no doubled dot.
fn name_ext_from_parts(name: &str, ext: &str) -> String {
    if name.is_empty() {
        ext.to_string()
    } else if ext.is_empty() {
        name.to_string()
    } else if ext.starts_with('.') {
        format!("{name}{ext}")
    } else {
        format!("{name}.{ext}")
    }
}

#[cfg(test)]
mod tests {
    use super::name_ext_from_parts;

    #[test]
    fn the_name_and_extension_meet_at_one_dot() {
        assert_eq!(name_ext_from_parts("MVI_0410", ".mp4"), "MVI_0410.mp4");
        assert_eq!(name_ext_from_parts("MVI_0410", "mp4"), "MVI_0410.mp4");
        assert_eq!(name_ext_from_parts("", ".mp4"), ".mp4");
        assert_eq!(name_ext_from_parts("notes", ""), "notes");
    }
}
