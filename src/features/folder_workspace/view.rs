//! UI for folder workspace: only this module knows the workspace layout (row, splitters, region sizes).
//!
//! We pass only data to each feature view (directory, current_file, selected_file, etc.).
//! We do not tell any feature how to look (scrollable, rectangular, etc.); each feature view owns its appearance.

use iced::widget::{column, container, mouse_area, row, stack};
use iced::{Alignment, Element, Length};

use crate::features::folder::view::ListProps;
use crate::features::folder_controls::view::ToolbarProps;
use crate::features::{
    batch, file_menu, file_workspace, folder, folder_controls, media_viewer, recent_folders,
};
use crate::ui::icons::{icon, spinner, Icon};
use crate::ui::palette::TagPalette;
use crate::ui::tokens::*;
use crate::ui::{button, style, text};
use crate::widgets::right_press_reporter::RightPressReporter;
use crate::widgets::splitter::Splitter;

use super::{FolderWorkspace, Message};

/// Workspace layout: one big drop panel when no folder is open; otherwise regions and splitters.
/// `update_available` is the newer frename version found by the update check, if any.
pub fn view(
    state: &FolderWorkspace,
    tag_palette: TagPalette,
    update_available: Option<String>,
) -> Element<'_, Message> {
    let seg_start = state.file_workspace().segment_start_secs();
    let seg_end = state.file_workspace().segment_end_secs();
    let markers = media_viewer::video::view::MarkersView {
        markers: state.file_workspace().markers(),
        state: state.markers(),
        pane_width: state.left_width(),
        spinner_frame: state.spinner_frame(),
        unnamed: state.unnamed_marker_guids().len(),
    };

    if state.directory().is_none() && !state.media_fullscreen() {
        return empty_window(
            state.is_loading(),
            state.spinner_frame(),
            state.recent_folders(),
        );
    }

    // When fullscreen overlay is active, render blank space here — otherwise the video
    // controls' tooltips (rendered at window level) bleed through the fullscreen overlay.
    let video = container(if state.media_fullscreen() {
        iced::widget::Space::new().into()
    } else {
        media_viewer::view::view(state.media_viewer(), false, seg_start, seg_end, markers)
            .map(Message::MediaViewer)
    })
    .width(Length::Fixed(state.left_width()))
    .height(Length::Fill);

    // Each splitter keeps the columns on both sides at least as wide as their minimum.
    let left_splitter = Splitter::new(Message::LeftSplitterDragged)
        .min_left(VIDEO_MIN_WIDTH)
        .min_right(FILE_LIST_MIN_WIDTH + SPLITTER_HIT + TAGS_MIN_WIDTH);

    let (has_previous, has_next) = state.has_previous_next();
    let has_selected = state.current_file().is_some();

    // Bound to a local so the folder list can borrow it instead of taking a clone per frame.
    let tag_color_mapping = state.file_workspace().tag_color_mapping();
    let folder_col: Element<'_, folder::Message> = column![
        container(folder::view::view(ListProps {
            directory: state.directory(),
            loading: state.is_loading(),
            tag_color_mapping: &tag_color_mapping,
            tag_palette,
            rename: state.inline_rename(),
            spinner_frame: state.spinner_frame(),
            batch: state.batch().is_active().then_some(state.batch()),
            markers_not_saved: state.unsaved_markers(),
            filter_menu_open: state.filter_menu_open(),
            width: state.folder_width(),
        }))
        .height(Length::Fill),
        folder_controls::view::view(ToolbarProps {
            has_previous,
            has_next,
            has_selected,
            batch_mode: state.batch().is_active(),
            batch_running: state.batch().is_running(),
            recent_open: state.recent_folders().is_open(),
            update_available,
        }),
    ]
    .height(Length::Fill)
    .into();
    let folder_with_controls = folder_col.map(Message::Folder);

    let folder_list = container(folder_with_controls)
        .width(Length::Fixed(state.folder_width()))
        .height(Length::Fill);

    let right_min_left = state.left_width() + SPLITTER_HIT + FILE_LIST_MIN_WIDTH;
    let right_splitter = Splitter::new(Message::RightSplitterDragged)
        .min_left(right_min_left)
        .min_right(TAGS_MIN_WIDTH);

    // Batch mode shows the batch actions where the open file's tags and name are.
    let right_panel: Element<'_, Message> = if state.batch().is_active() {
        batch::view::view(state.batch(), state.directory(), state.spinner_frame())
            .map(Message::Batch)
    } else {
        let file_ws = state.file_workspace();
        let is_synced = file_ws.tag_list().is_selected_match_display_order();
        file_workspace::view::view(
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
            file_workspace::Message::CommentLayout(l) => Message::CommentLayout(l),
        })
    };

    let normal_layout = container(
        row![
            video,
            left_splitter,
            folder_list,
            right_splitter,
            right_panel
        ]
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(style::panel);

    // Always use stack so the root element type never changes — iced preserves
    // scrollable positions only when the widget-tree structure stays identical.
    // The overlay is the fullscreen media view when active, or an invisible space.
    let overlay: Element<'_, Message> = if state.media_fullscreen() {
        // `opaque`: the rows and cells under the fullscreen video must not get the hover (and
        // show their tooltips over it).
        iced::widget::opaque(
            container(
                media_viewer::view::view(state.media_viewer(), true, seg_start, seg_end, markers)
                    .map(Message::MediaViewer),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        )
    } else {
        iced::widget::Space::new().into()
    };

    // The file menu over everything, once a file is right-clicked (nothing otherwise).
    let file_menu = file_menu::view::view(state.file_menu()).map(Message::FileMenu);
    // The recent folders over the toolbar's open button, while their list is open.
    let recent = recent_folders::view::dropdown(state.recent_folders(), state.left_width())
        .map(Message::RecentFolders);

    // The right button's position is taken here, over the whole window: a row inside the
    // scrolled file list does not know where it is on screen.
    RightPressReporter::new(
        stack![normal_layout, overlay, file_menu, recent]
            .width(Length::Fill)
            .height(Length::Fill),
        |position| Message::FileMenu(file_menu::Message::RightPressed(position)),
    )
    .into()
}

/// The whole window when no folder is open (§13.7): what to do first and the button that does
/// it, or the folder opening. A click anywhere picks a folder too; a right-click picks one file.
/// With folders opened before, their list stands beside it, so the first click can be one of them.
fn empty_window<'a>(
    loading: bool,
    spinner_frame: usize,
    recent: &'a recent_folders::RecentFoldersState,
) -> Element<'a, Message> {
    if loading {
        return container(
            column![
                spinner(spinner_frame, ICON_L, TEXT_SECONDARY),
                text::secondary(fl!("folder-opening"))
            ]
            .spacing(SPACE_S)
            .align_x(Alignment::Center),
        )
        .center(Length::Fill)
        .style(style::panel)
        .into();
    }
    let block = column![
        icon(Icon::FolderOpen, ICON_XL, TEXT_SECONDARY),
        text::heading(fl!("folder-window-empty-title")),
        text::secondary(fl!("folder-window-empty-line")),
        button::primary(fl!("folder-window-open")).on_press(Message::OpenFolderPicker),
    ]
    .spacing(SPACE_M)
    .max_width(EMPTY_SCREEN_MAX_WIDTH);
    let content: Element<'a, Message> = if recent.is_empty() {
        block.into()
    } else {
        row![
            block,
            recent_folders::view::on_empty_screen(recent).map(Message::RecentFolders)
        ]
        .spacing(SPACE_XL)
        .align_y(Alignment::Center)
        .into()
    };
    mouse_area(
        container(content)
            .center(Length::Fill)
            .padding(PAGE_PADDING_Y)
            .style(style::panel),
    )
    .on_press(Message::OpenFolderPicker)
    .on_right_press(Message::OpenFilePicker)
    .into()
}
