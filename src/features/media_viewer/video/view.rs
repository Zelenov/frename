//! View for the video player sub-feature.

use frename_core::Subtitles;
use iced::widget::{
    button, column, container, mouse_area, row, scrollable, stack, text, tooltip, Column,
};
use iced::{Alignment, Element, Length};
use iced_video_player::VideoPlayer;

use super::{Message, VideoPlayerState};
use crate::features::video_controls;
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

/// Render the video player with controls below.
/// `is_fullscreen` controls which icon the fullscreen button shows.
/// `segment_start` and `segment_end` are passed to the progress bar for highlighting.
pub fn view<'a>(
    state: &'a VideoPlayerState,
    is_fullscreen: bool,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    screenshot_positions_secs: Vec<f32>,
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

        // One position for the caption, the list and the progress bar, so they always agree.
        let position = state.display_position();
        let position_secs = position.as_secs_f32();
        let subtitles = state.subtitles();
        let active_cue = subtitles.and_then(|s| s.cue_index_at(position));
        // The list keeps the last cue lit through the gaps, so the place is never lost.
        let list_cue = subtitles.and_then(|s| s.last_started_index(position));

        // Fullscreen lays the subtitles over the picture; otherwise they get a strip
        // of their own between the picture and the controls.
        let (video_area, subtitle_strip): (Element<'_, Message>, Option<Element<'_, Message>>) =
            match subtitles {
                Some(subs) if is_fullscreen => (
                    stack![
                        video_area,
                        subtitle_overlay(subs, active_cue, list_cue, true)
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
                    None,
                ),
                Some(subs) if state.show_cue_list() => (
                    stack![
                        video_area,
                        subtitle_overlay(subs, active_cue, list_cue, false)
                    ]
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
                    Some(subtitle_strip(subs, active_cue)),
                ),
                Some(subs) => (video_area.into(), Some(subtitle_strip(subs, active_cue))),
                None => (video_area.into(), None),
            };

        // Windowed mode only: fullscreen always shows the list.
        let cue_list_btn: Option<Element<'_, Message>> =
            (subtitles.is_some() && !is_fullscreen).then(|| cue_list_button(state.show_cue_list()));

        let controls_inner = video_controls::view::view(
            state.controls(),
            position_secs,
            segment_start,
            segment_end,
            screenshot_positions_secs,
        )
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
            row![controls_inner]
                .push(cue_list_btn)
                .push(fullscreen_btn)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(CONTROLS_HEIGHT)
        .style(theme::panel_container_style);

        column![video_area]
            .push(subtitle_strip)
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
            fl!("media-viewer-video-hide-subtitles")
        } else {
            fl!("media-viewer-video-show-subtitles")
        }),
        tooltip::Position::Top,
    )
    .into()
}

/// Subtitles laid over the picture: a scrollable list of every cue on the right, plus
/// (fullscreen only) the current cue captioned over the lower part of the picture —
/// windowed mode already shows it on the strip below. Empty areas let clicks through
/// to the video underneath (pause, double-click to toggle fullscreen).
fn subtitle_overlay<'a>(
    subtitles: &'a Subtitles,
    active_cue: Option<usize>,
    list_cue: Option<usize>,
    with_caption: bool,
) -> Element<'a, Message> {
    let current = if with_caption {
        cue_text(subtitles, active_cue)
    } else {
        ""
    };
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
    let list = container(
        scrollable(
            Column::with_children(rows)
                .spacing(CUE_ROW_SPACING)
                .padding([0, 12]),
        )
        .id(iced::widget::Id::new(CUE_LIST_SCROLLABLE_ID))
        .height(Length::Fill)
        .style(theme::dark_scrollable_style),
    )
    .width(Length::Fill)
    .max_width(CUE_LIST_WIDTH)
    .height(Length::Fill)
    .style(theme::subtitle_list_style);
    // Shares the width with the caption area so a narrow windowed pane is not covered
    // whole; on a wide screen the list stops at its max width and stays flush right.
    let list = container(list)
        .align_right(Length::FillPortion(2))
        .height(Length::Fill);

    row![caption_area, list]
        .width(Length::Fill)
        .height(Length::Fill)
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
