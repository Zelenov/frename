//! Reusable search bar widget. Stateless: parent holds value and provides on_input and on_clear.
//! Icon inside the bar (search left, clear right when non-empty); user-typed content is the only text.

use iced::widget::{container, mouse_area, row, text, text_input};
use iced::{mouse, Element, Length};

use crate::theme;

/// Widget id for the search bar text input (for focus and global key capture).
pub const SEARCH_BAR_INPUT_ID: &str = "search-bar-input";

/// Render a search bar: search icon left, text input, optional create-tag "○" button (when text is
/// a new tag name), optional clear "×" button (when non-empty).
///
/// `on_create` – when `Some(f)`, a "○" button appears at the right end. Pressing it or hitting
/// Enter calls `f(current_input_text)` to produce the message. Pass `Some(...)` only when the
/// current `value` does not match any existing tag name.
pub fn view<'a, Message: Clone + 'a>(
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    on_clear: impl Fn() -> Message + 'a,
    on_create: Option<impl Fn(String) -> Message + 'a>,
) -> Element<'a, Message> {
    // Evaluate the closure now (at render time) so we have a cloneable Message for both
    // on_submit and the "○" button press.
    let create_msg: Option<Message> = on_create.map(|f| f(value.to_string()));

    let search_icon = text("🔍").size(12).color(theme::TEXT_MUTED);
    let mut input = text_input("", value)
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
    if let Some(ref msg) = create_msg {
        input = input.on_submit(msg.clone());
    }

    let clear_icon: Option<Element<'a, Message>> = if value.is_empty() {
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

    // "○" create button: only when create_msg is Some (text is non-empty and not an existing tag)
    let create_icon: Option<Element<'a, Message>> = create_msg.map(|msg| {
        let icon = text("○").size(14).color(theme::TEXT_MUTED);
        mouse_area(container(icon).padding(4))
            .on_press(msg)
            .interaction(mouse::Interaction::Pointer)
            .into()
    });

    let mut row_elems: Vec<Element<'a, Message>> = vec![search_icon.into(), input.into()];
    if let Some(create) = create_icon {
        row_elems.push(create);
    }
    if let Some(clear) = clear_icon {
        row_elems.push(clear);
    }
    let inner = row(row_elems)
        .spacing(6)
        .align_y(iced::Alignment::Center);

    container(inner)
        .padding([6, 8])
        .width(Length::Fill)
        .style(theme::elevated_container_bordered_style)
        .into()
}
