//! UI rendering for file handler feature

use iced::widget::{container, row};
use iced::{Element, Length};

use crate::features::{rename_panel, video_player};
use crate::widgets::splitter::Splitter;
use super::{FileHandlerState, Message};

/// Render the file handler view: video player on the left, splitter, rename panel on the right
pub fn view(state: &FileHandlerState) -> Element<'_, Message> {
    let video = container(
        video_player::view::view(state.video_player()).map(Message::VideoPlayer),
    )
    .width(Length::Fixed(state.left_width()))
    .height(Length::Fill);

    let splitter = Splitter::new(Message::SplitterDragged)
        .min_left(150.0)
        .min_right(200.0);

    let panel = rename_panel::view::view(state.rename_panel()).map(Message::RenamePanel);

    row![video, splitter, panel]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
