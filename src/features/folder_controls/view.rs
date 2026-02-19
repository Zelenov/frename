//! UI for folder controls (prev/next). Only this module knows they are buttons; receives only booleans.

use iced::widget::{button, container, row, text, tooltip};
use iced::{Background, Color, Element};

use crate::features::folder;
use crate::theme;

const CONTROLS_HEIGHT: f32 = 32.0;

/// Render the folder controls: Previous File and Next File buttons.
/// Buttons are enabled only when there is a previous/next file available.
pub fn view(has_previous: bool, has_next: bool) -> Element<'static, folder::Message> {
    let button_style = |enabled: bool| {
        move |_theme: &iced::Theme, status: iced::widget::button::Status| {
            let (bg, text_color) = if enabled {
                let bg = match status {
                    iced::widget::button::Status::Hovered => theme::TRACK,
                    iced::widget::button::Status::Pressed => theme::SPLITTER_ACTIVE,
                    _ => Color::TRANSPARENT,
                };
                (bg, theme::TEXT)
            } else {
                (Color::TRANSPARENT, theme::TEXT_MUTED)
            };
            iced::widget::button::Style {
                background: Some(Background::Color(bg)),
                text_color,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: true,
            }
        }
    };

    let prev_btn: Element<'_, folder::Message> = tooltip(
        button(
            container(text("◀").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
            .on_press(folder::Message::PreviousFile)
            .width(CONTROLS_HEIGHT)
            .height(iced::Length::Fill)
            .padding(0)
            .style(button_style(has_previous)),
        text("Page Up"),
        iced::widget::tooltip::Position::Top,
    )
        .into();

    let next_btn: Element<'_, folder::Message> = tooltip(
        button(
            container(text("▶").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
            .on_press(folder::Message::NextFile)
            .width(CONTROLS_HEIGHT)
            .height(iced::Length::Fill)
            .padding(0)
            .style(button_style(has_next)),
        text("Page Down"),
        iced::widget::tooltip::Position::Top,
    )
        .into();

    let controls = row![prev_btn, next_btn]
        .spacing(8)
        .height(iced::Length::Fill)
        .align_y(iced::Alignment::Center);

    container(controls)
        .padding([0, 8])
        .width(iced::Length::Fill)
        .height(CONTROLS_HEIGHT)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_PANEL)),
            ..container::Style::default()
        })
        .into()
}
