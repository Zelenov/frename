//! UI rendering for video controls feature

use iced::widget::{button, container, row, text, tooltip};
use iced::Element;

use crate::theme;
use super::progress_bar::ProgressBar;
use super::{Message, VideoControlsState};

const CONTROLS_HEIGHT: f32 = 32.0;

/// Render the video player controls.
/// `position_secs` is the live playback position read from the video at view time.
/// `segment_start` and `segment_end` are the optional segment markers (in seconds) for the current file.
pub fn view(
    state: &VideoControlsState,
    position_secs: f32,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
) -> Element<'_, Message> {
    let back10_btn: Element<'_, Message> = tooltip(
        button(
            container(text("⏪").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
            .on_press(Message::SeekBack10)
            .width(CONTROLS_HEIGHT)
            .height(iced::Length::Fill)
            .padding(0)
            .style(theme::icon_button_style(true)),
        text("F1"),
        tooltip::Position::Top,
    )
        .into();

    let play_pause_label = if state.is_playing() { "⏸" } else { "▶" };
    let play_pause_btn: Element<'_, Message> = tooltip(
        button(
            container(text(play_pause_label).size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
            .on_press(Message::TogglePlayPause)
            .width(CONTROLS_HEIGHT)
            .height(iced::Length::Fill)
            .padding(0)
            .style(theme::icon_button_style(true)),
        text("F2"),
        tooltip::Position::Top,
    )
        .into();

    let forward10_btn: Element<'_, Message> = tooltip(
        button(
            container(text("⏩").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
            .on_press(Message::SeekForward10)
            .width(CONTROLS_HEIGHT)
            .height(iced::Length::Fill)
            .padding(0)
            .style(theme::icon_button_style(true)),
        text("F3"),
        tooltip::Position::Top,
    )
        .into();

    let seg_in_btn: Element<'_, Message> = tooltip(
        button(
            container(text("[").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
            .on_press(Message::SetSegmentStart)
            .width(CONTROLS_HEIGHT)
            .height(iced::Length::Fill)
            .padding(0)
            .style(theme::icon_button_style(true)),
        text("[  Set In"),
        tooltip::Position::Top,
    )
        .into();

    let seg_out_btn: Element<'_, Message> = tooltip(
        button(
            container(text("]").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
            .on_press(Message::SetSegmentEnd)
            .width(CONTROLS_HEIGHT)
            .height(iced::Length::Fill)
            .padding(0)
            .style(theme::icon_button_style(true)),
        text("]  Set Out"),
        tooltip::Position::Top,
    )
        .into();

    let duration = state.duration_secs();
    // While seeking, show the drag position; otherwise use live video position
    let current_pos = if state.is_seeking() {
        state.seek_position_secs()
    } else {
        position_secs
    };

    let bar = ProgressBar::new(0.0..=duration, current_pos, Message::Seek)
        .on_release(Message::SeekReleased)
        .segment_range(segment_start, segment_end);

    let controls = row![back10_btn, play_pause_btn, forward10_btn, seg_in_btn, seg_out_btn, bar]
        .spacing(8)
        .height(iced::Length::Fill)
        .align_y(iced::Alignment::Center);

    container(controls)
        .padding([0, 8])
        .width(iced::Length::Fill)
        .height(CONTROLS_HEIGHT)
        .style(theme::panel_container_style)
        .into()
}
