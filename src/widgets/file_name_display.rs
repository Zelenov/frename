//! File name display widget. Shows how the new file name is combined: each tag as a colored
//! box with label (dot-separated), then the name and extension concatenated (no separator between them).
//! Display-only; no interactions.

use iced::widget::{container, row, text};
use iced::{Background, Element, Length};

use frename_core::{File, StoredTagStore, TagList};

use crate::tag_colors;
use crate::theme;

/// Dot separator between parts (tags, name, extension).
const DOT: &str = " . ";

/// When true, the row of tag chips and name/extension wraps to multiple lines when width is limited.
pub const DEFAULT_WRAP: bool = true;

/// Render the file name display: tag boxes (label + tag color) separated by dots, then name + extension as one part.
/// If `wrap` is true, parts wrap to the next line when horizontal space is insufficient.
pub fn view<'a, S, Message: 'a>(
    file: Option<&'a File>,
    tag_list: Option<&'a TagList<S>>,
    wrap: bool,
) -> Element<'a, Message>
where
    S: StoredTagStore + Clone,
{
    let inner: Element<'a, Message> = match (file, tag_list) {
        (None, _) => placeholder("📄"),
        (Some(_), None) => placeholder("🏷"),
        (Some(_), Some(list)) => {
            let snapshot = list.file_snapshot();
            let checked_tags: Vec<_> = list.tags().iter().filter(|t| t.is_checked()).collect();
            let name = snapshot.name_without_extension().to_string();
            let ext = snapshot.extension().to_string();

            if checked_tags.is_empty() && name.is_empty() && ext.is_empty() {
                return container(placeholder("🏷"))
                    .padding([8, 8])
                    .width(Length::Fill)
                    .style(|_theme| iced::widget::container::Style {
                        background: Some(Background::Color(theme::BG_ELEVATED)),
                        ..Default::default()
                    })
                    .into();
            }

            let mut parts: Vec<Element<'a, Message>> = Vec::new();

            for (i, tag) in checked_tags.iter().enumerate() {
                if i > 0 {
                    parts.push(dot_text().into());
                }
                let tag_color = tag_colors::TagColors::color(tag.color_index());
                parts.push(
                    container(
                        text(tag.tag())
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

            let name_ext = if name.is_empty() {
                ext
            } else if ext.is_empty() {
                name
            } else {
                format!("{}{}", name, ext)
            };
            if !name_ext.is_empty() {
                if !checked_tags.is_empty() {
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
    };

    container(inner)
        .padding([8, 8])
        .width(Length::Fill)
        .style(|_theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::BG_ELEVATED)),
            ..Default::default()
        })
        .into()
}

fn placeholder<'a, Message: 'a>(icon: &'static str) -> Element<'a, Message> {
    text(icon)
        .size(14)
        .color(theme::TEXT_MUTED)
        .into()
}

fn dot_text<'a, Message: 'a>() -> Element<'a, Message> {
    text(DOT)
        .size(14)
        .color(theme::TEXT_MUTED)
        .into()
}
