//! The order strip between the tag grid and the file name card (design system §13.5.5): in
//! words, whether reordering the file's tags also reorders the folder's, and, when the two orders
//! differ, the buttons that make them the same again.

use iced::widget::{container, row};
use iced::{Alignment, Color, Element, Length};

use super::Message;
use crate::ui::button;
use crate::ui::icons::{icon, Icon};
use crate::ui::style::ButtonKind;
use crate::ui::text;
use crate::ui::tokens::*;
use crate::ui::tooltip::{self, Position};

/// Render the order strip.
///
/// `is_synced` – the file's tags are in the folder's order.
/// `locked`    – when synced, whether reordering the file reorders the folder too.
pub fn view(is_synced: bool, locked: bool) -> Element<'static, Message> {
    let content = if is_synced {
        let (glyph, sentence, toggle) = if locked {
            (
                Icon::Lock,
                fl!("sync-panel-locked"),
                fl!("sync-panel-unlock"),
            )
        } else {
            (
                Icon::LockOpen,
                fl!("sync-panel-unlocked"),
                fl!("sync-panel-lock"),
            )
        };
        line(glyph, TEXT_SECONDARY, sentence)
            .push(button::ghost(toggle).on_press(Message::ToggleLock))
    } else {
        line(Icon::TriangleAlert, WARNING, fl!("sync-panel-differs"))
            .push(order_button(
                Icon::ArrowUp,
                fl!("sync-panel-use-for-folder"),
                Message::SyncUp,
            ))
            .push(order_button(
                Icon::ArrowDown,
                fl!("sync-panel-sort-like-folder"),
                Message::SyncDown,
            ))
    };
    container(content.spacing(SPACE_S).align_y(Alignment::Center))
        .width(Length::Fill)
        .height(ORDER_STRIP_HEIGHT)
        .center_y(ORDER_STRIP_HEIGHT)
        .into()
}

/// The icon and the sentence, cut at the strip's end rather than wrapped.
fn line(glyph: Icon, color: Color, sentence: String) -> iced::widget::Row<'static, Message> {
    let sentence =
        container(text::secondary(sentence).wrapping(iced::widget::text::Wrapping::None))
            .width(Length::Fill)
            .clip(true);
    row![icon(glyph, ICON_MARK, color), sentence]
}

/// A button that copies one order onto the other; neither can be undone yet, so they say so.
fn order_button(glyph: Icon, label: String, message: Message) -> Element<'static, Message> {
    tooltip::tip_text(
        button::with_icon(ButtonKind::Secondary, glyph, label, true).on_press(message),
        fl!("sync-panel-no-undo"),
        Position::Top,
    )
}
