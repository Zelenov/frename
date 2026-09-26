//! View for the unified media viewer feature.

use iced::widget::{container, text};
use iced::{Element, Length};

use super::state::ActiveMedia;
use super::{image, video, MediaViewerState, Message};
use crate::theme;

/// Render the appropriate sub-view based on which media type is active.
/// `is_fullscreen` is forwarded to sub-views so they can show the correct button icon.
/// `segment_start` and `segment_end` are only used for video (progress bar highlight).
pub fn view<'a>(
    state: &'a MediaViewerState,
    is_fullscreen: bool,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    screenshot_positions_secs: Vec<f32>,
) -> Element<'a, Message> {
    match &state.active {
        ActiveMedia::Video => video::view::view(
            &state.video,
            is_fullscreen,
            segment_start,
            segment_end,
            screenshot_positions_secs,
        )
        .map(Message::Video),
        ActiveMedia::Image => image::view::view(&state.image, is_fullscreen).map(Message::Image),
        ActiveMedia::Unsupported => unsupported_file_view(),
        ActiveMedia::None => container(text("🎬").size(48).color(theme::TEXT_MUTED))
            .center(Length::Fill)
            .into(),
    }
}

fn unsupported_file_view() -> Element<'static, Message> {
    container(text("📄").size(72).color(theme::TEXT_MUTED))
        .center(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_container_style)
        .into()
}
