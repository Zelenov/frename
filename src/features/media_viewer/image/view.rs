//! View for the image viewer sub-feature.

use iced::widget::{button, column, container, image, row, text, tooltip};
use iced::{ContentFit, Element, Length};

use crate::theme;
use super::{ImageViewerState, Message};

const CONTROLS_HEIGHT: f32 = 32.0;

/// Render the image viewer with a controls bar (fullscreen button) at the bottom right.
/// `is_fullscreen` controls which icon the fullscreen button shows.
pub fn view(state: &ImageViewerState, is_fullscreen: bool) -> Element<'_, Message> {
    if let Some(handle) = state.current_handle() {
        let img = image(handle.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Contain);

        let fullscreen_icon = if is_fullscreen { "⊡" } else { "⛶" };
        let fullscreen_btn: Element<'_, Message> = tooltip(
            button(
                container(text(fullscreen_icon).size(14))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
            )
            .on_press(Message::ToggleFullscreen)
            .width(CONTROLS_HEIGHT)
            .height(CONTROLS_HEIGHT)
            .padding(0)
            .style(theme::icon_button_style(true)),
            text("F5"),
            tooltip::Position::Top,
        )
        .into();

        // Spacer pushes the button to the right.
        let controls = container(
            row![
                container(text("")).width(Length::Fill),
                fullscreen_btn,
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(iced::Alignment::Center),
        )
        .padding([0, 8])
        .width(Length::Fill)
        .height(CONTROLS_HEIGHT)
        .style(theme::panel_container_style);

        column![img, controls]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else if state.is_loading() {
        container(text("⏳").size(48).color(theme::TEXT_MUTED))
            .center(Length::Fill)
            .into()
    } else if state.load_failed() {
        container(text("✕").size(80).color(theme::ERROR))
            .center(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::panel_container_style)
            .into()
    } else {
        container(text("🖼").size(48).color(theme::TEXT_MUTED))
            .center(Length::Fill)
            .into()
    }
}
