//! File name display widget. Shows how the new file name is combined: each tag as a colored
//! chip (dot-separated), then the name and extension concatenated (no separator between them).
//! Display-only; no interactions. Uses [crate::widgets::tag_chip] for tag pills.

use iced::widget::{row, text};
use iced::Element;

use frename_core::{FileSnapshot, TagColorMapping};

use crate::tag_colors;
use crate::theme;
use crate::widgets::tag_chip;

/// Dot separator between parts (tags, name, extension).
const DOT: &str = " . ";

/// Renders the file name as tag chips + name.extension (no outer container).
/// Callers wrap in a container when they need panel style (e.g. file workspace, folder list rows).
pub fn view<Message: 'static>(
    snapshot: FileSnapshot,
    color_mapping: &TagColorMapping,
    wrap: bool,
) -> Element<'static, Message> {
    let tags: Vec<String> = snapshot.tags().to_vec();
    let name = snapshot.name_without_extension().to_string();
    let ext = snapshot.extension().to_string();
    let name_ext = name_ext_from_parts(&name, &ext);

    let mut parts: Vec<Element<'static, Message>> = Vec::new();
    for (i, tag_name) in tags.iter().enumerate() {
        if i > 0 {
            parts.push(dot_text().into());
        }
        let color_index = color_mapping.color_index_for(tag_name);
        let tag_color = tag_colors::TagColors::color(color_index);
        parts.push(tag_chip::view_display_only(tag_name.clone(), tag_color));
    }
    if !name_ext.is_empty() {
        if !tags.is_empty() {
            parts.push(dot_text().into());
        }
        parts.push(
            text(name_ext)
                .size(14)
                .color(theme::TEXT)
                .into(),
        );
    }

    let row = row(parts)
        .spacing(0)
        .align_y(iced::Alignment::Center);
    if wrap {
        row.wrap()
            .vertical_spacing(4)
            .align_x(iced::Alignment::Start)
            .into()
    } else {
        row.into()
    }
}

fn dot_text<Message: 'static>() -> Element<'static, Message> {
    text(DOT)
        .size(14)
        .color(theme::TEXT_MUTED)
        .into()
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
