//! A file name as a file list row shows it (`docs/design/design-system.md` §13.4.2): each tag as a
//! mini chip, a `·` where the name has a dot, then the rest of the name with its extension in
//! mono. Display-only. In/out points are not part of the file name, so they are not shown here.
//!
//! Fitting: chips are never cut in the middle. Those that do not fit become a `+N` badge before
//! the name, and chips give way before the name, which keeps at least `NAME_MIN_WIDTH`. The name
//! is cut with "…" at the end: mono text has a fixed width, so the cut is exact.

use std::borrow::Cow;

use iced::widget::{row, Row};
use iced::{Alignment, Element};

use frename_core::{FileSnapshot, TagColorMapping};

use crate::ui::badge::{badge, BadgeKind};
use crate::ui::palette::TagPalette;
use crate::ui::text;
use crate::ui::tokens::*;
use crate::widgets::tag_chip;

/// Where the name has a dot between its parts.
const DOT: &str = "·";

/// What a name `width` px wide shows: the first `chips` tags, how many are `hidden` in the
/// `+N` badge, and the name part, cut to fit.
#[derive(Debug, PartialEq, Eq)]
struct Fit<'a> {
    chips: usize,
    hidden: usize,
    name: Cow<'a, str>,
}

/// A mini chip's width for a tag of `chars` characters.
fn chip_width(chars: usize) -> f32 {
    2.0 * SPACE_TIGHT + CHIP_MINI_CHAR_WIDTH * chars as f32
}

/// The `·` between two parts, with the row's spacing on both sides.
fn separator() -> f32 {
    CAPTION_CHAR_WIDTH + 2.0 * SPACE_XS
}

/// The `+N` badge's width.
fn hidden_badge_width(hidden: usize) -> f32 {
    2.0 * SPACE_TIGHT + CAPTION_CHAR_WIDTH * (hidden.to_string().len() + 1) as f32
}

/// `name` cut with "…" to at most `width` px of mono text.
fn fit_name(name: &str, width: f32) -> Cow<'_, str> {
    text::fit(name, width, MONO_CHAR_WIDTH)
}

/// How `tags` and `name` share `width` px.
fn fit<'a>(tags: &[String], name: &'a str, width: f32) -> Fit<'a> {
    let name_width = MONO_CHAR_WIDTH * name.chars().count() as f32;
    let name_keeps = name_width.min(NAME_MIN_WIDTH);
    let mut used = 0.0;
    let mut chips = 0;
    for (i, tag) in tags.iter().enumerate() {
        let gap = if i > 0 { separator() } else { 0.0 };
        let after = tags.len() - i - 1;
        let badge = if after > 0 {
            separator() + hidden_badge_width(after)
        } else {
            0.0
        };
        let before_name = if name.is_empty() { 0.0 } else { separator() };
        let need = gap + chip_width(tag.chars().count());
        if used + need + badge + before_name + name_keeps > width {
            break;
        }
        used += need;
        chips += 1;
    }
    let hidden = tags.len() - chips;
    if hidden > 0 {
        used += if chips > 0 { separator() } else { 0.0 } + hidden_badge_width(hidden);
    }
    if used > 0.0 && !name.is_empty() {
        used += separator();
    }
    Fit {
        chips,
        hidden,
        name: fit_name(name, width - used),
    }
}

/// Renders the file name in `width` px as mini chips + name.extension, on one line.
///
/// Borrows the snapshot: the folder list renders one of these per row on every redraw, so taking
/// it by value cost a full `FileSnapshot` clone (plus a `Vec<String>` and a `String` per tag) per
/// row per frame. `color_mapping` is only read for colour lookups, so its borrow does not escape.
pub fn view<'a, Message: 'a>(
    snapshot: &'a FileSnapshot,
    color_mapping: &TagColorMapping,
    tag_palette: TagPalette,
    width: f32,
) -> Row<'a, Message> {
    let tags = snapshot.tags();
    let name_ext = name_ext_from_parts(snapshot.name_without_extension(), snapshot.extension());
    let shown = fit(tags, &name_ext, width);

    let mut parts: Vec<Element<'a, Message>> = Vec::with_capacity(2 * tags.len() + 3);
    for tag_name in tags.iter().take(shown.chips) {
        if !parts.is_empty() {
            parts.push(dot());
        }
        let tag_color = tag_palette.color(
            color_mapping.color_index_for(tag_name),
            color_mapping.contains(tag_name),
        );
        parts.push(tag_chip::mini(tag_name, tag_color));
    }
    if shown.hidden > 0 {
        if !parts.is_empty() {
            parts.push(dot());
        }
        parts.push(badge(BadgeKind::Neutral, format!("+{}", shown.hidden)));
    }
    if !shown.name.is_empty() {
        if !parts.is_empty() {
            parts.push(dot());
        }
        parts.push(
            text::mono(shown.name.into_owned())
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
    use super::*;

    fn tags(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn the_name_and_extension_meet_at_one_dot() {
        assert_eq!(name_ext_from_parts("MVI_0410", ".mp4"), "MVI_0410.mp4");
        assert_eq!(name_ext_from_parts("MVI_0410", "mp4"), "MVI_0410.mp4");
        assert_eq!(name_ext_from_parts("", ".mp4"), ".mp4");
        assert_eq!(name_ext_from_parts("notes", ""), "notes");
    }

    #[test]
    fn everything_shows_when_there_is_room() {
        let shown = fit(&tags(&["pick", "wide"]), "MVI_0410.mp4", 1000.0);
        assert_eq!(shown.chips, 2);
        assert_eq!(shown.hidden, 0);
        assert_eq!(shown.name, "MVI_0410.mp4");
    }

    #[test]
    fn chips_that_do_not_fit_become_a_count_and_are_never_cut() {
        let all = tags(&["pick", "wide", "landscape", "golden-hour"]);
        let shown = fit(&all, "MVI_0410.mp4", 260.0);
        assert!(shown.chips < all.len(), "{shown:?}");
        assert_eq!(shown.chips + shown.hidden, all.len());
    }

    #[test]
    fn chips_give_way_before_the_name_keeps_its_minimum() {
        let shown = fit(&tags(&["landscape"]), "MVI_0410_long_name.mp4", 120.0);
        assert_eq!(shown.chips, 0);
        assert_eq!(shown.hidden, 1);
        assert!(shown.name.ends_with('…'), "{shown:?}");
    }

    #[test]
    fn a_long_name_is_cut_with_an_ellipsis_to_its_room() {
        let name = "a_very_long_file_name_from_the_camera.mp4";
        let cut = fit_name(name, 10.0 * MONO_CHAR_WIDTH);
        assert_eq!(cut.chars().count(), 10);
        assert!(cut.ends_with('…'));
        assert_eq!(fit_name("short.mp4", 400.0), "short.mp4");
    }
}
