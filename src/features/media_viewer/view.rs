//! View for the media_viewer feature: video, or a placeholder for anything else.

use iced::widget::stack;
use iced::{Element, Length};

use super::placeholder;
use super::state::ActiveMedia;
use super::{video, MediaViewerState, Message};

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
        ActiveMedia::Unsupported => with_notice(
            state,
            placeholder::cannot_play(Some(fl!("media-viewer-no-picture"))),
        ),
        ActiveMedia::None => with_notice(state, placeholder::blank()),
    }
}

fn with_notice<'a>(
    state: &'a MediaViewerState,
    base: Element<'a, Message>,
) -> Element<'a, Message> {
    let Some(notice) = state.notice() else {
        return base;
    };
    stack![base, placeholder::floating_notice(notice, None)]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
