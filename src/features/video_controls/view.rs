//! UI rendering for video controls feature

use iced::widget::{button, container, mouse_area, row, text, tooltip, Space};
use iced::{Element, Length};

use super::progress_bar::{BarMarker, ProgressBar};
use super::{Message, VideoControlsState};
use crate::theme;

const CONTROLS_HEIGHT: f32 = 32.0;
/// ↺ and ↻ sit side by side as one pair, a little narrower than the other buttons.
const ROTATE_BUTTON_WIDTH: f32 = 24.0;

/// Render the video player controls.
/// `position_secs` is the live playback position read from the video at view time.
/// `segment_start` and `segment_end` are the optional segment markers (in seconds) for the current file.
/// `can_add_markers` is false when the file cannot hold markers; `marker_held` shows 📍 pressed;
/// `cannot_rotate` says why the file cannot be turned (↺ ↻ are off then, with it as their
/// tooltip).
/// The progress bar is not part of it: the caller puts [`progress_bar`] on a row of its own
/// above the buttons, which keep to the left, the volume to the right.
pub fn view(
    state: &VideoControlsState,
    can_add_markers: bool,
    marker_held: bool,
    cannot_rotate: Option<String>,
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
        text(fl!("video-controls-set-in")),
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
        text(fl!("video-controls-set-out")),
        tooltip::Position::Top,
    )
    .into();

    // The bar is on its own row above: a gap pushes the volume to the right.
    let bar: Element<'_, Message> = Space::new().width(Length::Fill).into();

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
        text(fl!("video-controls-screenshot")),
        tooltip::Position::Top,
    )
    .into();

    let add_marker_btn: Element<'_, Message> = tooltip(
        // Held like `F2`: pressed, a marker starts; released (or left), it ends there, so
        // holding the button while the clip plays marks a range. The content takes the press,
        // so the button's own message is only there to draw it enabled.
        button({
            let icon = container(text("📍").size(14))
                .center_x(iced::Length::Fill)
                .center_y(iced::Length::Fill);
            let held: Element<'_, Message> = if can_add_markers {
                mouse_area(icon)
                    .on_press(Message::MarkerKeyPressed)
                    .on_release(Message::MarkerKeyReleased)
                    .on_exit(Message::MarkerKeyReleased)
                    .into()
            } else {
                icon.into()
            };
            held
        })
        .on_press_maybe(can_add_markers.then_some(Message::MarkerKeyReleased))
        .width(CONTROLS_HEIGHT)
        .height(iced::Length::Fill)
        .padding(0)
        // Pressed while `F2` or the button is held: a marker is being drawn.
        .style(move |t, status| {
            let status = if marker_held {
                button::Status::Pressed
            } else {
                status
            };
            theme::icon_button_style(can_add_markers)(t, status)
        }),
        text(if can_add_markers {
            fl!("video-controls-add-marker")
        } else {
            fl!("video-controls-cannot-hold-markers")
        }),
        tooltip::Position::Top,
    )
    .into();

    let can_rotate = cannot_rotate.is_none();
    let rotate_btn =
        |icon: &'static str, quarter_turns: i32, tip: String| -> Element<'_, Message> {
            tooltip(
                button(
                    container(text(icon).size(16))
                        .center_x(iced::Length::Fill)
                        .center_y(iced::Length::Fill),
                )
                .on_press_maybe(can_rotate.then_some(Message::Rotate(quarter_turns)))
                .width(ROTATE_BUTTON_WIDTH)
                .height(iced::Length::Fill)
                .padding(0)
                .style(theme::icon_button_style(can_rotate)),
                text(cannot_rotate.clone().unwrap_or(tip)),
                tooltip::Position::Top,
            )
            .into()
        };
    let rotate_left_btn = rotate_btn("↺", -1, fl!("video-controls-rotate-left"));
    let rotate_right_btn = rotate_btn("↻", 1, fl!("video-controls-rotate-right"));

    // The two turns are one tight pair, and the row is packed a little closer than before
    // they came, so the volume still fits the default player width.
    let rotate_pair: Element<'_, Message> = row![rotate_left_btn, rotate_right_btn]
        .height(iced::Length::Fill)
        .into();

    let controls = row![
        back10_btn,
        play_pause_btn,
        forward10_btn,
        seg_in_btn,
        seg_out_btn,
        screenshot_btn,
        add_marker_btn,
        rotate_pair,
        bar,
        volume_icon,
        volume_bar
    ]
    .spacing(4)
    .height(iced::Length::Fill)
    .align_y(iced::Alignment::Center);

    container(controls)
        .padding([0, 8])
        .width(iced::Length::Fill)
        .height(CONTROLS_HEIGHT)
        .style(theme::panel_container_style)
        .into()
}

/// The seek bar with the in/out range, the clip markers as pins and the label of the marker the
/// playhead is on, filling the width it is given.
pub fn progress_bar<'a>(
    state: &'a VideoControlsState,
    position_secs: f32,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    markers: Vec<BarMarker>,
    marker_label: Option<MarkerLabel<'a>>,
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
        .on_marker_span(Message::SetMarkerSpan)
        .on_new_range(!state.is_playing(), Message::AddRange)
        .on_play_range(Message::PlayRange)
        .label(marker_label.map(|label| (label.at, marker_label_button(label))))
        .label_right_edge(marker_label.and_then(|label| label.right_edge))
        .into()
}

/// The marker the playhead is on, as the progress bar labels it.
#[derive(Debug, Clone, Copy)]
pub struct MarkerLabel<'a> {
    /// Where the label is centred, in seconds: a point's tick, or the middle of a range.
    pub at: f32,
    pub name: &'a str,
    /// `None` for a marker frename cannot change (it has no GUID).
    pub guid: Option<&'a str>,
    /// Its pin's color: the label is the pin's head, outlined in it.
    pub color: iced::Color,
    /// The player's right edge (window x) the label stays left of; `None` in fullscreen,
    /// where the player is the whole window.
    pub right_edge: Option<f32>,
}

/// Room the label's padding, border and ✎ take besides the name (px).
const LABEL_CHROME: f32 = 48.0;
/// A generous average width of a character of the label's 12 px text (px).
const LABEL_CHAR_WIDTH: f32 = 6.8;

/// `name`, cut with "…" so the label fits a player `width` px wide.
fn fit_label(name: &str, width: f32) -> String {
    let room = ((width - LABEL_CHROME) / LABEL_CHAR_WIDTH).max(1.0) as usize;
    if name.chars().count() <= room {
        return name.to_string();
    }
    let cut: String = name.chars().take(room.saturating_sub(1)).collect();
    format!("{}…", cut.trim_end())
}

/// The label over the marker's tick: its name and ✎. A click opens the marker's row in the
/// marker list with the name field focused.
fn marker_label_button(label: MarkerLabel<'_>) -> Element<'_, Message> {
    let name = if label.name.trim().is_empty() {
        text(fl!("video-controls-add-a-name"))
            .size(12)
            .color(theme::TEXT_MUTED)
    } else {
        let fitted = match label.right_edge {
            Some(width) => fit_label(label.name, width),
            None => label.name.to_string(),
        };
        text(fitted)
            .size(12)
            .color(theme::TEXT)
            .wrapping(iced::widget::text::Wrapping::None)
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
        .style(theme::marker_label_style(label.color))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_marker_name_is_cut_to_the_player() {
        assert_eq!(fit_label("Lion", 400.0), "Lion");
        let long = "Close-up of the blue Turkish Airlines sign hanging from the ceiling";
        let fitted = fit_label(long, 300.0);
        assert!(fitted.ends_with('…'), "{fitted}");
        assert!(fitted.chars().count() as f32 * LABEL_CHAR_WIDTH + LABEL_CHROME <= 300.0);
    }
}
