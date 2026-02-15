//! UI rendering for file handler feature

use iced::widget::{container, row};
use iced::{Element, Length};

use crate::features::{folder, rename_panel, video_player};
use crate::widgets::splitter::Splitter;
use super::{FileHandlerState, Message};

/// Render the file handler view:
/// [Video Player] | splitter | [Folder] | [Rename Panel]
pub fn view(state: &FileHandlerState) -> Element<'_, Message> {
    let video = container(
        video_player::view::view(state.video_player()).map(Message::VideoPlayer),
    )
    .width(Length::Fixed(state.left_width()))
    .height(Length::Fill);

    let splitter = Splitter::new(Message::SplitterDragged)
        .min_left(150.0)
        .min_right(200.0);

    let folder_list = container(
        folder::view::view(state.folder()).map(Message::Folder),
    )
    .width(Length::Fixed(200.0))
    .height(Length::Fill);

    let panel = rename_panel::view::view(state.rename_panel()).map(Message::RenamePanel);

    row![video, splitter, folder_list, panel]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
