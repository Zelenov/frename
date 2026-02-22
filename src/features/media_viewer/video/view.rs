//! View for the video player sub-feature.

use iced::widget::{column, container, mouse_area, text};
use iced::Element;
use iced_video_player::VideoPlayer;

use crate::features::video_controls;
use crate::theme;
use super::{Message, VideoPlayerState};

/// Render the video player with controls below.
pub fn view(state: &VideoPlayerState) -> Element<'_, Message> {
    if let Some(video) = state.current_video() {
        let player = VideoPlayer::new(video)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .content_fit(iced::ContentFit::Contain)
            .on_end_of_stream(Message::EndOfStream);

        let video_area = mouse_area(
            container(player)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .style(theme::panel_container_style),
        )
        .on_press(Message::TogglePause);

        let position_secs = video.position().as_secs_f32();
        let controls =
            video_controls::view::view(state.controls(), position_secs).map(Message::Controls);

        column![video_area, controls]
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    } else if state.is_loading() {
        container(text("⏳").size(48).color(theme::TEXT_MUTED))
            .center(iced::Length::Fill)
            .into()
    } else if state.load_failed() {
        container(text("✕").size(80).color(theme::ERROR))
            .center(iced::Length::Fill)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(theme::panel_container_style)
            .into()
    } else {
        container(text("🎬").size(48).color(theme::TEXT_MUTED))
            .center(iced::Length::Fill)
            .into()
    }
}
