//! View for the media_viewer feature: video, or a placeholder for anything else.

use iced::widget::{container, stack, text};
use iced::{Element, Length};

use super::state::ActiveMedia;
use super::{video, MediaViewerState, Message};
use crate::theme;

/// Widest a note over a placeholder gets before it wraps (px); matches the video view's own.
const NOTICE_MAX_WIDTH: f32 = 260.0;

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
        // The video component (the only place a note is normally drawn) is not on screen here,
        // but a save can be refused right as the video unloads for the next file or a batch job
        // (issue #84) — its note must still reach the editor, not vanish behind the placeholder.
        ActiveMedia::Unsupported => with_notice(state, unsupported_file_view()),
        ActiveMedia::None => with_notice(
            state,
            container(text("🎬").size(48).color(theme::TEXT_MUTED))
                .center(Length::Fill)
                .into(),
        ),
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

fn with_notice<'a>(
    state: &'a MediaViewerState,
    base: Element<'a, Message>,
) -> Element<'a, Message> {
    let Some(notice) = state.notice() else {
        return base;
    };
    let place = container(
        container(text(notice).size(13).color(theme::TEXT))
            .max_width(NOTICE_MAX_WIDTH)
            .padding([4, 10])
            .style(theme::panel_container_style),
    )
    .padding(8)
    .width(Length::Fill)
    .height(Length::Fill)
    .align_left(Length::Fill)
    .align_bottom(Length::Fill);
    stack![base, place]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
