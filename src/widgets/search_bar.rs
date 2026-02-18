//! Reusable search bar widget. Stateless: parent holds value and provides on_input message.
//! Uses an icon only (no UI text); user-typed content is the only text.

use iced::widget::{container, row, text, text_input};
use iced::{Element, Length};

use crate::theme;

/// Render a search bar: icon + single-line text input (no placeholder text).
/// Parent passes current value and maps input to its message.
pub fn view<'a, Message: Clone + 'a>(
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let icon = text("🔍").size(16).color(theme::TEXT_MUTED);
    let input = text_input("", value)
        .on_input(on_input)
        .padding(8)
        .size(14)
        .style(|_theme: &iced::Theme, _status: iced::widget::text_input::Status| {
            iced::widget::text_input::Style {
                background: iced::Background::Color(theme::BG_ELEVATED),
                border: iced::Border {
                    radius: 2.0.into(),
                    width: 1.0,
                    color: theme::TEXT_MUTED,
                },
                icon: theme::TEXT_MUTED,
                placeholder: theme::TEXT_MUTED,
                value: theme::TEXT,
                selection: theme::ACCENT,
            }
        });

    let content = row![icon, input]
        .spacing(6)
        .align_y(iced::Alignment::Center);

    container(content)
        .width(Length::Fill)
        .padding([4, 4])
        .into()
}
