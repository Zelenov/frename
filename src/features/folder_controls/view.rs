//! UI for folder controls (prev/next). Only this module knows they are buttons; receives only booleans.

use iced::widget::{button, container, row, text, tooltip};
use iced::Element;

use crate::features::folder;
use crate::theme;

const CONTROLS_HEIGHT: f32 = 32.0;

/// Render the folder controls: Previous File, Next File, and Scroll-to-Selected buttons.
/// Buttons are enabled only when applicable.
pub fn view(has_previous: bool, has_next: bool, has_selected: bool) -> Element<'static, folder::Message> {
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
            .style(theme::icon_button_style(has_previous)),
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
            .style(theme::icon_button_style(has_next)),
        text("Page Down"),
        iced::widget::tooltip::Position::Top,
    )
        .into();

    let scroll_btn: Element<'_, folder::Message> = tooltip(
        button(
            container(text("⊙").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
            .on_press(folder::Message::ScrollToSelected)
            .width(CONTROLS_HEIGHT)
            .height(iced::Length::Fill)
            .padding(0)
            .style(theme::icon_button_style(has_selected)),
        text("Scroll to file"),
        iced::widget::tooltip::Position::Top,
    )
        .into();

    let controls = row![prev_btn, next_btn, scroll_btn]
        .spacing(8)
        .height(iced::Length::Fill)
        .align_y(iced::Alignment::Center);

    container(controls)
        .padding([0, 8])
        .width(iced::Length::Fill)
        .height(CONTROLS_HEIGHT)
        .style(theme::panel_container_style)
        .into()
}
