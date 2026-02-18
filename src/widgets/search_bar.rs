//! Reusable search bar widget. Stateless: parent holds value and provides on_input and on_clear.
//! Icon inside the bar (search left, clear right when non-empty); user-typed content is the only text.

use iced::widget::{container, mouse_area, row, text, text_input};
use iced::{mouse, Element, Length};

use crate::theme;

/// Widget id for the search bar text input (for focus and global key capture).
pub const SEARCH_BAR_INPUT_ID: &str = "search-bar-input";

/// Render a search bar: one visual unit with search icon on the left, text input, clear (×) on the right when non-empty.
pub fn view<'a, Message: Clone + 'a>(
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    on_clear: impl Fn() -> Message + 'a,
) -> Element<'a, Message> {
    let search_icon = text("🔍").size(12).color(theme::TEXT_MUTED);
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

    let clear_icon = if value.is_empty() {
        None
    } else {
        let icon = text("×").size(16).color(theme::TEXT_MUTED);
        Some(
            mouse_area(container(icon).padding(4))
                .on_press(on_clear())
                .interaction(mouse::Interaction::Pointer)
                .into(),
        )
    };

    let mut row_elems: Vec<Element<'a, Message>> = vec![search_icon.into(), input.into()];
    if let Some(clear) = clear_icon {
        row_elems.push(clear);
    }
    let inner = row(row_elems)
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
