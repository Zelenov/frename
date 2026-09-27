//! View for the unified media viewer feature.

use iced::widget::{container, text};
use iced::{Element, Length};

use super::state::ActiveMedia;
use super::{video, MediaViewerState, Message};
use crate::theme;

/// Render the video player, or a placeholder when no video is open.
/// `is_fullscreen` is forwarded to the video view so it can show the correct button icon.
/// `segment_start` and `segment_end` highlight the segment on the progress bar.
pub fn view<'a>(
    state: &'a MediaViewerState,
    is_fullscreen: bool,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    markers: video::view::MarkersView<'a>,
) -> Element<'a, Message> {
    match &state.active {
        ActiveMedia::Video => video::view::view(
            &state.video,
            is_fullscreen,
            segment_start,
            segment_end,
            markers,
        )
        .map(Message::Video),
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
