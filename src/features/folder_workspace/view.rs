//! UI for folder workspace: only this module knows the workspace layout (row, splitters, region sizes).
//!
//! We pass only data to each feature view (directory, current_file, selected_file, etc.).
//! We do not tell any feature how to look (scrollable, rectangular, etc.); each feature view owns its appearance.

use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::features::{file_workspace, folder, folder_controls, media_viewer};
use crate::theme;
use crate::widgets::splitter::{Splitter, HIT_WIDTH};

use super::{FolderWorkspace, Message};

/// Workspace layout: one big drop panel when no folder is open; otherwise regions and splitters.
pub fn view(state: &FolderWorkspace) -> Element<'_, Message> {
    if state.directory().is_none() {
        let icon = if state.is_loading() { "⏳" } else { "📂" };
        return container(
            container(
                text(icon)
                    .size(120)
                    .color(theme::TEXT_MUTED),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_container_style)
        .into();
    }

    let video = container(
        media_viewer::view::view(state.media_viewer()).map(Message::MediaViewer),
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
            state.is_loading(),
            state.file_workspace().tag_color_mapping(),
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
    let file_workspace_panel = file_workspace::view::view(
        file_ws,
        state.tag_panel(),
        state.file_name_panel(),
        file_ws.tag_list(),
    )
    .map(|m| match m {
        file_workspace::Message::TagPanel(m) => Message::TagPanel(m),
        file_workspace::Message::FileNamePanel(m) => Message::FileNamePanel(m),
    });

    container(
        row![video, left_splitter, folder_list, right_splitter, file_workspace_panel]
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::main_container_style)
    .into()
}
