//! File name display widget. Shows how the new file name is combined: each tag as a colored
//! box with label (dot-separated), then the name and extension concatenated (no separator between them).
//! Display-only; no interactions.
//!
//! Single entry point: [view] takes a snapshot and color mapping, returns the chip row.
//! Callers wrap in a container when they need the elevated panel style (e.g. file workspace).

use iced::widget::{container, row, text};
use iced::{Background, Element, Length};

use frename_core::{FileSnapshot, TagColorMapping};

use crate::tag_colors;
use crate::theme;

/// Dot separator between parts (tags, name, extension).
const DOT: &str = " . ";

/// When true, the row of tag chips and name/extension wraps to multiple lines when width is limited.
pub const DEFAULT_WRAP: bool = true;

/// Renders the file name as tag chips + name.extension (no outer container).
/// Use this for both the file workspace panel (wrap in [view_in_panel]) and the folder list rows.
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
        parts.push(
            container(
                text(tag_name.clone())
                    .size(14)
                    .color(iced::Color::from_rgb(0.0, 0.0, 0.0)),
            )
            .padding([4, 6])
            .style(move |_theme| iced::widget::container::Style {
                background: Some(Background::Color(tag_color)),
                border: iced::border::rounded(2),
                ..Default::default()
            })
            .into(),
        );
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

/// Same as [view] but wrapped in the elevated panel container (padding + background). Use in the file workspace.
pub fn view_in_panel<Message: 'static>(
    snapshot: FileSnapshot,
    color_mapping: &TagColorMapping,
    wrap: bool,
) -> Element<'static, Message> {
    container(view(snapshot, color_mapping, wrap))
        .padding([8, 8])
        .width(Length::Fill)
        .style(theme::elevated_container_style)
        .into()
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
