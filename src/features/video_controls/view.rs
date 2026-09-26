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
/// the file cannot hold them.
pub fn view<'a>(
    state: &'a VideoControlsState,
    position_secs: f32,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    markers: Vec<BarMarker>,
    marker_label: Option<MarkerLabel<'a>>,
    can_add_markers: bool,
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

    let duration = state.duration_secs();
    // While seeking, show the drag position; otherwise use live video position
    let current_pos = if state.is_seeking() {
        state.seek_position_secs()
    } else {
        position_secs
    };

    let bar = ProgressBar::new(0.0..=duration, current_pos, Message::Seek)
        .on_release(Message::SeekReleased)
        .segment_range(segment_start, segment_end)
        .markers(markers)
        .label(marker_label.map(|label| (label.at, marker_label_button(label))));

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
            container(text("📍").size(14))
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

/// The marker the playhead is on, as the progress bar labels it.
#[derive(Debug, Clone, Copy)]
pub struct MarkerLabel<'a> {
    /// Where its tick is, in seconds.
    pub at: f32,
    pub name: &'a str,
    /// `None` for a marker frename cannot change (it has no GUID).
    pub guid: Option<&'a str>,
}

/// The label over the marker's tick: its name and ✎. A click opens the marker's row in the
/// marker list with the name field focused.
fn marker_label_button(label: MarkerLabel<'_>) -> Element<'_, Message> {
    let name = if label.name.trim().is_empty() {
        text("Add a name").size(12).color(theme::TEXT_MUTED)
    } else {
        text(label.name).size(12).color(theme::TEXT)
    };
    let content = row![name]
        .push(
            label
                .guid
                .map(|_| text("✎").size(11).color(theme::TEXT_MUTED)),
        )
        .spacing(6)
        .align_y(iced::Alignment::Center);
    button(content)
        .on_press_maybe(label.guid.map(|guid| Message::EditMarker(guid.to_string())))
        .padding([2, 6])
        .style(theme::marker_label_style)
        .into()
}
