//! The marker list: one row per marker, read-only like a subtitle cue, except the one row open
//! for editing. Rows have a fixed height (the open one a taller fixed height), so the list is
//! scrolled to a row by arithmetic, as the subtitle list is.

use frename_core::{format_marker_time, Marker, MarkerColor};
use iced::widget::text_editor::Binding;
use iced::widget::{
    button, column, container, mouse_area, row, scrollable, text, text_editor, text_input, tooltip,
    Column, Space,
};
use iced::{Alignment, Element, Length};

use super::{MarkersState, Message};
use crate::theme;

pub const MARKER_LIST_SCROLLABLE_ID: &str = "marker_list";
pub const MARKER_NAME_INPUT_ID: &str = "marker_name_input";
/// Room for the first line, the name and two lines of the comment; longer text is clipped.
const ROW_HEIGHT: f32 = 78.0;
/// The open row: first line, name field and comment editor.
const OPEN_ROW_HEIGHT: f32 = 170.0;
const ROW_SPACING: f32 = 2.0;
/// Distance from one closed row's top to the next.
pub const ROW_PITCH: f32 = ROW_HEIGHT + ROW_SPACING;
const DOT_SIZE: f32 = 14.0;
const ICON_SIZE: f32 = 22.0;
const COMMENT_EDITOR_HEIGHT: f32 = 76.0;

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
        return note("No markers yet — press F2 to add one");
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

/// `m:ss` or `m:ss–m:ss`.
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
            let length: Element<'a, Message> = if marker.duration_ms > 0 {
                icon_button(
                    "⇤",
                    "Make it a point marker",
                    Some(Message::ClearEnd(guid.to_string())),
                )
            } else {
                icon_button(
                    "⇥",
                    "End it at the playhead",
                    Some(Message::SetEndHere(guid.to_string())),
                )
            };
            let edit: Element<'a, Message> = if open.is_some() {
                icon_button("✓", "Done (Enter)", Some(Message::Close))
            } else {
                icon_button(
                    "✎",
                    "Name and comment",
                    Some(Message::Open(guid.to_string())),
                )
            };
            row![
                dot(
                    marker.color,
                    false,
                    Some(Message::ToggleColorPicker(guid.to_string()))
                ),
                time,
                Space::new().width(Length::Fill),
                length,
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
        Some(edit) => {
            let name = text_input("Name", &marker.name)
                .id(iced::widget::Id::new(MARKER_NAME_INPUT_ID))
                .on_input(Message::NameInput)
                .on_submit(Message::Close)
                .size(13)
                .padding([3, 6]);
            let comment = text_editor(&edit.comment)
                .on_action(Message::CommentAction)
                .placeholder("Comment...")
                .height(COMMENT_EDITOR_HEIGHT)
                .size(12)
                .padding([3, 6])
                // Enter is a line break here: the name field's Enter closes the row.
                .key_binding(|kp| {
                    if matches!(
                        kp.key,
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
                    ) {
                        Some(Binding::Enter)
                    } else {
                        Binding::from_key_press(kp)
                    }
                });
            (
                column![first_line, name, comment].spacing(4).into(),
                OPEN_ROW_HEIGHT,
            )
        }
        None => {
            let name = text(if marker.name.is_empty() {
                "—"
            } else {
                marker.name.as_str()
            })
            .size(13)
            .color(if lit { theme::TEXT } else { theme::TEXT_SOFT });
            let comment = text(marker.comment.as_str())
                .size(12)
                .color(theme::TEXT_MUTED);
            (
                column![first_line, name, comment].spacing(2).into(),
                ROW_HEIGHT,
            )
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
    fn ranged_markers_show_both_ends() {
        let mut marker = Marker::new(41_000);
        assert_eq!(time_label(&marker), "0:41");
        marker.duration_ms = 6_000;
        assert_eq!(time_label(&marker), "0:41–0:47");
    }
}
