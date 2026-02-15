//! File name display widget. Shows the generated file name for a file (or placeholders).
//! Display-only; no interactions.

use iced::widget::{container, text};
use iced::{Background, Element, Length};

use frename_core::File;

use crate::theme;

/// Render the file name display: shows the current file name (tags + initial name) or a placeholder.
pub fn view<'a, Message: 'a>(file: Option<&'a File>) -> Element<'a, Message> {
    let (label, color) = match file {
        None => ("Select a file", theme::TEXT_MUTED),
        Some(f) => (
            if f.file_name().is_empty() {
                "(no tags selected)"
            } else {
                f.file_name()
            },
            theme::TEXT,
        ),
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
