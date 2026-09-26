//! View for the video player sub-feature.

use frename_core::{Marker, Subtitles};
use iced::widget::{
    button, column, container, mouse_area, row, scrollable, stack, text, tooltip, Column,
};
use iced::{Alignment, Element, Length};
use iced_video_player::VideoPlayer;

use super::{Message, Overlay, VideoPlayerState};
use crate::features::markers::{self, MarkersState};
use crate::features::video_controls::{self, BarMarker};
use crate::theme;

const CONTROLS_HEIGHT: f32 = 32.0;
/// Fixed so the video does not jump as cues of one or two lines come and go.
const SUBTITLE_STRIP_HEIGHT: f32 = 48.0;
const SUBTITLE_STRIP_TEXT_SIZE: f32 = 14.0;
const FULLSCREEN_CUE_TEXT_SIZE: f32 = 28.0;
const CUE_LIST_WIDTH: f32 = 340.0;
const CUE_LIST_TEXT_SIZE: f32 = 13.0;
/// Every row has the same height so the list can be scrolled to a cue by arithmetic.
/// Room for the time plus three lines of text; longer cues are clipped.
const CUE_ROW_HEIGHT: f32 = 78.0;
const CUE_ROW_SPACING: f32 = 2.0;
/// Distance from one row's top to the next.
pub const CUE_ROW_PITCH: f32 = CUE_ROW_HEIGHT + CUE_ROW_SPACING;
pub const CUE_LIST_SCROLLABLE_ID: &str = "subtitle_cue_list";

/// The open file's clip markers and the list's state, as the video view shows them.
#[derive(Clone, Copy)]
pub struct MarkersView<'a> {
    /// `None` when the file cannot hold markers.
    pub markers: Option<&'a [Marker]>,
    pub state: &'a MarkersState,
    /// Width of the video pane, which starts at the window's left edge. Windowed only:
    /// fullscreen, the player is the whole window.
    pub pane_width: f32,
}

/// Render the video player with controls below.
/// `is_fullscreen` controls which icon the fullscreen button shows.
/// `segment_start` and `segment_end` are passed to the progress bar for highlighting.
pub fn view<'a>(
    state: &'a VideoPlayerState,
    is_fullscreen: bool,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    markers: MarkersView<'a>,
) -> Element<'a, Message> {
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
        .on_press(Message::TogglePause)
        .on_double_click(Message::ToggleFullscreen);

        // One position for the caption, the lists and the progress bar, so they always agree.
        let position = state.display_position();
        let position_secs = position.as_secs_f32();
        let subtitles = state.subtitles();
        let active_cue = subtitles.and_then(|s| s.cue_index_at(position));
        // The list keeps the last cue lit through the gaps, so the place is never lost.
        let list_cue = subtitles.and_then(|s| s.last_started_index(position));
        let can_hold_markers = markers.markers.is_some();
        let caption = |subs| {
            if is_fullscreen {
                cue_text(subs, active_cue)
            } else {
                ""
            }
        };

        // Fullscreen lays the subtitles over the picture; otherwise they get a strip
        // of their own between the picture and the controls. The side list shows the
        // markers or the subtitles when they were asked for (CC / ◆, in fullscreen too).
        let strip = subtitles
            .filter(|_| !is_fullscreen)
            .map(|subs| subtitle_strip(subs, active_cue));
        let side_list: Option<Element<'_, Message>> = if state.show_marker_list() {
            let list = markers::view::view(markers.markers, markers.state, state.position_ms())
                .map(Message::Markers);
            let tabs = subtitles.map(|_| overlay_tabs(Overlay::Markers));
            Some(side_overlay(
                subtitles.map_or("", caption),
                tabs,
                Some(list),
            ))
        } else {
            subtitles
                .filter(|_| is_fullscreen || state.show_cue_list())
                .map(|subs| {
                    let list = state.show_cue_list().then(|| cue_list(subs, list_cue));
                    let tabs = (list.is_some() && can_hold_markers)
                        .then(|| overlay_tabs(Overlay::Subtitles));
                    side_overlay(caption(subs), tabs, list)
                })
        };
        let video_area: Element<'_, Message> = match side_list {
            Some(list) => stack![video_area, list]
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            None => video_area.into(),
        };

        let cue_list_btn: Option<Element<'_, Message>> =
            subtitles.map(|_| cue_list_button(state.show_cue_list()));
        let marker_list_btn = marker_list_button(state.show_marker_list());

        // Like a subtitle line: the name of the marker the playhead is on, as its pin's head.
        let labelled = markers
            .markers
            .and_then(|m| markers::view::marker_at(m, state.position_ms()));
        let bar_markers = markers
            .markers
            .unwrap_or_default()
            .iter()
            .map(|m| BarMarker {
                start: m.start_ms as f32 / 1000.0,
                end: m.end_ms() as f32 / 1000.0,
                color: theme::marker_color(m.color),
                active: labelled.is_some_and(|l| std::ptr::eq(l, m)),
            })
            .collect();
        let marker_label = labelled.map(|m| video_controls::view::MarkerLabel {
            at: m.start_ms as f32 / 1000.0,
            name: m.name.as_str(),
            guid: m.guid.as_deref(),
            color: theme::marker_color(m.color),
            // Within the player, not within the bar.
            right_edge: (!is_fullscreen).then_some(markers.pane_width),
        });
        let controls_inner = video_controls::view::view(
            state.controls(),
            position_secs,
            segment_start,
            segment_end,
            bar_markers,
            marker_label,
            markers.markers.is_some(),
        )
        .map(Message::Controls);

        let notice: Option<Element<'_, Message>> = state.notice().map(|notice| {
            container(text(notice).size(12).color(theme::TEXT_SOFT))
                .padding([0, 8])
                .center_y(Length::Fill)
                .into()
        });

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
            row![controls_inner]
                .push(notice)
                // Same order as the tabs over the side list: Subtitles, Markers.
                .push(cue_list_btn)
                .push(marker_list_btn)
                .push(fullscreen_btn)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(CONTROLS_HEIGHT)
        .style(theme::panel_container_style);

        column![video_area]
            .push(strip)
            .push(controls)
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

/// Text of the cue at `index`, or nothing between cues.
fn cue_text(subtitles: &Subtitles, index: Option<usize>) -> &str {
    index
        .and_then(|i| subtitles.cues().get(i))
        .map_or("", |cue| cue.text.as_str())
}

/// The current cue on a strip of its own below the picture (windowed mode).
fn subtitle_strip<'a>(subtitles: &'a Subtitles, active_cue: Option<usize>) -> Element<'a, Message> {
    container(
        text(cue_text(subtitles, active_cue))
            .size(SUBTITLE_STRIP_TEXT_SIZE)
            .color(theme::TEXT)
            .align_x(Alignment::Center),
    )
    .center_x(Length::Fill)
    .align_y(Alignment::Center)
    .height(SUBTITLE_STRIP_HEIGHT)
    .padding([4, 12])
    .clip(true)
    .style(theme::panel_container_style)
    .into()
}

/// Controls-bar button that shows or hides the subtitle list in windowed mode.
fn cue_list_button<'a>(shown: bool) -> Element<'a, Message> {
    tooltip(
        button(
            container(
                text("CC")
                    .size(11)
                    .color(if shown { theme::ACCENT } else { theme::TEXT }),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        )
        .on_press(Message::ToggleCueList)
        .width(CONTROLS_HEIGHT)
        .height(CONTROLS_HEIGHT)
        .padding(0)
        .style(theme::icon_button_style(true)),
        text(if shown {
            "Hide subtitle list"
        } else {
            "Show subtitle list"
        }),
        tooltip::Position::Top,
    )
    .into()
}

/// Controls-bar button that shows or hides the marker list, in fullscreen too.
fn marker_list_button<'a>(shown: bool) -> Element<'a, Message> {
    tooltip(
        button(
            container(
                text("◆")
                    .size(12)
                    .color(if shown { theme::ACCENT } else { theme::TEXT }),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        )
        .on_press(Message::ToggleMarkerList)
        .width(CONTROLS_HEIGHT)
        .height(CONTROLS_HEIGHT)
        .padding(0)
        .style(theme::icon_button_style(true)),
        text("Markers (Shift+F1 / Shift+F3 to jump, Shift+drag to snap)"),
        tooltip::Position::Top,
    )
    .into()
}

/// `Subtitles` and `Markers` tabs over the side list, `active` lit.
fn overlay_tabs<'a>(active: Overlay) -> Element<'a, Message> {
    // Each tab leads with the icon of its button in the controls bar.
    let tab = |icon: &'a str, label: &'a str, overlay: Overlay| {
        button(
            row![text(icon).size(11), text(label).size(12)]
                .spacing(6)
                .align_y(Alignment::Center),
        )
        .on_press(Message::ShowOverlay(overlay))
        .padding([3, 10])
        .style(theme::overlay_tab_style(active == overlay))
    };
    row![
        tab("CC", "Subtitles", Overlay::Subtitles),
        tab("◆", "Markers", Overlay::Markers)
    ]
    .spacing(4)
    .padding([6, 12])
    .into()
}

/// A list laid over the picture on the right, with `tabs` above it, plus (fullscreen only)
/// the current cue captioned over the lower part of the picture — windowed mode already
/// shows it on the strip below. Without a list the caption alone is laid over the picture.
/// Empty areas let clicks through to the video underneath (pause, double-click to toggle
/// fullscreen).
fn side_overlay<'a>(
    current: &'a str,
    tabs: Option<Element<'a, Message>>,
    list: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let caption: Element<'a, Message> = if current.is_empty() {
        iced::widget::Space::new().into()
    } else {
        container(
            text(current)
                .size(FULLSCREEN_CUE_TEXT_SIZE)
                .color(theme::TEXT)
                .align_x(Alignment::Center),
        )
        .padding([8, 18])
        .max_width(1100)
        .style(theme::subtitle_caption_style)
        .into()
    };
    let caption_area = container(caption)
        .width(Length::FillPortion(3))
        .center_x(Length::FillPortion(3))
        .align_bottom(Length::Fill)
        .padding([48, 24]);
    let Some(list) = list else {
        return caption_area.width(Length::Fill).into();
    };

    let panel = container(column![].push(tabs).push(list).height(Length::Fill))
        .width(Length::Fill)
        .max_width(CUE_LIST_WIDTH)
        .height(Length::Fill)
        .style(theme::subtitle_list_style);
    // Shares the width with the caption area so a narrow windowed pane is not covered
    // whole; on a wide screen the list stops at its max width and stays flush right.
    let panel = container(panel)
        .align_right(Length::FillPortion(2))
        .height(Length::Fill);

    row![caption_area, panel]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// A scrollable list of every cue; the one last started is lit.
fn cue_list<'a>(subtitles: &'a Subtitles, list_cue: Option<usize>) -> Element<'a, Message> {
    let rows = subtitles.cues().iter().enumerate().map(|(index, cue)| {
        let is_active = list_cue == Some(index);
        button(
            column![
                text(format_cue_time(cue.start))
                    .size(11)
                    .color(theme::TEXT_MUTED),
                text(cue.text.as_str())
                    .size(CUE_LIST_TEXT_SIZE)
                    .color(if is_active {
                        theme::TEXT
                    } else {
                        theme::TEXT_SOFT
                    }),
            ]
            .spacing(2),
        )
        .on_press(Message::SeekToCue(index))
        .width(Length::Fill)
        .height(CUE_ROW_HEIGHT)
        .clip(true)
        .padding([6, 10])
        .style(theme::cue_row_style(is_active))
        .into()
    });
    scrollable(
        Column::with_children(rows)
            .spacing(CUE_ROW_SPACING)
            .padding([0, 12]),
    )
    .id(iced::widget::Id::new(CUE_LIST_SCROLLABLE_ID))
    .height(Length::Fill)
    .style(theme::dark_scrollable_style)
    .into()
}

/// `m:ss` below an hour, `h:mm:ss` above.
fn format_cue_time(at: std::time::Duration) -> String {
    let total = at.as_secs();
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}
