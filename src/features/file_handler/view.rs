//! UI rendering for file handler feature

use iced::widget::{container, row};
use iced::{Element, Length};

use crate::features::{folder, rename_panel, video_player};
use crate::widgets::splitter::{Splitter, HIT_WIDTH};
use super::{FileHandlerState, Message};

/// Render the file handler view:
/// [Video Player] | splitter | [Folder] | splitter | [Rename Panel]
pub fn view(state: &FileHandlerState) -> Element<'_, Message> {
    let video = container(
        video_player::view::view(state.video_player()).map(Message::VideoPlayer),
    )
    .width(Length::Fixed(state.left_width()))
    .height(Length::Fill);

    let left_splitter = Splitter::new(Message::LeftSplitterDragged)
        .min_left(150.0)
        .min_right(200.0);

    let folder_list = container(
        folder::view::view(state.folder()).map(Message::Folder),
    )
    .width(Length::Fixed(state.folder_width()))
    .height(Length::Fill);

    // Right splitter: left bound = after video + left splitter + min folder
    let right_min_left = state.left_width() + HIT_WIDTH + 120.0;
    let right_splitter = Splitter::new(Message::RightSplitterDragged)
        .min_left(right_min_left)
        .min_right(200.0);

    let selected_file = state
        .folder()
        .directory()
        .and_then(|d| d.selected_file());
    let panel =
        rename_panel::view::view(state.rename_panel(), selected_file).map(Message::RenamePanel);

    row![video, left_splitter, folder_list, right_splitter, panel]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
