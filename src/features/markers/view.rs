//! The marker list: one row per marker, read-only like a subtitle cue, except the one row open
//! for editing. Rows have a fixed height (the open one a taller fixed height), so the list is
//! scrolled to a row by arithmetic, as the subtitle list is.

use frename_core::{format_marker_time, Marker, MarkerColor, MARKER_SNAP_MS};
use iced::widget::{
    button, column, container, mouse_area, row, scrollable, text, text_input, tooltip, Column,
    Space,
};
use iced::{Alignment, Element, Length};

use super::{MarkersState, Message};
use crate::theme;

pub const MARKER_LIST_SCROLLABLE_ID: &str = "marker_list";
pub const MARKER_NAME_INPUT_ID: &str = "marker_name_input";
/// Room for the first line and the name; a longer name is clipped.
const ROW_HEIGHT: f32 = 56.0;
/// The open row: first line and name field.
const OPEN_ROW_HEIGHT: f32 = 66.0;
const ROW_SPACING: f32 = 2.0;
/// Distance from one closed row's top to the next.
pub const ROW_PITCH: f32 = ROW_HEIGHT + ROW_SPACING;
const DOT_SIZE: f32 = 14.0;
const ICON_SIZE: f32 = 22.0;

/// How long after a point marker its label stays shown over the progress bar, as a subtitle
/// line stays for its cue.
const NAME_HOLD_MS: u64 = 2_000;

/// The marker the playhead is on, which the progress bar labels (its name, or a prompt to
/// name it): from just before its start to its end, or to [`NAME_HOLD_MS`] after a point
/// marker. A marker already started wins over the next one coming up, then the latest start.
pub fn marker_at(markers: &[Marker], position_ms: u64) -> Option<&Marker> {
    markers
        .iter()
        .filter(|m| {
            let from = m.start_ms.saturating_sub(MARKER_SNAP_MS);
            let to = m.end_ms().max(m.start_ms + NAME_HOLD_MS);
            (from..=to).contains(&position_ms)
        })
        .max_by_key(|m| (m.start_ms <= position_ms, m.start_ms))
}

/// Index of the lit row: the last marker at or before `position_ms`.
pub fn lit_index(markers: &[Marker], position_ms: u64) -> Option<usize> {
    markers.iter().rposition(|m| m.start_ms <= position_ms)
}

/// The marker list for the side overlay. `markers` is `None` when the file cannot hold them.
pub fn view<'a>(
    markers: Option<&'a [Marker]>,
    state: &'a MarkersState,
    position_ms: u64,
) -> Element<'a, Message> {
    let Some(markers) = markers else {
        return note("This file cannot hold markers");
    };
    if markers.is_empty() {
        return empty_list();
    }
    let lit = lit_index(markers, position_ms);
    let rows = markers
        .iter()
        .enumerate()
        .map(|(index, marker)| marker_row(marker, state, lit == Some(index)));
    scrollable(
        Column::with_children(rows)
            .spacing(ROW_SPACING)
            .padding([0, 12]),
    )
    .id(iced::widget::Id::new(MARKER_LIST_SCROLLABLE_ID))
    .height(Length::Fill)
    .style(theme::dark_scrollable_style)
    .into()
}

fn note<'a>(message: &'a str) -> Element<'a, Message> {
    container(text(message).size(13).color(theme::TEXT_MUTED))
        .padding([8, 16])
        .width(Length::Fill)
        .into()
}

/// An empty list: a note and a button that adds the first marker.
fn empty_list<'a>() -> Element<'a, Message> {
    column![
        text("No markers yet").size(13).color(theme::TEXT_MUTED),
        button(text("📍 Add a marker (F2)").size(13))
            .on_press(Message::Add)
            .padding([4, 10])
            .style(theme::overlay_tab_style(false)),
    ]
    .spacing(8)
    .padding([8, 16])
    .width(Length::Fill)
    .into()
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

fn icon_button<'a>(icon: &'a str, tip: &'a str, message: Option<Message>) -> Element<'a, Message> {
    tooltip(
        button(
            container(text(icon).size(13))
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .on_press_maybe(message)
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .padding(0)
        .style(theme::icon_button_style(true)),
        text(tip).size(12),
        tooltip::Position::Top,
    )
    .into()
}

fn dot<'a>(color: MarkerColor, selected: bool, message: Option<Message>) -> Element<'a, Message> {
    button(Space::new().width(DOT_SIZE).height(DOT_SIZE))
        .on_press_maybe(message)
        .width(DOT_SIZE)
        .height(DOT_SIZE)
        .padding(0)
        .style(theme::marker_dot_style(
            theme::marker_color(color),
            selected,
        ))
        .into()
}

fn marker_row<'a>(marker: &'a Marker, state: &'a MarkersState, lit: bool) -> Element<'a, Message> {
    let guid = marker.guid.as_deref();
    let open = state.edit().filter(|edit| guid == Some(edit.guid.as_str()));
    let time = button(text(time_label(marker)).size(11).color(theme::TEXT_MUTED))
        .on_press(Message::JumpTo(marker.start_ms))
        .padding([2, 4])
        .style(theme::icon_button_style(true));

    let first_line: Element<'a, Message> = match guid {
        Some(guid) if state.color_picker() == Some(guid) => {
            let colors = MarkerColor::ALL.into_iter().map(|color| {
                dot(
                    color,
                    color == marker.color,
                    Some(Message::SetColor(guid.to_string(), color)),
                )
            });
            row(colors)
                .push(Space::new().width(Length::Fill))
                .push(icon_button(
                    "✕",
                    "Keep the color",
                    Some(Message::ToggleColorPicker(guid.to_string())),
                ))
                .spacing(6)
                .align_y(Alignment::Center)
                .into()
        }
        Some(guid) => {
            let edit: Element<'a, Message> = if open.is_some() {
                icon_button("✓", "Done (Enter)", Some(Message::Close))
            } else {
                icon_button("✎", "Rename", Some(Message::Open(guid.to_string())))
            };
            row![
                dot(
                    marker.color,
                    false,
                    Some(Message::ToggleColorPicker(guid.to_string()))
                ),
                time,
                Space::new().width(Length::Fill),
                edit,
                icon_button(
                    "✕",
                    "Delete the marker",
                    Some(Message::Delete(guid.to_string()))
                ),
            ]
            .spacing(4)
            .align_y(Alignment::Center)
            .into()
        }
        None => row![
            dot(marker.color, false, None),
            time,
            Space::new().width(Length::Fill),
            text("read-only").size(11).color(theme::TEXT_MUTED),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into(),
    };

    let (body, height): (Element<'a, Message>, f32) = match open {
        Some(_) => {
            let name = text_input("Name", &marker.name)
                .id(iced::widget::Id::new(MARKER_NAME_INPUT_ID))
                .on_input(Message::NameInput)
                .on_submit(Message::Close)
                .size(13)
                .padding([3, 6]);
            (column![first_line, name].spacing(4).into(), OPEN_ROW_HEIGHT)
        }
        None => {
            let name = text(if marker.name.is_empty() {
                "—"
            } else {
                marker.name.as_str()
            })
            .size(13)
            .color(if lit { theme::TEXT } else { theme::TEXT_SOFT })
            .wrapping(iced::widget::text::Wrapping::None);
            (column![first_line, name].spacing(2).into(), ROW_HEIGHT)
        }
    };
    let row_box = container(body)
        .width(Length::Fill)
        .height(height)
        .clip(true)
        .padding([6, 10])
        .style(move |theme| {
            let mut style = container::Style::default();
            let button_style = theme::cue_row_style(lit)(theme, button::Status::Active);
            style.background = button_style.background;
            style.border = button_style.border;
            style
        });
    // A click on the row (not on one of its buttons) opens it for editing.
    match guid.filter(|_| open.is_none()) {
        Some(guid) => mouse_area(row_box)
            .on_press(Message::Open(guid.to_string()))
            .into(),
        None => row_box.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lit_row_is_the_last_marker_at_or_before_the_playhead() {
        let markers = [Marker::new(1_000), Marker::new(5_000)];
        assert_eq!(lit_index(&markers, 0), None);
        assert_eq!(lit_index(&markers, 1_000), Some(0));
        assert_eq!(lit_index(&markers, 4_999), Some(0));
        assert_eq!(lit_index(&markers, 9_000), Some(1));
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
    fn ranged_markers_show_both_ends() {
        let mut marker = Marker::new(41_000);
        assert_eq!(time_label(&marker), "0:41");
        marker.duration_ms = 6_000;
        assert_eq!(time_label(&marker), "0:41–0:47");
    }
}
