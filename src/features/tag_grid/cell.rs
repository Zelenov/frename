//! One cell of the tag grid and of the starred strip (design system §13.5.2): the tag's chip with
//! its checkbox, label, star and action, and the cursor drawn as a filled cell and a ring.
//! Checked is the box and a SemiBold label; the chip's color never changes with it.

use frename_core::Tag;
use iced::widget::{checkbox, container, mouse_area, row, space};
use iced::{mouse, Alignment, Border, Color, Element, Length, Padding};

use super::layout::{fit_label, Columns};
use crate::features::tag_panel::{Message, GRID_CELL_HEIGHT, GRID_CELL_INSET};
use crate::ui::icons::{icon, Icon};
use crate::ui::tokens::*;
use crate::ui::tooltip::{self, Position, Tip};
use crate::ui::{list, style, text};

/// A chip's inset: 0 × 8.
const CHIP_PADDING: Padding = Padding {
    top: 0.0,
    bottom: 0.0,
    left: SPACE_S,
    right: SPACE_S,
};

/// The cell of `tag`, drawn in `color`, `columns` wide; `cursor` when the keys act on it.
pub fn cell<'a>(
    tag: &'a Tag,
    color: Color,
    cursor: bool,
    columns: Columns,
) -> Element<'a, Message> {
    let id = tag.id();
    let chip = mouse_area(chip(tag, color, cursor, columns.chip_width()))
        .on_press(Message::ToggleTag(id))
        .interaction(mouse::Interaction::Pointer);
    let ring = container(chip)
        .padding(GRID_CELL_INSET)
        .style(move |_theme: &iced::Theme| cursor_ring(cursor));
    container(ring)
        .width(columns.cell_width)
        .height(GRID_CELL_HEIGHT)
        .center_y(GRID_CELL_HEIGHT)
        .style(style::selectable(cursor))
        .id(iced::widget::Id::from(id.widget_id()))
        .into()
}

/// The 2-px ring 1 px outside the chip of the cursor cell.
fn cursor_ring(cursor: bool) -> container::Style {
    if !cursor {
        return container::Style::default();
    }
    container::Style {
        border: Border {
            color: ACCENT_TEXT,
            width: RING,
            radius: (RADIUS_S + GRID_CELL_INSET).into(),
        },
        ..container::Style::default()
    }
}

/// The chip: filled with the tag color when the tag is in the folder's tags, an outline when it
/// is only in this file's name.
fn chip<'a>(tag: &'a Tag, color: Color, cursor: bool, width: f32) -> Element<'a, Message> {
    let id = tag.id();
    let stored = tag.is_stored();
    let label_color = chip_label_ink(stored);
    let name = fit_label(tag.tag(), width);
    let cut = name.len() != tag.tag().len();
    let label = if tag.is_checked() {
        text::strong(name.into_owned())
    } else {
        text::body(name.into_owned())
    }
    .color(label_color)
    .wrapping(iced::widget::text::Wrapping::None);
    let label: Element<'a, Message> = if cut {
        tooltip::tip_text(label, tag.tag(), Position::Top)
    } else {
        label.into()
    };
    let check = checkbox(tag.is_checked())
        .size(CHECK_SIZE)
        .spacing(0)
        .style(if stored {
            style::chip_checkbox
        } else {
            style::checkbox
        })
        .on_toggle(move |_| Message::ToggleTag(id));
    let marks = row![star(tag), action(tag, cursor)]
        .spacing(SPACE_XS)
        .align_y(Alignment::Center);
    let content = row![check, label, marks]
        .spacing(SPACE_TIGHT)
        .align_y(Alignment::Center);
    let body = container(content)
        .padding(CHIP_PADDING)
        .height(CHIP_HEIGHT)
        .center_y(CHIP_HEIGHT);
    let (body, hover) = if stored {
        (body.style(style::badge(color)), CHIP_HOVER)
    } else {
        (body.style(style::outline(false)), HOVER)
    };
    list::hoverable(body, hover, RADIUS_S)
}

/// A 14-px mark on the chip that does `message`, with its tooltip, in `ink`.
fn mark<'a>(glyph: Icon, ink: Color, message: Message, tip: Tip) -> Element<'a, Message> {
    let target = mouse_area(icon(glyph, ICON_MARK, ink))
        .on_press(message)
        .interaction(mouse::Interaction::Pointer);
    tooltip::tip(target, tip, Position::Top)
}

/// An empty mark's place, so the chip keeps its width.
fn no_mark<'a>() -> Element<'a, Message> {
    space().width(ICON_MARK).height(ICON_MARK).into()
}

/// The star, on saved tags only: filled when starred.
fn star<'a>(tag: &'a Tag) -> Element<'a, Message> {
    if !tag.is_stored() {
        return no_mark();
    }
    let (glyph, tip) = if tag.is_starred() {
        (Icon::StarFilled, fl!("tag-grid-unstar"))
    } else {
        (Icon::Star, fl!("tag-grid-star"))
    };
    mark(
        glyph,
        chip_mark_ink(true),
        Message::ToggleStar(tag.id()),
        Tip::new(tip),
    )
}

/// The action: adding an unsaved tag to the folder's tags, or deleting the cursor's tag.
fn action<'a>(tag: &'a Tag, cursor: bool) -> Element<'a, Message> {
    let id = tag.id();
    match (tag.is_stored(), cursor) {
        (false, _) => mark(
            Icon::Plus,
            chip_mark_ink(false),
            Message::SaveTag(id),
            Tip::new(fl!("tag-grid-save")).keys(&["Enter"]),
        ),
        (true, true) => mark(
            Icon::Trash,
            chip_mark_ink(true),
            Message::DeleteTag(id),
            Tip::new(fl!("tag-grid-delete", tag = tag.tag())).keys(&["Delete"]),
        ),
        (true, false) => no_mark(),
    }
}

/// The first cell while the search names no tag: "Create “text”" and its key.
pub fn create<'a>(name: &str) -> Element<'a, Message> {
    let content = row![
        icon(Icon::Plus, ICON_MARK, TEXT_SECONDARY),
        text::body(fl!("tag-grid-create", tag = name)),
    ]
    .push(crate::ui::badge::key_cap("Enter", false))
    .spacing(SPACE_TIGHT)
    .align_y(Alignment::Center);
    let body = container(content)
        .padding(CHIP_PADDING)
        .height(CHIP_HEIGHT)
        .center_y(CHIP_HEIGHT)
        .style(style::outline(false));
    let target = mouse_area(list::hoverable(body, HOVER, RADIUS_S))
        .on_press(Message::CreateTag(name.to_string()))
        .interaction(mouse::Interaction::Pointer);
    container(target)
        .padding(GRID_CELL_INSET)
        .width(Length::Shrink)
        .into()
}
