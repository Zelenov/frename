//! Reusable search bar widget. Stateless: parent holds value and provides on_input message.
//! Icon inside the bar (first symbol, small and muted); user-typed content is the only text.

use iced::widget::{container, row, text, text_input};
use iced::{Element, Length};

use crate::theme;

/// Widget id for the search bar text input (for focus and global key capture).
pub const SEARCH_BAR_INPUT_ID: &str = "search-bar-input";

/// Render a search bar: one visual unit with icon inside on the left, then text input.
/// Icon is small and muted; cursor/text starts to the right of the icon.
pub fn view<'a, Message: Clone + 'a>(
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let icon = text("🔍").size(12).color(theme::TEXT_MUTED);
    let input = text_input("", value)
        .id(iced::widget::Id::from(SEARCH_BAR_INPUT_ID))
        .on_input(on_input)
        .padding([8, 8])
        .size(14)
        .style(|_theme: &iced::Theme, _status: iced::widget::text_input::Status| {
            iced::widget::text_input::Style {
                background: iced::Background::Color(theme::BG_ELEVATED),
                border: iced::Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: theme::BG_ELEVATED,
                },
                icon: theme::TEXT_MUTED,
                placeholder: theme::TEXT_MUTED,
                value: theme::TEXT,
                selection: theme::ACCENT,
            }
        });

    let inner = row![icon, input]
        .spacing(6)
        .align_y(iced::Alignment::Center);

    container(inner)
        .padding([6, 8])
        .width(Length::Fill)
        .style(|_theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::BG_ELEVATED)),
            border: iced::Border {
                radius: 4.0.into(),
                width: 1.0,
                color: theme::TEXT_MUTED,
            },
            ..Default::default()
        })
        .into()
}
