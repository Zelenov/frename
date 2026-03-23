//! View for the sync panel.
//!
//! When orders differ: Sync Up and Sync Down buttons are active.
//! When orders equal:  Lock or Unlock button is shown (reflects `locked` state).

use iced::widget::{button, container, row, text};
use iced::{Background, Color, Element, Length};

use super::Message;

pub const PANEL_HEIGHT: f32 = 36.0;

const BG_BRIDGE: Color = Color::from_rgb(0.098, 0.098, 0.098); // #191919
const BG_BLOCK: Color = Color::from_rgb(0.18, 0.18, 0.18);     // #2e2e2e
const BG_BLOCK_HOV: Color = Color::from_rgb(0.24, 0.24, 0.24);
const BG_BLOCK_PRS: Color = Color::from_rgb(0.30, 0.30, 0.30);
const BG_BLOCK_DIM: Color = Color::from_rgb(0.12, 0.12, 0.12); // grayed-out

const TEXT_ACTIVE: Color = Color::WHITE;
const TEXT_DIM: Color = Color::from_rgb(0.35, 0.35, 0.35);

/// Render the sync panel.
///
/// `is_synced` – checked-tag order in grid/DB matches file name panel order.
/// `locked`    – when synced, whether lock mode is active.
pub fn view(is_synced: bool, locked: bool) -> Element<'static, Message> {
    let blocks: Element<'static, Message> = if is_synced {
        let lock_icon: &'static str = if locked { "🔒" } else { "🔓" };
        row![make_block(lock_icon, Message::ToggleLock, true)]
            .spacing(8.0)
            .height(Length::Fill)
            .into()
    } else {
        row![
            make_block("🔓↑", Message::SyncUp, true),
            make_block("🔓↓", Message::SyncDown, true),
        ]
        .spacing(8.0)
        .height(Length::Fill)
        .into()
    };

    container(
        container(blocks)
            .center_x(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(PANEL_HEIGHT)
    .style(|_: &iced::Theme| iced::widget::container::Style {
        background: Some(Background::Color(BG_BRIDGE)),
        ..Default::default()
    })
    .into()
}

fn make_block(icon: &'static str, msg: Message, active: bool) -> Element<'static, Message> {
    if active {
        button(
            container(text(icon).size(13).color(TEXT_ACTIVE))
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .on_press(msg)
        .width(PANEL_HEIGHT)
        .height(Length::Fill)
        .padding(0)
        .style(move |_theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => BG_BLOCK_HOV,
                iced::widget::button::Status::Pressed => BG_BLOCK_PRS,
                _ => BG_BLOCK,
            };
            iced::widget::button::Style {
                background: Some(Background::Color(bg)),
                text_color: TEXT_ACTIVE,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                snap: true,
            }
        })
        .into()
    } else {
        container(
            container(text(icon).size(13).color(TEXT_DIM))
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .width(PANEL_HEIGHT)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(Background::Color(BG_BLOCK_DIM)),
            ..Default::default()
        })
        .into()
    }
}
