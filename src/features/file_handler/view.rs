//! UI rendering for file handler feature

use iced::widget::row;
use iced::{Element, Length};

use crate::features::{rename_panel, video_player};

use super::{FileHandlerState, Message};

/// Render the file handler view: video player on the left, rename panel on the right
pub fn view(state: &FileHandlerState) -> Element<'_, Message> {
    let video = video_player::view::view(state.video_player()).map(Message::VideoPlayer);
    let panel = rename_panel::view::view(state.rename_panel()).map(Message::RenamePanel);

    row![video, panel]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
