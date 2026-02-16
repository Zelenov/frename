//! File name display widget. Shows the file name from workspace tag list + file stem (not the file's own tags).
//! Display-only; no interactions.

use iced::widget::{container, text};
use iced::{Background, Element, Length};

use frename_core::{File, TagList};

use crate::theme;

/// Render the file name display: workspace tags + file stem, or a placeholder.
pub fn view<'a, Message: 'a>(
    file: Option<&'a File>,
    tag_list: Option<&'a TagList>,
) -> Element<'a, Message> {
    let (label, color) = match (file, tag_list) {
        (None, _) => ("📄".to_string(), theme::TEXT_MUTED),
        (Some(_), None) => ("🏷".to_string(), theme::TEXT_MUTED),
        (Some(f), Some(list)) => {
            let name = list.checked_file_tags().file_name(f.initial_filename());
            (
                if name.is_empty() {
                    "🏷".to_string()
                } else {
                    name
                },
                theme::TEXT,
            )
        }
    };

    let content = container(
        text(label)
            .size(14)
            .color(color)
            .wrapping(text::Wrapping::WordOrGlyph),
    )
    .padding([8, 8])
    .width(Length::Fill)
    .style(|_theme| iced::widget::container::Style {
        background: Some(Background::Color(theme::BG_ELEVATED)),
        ..Default::default()
    });

    content.into()
}
