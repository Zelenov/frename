//! UI for folder workspace: only this module knows the workspace layout (row, splitters, region sizes).
//!
//! We pass only data to each feature view (directory, current_file, selected_file, etc.).
//! We do not tell any feature how to look (scrollable, rectangular, etc.); each feature view owns its appearance.

use iced::widget::{column, container, mouse_area, row, stack, text};
use iced::{Element, Length};

use crate::features::{file_workspace, folder, folder_controls, media_viewer};
use crate::tag_colors::TagPalette;
use crate::theme;
use crate::widgets::splitter::{Splitter, HIT_WIDTH};

use super::{FolderWorkspace, Message};

/// Workspace layout: one big drop panel when no folder is open; otherwise regions and splitters.
pub fn view(state: &FolderWorkspace, tag_palette: TagPalette) -> Element<'_, Message> {
    let seg_start = state.file_workspace().segment_start_secs();
    let seg_end = state.file_workspace().segment_end_secs();
    let screenshot_secs: Vec<f32> = state.file_workspace().screenshots()
        .iter()
        .map(|s| s.position_secs())
        .collect();

    if state.directory().is_none() && !state.media_fullscreen() {
        let icon = if state.is_loading() { "⏳" } else { "📂" };
        let inner = container(
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
        .style(theme::panel_container_style);

        return if state.is_loading() {
            inner.into()
        } else {
            mouse_area(inner)
                .on_press(Message::OpenFilePicker)
                .into()
        };
    }

    // When fullscreen overlay is active, render blank space here — otherwise the video
    // controls' tooltips (rendered at window level) bleed through the fullscreen overlay.
    let video = container(if state.media_fullscreen() {
        iced::widget::Space::new().into()
    } else {
        media_viewer::view::view(state.media_viewer(), false, seg_start, seg_end, screenshot_secs.clone())
            .map(Message::MediaViewer)
    })
    .width(Length::Fixed(state.left_width()))
    .height(Length::Fill);

    let left_splitter = Splitter::new(Message::LeftSplitterDragged)
        .min_left(150.0)
        .min_right(200.0);

    let (has_previous, has_next) = state.has_previous_next();
    let has_selected = state.current_file().is_some();
    let filters = state
        .directory()
        .map(|d| folder_controls::view::ListFilters {
            untagged_only: d.untagged_only(),
            untagged_count: d.untagged_count(),
            subtitled_only: d.subtitled_only(),
            subtitled_count: d.subtitled_count(),
            commented_only: d.commented_only(),
            commented_count: d.commented_count(),
        })
        .unwrap_or_default();

    // Bound to a local so the folder list can borrow it instead of taking a clone per frame.
    let tag_color_mapping = state.file_workspace().tag_color_mapping();
    let folder_col: Element<'_, folder::Message> = column![
        container(folder::view::view(
            state.directory(),
            state.is_loading(),
            &tag_color_mapping,
            tag_palette,
            state.inline_rename(),
        ))
        .height(Length::Fill),
        folder_controls::view::view(
            has_previous,
            has_next,
            has_selected,
            filters,
        ),
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
    let is_synced = file_ws.tag_list().is_selected_match_display_order();
    let file_workspace_panel = file_workspace::view::view(
        file_ws,
        state.tag_panel(),
        state.file_name_panel(),
        is_synced,
        state.sync_locked(),
        file_ws.tag_list(),
        tag_palette,
    )
    .map(|m| match m {
        file_workspace::Message::TagPanel(m) => Message::TagPanel(m),
        file_workspace::Message::FileNamePanel(m) => Message::FileNamePanel(m),
        file_workspace::Message::SyncPanel(m) => Message::SyncPanel(m),
        file_workspace::Message::CommentAction(a) => Message::CommentAction(a),
    });

    let normal_layout = container(
        row![video, left_splitter, folder_list, right_splitter, file_workspace_panel]
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::main_container_style);

    // Always use stack so the root element type never changes — iced preserves
    // scrollable positions only when the widget-tree structure stays identical.
    // The overlay is the fullscreen media view when active, or an invisible space.
    let overlay: Element<'_, Message> = if state.media_fullscreen() {
        container(
            media_viewer::view::view(state.media_viewer(), true, seg_start, seg_end, screenshot_secs)
                .map(Message::MediaViewer),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        iced::widget::Space::new().into()
    };

    stack![normal_layout, overlay]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
