//! View for the video player sub-feature (design system §13.3): the picture on black with the
//! side list over it, the subtitle strip, the timeline and the controls bar, which gives up
//! whole groups into **More** on a narrow pane.

use frename_core::{Marker, Subtitles};
use iced::widget::{column, container, mouse_area, row, space, stack, Column, Row};
use iced::{Alignment, Element, Length, Padding};
use iced_video_player::VideoPlayer;

use super::{Message, Overlay, VideoPlayerState};
use crate::features::markers::{self, MarkersState};
use crate::features::media_viewer::placeholder;
use crate::features::video_controls::view::{self as controls, Command};
use crate::features::video_controls::{self, BarMarker, Fold};
use crate::ui::icon_button::IconButton;
use crate::ui::icons::Icon;
use crate::ui::menu::{self, MenuItem};
use crate::ui::palette::marker_color;
use crate::ui::segmented::{segmented, Segment};
use crate::ui::style;
use crate::ui::tokens::*;
use crate::ui::tooltip::{self, Position, Tip};
use crate::ui::{list, scroll, text};

/// The in/out points in milliseconds, when both are set and in comes first.
fn in_out_ms(start: Option<f32>, end: Option<f32>) -> Option<(u64, u64)> {
    match (start, end) {
        (Some(start), Some(end)) if start < end => {
            Some(((start * 1000.0) as u64, (end * 1000.0) as u64))
        }
        _ => None,
    }
}

/// How tall the side list's row of a cue with `text` is, as an estimate good enough to scroll a
/// row into view: rows take their natural height, so short cues leave no holes.
fn cue_row_height(text: &str) -> f32 {
    let per_line = (CUE_TEXT_WIDTH / BODY_CHAR_WIDTH).max(1.0) as usize;
    let lines: usize = text
        .lines()
        .map(|line| line.chars().count().div_ceil(per_line).max(1))
        .sum::<usize>()
        .max(1);
    CUE_ROW_CHROME + LINE_BODY * lines as f32
}

/// Distance from the top of the cue list to the row of cue `index`.
pub fn cue_offset(subtitles: &Subtitles, index: usize) -> f32 {
    subtitles
        .cues()
        .iter()
        .take(index)
        .map(|cue| cue_row_height(&cue.text) + SPACE_XXS)
        .sum()
}
pub const CUE_LIST_SCROLLABLE_ID: &str = "subtitle_cue_list";
/// The share of a wide pane the side list takes (at most `SIDE_LIST_MAX_WIDTH`).
const LIST_SHARE_OF_PANE: f32 = 0.4;

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

/// Render the video player with its strip, timeline and controls below.
/// `is_fullscreen` controls which icon the fullscreen button shows.
/// `segment_start` and `segment_end` are passed to the progress bar for highlighting.
pub fn view<'a>(
    state: &'a VideoPlayerState,
    is_fullscreen: bool,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
    markers: MarkersView<'a>,
) -> Element<'a, Message> {
    let Some(video) = state.current_video() else {
        return if state.is_loading() {
            placeholder::loading(state.loading_name(), state.loading_ticks())
        } else if state.load_failed() {
            placeholder::cannot_play(None)
        } else {
            placeholder::blank()
        };
    };
    let player = VideoPlayer::new(video)
        .width(Length::Fill)
        .height(Length::Fill)
        .content_fit(iced::ContentFit::Contain)
        .on_end_of_stream(Message::EndOfStream);
    let picture = mouse_area(
        container(player)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::video),
    )
    .on_press(Message::TogglePause)
    .on_double_click(Message::ToggleFullscreen);

    // One position for the caption, the lists and the progress bar, so they always agree.
    let position = state.display_position();
    let position_secs = position.as_secs_f32();
    let subtitles = state.subtitles();
    let active_cue = subtitles.and_then(|s| s.cue_index_at(position));
    // Fullscreen, the whole window is the pane.
    let pane_width = if is_fullscreen {
        f32::INFINITY
    } else {
        markers.pane_width
    };
    let list_buttons = usize::from(subtitles.is_some()) + 1;
    let fold = Fold::for_width(pane_width, list_buttons);

    let mut layers: Vec<Element<'a, Message>> = vec![picture.into()];
    if let Some(side) = side_overlay(
        state,
        markers,
        is_fullscreen,
        pane_width,
        active_cue,
        in_out_ms(segment_start, segment_end),
    ) {
        layers.push(side);
    }
    // The note that says where the clip continued is always over the picture, and starts the
    // clip over when clicked (#161).
    let starts_over = state.notice_starts_over();
    if let Some(notice) = state
        .notice()
        .filter(|_| starts_over || !fold.notice_in_bar)
    {
        layers.push(placeholder::floating_notice(
            notice,
            starts_over.then_some(Message::GoToStart),
        ));
    }
    if state.more_open() && fold.has_more() {
        layers.push(more_popover(state, markers, fold));
    }
    let picture_area = stack(layers).width(Length::Fill).height(Length::Fill);

    // Fullscreen lays the subtitles over the picture; otherwise they get a strip of their own.
    let strip = subtitles
        .filter(|_| !is_fullscreen)
        .map(|subs| subtitle_strip(subs, active_cue));
    let bars_style = if is_fullscreen {
        style::video
    } else {
        style::panel
    };
    column![picture_area]
        .push(strip)
        .push(
            timeline(
                state,
                markers,
                is_fullscreen,
                position_secs,
                segment_start,
                segment_end,
            )
            .style(bars_style),
        )
        .push(controls_bar(state, markers, fold, is_fullscreen, position_secs).style(bars_style))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// The timeline row: the seek bar with the in/out segment, the markers and the label of the
/// marker the playhead is on.
fn timeline<'a>(
    state: &'a VideoPlayerState,
    markers: MarkersView<'a>,
    is_fullscreen: bool,
    position_secs: f32,
    segment_start: Option<f32>,
    segment_end: Option<f32>,
) -> container::Container<'a, Message> {
    // Like a subtitle line: the name of the marker the playhead is on, as its pin's head.
    let labelled = markers
        .markers
        .and_then(|m| markers::view::marker_at(m, state.position_ms()));
    let bar_markers: Vec<BarMarker> = markers
        .markers
        .unwrap_or_default()
        .iter()
        .map(|m| BarMarker {
            start: m.start_ms as f32 / 1000.0,
            end: m.end_ms() as f32 / 1000.0,
            color: marker_color(m.color),
            active: labelled.is_some_and(|l| std::ptr::eq(l, m)),
            guid: m.guid.clone(),
        })
        .chain(video_controls::in_out_band(segment_start, segment_end))
        .collect();
    // The label gets a lane of its own when the clip has markers, so it never covers the
    // subtitle strip above the timeline.
    let label_lane = markers.markers.is_some_and(|m| !m.is_empty());
    let marker_label = labelled.map(|m| controls::MarkerLabel {
        // Over a point's pin, or over the middle of a range's band.
        at: (m.start_ms + m.end_ms()) as f32 / 2000.0,
        name: m.name.as_str(),
        guid: m.guid.as_deref(),
        color: marker_color(m.color),
        // Within the player, not within the bar.
        right_edge: (!is_fullscreen).then_some(markers.pane_width),
    });
    // Taller when overlapping ranges stack their bands in lanes.
    let height = TIMELINE_HEIGHT.max(video_controls::bar_height(&bar_markers, label_lane));
    container(
        controls::progress_bar(
            state.controls(),
            position_secs,
            segment_start,
            segment_end,
            bar_markers,
            marker_label,
            label_lane,
        )
        .map(Message::Controls),
    )
    .padding(Padding {
        left: SPACE_M,
        right: SPACE_M,
        ..Padding::ZERO
    })
    .width(Length::Fill)
    .center_y(height)
}

/// The controls the video pane's own messages drive: the lists and fullscreen.
fn list_commands(state: &VideoPlayerState, markers: MarkersView<'_>) -> Vec<Command<Message>> {
    let subtitles = state.subtitles().map(|_| {
        Command::icon(
            Icon::Captions,
            fl!("media-viewer-video-subtitle-list"),
            &[],
            Some(Message::ToggleCueList),
        )
        .latched(state.show_cue_list())
    });
    let marker_list = Command::icon(
        Icon::MapPin,
        fl!("media-viewer-video-marker-list"),
        &[],
        Some(Message::ToggleMarkerList),
    )
    .latched(state.show_marker_list());
    let marker_list = if markers.markers.is_some() {
        marker_list.detail(fl!("media-viewer-video-markers-hint"))
    } else {
        marker_list
    };
    // Same order as the tabs over the side list: Subtitles, Markers.
    subtitles.into_iter().chain([marker_list]).collect()
}

/// Why ↺ ↻ are off; while the clip loads its rotation is not known yet, so they stay on.
fn cannot_rotate(state: &VideoPlayerState) -> Option<String> {
    state
        .rotation()
        .and_then(|read| read.as_ref().err())
        .map(|error| {
            let reason = crate::features::rotation_text::why_not_rotated(error);
            fl!("rotate-cannot", reason = reason)
        })
}

fn controls_of<'a>(
    commands: impl IntoIterator<Item = Command<video_controls::Message>>,
    quiet: bool,
) -> Row<'a, Message> {
    controls::group(
        commands
            .into_iter()
            .map(|c| c.map(Message::Controls).quiet(quiet)),
    )
}

/// The controls bar (§13.3.5): transport · in/out · mark · rotate · the notice slot · time ·
/// volume · views, the groups 12 px apart, their buttons touching.
fn controls_bar<'a>(
    state: &'a VideoPlayerState,
    markers: MarkersView<'a>,
    fold: Fold,
    is_fullscreen: bool,
    position_secs: f32,
) -> container::Container<'a, Message> {
    let controls_state = state.controls();
    let [back, play, forward] = controls::transport(controls_state);
    let transport = if fold.frame_step {
        let [frame_back, frame_forward] = controls::frame_step();
        vec![back, frame_back, play, frame_forward, forward]
    } else {
        vec![back, play, forward]
    };
    let mut groups: Vec<Element<'a, Message>> = vec![
        controls_of(transport, is_fullscreen).into(),
        controls_of(controls::in_out(), is_fullscreen).into(),
    ];
    if fold.mark {
        groups.push(controls_of(mark_commands(markers), is_fullscreen).into());
    }
    if fold.rotate {
        groups.push(controls_of(controls::rotate(cannot_rotate(state)), is_fullscreen).into());
    }
    // The free space holds the notice when it is wide enough (otherwise it floats over the
    // picture).
    let notice: Element<'a, Message> = match state
        .notice()
        .filter(|_| fold.notice_in_bar && !state.notice_starts_over())
    {
        Some(notice) => tooltip::tip_text(
            text::secondary(notice).wrapping(iced::widget::text::Wrapping::None),
            notice,
            Position::Top,
        ),
        None => space().into(),
    };
    groups.push(container(notice).width(Length::Fill).clip(true).into());
    if fold.time {
        groups.push(controls::time_readout(controls_state, position_secs).map(Message::Controls));
    }
    groups.push(if fold.volume_slider {
        controls::volume(controls_state).map(Message::Controls)
    } else {
        {
            // Folded, the slider is in More; the wheel over the button still changes the volume.
            let volume = controls_state.volume();
            mouse_area(
                IconButton::new(Icon::Volume)
                    .latched(state.more_open())
                    .quiet(is_fullscreen)
                    .tip(Tip::new(fl!("video-controls-volume-scroll")), Position::Top)
                    .on_press(Message::ToggleMore),
            )
            .on_scroll(move |delta| {
                Message::Controls(video_controls::Message::SetVolume(
                    video_controls::volume_after_scroll(volume, delta),
                ))
            })
            .into()
        }
    });
    let mut views: Vec<Element<'a, Message>> = Vec::new();
    if fold.lists {
        views.extend(
            list_commands(state, markers)
                .into_iter()
                .map(|c| c.quiet(is_fullscreen).button()),
        );
    }
    if fold.has_more() {
        views.push(
            IconButton::new(Icon::Ellipsis)
                .latched(state.more_open())
                .quiet(is_fullscreen)
                .tip(Tip::new(fl!("video-controls-more")), Position::Top)
                .on_press(Message::ToggleMore)
                .into(),
        );
    }
    views.push(
        fullscreen_command(is_fullscreen)
            .quiet(is_fullscreen)
            .button(),
    );
    groups.push(Row::with_children(views).align_y(Alignment::Center).into());

    container(
        Row::with_children(groups)
            .spacing(SPACE_M)
            .height(Length::Fill)
            .align_y(Alignment::Center),
    )
    .padding(Padding {
        left: SPACE_S,
        right: SPACE_S,
        ..Padding::ZERO
    })
    .width(Length::Fill)
    .height(BAR_HEIGHT)
}

fn mark_commands(markers: MarkersView<'_>) -> [Command<video_controls::Message>; 2] {
    controls::mark(
        markers.markers.is_some(),
        markers.state.recording().is_some(),
    )
}

fn fullscreen_command(is_fullscreen: bool) -> Command<Message> {
    let glyph = if is_fullscreen {
        Icon::Minimize
    } else {
        Icon::Maximize
    };
    Command::icon(
        glyph,
        fl!("media-viewer-video-fullscreen"),
        &["F5"],
        Some(Message::ToggleFullscreen),
    )
    .latched(is_fullscreen)
}

/// **More**, open over the bottom right of the picture: the controls the bar has no room for,
/// and the volume slider when it is folded. A click outside it closes it.
fn more_popover<'a>(
    state: &'a VideoPlayerState,
    markers: MarkersView<'a>,
    fold: Fold,
) -> Element<'a, Message> {
    let picked = Message::MorePicked;
    let mut items: Vec<MenuItem<Message>> = Vec::new();
    if !fold.frame_step {
        items.extend(
            controls::frame_step()
                .into_iter()
                .map(|c| c.map(Message::Controls).menu_item(picked)),
        );
    }
    if !fold.mark {
        items.extend(
            mark_commands(markers)
                .into_iter()
                .map(|c| c.map(Message::Controls).menu_item(picked)),
        );
    }
    if !fold.rotate {
        items.extend(
            controls::rotate(cannot_rotate(state))
                .into_iter()
                .map(|c| c.map(Message::Controls).menu_item(picked)),
        );
    }
    if !fold.lists {
        items.extend(
            list_commands(state, markers)
                .into_iter()
                .map(|c| c.menu_item(picked)),
        );
    }
    let mut rows: Vec<Element<'a, Message>> = Vec::new();
    if !fold.volume_slider {
        rows.push(
            container(controls::volume(state.controls()).map(Message::Controls))
                .padding(SPACE_S)
                .into(),
        );
    }
    rows.extend(items.into_iter().map(menu::item));
    let popup = container(menu::menu(rows, Length::Fixed(MENU_WIDTH)))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(SPACE_S)
        .align_right(Length::Fill)
        .align_bottom(Length::Fill);
    let outside = mouse_area(container(space()).width(Length::Fill).height(Length::Fill))
        .on_press(Message::CloseMore);
    stack![iced::widget::opaque(outside), popup]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Text of the cue at `index`, or nothing between cues.
fn cue_text(subtitles: &Subtitles, index: Option<usize>) -> &str {
    index
        .and_then(|i| subtitles.cues().get(i))
        .map_or("", |cue| cue.text.as_str())
}

/// The current cue on a strip of its own below the picture (windowed mode): at most two lines,
/// at a fixed height so the picture never jumps between cues.
fn subtitle_strip<'a>(subtitles: &'a Subtitles, active_cue: Option<usize>) -> Element<'a, Message> {
    container(text::subtitle(cue_text(subtitles, active_cue)).align_x(Alignment::Center))
        .center_x(Length::Fill)
        .align_y(Alignment::Center)
        .height(SUBTITLE_STRIP_HEIGHT)
        .padding(Padding {
            top: SPACE_XS,
            bottom: SPACE_XS,
            left: SPACE_M,
            right: SPACE_M,
        })
        .clip(true)
        .style(style::panel)
        .into()
}

/// How wide the side list is over a pane `pane_width` wide, and whether it covers the whole
/// picture (a narrow pane): then it has its own close button.
fn side_list_width(pane_width: f32) -> (f32, bool) {
    if pane_width < SIDE_LIST_PARTIAL_FROM {
        (pane_width, true)
    } else {
        (
            (pane_width * LIST_SHARE_OF_PANE).min(SIDE_LIST_MAX_WIDTH),
            false,
        )
    }
}

/// What lies over the picture: the side list on the right when one was asked for, and
/// (fullscreen only) the current cue captioned over the lower part of the picture the list
/// leaves free. Empty areas let clicks through to the video (pause, double-click).
fn side_overlay<'a>(
    state: &'a VideoPlayerState,
    markers: MarkersView<'a>,
    is_fullscreen: bool,
    pane_width: f32,
    active_cue: Option<usize>,
    in_out: Option<(u64, u64)>,
) -> Option<Element<'a, Message>> {
    let subtitles = state.subtitles();
    let caption = subtitles
        .filter(|_| is_fullscreen)
        .map_or("", |subs| cue_text(subs, active_cue));
    let shown = if state.show_marker_list() {
        Some(Overlay::Markers)
    } else if state.show_cue_list() && subtitles.is_some() {
        Some(Overlay::Subtitles)
    } else {
        None
    };
    let caption_area = fullscreen_caption(caption);
    let Some(shown) = shown else {
        return (!caption.is_empty()).then(|| caption_area.width(Length::Fill).into());
    };
    let list: Element<'a, Message> = match (shown, subtitles) {
        (Overlay::Subtitles, Some(subs)) => {
            cue_list(subs, subs.last_started_index(state.display_position()))
        }
        _ => markers::view::view(
            markers.markers,
            markers.state,
            state.position_ms(),
            in_out,
            is_fullscreen,
        )
        .map(Message::Markers),
    };
    let (width, covers) = side_list_width(pane_width);
    let panel = container(
        column![
            side_list_header(state, markers, shown, covers, is_fullscreen),
            list
        ]
        .height(Length::Fill),
    )
    .width(if covers {
        Length::Fill
    } else {
        Length::Fixed(width)
    })
    .height(Length::Fill)
    .style(style::overlay_list);
    Some(if covers {
        panel.into()
    } else {
        row![caption_area, panel]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    })
}

/// The fullscreen caption: a pill centred at the bottom of the area it is given.
fn fullscreen_caption<'a>(caption: &'a str) -> container::Container<'a, Message> {
    let pill: Element<'a, Message> = if caption.is_empty() {
        space().into()
    } else {
        container(text::video_caption(caption).align_x(Alignment::Center))
            .padding(Padding {
                top: CAPTION_PADDING_Y,
                bottom: CAPTION_PADDING_Y,
                left: CAPTION_PADDING_X,
                right: CAPTION_PADDING_X,
            })
            .max_width(CAPTION_MAX_WIDTH)
            .style(style::caption_pill)
            .into()
    };
    container(pill)
        .width(Length::Fill)
        .center_x(Length::Fill)
        .align_bottom(Length::Fill)
        .padding(Padding {
            bottom: CAPTION_LIFT,
            left: SPACE_XL,
            right: SPACE_XL,
            ..Padding::ZERO
        })
}

/// The side list's header: the two lists as tabs when the clip has both, otherwise the list's
/// name; with `close` (the list covers the picture) a button that closes it.
fn side_list_header<'a>(
    state: &'a VideoPlayerState,
    markers: MarkersView<'a>,
    shown: Overlay,
    close: bool,
    is_fullscreen: bool,
) -> Element<'a, Message> {
    let subtitles = state.subtitles();
    let count = |n: usize| (n > 0).then_some(n);
    let title: Element<'a, Message> = match (subtitles, markers.markers) {
        (Some(subs), Some(list)) => segmented([
            Segment {
                icon: Some(Icon::Captions),
                label: fl!("settings-subtitles"),
                count: count(subs.cues().len()),
                selected: shown == Overlay::Subtitles,
                on_press: Message::ShowOverlay(Overlay::Subtitles),
            },
            Segment {
                icon: Some(Icon::MapPin),
                label: fl!("media-viewer-video-tab-markers"),
                count: count(list.len()),
                selected: shown == Overlay::Markers,
                on_press: Message::ShowOverlay(Overlay::Markers),
            },
        ]),
        _ if shown == Overlay::Subtitles => text::title(fl!("settings-subtitles")).into(),
        _ => text::title(fl!("media-viewer-video-tab-markers")).into(),
    };
    let toggle = match shown {
        Overlay::Subtitles => Message::ToggleCueList,
        _ => Message::ToggleMarkerList,
    };
    let close = close.then(|| {
        IconButton::new(Icon::X)
            .small()
            .overlay()
            .quiet(is_fullscreen)
            .tip(
                Tip::new(fl!("media-viewer-video-close-list")),
                Position::Left,
            )
            .on_press(toggle)
    });
    container(
        row![title, space::horizontal()]
            .push(close)
            .align_y(Alignment::Center),
    )
    .padding(Padding {
        left: SPACE_S,
        right: SPACE_S,
        ..Padding::ZERO
    })
    .center_y(SIDE_LIST_HEADER_HEIGHT)
    .into()
}

/// A scrollable list of every cue; the one last started is lit.
fn cue_list<'a>(subtitles: &'a Subtitles, list_cue: Option<usize>) -> Element<'a, Message> {
    let rows = subtitles.cues().iter().enumerate().map(|(index, cue)| {
        let lit = list_cue == Some(index);
        let cue_text = text::body(cue.text.as_str()).color(if lit { TEXT } else { TEXT_SECONDARY });
        let body = column![text::mono(format_cue_time(cue.start)), cue_text].padding(Padding {
            top: SPACE_TIGHT,
            bottom: SPACE_TIGHT,
            ..Padding::ZERO
        });
        let item = list::row_item(body, lit, OVERLAY_HOVER, Length::Shrink);
        mouse_area(item)
            .on_press(Message::SeekToCue(index))
            .interaction(iced::mouse::Interaction::Pointer)
            .into()
    });
    scroll::vertical_with_id(
        CUE_LIST_SCROLLABLE_ID,
        Column::with_children(rows)
            .spacing(SPACE_XXS)
            .padding(Padding {
                left: SPACE_S,
                ..Padding::ZERO
            }),
    )
    .on_scroll(|viewport| {
        Message::CueListScrolled(viewport.absolute_offset().y, viewport.bounds().height)
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_pane_gives_the_side_list_two_fifths_at_most_its_widest() {
        assert_eq!(side_list_width(700.0), (280.0, false));
        assert_eq!(side_list_width(1600.0), (SIDE_LIST_MAX_WIDTH, false));
    }

    #[test]
    fn on_a_narrow_pane_the_side_list_covers_the_picture() {
        assert_eq!(side_list_width(500.0), (500.0, true));
    }
}
