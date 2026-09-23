//! UI for the settings window.

use iced::widget::{checkbox, column, container, text};
use iced::{Element, Length};

use crate::theme;

use super::{Message, SettingsState};

/// Render the settings window: one titled section per area, one checkbox per setting.
pub fn view(state: &SettingsState) -> Element<'_, Message> {
    let settings = state.settings();

    let video = section(
        "Video",
        checkbox(settings.autoplay_video)
            .label("Play videos automatically when opened")
            .on_toggle(Message::SetAutoplayVideo)
            .into(),
    );
    let tags = section(
        "Tags",
        checkbox(settings.monochrome_tags)
            .label("Monochrome tags")
            .on_toggle(Message::SetMonochromeTags)
            .into(),
    );

    container(column![video, tags].spacing(20))
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::main_container_style)
        .into()
}

fn section<'a>(title: &'a str, content: Element<'a, Message>) -> Element<'a, Message> {
    column![text(title).size(14).color(theme::TEXT_MUTED), content]
        .spacing(8)
        .into()
}
