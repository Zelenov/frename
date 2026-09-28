//! Tag chips (design system §8.16, §13.4.2, §13.5.6): a tag's name in black on its color. The
//! mini chip is for the file list's rows; the chip for the file name card, where it can be
//! dragged. The cells of the tag grid are `features::tag_grid::cell`.

use iced::widget::{column, container, space};
use iced::{Background, Color, Element, Length, Padding};

use crate::ui::tokens::*;
use crate::ui::{style, text};

/// A chip's inset: 0 × 8.
const PADDING: Padding = Padding {
    top: 0.0,
    bottom: 0.0,
    left: SPACE_S,
    right: SPACE_S,
};

/// The chip's surface: the tag color, lifted while dragged.
fn surface(color: Color, lifted: bool) -> impl Fn(&iced::Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        border: iced::border::rounded(RADIUS_S),
        shadow: if lifted {
            style::lift_shadow()
        } else {
            iced::Shadow::default()
        },
        ..container::Style::default()
    }
}

/// A mini chip (`chip.mini`, design system §13.4.2): 20 px high, 12/16 black text on the tag
/// color, for the file list's rows. It has no marks.
pub fn mini<'a, Message: 'a>(tag_name: &'a str, tag_color: Color) -> Element<'a, Message> {
    container(text::chip_mini(tag_name).wrapping(iced::widget::text::Wrapping::None))
        .padding(Padding {
            left: SPACE_TIGHT,
            right: SPACE_TIGHT,
            ..Padding::ZERO
        })
        .height(CHIP_MINI_HEIGHT)
        .center_y(CHIP_MINI_HEIGHT)
        .style(surface(tag_color, false))
        .into()
}

/// A 28-px chip that only shows the tag: the file name card, the chip over the trash.
pub fn view_display_only<'a, Message: 'a>(
    tag_name: &'a str,
    tag_color: Color,
) -> Element<'a, Message> {
    chip(tag_name, tag_color, false)
}

fn chip<'a, Message: 'a>(
    tag_name: &'a str,
    tag_color: Color,
    lifted: bool,
) -> Element<'a, Message> {
    container(
        text::body(tag_name)
            .color(TAG_TEXT)
            .wrapping(iced::widget::text::Wrapping::None),
    )
    .padding(PADDING)
    .height(CHIP_HEIGHT)
    .center_y(CHIP_HEIGHT)
    .style(surface(tag_color, lifted))
    .into()
}

/// A chip of the file name card, in a cell `CHIP_DRAG_LIFT` taller than it: while `dragging`
/// it floats at the top of the cell with the lift shadow, otherwise it sits at the bottom.
pub fn draggable<'a, Message: 'a>(
    tag_name: &'a str,
    tag_color: Color,
    dragging: bool,
) -> Element<'a, Message> {
    let chip = chip(tag_name, tag_color, dragging);
    let lift = space().height(CHIP_DRAG_LIFT);
    let cell = if dragging {
        column![chip, lift]
    } else {
        column![lift, chip]
    };
    cell.width(Length::Shrink).into()
}
