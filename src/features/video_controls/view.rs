//! UI rendering for video controls feature

use iced::widget::{button, container, row, text, tooltip, Space};
use iced::{Element, Length};

use super::progress_bar::{BarMarker, ProgressBar};
use super::{Message, VideoControlsState};
use crate::theme;

const CONTROLS_HEIGHT: f32 = 32.0;

/// Render the video player controls.
/// `position_secs` is the live playback position read from the video at view time.
/// `segment_start` and `segment_end` are the optional segment markers (in seconds) for the current file.
/// `markers` are the clip markers (in seconds) drawn on the bar; `can_add_markers` is false when
/// the file cannot hold them. With `bar_on_own_row` the progress bar is left out (the caller puts
/// [`progress_bar`] on a row of its own) and the buttons keep to the left, the volume to the right.
#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    state: &'a VideoControlsState,
    position_secs: f32,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    markers: Vec<BarMarker>,
    can_add_markers: bool,
    bar_on_own_row: bool,
) -> Element<'a, Message> {
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
        text("Space"),
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

    // On a row of its own the bar is left out here and a gap pushes the volume to the right.
    let bar: Element<'_, Message> = if bar_on_own_row {
        Space::new().width(Length::Fill).into()
    } else {
        progress_bar(state, position_secs, segment_start, segment_end, markers)
    };

    let volume_icon: Element<'_, Message> =
        container(text("🔊").size(13)).center_y(Length::Fill).into();

    let volume_bar: Element<'_, Message> = container(
        ProgressBar::new(0.0..=1.0, state.volume(), Message::SetVolume).fill_color(theme::VOLUME),
    )
    .width(Length::Fixed(72.0))
    .center_y(Length::Fill)
    .into();

    let screenshot_btn: Element<'_, Message> = tooltip(
        button(
            container(text("📷").size(16))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
        .on_press(Message::TakeScreenshot)
        .width(CONTROLS_HEIGHT)
        .height(iced::Length::Fill)
        .padding(0)
        .style(theme::icon_button_style(true)),
        text("Save this frame as a JPEG (F12)"),
        tooltip::Position::Top,
    )
    .into();

    let add_marker_btn: Element<'_, Message> = tooltip(
        button(
            container(text("◆+").size(13))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill),
        )
        .on_press_maybe(can_add_markers.then_some(Message::AddMarker))
        .width(CONTROLS_HEIGHT)
        .height(iced::Length::Fill)
        .padding(0)
        .style(theme::icon_button_style(can_add_markers)),
        text(if can_add_markers {
            "Add marker (F2, again to name it)"
        } else {
            "This file cannot hold markers"
        }),
        tooltip::Position::Top,
    )
    .into();

    let controls = row![
        back10_btn,
        play_pause_btn,
        forward10_btn,
        seg_in_btn,
        seg_out_btn,
        screenshot_btn,
        add_marker_btn,
        bar,
        Space::new().width(8),
        volume_icon,
        volume_bar
    ]
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

/// The seek bar with the in/out range and the clip markers, filling the width it is given.
pub fn progress_bar<'a>(
    state: &'a VideoControlsState,
    position_secs: f32,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    markers: Vec<BarMarker>,
) -> Element<'a, Message> {
    // While seeking, show the drag position; otherwise use live video position
    let current_pos = if state.is_seeking() {
        state.seek_position_secs()
    } else {
        position_secs
    };
    ProgressBar::new(0.0..=state.duration_secs(), current_pos, Message::Seek)
        .on_release(Message::SeekReleased)
        .segment_range(segment_start, segment_end)
        .markers(markers)
        .into()
}
