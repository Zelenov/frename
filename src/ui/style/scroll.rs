//! The scrollbar (`docs/design/design-system.md` §8.19, §13.2): one look for every scroll area.

use iced::widget::container;
use iced::widget::scrollable::{AutoScroll, Rail, Scroller, Status, Style};
use iced::{border, Background, Shadow, Theme};

use crate::ui::tokens::*;

pub fn scrollable(_theme: &Theme, status: Status) -> Style {
    let rail = |active: bool| Rail {
        background: Some(Background::Color(BG_RAISED)),
        border: border::rounded(RADIUS_CHECK),
        scroller: Scroller {
            background: Background::Color(if active { ACCENT_TEXT } else { BORDER_CONTROL }),
            border: border::rounded(RADIUS_CHECK),
        },
    };
    let (vertical, horizontal) = match status {
        Status::Hovered {
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_hovered,
            ..
        } => (
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_hovered,
        ),
        Status::Dragged {
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_dragged,
            ..
        } => (
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_dragged,
        ),
        Status::Active { .. } => (false, false),
    };
    Style {
        container: container::Style::default(),
        vertical_rail: rail(vertical),
        horizontal_rail: rail(horizontal),
        gap: None,
        auto_scroll: AutoScroll {
            background: Background::Color(BG_OVERLAY),
            border: border::rounded(RADIUS_L).width(LINE).color(BORDER_SUBTLE),
            shadow: Shadow::default(),
            icon: TEXT_SECONDARY,
        },
    }
}
