//! View for the unified media viewer feature.

use iced::widget::{container, text};
use iced::{Element, Length};

use crate::theme;
use super::state::ActiveMedia;
use super::{image, video, Message, MediaViewerState};

/// Render the appropriate sub-view based on which media type is active.
/// `is_fullscreen` is forwarded to sub-views so they can show the correct button icon.
pub fn view(state: &MediaViewerState, is_fullscreen: bool) -> Element<'_, Message> {
    match &state.active {
        ActiveMedia::Video => {
            video::view::view(&state.video, is_fullscreen).map(Message::Video)
        }
        ActiveMedia::Image => {
            image::view::view(&state.image, is_fullscreen).map(Message::Image)
        }
        ActiveMedia::Unsupported => unsupported_file_view(),
        ActiveMedia::None => container(text("🎬").size(48).color(theme::TEXT_MUTED))
            .center(Length::Fill)
            .into(),
    }
}

fn unsupported_file_view() -> Element<'static, Message> {
    container(
        text("📄").size(72).color(theme::TEXT_MUTED),
    )
    .center(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::panel_container_style)
    .into()
}
