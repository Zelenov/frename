//! UI rendering for video player feature

use iced::widget::{column, container, text};
use iced::Element;
use iced_video_player::VideoPlayer;

use crate::features::video_controls;
use super::{Message, VideoPlayerState};

/// Render the video player with controls below
pub fn view(state: &VideoPlayerState) -> Element<'_, Message> {
    if let Some(video) = state.current_video() {
        let player = VideoPlayer::new(video)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .content_fit(iced::ContentFit::Contain)
            .on_end_of_stream(Message::EndOfStream);

        // Wrap in container so column layout sees Fill height
        // (VideoPlayer widget's size() always reports Shrink)
        let video_area = container(player)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        let controls = video_controls::view::view(state.controls()).map(Message::Controls);

        column![video_area, controls]
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    } else if state.is_loading() {
        container(text("Loading video...").size(24))
            .center(iced::Length::Fill)
            .into()
    } else {
        container(text("Drop a video file here to play").size(24))
            .center(iced::Length::Fill)
            .into()
    }
}
