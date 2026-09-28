//! UI rendering for video controls feature

use iced::widget::{button, container, mouse_area, responsive, row, text, tooltip, Space};
use iced::{mouse, Alignment, Element, Length};

use super::progress_bar::{BarMarker, ProgressBar};
use super::{Message, VideoControlsState};
use crate::theme;

const CONTROLS_HEIGHT: f32 = 32.0;
/// ↺ and ↻ sit side by side as one pair, a little narrower than the other buttons.
const ROTATE_BUTTON_WIDTH: f32 = 24.0;
/// Room between items in the controls row (and inside the collapsed volume control).
const ROW_SPACING: f32 = 4.0;
/// Left + right padding of the controls row.
const ROW_PADDING: f32 = 16.0;
/// Fixed, rather than left to the "🔊" glyph's own metrics, so the width used to decide
/// when it and the buttons around it still fit (below) is exact, not a font-dependent guess.
const VOLUME_ICON_WIDTH: f32 = 24.0;
const VOLUME_BAR_WIDTH: f32 = 72.0;
/// How much one wheel notch changes the volume when the bar itself has no room to drag (#96).
const VOLUME_SCROLL_STEP: f32 = 0.05;

/// Width of the seven square buttons (⏪ ▶ ⏩ [ ] 📷 📍) plus the ↺↻ pair, with the gaps
/// between all eight of them — everything in the row except the volume control.
const BUTTONS_CONTENT_WIDTH: f32 = CONTROLS_HEIGHT * 7.0 + ROTATE_BUTTON_WIDTH * 2.0;
const BUTTONS_GAPS: f32 = ROW_SPACING * 7.0;

/// Narrowest the controls row can be and still show every button plus a reachable volume
/// control (icon only, no bar): the video panel's own splitter minimum is set to this so
/// nothing in the row is ever clipped, at any width frename allows (#96).
pub const MIN_CONTROLS_WIDTH: f32 =
    BUTTONS_CONTENT_WIDTH + BUTTONS_GAPS + ROW_SPACING + VOLUME_ICON_WIDTH + ROW_PADDING;

/// Below this, the volume bar is dropped in favor of the icon alone (scroll it to change
/// the volume) rather than being clipped or pushed off the row's right edge (#96).
const VOLUME_BAR_MIN_WIDTH: f32 = MIN_CONTROLS_WIDTH + ROW_SPACING + VOLUME_BAR_WIDTH;

// The relation between the constants above, checked once at compile time rather than as a
// runtime test (clippy's own suggestion for an assertion on values that never change): the
// panel's floor must actually be wider than the buttons alone, and narrower than the point
// where the bar fits, or one of the two thresholds would be pointless.
const _: () = assert!(MIN_CONTROLS_WIDTH > BUTTONS_CONTENT_WIDTH + BUTTONS_GAPS);
const _: () = assert!(MIN_CONTROLS_WIDTH < VOLUME_BAR_MIN_WIDTH);

/// Whether the controls row is wide enough to show the volume bar (not just its icon), at
/// `available_width` (#96).
fn show_volume_bar(available_width: f32) -> bool {
    available_width >= VOLUME_BAR_MIN_WIDTH
}

/// New volume after one wheel notch over the collapsed volume icon, clamped to 0.0..=1.0.
fn volume_after_scroll(current: f32, delta: mouse::ScrollDelta) -> f32 {
    let y = match delta {
        mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => y,
    };
    let step = if y > 0.0 {
        VOLUME_SCROLL_STEP
    } else if y < 0.0 {
        -VOLUME_SCROLL_STEP
    } else {
        0.0
    };
    (current + step).clamp(0.0, 1.0)
}

/// Render the video player controls.
/// `position_secs` is the live playback position read from the video at view time.
/// `segment_start` and `segment_end` are the optional segment markers (in seconds) for the current file.
/// `can_add_markers` is false when the file cannot hold markers; `marker_held` shows 📍 pressed;
/// `cannot_rotate` says why the file cannot be turned (↺ ↻ are off then, with it as their
/// tooltip).
/// The progress bar is not part of it: the caller puts [`progress_bar`] on a row of its own
/// above the buttons, which keep to the left, the volume to the right.
/// Below a width threshold the volume bar gives way to the icon alone (#96) — measured live via
/// `responsive`, so this also covers fullscreen, where the row is the whole window.
pub fn view<'a>(
    state: &'a VideoControlsState,
    can_add_markers: bool,
    marker_held: bool,
    cannot_rotate: Option<String>,
) -> Element<'a, Message> {
    responsive(move |size| {
        view_at_width(
            state,
            can_add_markers,
            marker_held,
            cannot_rotate.clone(),
            size.width,
        )
    })
    .width(Length::Fill)
    .height(Length::Fixed(CONTROLS_HEIGHT))
    .into()
}

fn view_at_width(
    state: &VideoControlsState,
    can_add_markers: bool,
    marker_held: bool,
    cannot_rotate: Option<String>,
    available_width: f32,
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

    let volume_icon = || {
        container(text("🔊").size(13))
            .width(Length::Fixed(VOLUME_ICON_WIDTH))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
    };

    // Below VOLUME_BAR_MIN_WIDTH the bar has nowhere to go without clipping something (#96):
    // the icon alone stays, and the wheel changes the volume instead of a drag on the bar.
    let volume_control: Element<'_, Message> = if show_volume_bar(available_width) {
        let volume_bar: Element<'_, Message> = container(
            ProgressBar::new(0.0..=1.0, state.volume(), Message::SetVolume)
                .fill_color(theme::VOLUME),
        )
        .width(Length::Fixed(VOLUME_BAR_WIDTH))
        .center_y(Length::Fill)
        .into();
        row![volume_icon(), volume_bar]
            .spacing(ROW_SPACING)
            .align_y(Alignment::Center)
            .into()
    } else {
        let volume = state.volume();
        mouse_area(volume_icon())
            .on_scroll(move |delta| Message::SetVolume(volume_after_scroll(volume, delta)))
            .into()
    };

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
        volume_control
    ]
    .spacing(ROW_SPACING)
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

    /// Below the threshold the bar has no room and gives way to the icon; at or above it, it
    /// fits (#96) — including at the panel's own minimum, the narrowest this is ever asked.
    #[test]
    fn the_volume_bar_only_shows_once_it_fits() {
        assert!(!show_volume_bar(MIN_CONTROLS_WIDTH));
        assert!(!show_volume_bar(VOLUME_BAR_MIN_WIDTH - 1.0));
        assert!(show_volume_bar(VOLUME_BAR_MIN_WIDTH));
        assert!(show_volume_bar(1600.0));
    }

    #[test]
    fn scrolling_up_raises_volume_and_down_lowers_it_clamped() {
        let up = mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 };
        let down = mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 };
        assert_eq!(volume_after_scroll(0.5, up), 0.5 + VOLUME_SCROLL_STEP);
        assert_eq!(volume_after_scroll(0.5, down), 0.5 - VOLUME_SCROLL_STEP);
        assert_eq!(volume_after_scroll(1.0, up), 1.0, "clamped at the top");
        assert_eq!(volume_after_scroll(0.0, down), 0.0, "clamped at the bottom");
        let no_move = mouse::ScrollDelta::Pixels { x: 0.0, y: 0.0 };
        assert_eq!(volume_after_scroll(0.5, no_move), 0.5);
    }
}
