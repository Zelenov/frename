//! Small timecode badge with a tooltip label and × remove button.

use iced::widget::{container, mouse_area, row, text, tooltip};
use iced::{mouse, Alignment, Background, Color, Element};

use crate::theme;

/// Format seconds as `MM:SS` or `HH:MM:SS` (hours shown only when non-zero).
pub fn fmt_timecode(secs: f32) -> String {
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

/// Renders a timecode badge: time text + × button, with `label` shown as a tooltip.
pub fn view<Message: Clone + 'static>(
    label: &'static str,
    time_str: String,
    clear_msg: Message,
) -> Element<'static, Message> {
    let remove_btn = mouse_area(
        container(text("×").size(12).color(Color::from_rgb(0.6, 0.6, 0.65))).padding([0, 2]),
    )
    .on_press(clear_msg)
    .interaction(mouse::Interaction::Pointer);

    let badge = container(
        row![text(time_str).size(13).color(theme::TEXT), remove_btn,]
            .spacing(4)
            .align_y(Alignment::Center),
    )
    .padding([3, 6])
    .style(|_theme: &_| iced::widget::container::Style {
        background: Some(Background::Color(Color::from_rgb(0.22, 0.22, 0.26))),
        border: iced::border::rounded(3),
        ..Default::default()
    });

    tooltip(badge, text(label), tooltip::Position::Top).into()
}
