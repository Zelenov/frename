//! The marker list (design system §13.3.6): one row per marker, read-only like a subtitle cue,
//! except the one row open for renaming. A row is as tall as its name's lines; the list is
//! scrolled to a row by an estimate of the rows above it ([`row_offset`]), as the subtitle list is.

use std::borrow::Cow;

use frename_core::{format_marker_time, Marker, MarkerColor, AI_MARKER_COLOR, MARKER_SNAP_MS};
use iced::widget::{
    button, column, container, hover, mouse_area, row, space, stack, text_editor, Column,
};
use iced::{Alignment, Background, Border, Element, Length, Padding};

use super::{MarkersState, Message};
use crate::ui::button::{self as ui_button};
use crate::ui::icon_button::IconButton;
use crate::ui::icons::{icon, spinner, Icon};
use crate::ui::palette::marker_color;
use crate::ui::style::{self, ButtonKind};
use crate::ui::tokens::*;
use crate::ui::tooltip::{self, Position, Tip};
use crate::ui::{empty, list, scroll, text};

pub const MARKER_LIST_SCROLLABLE_ID: &str = "marker_list";
pub const MARKER_NAME_INPUT_ID: &str = "marker_name_input";
/// (Scroll estimate only.) Room for a name's text in a row of the list at its widest: the list less its inset, the
/// row's inset and the scroll gutter.
const NAME_ROOM: f32 = SIDE_LIST_MAX_WIDTH - SPACE_S - SPACE_S - SPACE_S - SCROLL_GUTTER;
/// A generous average advance of a character of the name.
const NAME_CHAR_ADVANCE: f32 = 7.0;
/// Inside a row: 6 px above and below, level with the list's other rows.
const ROW_INSET: Padding = Padding {
    top: SPACE_TIGHT,
    bottom: SPACE_TIGHT,
    left: 0.0,
    right: 0.0,
};
/// The name field of the open row: a text field's inset.
const FIELD_INSET: Padding = Padding {
    top: CONTROL_PADDING_Y,
    bottom: CONTROL_PADDING_Y,
    left: SPACE_S,
    right: SPACE_S,
};

/// How many lines `text` wraps to in a row of the list, as an estimate: good enough to scroll a
/// row into view.
fn wrapped_lines(text: &str) -> usize {
    let per_line = (NAME_ROOM / NAME_CHAR_ADVANCE) as usize;
    text.lines()
        .map(|line| line.chars().count().div_ceil(per_line).max(1))
        .sum::<usize>()
        .max(1)
}

/// A comment shows at most this many lines in a row that is not open.
const COMMENT_LINES: usize = 3;

/// `comment` cut to [`COMMENT_LINES`] lines of a row (estimated like [`wrapped_lines`]), ending
/// in "…" when cut.
fn clamped_comment(comment: &str) -> Cow<'_, str> {
    let per_line = (NAME_ROOM / NAME_CHAR_ADVANCE) as usize;
    let all = comment.lines().count();
    let mut left = COMMENT_LINES;
    let mut kept: Vec<String> = Vec::new();
    for line in comment.lines() {
        let lines = line.chars().count().div_ceil(per_line).max(1);
        if lines > left {
            let cut: String = line.chars().take(left * per_line - 1).collect();
            kept.push(format!("{}…", cut.trim_end()));
            return Cow::Owned(kept.join("\n"));
        }
        kept.push(line.to_string());
        left -= lines;
        if left == 0 && kept.len() < all {
            let last = kept.pop().unwrap_or_default();
            kept.push(format!("{}…", last.trim_end()));
            return Cow::Owned(kept.join("\n"));
        }
    }
    Cow::Borrowed(comment)
}

/// Estimated height of `marker`'s row, for scrolling to a row only: the row itself is as tall as
/// its content. Its comment, if any, adds its lines under the name (at most
/// [`COMMENT_LINES`]).
fn row_height(marker: &Marker) -> f32 {
    let comment = if marker.comment.trim().is_empty() {
        0.0
    } else {
        let lines = wrapped_lines(&clamped_comment(marker.comment.trim()));
        SPACE_XXS + lines as f32 * LINE_BODY
    };
    MARKER_ROW_HEIGHT + (wrapped_lines(&marker.name) - 1) as f32 * LINE_BODY + comment
}

/// Distance from the top of the list to the top of row `index`, from the rows above it.
pub fn row_offset(markers: &[Marker], index: usize) -> f32 {
    markers
        .iter()
        .take(index)
        .map(|m| row_height(m) + SPACE_XXS)
        .sum()
}

/// The offset to scroll the list to once it was put back at `offset` in a viewport `viewport`
/// tall: `offset`, unless the lit marker (the last one the playhead passed) is out of view, then
/// the offset the follow uses for it. `None` when the list is right as it is.
pub fn offset_showing_lit(
    markers: &[Marker],
    passed: Option<usize>,
    offset: f32,
    viewport: f32,
) -> Option<f32> {
    let index = passed?;
    let top = row_offset(markers, index);
    let bottom = row_offset(markers, index + 1) - SPACE_XXS;
    let wanted = crate::ui::scroll::keep_row_in_view(
        offset,
        viewport,
        top,
        bottom,
        row_offset(markers, index.saturating_sub(1)),
    );
    (wanted != offset).then_some(wanted)
}

/// How long after a point marker its label stays shown over the progress bar, as a subtitle
/// line stays for its cue.
const NAME_HOLD_MS: u64 = 2_000;

/// The marker the playhead is on, which the progress bar labels (its name, or a prompt to
/// name it): from just before its start to its end, or to [`NAME_HOLD_MS`] after a point
/// marker. A marker already started wins over the next one coming up, then the latest start.
pub fn marker_at(markers: &[Marker], position_ms: u64) -> Option<&Marker> {
    lit_index(markers, position_ms).map(|index| &markers[index])
}

/// Index of the lit row: the marker the playhead is on, by the rule of [`marker_at`], so the
/// list and the progress bar agree. `None` between markers and after the last one.
pub fn lit_index(markers: &[Marker], position_ms: u64) -> Option<usize> {
    markers
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            let from = m.start_ms.saturating_sub(MARKER_SNAP_MS);
            let to = m.end_ms().max(m.start_ms + NAME_HOLD_MS);
            (from..=to).contains(&position_ms)
        })
        .max_by_key(|(_, m)| (m.start_ms <= position_ms, m.start_ms))
        .map(|(index, _)| index)
}

/// Index of the last marker at or before `position_ms`, however long ago: where the list is
/// kept scrolled to. Not the highlight (see [`lit_index`]).
pub fn passed_index(markers: &[Marker], position_ms: u64) -> Option<usize> {
    markers.iter().rposition(|m| m.start_ms <= position_ms)
}

/// The marker list for the side overlay. `markers` is `None` when the file cannot hold them.
pub fn view<'a>(
    markers: Option<&'a [Marker]>,
    state: &'a MarkersState,
    position_ms: u64,
    in_out: Option<(u64, u64)>,
    quiet: bool,
    spinner_frame: usize,
) -> Element<'a, Message> {
    let list = marker_list(markers, state, position_ms, quiet, spinner_frame);
    match in_out {
        Some(span) => column![in_out_line(span), list].into(),
        None => list,
    }
}

/// The in/out points, above the markers: not a marker (it has no name to change and is set
/// with `[` `]`), but a span of the clip like one. A click jumps to the in point.
fn in_out_line<'a>((start, end): (u64, u64)) -> Element<'a, Message> {
    let times = format!("{}–{}", format_marker_time(start), format_marker_time(end));
    let content = row![
        container(space())
            .width(MARKER_DOT)
            .height(ICON_S)
            .style(style::fill(VIDEO_SEGMENT_EDGE)),
        text::mono(times),
        text::secondary(fl!("markers-in-out")),
    ]
    .spacing(SPACE_S)
    .align_y(Alignment::Center);
    let item = list::row_item(content, false, OVERLAY_HOVER, Length::Fill);
    container(
        mouse_area(container(item).height(ROW_HEIGHT))
            .on_press(Message::JumpTo(start))
            .interaction(iced::mouse::Interaction::Pointer),
    )
    .padding(Padding {
        left: SPACE_S,
        right: SCROLL_GUTTER,
        ..Padding::ZERO
    })
    .into()
}

/// The markers, or why there are none.
fn marker_list<'a>(
    markers: Option<&'a [Marker]>,
    state: &'a MarkersState,
    position_ms: u64,
    quiet: bool,
    spinner_frame: usize,
) -> Element<'a, Message> {
    let Some(markers) = markers else {
        return cannot_hold();
    };
    if markers.is_empty() {
        return empty_list(quiet);
    }
    let lit = lit_index(markers, position_ms);
    let rows = markers
        .iter()
        .enumerate()
        .map(|(index, marker)| marker_row(marker, state, lit == Some(index), quiet, spinner_frame));
    scroll::vertical_with_id(
        MARKER_LIST_SCROLLABLE_ID,
        Column::with_children(rows)
            .spacing(SPACE_XXS)
            .padding(Padding {
                left: SPACE_S,
                bottom: SPACE_S,
                ..Padding::ZERO
            }),
    )
    .on_scroll(|viewport| Message::Scrolled(viewport.absolute_offset().y, viewport.bounds().height))
    .into()
}

/// A file whose format has no place for markers: say so, and where Premiere reads them.
fn cannot_hold<'a>() -> Element<'a, Message> {
    row![
        icon(Icon::CircleAlert, ICON_M, TEXT_SECONDARY),
        column![
            text::body(fl!("video-controls-cannot-hold-markers")),
            text::secondary(fl!("markers-cannot-hold-hint")),
        ],
    ]
    .spacing(SPACE_S)
    .padding(SPACE_S)
    .into()
}

/// An empty list: a line and the button that adds the first marker.
fn empty_list<'a>(quiet: bool) -> Element<'a, Message> {
    let add = ui_button::with_icon(
        ButtonKind::Secondary,
        Icon::MapPin,
        fl!("markers-add"),
        true,
    )
    .on_press(Message::Add);
    let add = tooltip::tip_unless(
        quiet,
        add,
        Tip::new(fl!("markers-add")).keys(&["F2"]),
        Position::Bottom,
    );
    empty::small(fl!("markers-empty"), Some(add)).into()
}

/// `m:ss` or `m:ss–m:ss` (a ranged marker read from the file).
fn time_label(marker: &Marker) -> String {
    let start = format_marker_time(marker.start_ms);
    if marker.duration_ms == 0 {
        start
    } else {
        format!("{start}–{}", format_marker_time(marker.end_ms()))
    }
}

/// A color's name, the tooltip of its dot in the picker.
fn color_name(color: MarkerColor) -> String {
    match color {
        MarkerColor::Green => fl!("markers-color-green"),
        MarkerColor::Red => fl!("markers-color-red"),
        MarkerColor::Orange => fl!("markers-color-orange"),
        MarkerColor::Yellow => fl!("markers-color-yellow"),
        MarkerColor::White => fl!("markers-color-white"),
        MarkerColor::Blue => fl!("markers-color-blue"),
        MarkerColor::Cyan => fl!("markers-color-cyan"),
        MarkerColor::Lavender => fl!("markers-color-lavender"),
        MarkerColor::Magenta => fl!("markers-color-magenta"),
        MarkerColor::Other(_) => fl!("markers-color-other"),
    }
}

/// A round dot in a marker's color: ringed on hover, and when it is the one chosen (`ringed`).
/// Without `message` it is not a button (a read-only marker).
fn dot<'a>(color: MarkerColor, ringed: bool, message: Option<Message>) -> Element<'a, Message> {
    let fill = marker_color(color);
    let editable = message.is_some();
    button(space().width(MARKER_DOT).height(MARKER_DOT))
        .on_press_maybe(message)
        .width(MARKER_DOT)
        .height(MARKER_DOT)
        .padding(0)
        .style(move |_theme, status| {
            let hovered = editable && matches!(status, button::Status::Hovered);
            let ring = if ringed {
                TEXT
            } else if hovered {
                TEXT_SECONDARY
            } else {
                iced::Color::TRANSPARENT
            };
            button::Style {
                background: Some(Background::Color(fill)),
                text_color: TEXT,
                border: Border {
                    radius: RADIUS_FULL.into(),
                    width: RING,
                    color: ring,
                },
                ..button::Style::default()
            }
        })
        .into()
}

/// A 24-px action over the list: shown on hover, on the lit row and on the open row.
fn row_action<'a>(glyph: Icon, tip: Tip, message: Message, quiet: bool) -> Element<'a, Message> {
    IconButton::new(glyph)
        .quiet(quiet)
        .small()
        .overlay()
        .tip(tip, Position::Left)
        .on_press(message)
        .into()
}

/// The color picker that replaces a row's first line: the editor's colors, then, set apart, the
/// AI's. White is what marks a marker as the AI's, so it is picked as "AI", not as a color.
fn color_picker<'a>(marker: &Marker, guid: &str, quiet: bool) -> Element<'a, Message> {
    let pick = |color: MarkerColor| {
        dot(
            color,
            color == marker.color,
            Some(Message::SetColor(guid.to_string(), color)),
        )
    };
    let colors = MarkerColor::ALL
        .into_iter()
        .filter(|&color| color != AI_MARKER_COLOR)
        .map(|color| {
            tooltip::tip_text_unless(quiet, pick(color), color_name(color), Position::Top)
        });
    let ai = tooltip::tip_text_unless(
        quiet,
        row![pick(AI_MARKER_COLOR), text::caption("AI")]
            .spacing(SPACE_XS)
            .align_y(Alignment::Center),
        fl!("markers-ai-hint"),
        Position::Top,
    );
    let divider = container(space())
        .width(LINE)
        .height(MARKER_DOT)
        .style(style::divider);
    row(colors)
        .push(divider)
        .push(ai)
        .push(space::horizontal())
        .push(row_action(
            Icon::X,
            Tip::new(fl!("markers-keep-color")),
            Message::ToggleColorPicker(guid.to_string()),
            quiet,
        ))
        .spacing(SPACE_XS)
        .align_y(Alignment::Center)
        .height(ICON_BUTTON_SMALL)
        .into()
}

fn marker_row<'a>(
    marker: &'a Marker,
    state: &'a MarkersState,
    lit: bool,
    quiet: bool,
    spinner_frame: usize,
) -> Element<'a, Message> {
    let guid = marker.guid.as_deref();
    let open = state.edit().filter(|edit| guid == Some(edit.guid.as_str()));
    let time = button(text::mono(time_label(marker)))
        .on_press(Message::JumpTo(marker.start_ms))
        .padding(Padding {
            left: SPACE_XS,
            right: SPACE_XS,
            ..Padding::ZERO
        })
        .style(style::button(ButtonKind::OverlayIcon));

    // The first line without its actions, and the actions, which show only on hover unless the
    // row is lit or open.
    let (first_line, actions): (Element<'a, Message>, Option<Element<'a, Message>>) = match guid {
        Some(guid) if state.color_picker() == Some(guid) => {
            (color_picker(marker, guid, quiet), None)
        }
        Some(guid) => {
            let describing = state.is_describing(guid);
            let done = open.is_some().then(|| {
                row_action(
                    Icon::Check,
                    Tip::new(fl!("markers-done")).keys(&["Enter"]),
                    Message::Close,
                    quiet,
                )
            });
            // While it is described, its stop sits on the "Describing…" line, apart from ✕.
            let ai = (!describing).then(|| {
                // `Ctrl+F2` describes the marker the playhead is on: the lit row.
                let tip = Tip::new(fl!("markers-ai-describe"));
                let tip = if lit { tip.keys(&["Ctrl", "F2"]) } else { tip };
                row_action(
                    Icon::Sparkles,
                    tip,
                    Message::Describe(guid.to_string()),
                    quiet,
                )
            });
            let delete = row_action(
                Icon::X,
                Tip::new(fl!("markers-delete")),
                Message::Delete(guid.to_string()),
                quiet,
            );
            let actions = row![]
                .push(done)
                .push(ai)
                .push(delete)
                .align_y(Alignment::Center);
            let line = row![
                dot(
                    marker.color,
                    false,
                    Some(Message::ToggleColorPicker(guid.to_string()))
                ),
                time,
            ]
            .spacing(SPACE_S)
            .align_y(Alignment::Center)
            .height(ICON_BUTTON_SMALL);
            (line.into(), Some(actions.into()))
        }
        None => {
            let line = row![
                dot(marker.color, false, None),
                time,
                space::horizontal(),
                icon(Icon::Lock, ICON_S, TEXT_SECONDARY),
                text::caption(fl!("markers-read-only")),
            ]
            .spacing(SPACE_S)
            .align_y(Alignment::Center)
            .height(ICON_BUTTON_SMALL);
            (line.into(), None)
        }
    };

    let name: Element<'a, Message> = match open {
        Some(edit) => text_editor(&edit.name)
            .id(iced::widget::Id::new(MARKER_NAME_INPUT_ID))
            .placeholder(fl!("markers-name-placeholder"))
            .on_action(Message::NameAction)
            // `Enter` closes the row: a name is one line, wrapped to fit.
            .key_binding(|press| {
                if matches!(
                    press.key,
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
                ) {
                    Some(text_editor::Binding::Custom(Message::Close))
                } else {
                    text_editor::Binding::from_key_press(press)
                }
            })
            .size(TEXT_BODY)
            .font(FONT)
            .padding(FIELD_INSET)
            .style(style::text_editor)
            .into(),
        None if marker.name.is_empty() => text::body("—").color(TEXT_SECONDARY).into(),
        None => text::body(marker.name.as_str()).into(),
    };
    // The comment (an AI's description, or Premiere's comment) under the name, read-only: a few
    // lines, the whole of it while the row is open.
    let comment = (!marker.comment.trim().is_empty()).then(|| {
        let comment = marker.comment.trim();
        if open.is_some() {
            text::secondary(comment)
        } else {
            text::secondary(clamped_comment(comment).into_owned())
        }
    });
    // A request on its way, under the name and its comment: shown whether or not the actions
    // are (it goes on in the background), and clear of them.
    let working = marker
        .guid
        .as_deref()
        .filter(|guid| state.is_describing(guid))
        .map(|guid| {
            row![
                spinner(spinner_frame, ICON_S, TEXT_SECONDARY),
                text::caption(fl!("markers-ai-describing")),
                row_action(
                    Icon::CircleX,
                    Tip::new(fl!("markers-ai-stop")),
                    Message::StopDescribing(guid.to_string()),
                    quiet,
                ),
            ]
            .spacing(SPACE_XS)
            .align_y(Alignment::Center)
        });
    let body = column![first_line, name]
        .push(comment)
        .push(working)
        .spacing(SPACE_XXS)
        .padding(ROW_INSET);
    // The row is as tall as its content, not as the estimate of [`row_height`]: a name that wraps
    // to more lines than estimated (a narrow list, a long name) must not be cut off, and the list
    // sizes its scroll range from what is rendered.
    let item = list::row_item(body, lit || open.is_some(), OVERLAY_HOVER, Length::Shrink);

    // The actions sit at the right end of the first line, over the row.
    let item: Element<'a, Message> = match actions {
        Some(actions) => {
            let place = container(actions)
                .padding(Padding {
                    top: SPACE_TIGHT,
                    right: SPACE_S,
                    ..Padding::ZERO
                })
                .align_right(Length::Fill);
            if lit || open.is_some() {
                stack![item, place].into()
            } else {
                hover(item, place)
            }
        }
        None => item,
    };
    // A click on the row (not on one of its buttons) opens it for renaming.
    match guid.filter(|_| open.is_none() && state.color_picker().is_none()) {
        Some(guid) => mouse_area(item)
            .on_press(Message::Open(guid.to_string()))
            .into(),
        None => item,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lit_row_is_the_marker_the_playhead_is_on() {
        let mut range = Marker::new(20_000);
        range.duration_ms = 6_000;
        let markers = [Marker::new(1_000), Marker::new(5_000), range];
        assert_eq!(lit_index(&markers, 0), None);
        assert_eq!(lit_index(&markers, 1_000), Some(0));
        assert_eq!(lit_index(&markers, 2_999), Some(0));
        assert_eq!(lit_index(&markers, 3_001), None, "between two markers");
        assert_eq!(lit_index(&markers, 5_500), Some(1));
        assert_eq!(lit_index(&markers, 23_000), Some(2), "inside a range");
        assert_eq!(lit_index(&markers, 26_000), Some(2), "at its end");
        assert_eq!(lit_index(&markers, 26_001), None, "after the last one");
    }

    #[test]
    fn a_point_marker_is_lit_for_two_seconds_and_then_not_at_all() {
        let markers = [Marker::new(5_000)];
        assert_eq!(lit_index(&markers, 7_000), Some(0));
        assert_eq!(lit_index(&markers, 7_001), None);
        assert_eq!(lit_index(&markers, 90_000), None);
    }

    #[test]
    fn overlapping_markers_light_the_one_the_bar_labels() {
        let mut long = Marker::new(1_000);
        long.duration_ms = 20_000;
        let markers = [long, Marker::new(5_000)];
        for ms in [500, 1_000, 4_999, 5_000, 6_000, 8_000, 21_000] {
            let lit = lit_index(&markers, ms).map(|i| &markers[i]);
            assert_eq!(lit, marker_at(&markers, ms), "at {ms}");
        }
    }

    #[test]
    fn the_list_follows_the_last_marker_passed() {
        let markers = [Marker::new(1_000), Marker::new(5_000)];
        assert_eq!(passed_index(&markers, 0), None);
        assert_eq!(passed_index(&markers, 4_999), Some(0));
        assert_eq!(passed_index(&markers, 90_000), Some(1));
    }

    #[test]
    fn the_bar_labels_the_marker_the_playhead_is_on() {
        let mut first = Marker::new(10_000);
        first.name = "Lion".into();
        let mut second = Marker::new(11_000);
        second.name = "Cub".into();
        let unnamed = Marker::new(20_000);
        let markers = [first, second, unnamed];
        let name = |ms| marker_at(&markers, ms).map(|m| m.name.as_str());
        assert_eq!(name(9_000), None);
        assert_eq!(name(9_600), Some("Lion"), "just before it");
        assert_eq!(name(10_900), Some("Lion"));
        assert_eq!(name(11_500), Some("Cub"), "the later one wins");
        assert_eq!(name(13_001), None, "held two seconds");
        assert_eq!(
            name(20_000),
            Some(""),
            "unnamed: labelled so it can be named"
        );
    }

    #[test]
    fn a_long_comment_shows_three_lines_and_an_ellipsis() {
        let per_line = (NAME_ROOM / NAME_CHAR_ADVANCE) as usize;
        let short = "A lion walks past.";
        assert_eq!(clamped_comment(short), short);
        let long = "x".repeat(per_line * 5);
        let cut = clamped_comment(&long);
        assert!(cut.ends_with('…'), "{cut}");
        assert_eq!(wrapped_lines(&cut), COMMENT_LINES);
        let many = "one\ntwo\nthree\nfour";
        assert_eq!(clamped_comment(many), "one\ntwo\nthree…");
        let mut marker = Marker::new(0);
        marker.comment = long;
        assert_eq!(
            row_height(&marker),
            MARKER_ROW_HEIGHT + SPACE_XXS + COMMENT_LINES as f32 * LINE_BODY
        );
    }

    #[test]
    fn ranged_markers_show_both_ends() {
        let mut marker = Marker::new(41_000);
        assert_eq!(time_label(&marker), "0:41");
        marker.duration_ms = 6_000;
        assert_eq!(time_label(&marker), "0:41–0:47");
    }

    #[test]
    fn a_long_name_makes_its_row_taller_and_moves_the_rows_below() {
        let short = Marker::new(1_000);
        let mut long = Marker::new(2_000);
        long.name = "x".repeat(200);
        assert_eq!(row_height(&short), MARKER_ROW_HEIGHT);
        assert!(row_height(&long) > MARKER_ROW_HEIGHT + LINE_BODY);
        let markers = [long.clone(), short];
        assert_eq!(row_offset(&markers, 1), row_height(&long) + SPACE_XXS);
    }

    #[test]
    fn a_restored_list_keeps_its_offset_unless_the_lit_marker_is_out_of_view() {
        let markers: Vec<Marker> = (0..30).map(|i| Marker::new(i * 1_000)).collect();
        let row = row_offset(&markers, 1);
        let lit = Some(20);
        let top = row_offset(&markers, 20);
        // Shown: kept.
        assert_eq!(
            offset_showing_lit(&markers, lit, top - row, 3.0 * row),
            None
        );
        // Below the fold (the viewport got shorter): the follow's offset, one row of context.
        assert_eq!(
            offset_showing_lit(&markers, lit, 0.0, 3.0 * row),
            Some(row_offset(&markers, 19))
        );
        // No marker passed yet: nothing to show.
        assert_eq!(offset_showing_lit(&markers, None, 0.0, 3.0 * row), None);
    }
}
