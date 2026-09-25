//! File name display widget. Shows how the new file name is combined: each tag as a colored
//! chip (dot-separated), then the name and extension concatenated (no separator between them).
//! Display-only; no interactions. Uses [crate::widgets::tag_chip] for tag pills.

use iced::widget::{row, text};
use iced::Element;

use frename_core::{FileSnapshot, TagColorMapping};

use crate::tag_colors::TagPalette;
use crate::theme;
use crate::widgets::tag_chip;

/// Dot separator between parts (tags, name, extension).
const DOT: &str = " . ";

/// Format seconds as `MM:SS` or `HH:MM:SS`.
fn fmt_timecode(secs: f32) -> String {
    let total = secs as u32;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h == 0 {
        format!("{:02}:{:02}", m, s)
    } else {
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
}

/// Renders the file name as tag chips + optional timecodes + name.extension (no outer container).
/// `seg_start`/`seg_end`: optional IN/OUT timecodes shown between tags and file name.
/// Callers wrap in a container when they need panel style (e.g. file workspace, folder list rows).
///
/// Borrows the snapshot: the folder list renders one of these per row on every redraw, so taking
/// it by value cost a full `FileSnapshot` clone (plus a `Vec<String>` and a `String` per tag) per
/// row per frame. `color_mapping` is only read for colour lookups, so its borrow does not escape.
pub fn view<'a, Message: 'a>(
    snapshot: &'a FileSnapshot,
    color_mapping: &TagColorMapping,
    tag_palette: TagPalette,
    wrap: bool,
) -> Element<'a, Message> {
    let seg_start = snapshot.segment_start();
    let seg_end = snapshot.segment_end();
    let tags = snapshot.tags();
    let name_ext = name_ext_from_parts(snapshot.name_without_extension(), snapshot.extension());

    let mut parts: Vec<Element<'a, Message>> = Vec::new();
    for (i, tag_name) in tags.iter().enumerate() {
        if i > 0 {
            parts.push(dot_text());
        }
        let color_index = color_mapping.color_index_for(tag_name);
        let tag_color = tag_palette.color(color_index);
        parts.push(tag_chip::view_display_only(tag_name, tag_color));
    }
    // Timecode badges between tags and file name.
    for secs in seg_start.into_iter().chain(seg_end) {
        parts.push(dot_text());
        parts.push(
            text(fmt_timecode(secs))
                .size(12)
                .color(theme::TEXT_MUTED)
                .into(),
        );
    }
    if !name_ext.is_empty() {
        if !tags.is_empty() || seg_start.is_some() || seg_end.is_some() {
            parts.push(dot_text());
        }
        parts.push(text(name_ext).size(14).color(theme::TEXT).into());
    }

    let row = row(parts).spacing(0).align_y(iced::Alignment::Center);
    if wrap {
        row.wrap()
            .vertical_spacing(4)
            .align_x(iced::Alignment::Start)
            .into()
    } else {
        row.into()
    }
}

fn dot_text<'a, Message: 'a>() -> Element<'a, Message> {
    text(DOT).size(14).color(theme::TEXT_MUTED).into()
}

fn name_ext_from_parts(name: &str, ext: &str) -> String {
    if name.is_empty() {
        ext.to_string()
    } else if ext.is_empty() {
        name.to_string()
    } else if ext.starts_with('.') {
        format!("{}{}", name, ext)
    } else {
        format!("{}.{}", name, ext)
    }
}
