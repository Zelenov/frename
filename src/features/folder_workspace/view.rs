//! UI for folder workspace: only this module knows the workspace layout (row, splitters, region sizes).
//!
//! We pass only data to each feature view (directory, current_file, selected_file, etc.).
//! We do not tell any feature how to look (scrollable, rectangular, etc.); each feature view owns its appearance.

use iced::widget::{column, container, row};
use iced::{Background, Element, Length};

use crate::features::{file_workspace, folder, folder_controls, video_player};
use crate::theme;
use crate::widgets::splitter::{Splitter, HIT_WIDTH};

use super::{FolderWorkspace, Message};

/// Workspace layout: regions and splitters. Child views receive only data; they decide how they look.
pub fn view(state: &FolderWorkspace) -> Element<'_, Message> {
    let video = container(
        video_player::view::view(state.video_player()).map(Message::VideoPlayer),
    )
    .width(Length::Fixed(state.left_width()))
    .height(Length::Fill);

    let left_splitter = Splitter::new(Message::LeftSplitterDragged)
        .min_left(150.0)
        .min_right(200.0);

    let (has_previous, has_next) = state.has_previous_next();

    let folder_col: Element<'_, folder::Message> = column![
        container(folder::view::view(
            state.directory(),
            state.current_file(),
            state.is_loading(),
        ))
        .height(Length::Fill),
        folder_controls::view::view(has_previous, has_next),
    ]
    .height(Length::Fill)
    .into();
    let folder_with_controls = folder_col.map(Message::Folder);

    let folder_list = container(folder_with_controls)
        .width(Length::Fixed(state.folder_width()))
        .height(Length::Fill);

    let right_min_left = state.left_width() + HIT_WIDTH + 120.0;
    let right_splitter = Splitter::new(Message::RightSplitterDragged)
        .min_left(right_min_left)
        .min_right(200.0);

    let file_ws = state.file_workspace();
    let file_workspace_panel =
        file_workspace::view::view(file_ws, state.tag_panel()).map(Message::TagPanel);

    container(
        row![video, left_splitter, folder_list, right_splitter, file_workspace_panel]
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|_theme| iced::widget::container::Style {
        background: Some(Background::Color(theme::BG_MAIN)),
        ..Default::default()
    })
    .into()
}
