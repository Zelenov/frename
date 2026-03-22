//! View for the video player sub-feature.

use iced::widget::{button, column, container, mouse_area, row, text, tooltip};
use iced::{Element, Length};
use iced_video_player::VideoPlayer;

use crate::features::video_controls;
use crate::theme;
use super::{Message, VideoPlayerState};

const CONTROLS_HEIGHT: f32 = 32.0;

/// Render the video player with controls below.
/// `is_fullscreen` controls which icon the fullscreen button shows.
/// `segment_start` and `segment_end` are passed to the progress bar for highlighting.
pub fn view(
    state: &VideoPlayerState,
    is_fullscreen: bool,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
) -> Element<'_, Message> {
    if let Some(video) = state.current_video() {
        let player = VideoPlayer::new(video)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Contain)
            .on_end_of_stream(Message::EndOfStream);

        let video_area = mouse_area(
            container(player)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::panel_container_style),
        )
        .on_press(Message::TogglePause);

        let position_secs = video.position().as_secs_f32();
        let controls_inner =
            video_controls::view::view(state.controls(), position_secs, segment_start, segment_end)
                .map(Message::Controls);

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

        let controls = container(
            row![controls_inner, fullscreen_btn]
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(CONTROLS_HEIGHT)
        .style(theme::panel_container_style);

        column![video_area, controls]
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
        container(text("🎬").size(48).color(theme::TEXT_MUTED))
            .center(Length::Fill)
            .into()
    }
}
