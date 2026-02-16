//! UI for the video player. Only this module knows how the player and controls look.

use iced::widget::{column, container, text};
use iced::{Background, Element};
use iced_video_player::VideoPlayer;

use crate::features::video_controls;
use crate::theme;
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
            .height(iced::Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(theme::BG_PANEL)),
                ..container::Style::default()
            });

        // Read live position from the video at view time (like the Slider pattern)
        let position_secs = video.position().as_secs_f32();
        let controls =
            video_controls::view::view(state.controls(), position_secs).map(Message::Controls);

        column![video_area, controls]
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    } else if state.is_loading() {
        container(
            text("⏳")
                .size(48)
                .color(theme::TEXT_MUTED),
        )
        .center(iced::Length::Fill)
        .into()
    } else if state.load_failed() {
        container(
            text("✕")
                .size(80)
                .color(theme::ERROR),
        )
        .center(iced::Length::Fill)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme::BG_PANEL)),
            ..container::Style::default()
        })
        .into()
    } else {
        container(
            text("🎬")
                .size(48)
                .color(theme::TEXT_MUTED),
        )
        .center(iced::Length::Fill)
        .into()
    }
}
