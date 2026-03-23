//! State for folder workspace: only data and selection logic.
//!
//! Owns: loaded folder (directory + files), currently selected file, loading flag.
//! Also holds child feature state (video, rename panel) and workspace layout (splitter positions).
//! No UI concepts (scrollable, shape, size of children)—only data passed to feature views.
//! Each feature view decides how it looks; the workspace view only arranges regions.

use std::path::PathBuf;

use arboard;
use frename_core::{
    AppDatabase, AppStateStore, File, FileId, FileSnapshot, FolderAndFile, LoggingAppStateStore,
    NavigateFileCommand, ReorderTagCommand, ToggleTagCommand, PasteTagsCommand,
    DeleteTagCommand, CreateTagCommand, SaveTagCommand, StarTagCommand,
    SetSegmentStartCommand, SetSegmentEndCommand,
    SaveAndReparse, UndoContext, UndoError,
};
use frename_core::undo::History;

use super::messages::GlobalSearchKey;
use super::Directory;
use iced::{Subscription, Task};
use iced::widget::operation;

use crate::features::file_name_panel::{self, FileNamePanelState};
use crate::features::file_workspace::FileWorkspace;
use crate::features::folder;
use crate::features::media_viewer::{self, video as media_viewer_video, MediaViewerState};
use crate::features::sync_panel;
use crate::features::tag_panel::{self, TagPanelState, TAG_LIST_SCROLLABLE_ID};
use crate::widgets::search_bar::SEARCH_BAR_INPUT_ID;
use crate::widgets::splitter::HIT_WIDTH;

use super::Message;

const DEFAULT_LEFT_WIDTH: f32 = 460.0;
const DEFAULT_FOLDER_WIDTH: f32 = 200.0;
const MIN_FOLDER_WIDTH: f32 = 120.0;

/// Concrete history type for this workspace: Directory uses LoggingAppStateStore<AppDatabase>,
/// TagList uses AppDatabase.
type WorkspaceHistory = History<LoggingAppStateStore<AppDatabase>, AppDatabase>;

/// Folder workspace: owns directory, loading. Current file is the directory's selection.
pub struct FolderWorkspace {
    directory: Option<Directory>,
    loading: bool,
    /// File currently being edited: copy of file + tag selection. Rename panel reads/updates this.
    file_workspace: FileWorkspace<AppDatabase>,
    media_viewer: MediaViewerState,
    tag_panel: TagPanelState,
    file_name_panel: FileNamePanelState,
    /// Whether lock mode is active (sync drag → DB). Shown when sequences are equal.
    sync_locked: bool,
    /// Deferred rename: set when media must unload before the previous file can be renamed.
    /// Stores the file's stable ID and the tag snapshot to save.
    pending_file_updated: Option<(FileId, FileSnapshot)>,
    /// The ID of the file navigated to (captured after dir.select_*).
    /// Consumed by apply_file_updated to build NavigateFileCommand.
    pending_to_file_id: Option<FileId>,
    left_width: f32,
    folder_width: f32,
    /// Last reported tag list scroll offset and viewport height (for scroll-into-view).
    tag_list_scroll_y: Option<f32>,
    tag_list_viewport_height: Option<f32>,
    /// Last reported folder list scroll offset and viewport height (for scroll-into-view).
    folder_scroll_y: Option<f32>,
    folder_viewport_height: Option<f32>,
    /// Internally copied tag names (for paste onto another file).
    copied_tags: Option<Vec<String>>,
    /// Undo/redo history for all undoable actions.
    history: WorkspaceHistory,
    /// Whether the media viewer is currently shown fullscreen (F5).
    media_fullscreen: bool,
}

impl FolderWorkspace {
    pub fn new() -> Self {
        let (left_width, folder_width) = AppDatabase::new().get_window_state()
            .and_then(|w| {
                if w.left_panel_width > 0.0 && w.folder_panel_width > 0.0 {
                    Some((w.left_panel_width, w.folder_panel_width))
                } else {
                    None
                }
            })
            .unwrap_or((DEFAULT_LEFT_WIDTH, DEFAULT_FOLDER_WIDTH));
        Self {
            directory: None,
            loading: false,
            file_workspace: FileWorkspace::<AppDatabase>::default(),
            media_viewer: MediaViewerState::default(),
            tag_panel: TagPanelState::default(),
            file_name_panel: FileNamePanelState::default(),
            sync_locked: true,
            pending_file_updated: None,
            pending_to_file_id: None,
            left_width,
            folder_width,
            tag_list_scroll_y: None,
            tag_list_viewport_height: None,
            folder_scroll_y: None,
            folder_viewport_height: None,
            copied_tags: None,
            history: WorkspaceHistory::new(50),
            media_fullscreen: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile(path) => self.open_file(path),
            Message::LoadLastSession => self.load_last_session(),
            Message::ScanFolder(pair) => self.scan_folder(pair),
            Message::FolderLoaded {
                directory,
                target_file,
            } => self.folder_loaded(directory, target_file),
            Message::FolderLoadFailed => self.folder_load_failed(),
            Message::FileOpened(file) => self.apply_file_opened(file),
            Message::FileUpdated { id, snapshot } => self.apply_file_updated(id, snapshot),
            Message::Folder(folder_msg) => self.handle_folder_message(folder_msg),
            Message::MediaViewer(msg) => match msg {
                media_viewer::Message::Unloaded => self.on_media_unloaded(),
                media_viewer::Message::ToggleFullscreen => {
                    if self.media_viewer.is_previewable() {
                        self.media_fullscreen = !self.media_fullscreen;
                    }
                    Task::none()
                }
                media_viewer::Message::SegmentStartMarked(secs) => self.set_segment_start(secs),
                media_viewer::Message::SegmentEndMarked(secs) => self.set_segment_end(secs),
                other => self.media_viewer.update(other).map(Message::MediaViewer),
            },
            Message::TagPanel(msg) => self.handle_tag_panel(msg),
            Message::FileNamePanel(msg) => self.handle_file_name_panel(msg),
            Message::SyncPanel(msg) => self.handle_sync_panel(msg),
            Message::LeftSplitterDragged(x) => {
                self.left_width = x;
                let folder_start = self.left_width + HIT_WIDTH;
                let folder_end = folder_start + self.folder_width;
                let new_folder_width = folder_end - x - HIT_WIDTH;
                self.folder_width = new_folder_width.max(MIN_FOLDER_WIDTH);
                AppDatabase::new().set_panel_widths(self.left_width, self.folder_width);
                Task::none()
            }
            Message::RightSplitterDragged(x) => {
                let new_folder_width = x - self.left_width - HIT_WIDTH;
                self.folder_width = new_folder_width.max(MIN_FOLDER_WIDTH);
                AppDatabase::new().set_panel_widths(self.left_width, self.folder_width);
                Task::none()
            }
            Message::FocusSearchBarAndKey(key) => self.focus_search_bar_and_key(key),
            Message::Noop => Task::none(),
            Message::ScrollFolderListToSelected => self.scroll_folder_list_to_selected(),
            Message::FolderListScrollAdjusted(y) => {
                self.folder_scroll_y = Some(y);
                Task::none()
            }
            Message::ScrollTagListToSelection => self.scroll_tag_list_to_selection(),
            Message::TagListScrollAdjusted(scroll_y) => {
                self.tag_list_scroll_y = Some(scroll_y);
                Task::none()
            }
            Message::RemoveTag => self.handle_tag_panel(tag_panel::Message::DeleteSelectedTag),
            Message::SaveSelectedTag => {
                if let Some(id) = self.tag_panel.selected_tag_id() {
                    self.handle_tag_panel(tag_panel::Message::SaveTag(id))
                } else {
                    Task::none()
                }
            }
            Message::CopyTags => self.copy_tags(),
            Message::PasteTags => self.paste_tags(),
            Message::Undo => self.perform_undo(),
            Message::Redo => self.perform_redo(),
            Message::ToggleMediaFullscreen => {
                // Only toggle when media is active (video or image).
                if self.media_viewer.is_previewable() {
                    self.media_fullscreen = !self.media_fullscreen;
                }
                Task::none()
            }
            Message::SetSegmentStart => Task::done(Message::MediaViewer(
                media_viewer::Message::Video(media_viewer_video::Message::CaptureSegmentStart),
            )),
            Message::SetSegmentEnd => Task::done(Message::MediaViewer(
                media_viewer::Message::Video(media_viewer_video::Message::CaptureSegmentEnd),
            )),
            Message::EscapePressed => {
                if self.media_fullscreen {
                    self.media_fullscreen = false;
                    Task::none()
                } else {
                    self.handle_tag_panel(tag_panel::Message::SetFilter(String::new()))
                }
            }
        }
    }

    /// Emulate the key into the filter and focus the search bar (Iced cannot replay the event to the widget).
    fn focus_search_bar_and_key(&mut self, key: GlobalSearchKey) -> Task<Message> {
        let current = self.file_workspace.tag_list().filter_query().to_string();
        let new_value = match key {
            GlobalSearchKey::Char(c) => format!("{}{}", current, c),
            GlobalSearchKey::Backspace | GlobalSearchKey::Delete => {
                let mut s = current;
                s.pop();
                s
            }
        };
        self.file_workspace.set_tag_filter(new_value);
        operation::focus(iced::widget::Id::from(SEARCH_BAR_INPUT_ID))
            .map(|_: ()| Message::Noop)
    }

    fn set_segment_start(&mut self, secs: f32) -> Task<Message> {
        if self.file_workspace.file().is_none() { return Task::none(); }
        let old_secs = self.file_workspace.segment_start_secs();
        self.file_workspace.set_segment_start_secs(Some(secs));
        self.history.push(Box::new(SetSegmentStartCommand { old_secs, new_secs: Some(secs) }));
        Task::none()
    }

    fn set_segment_end(&mut self, secs: f32) -> Task<Message> {
        if self.file_workspace.file().is_none() { return Task::none(); }
        let old_secs = self.file_workspace.segment_end_secs();
        self.file_workspace.set_segment_end_secs(Some(secs));
        self.history.push(Box::new(SetSegmentEndCommand { old_secs, new_secs: Some(secs) }));
        Task::none()
    }

    fn copy_tags(&mut self) -> Task<Message> {
        let Some((_, snapshot)) = self.file_workspace.get_snapshot() else {
            return Task::none();
        };
        self.copied_tags = Some(snapshot.tags().to_vec());
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(snapshot.file_name());
        }
        Task::none()
    }

    fn paste_tags(&mut self) -> Task<Message> {
        let Some(names) = self.copied_tags.clone() else {
            return Task::none();
        };
        let Some((_, snapshot_before)) = self.file_workspace.get_snapshot() else {
            return Task::none();
        };
        let snapshot_after = FileSnapshot::new(
            names,
            snapshot_before.name_without_extension(),
            snapshot_before.extension(),
            snapshot_before.initial_file_name(),
        );
        self.file_workspace.reinitialize_tags_from_snapshot(snapshot_after.clone());
        self.clamp_selection_to_filtered();
        self.history.push(Box::new(PasteTagsCommand { snapshot_before, snapshot_after }));
        Task::none()
    }

    fn load_last_session(&self) -> Task<Message> {
        let store = LoggingAppStateStore::new(AppDatabase::new());
        let Some(session) = store.get_last_session() else {
            return Task::none();
        };
        Task::done(Message::ScanFolder(session))
    }

    fn open_file(&mut self, path: PathBuf) -> Task<Message> {
        let Some(file) = self
            .directory
            .as_mut()
            .and_then(|dir| dir.open_path(&path))
        else {
            self.file_workspace.set_file(None);
            self.pending_file_updated = None;
            return Task::done(Message::ScanFolder(FolderAndFile::new(
                path.parent().unwrap_or(&path),
                Some(path.clone()),
            )));
        };
        Task::done(Message::FileOpened(file))
    }

    fn scan_folder(&mut self, pair: FolderAndFile) -> Task<Message> {
        let folder = pair.folder().to_path_buf();
        let target_file = pair.file().map(|p| p.to_path_buf());
        let store = LoggingAppStateStore::new(AppDatabase::new());
        self.loading = true;
        self.file_workspace.set_file(None);
        self.pending_file_updated = None;

        Task::future(async move {
            match Directory::open(&folder, store).await {
                Ok(dir) => Message::FolderLoaded {
                    directory: dir,
                    target_file,
                },
                Err(_) => Message::FolderLoadFailed,
            }
        })
    }

    fn folder_load_failed(&mut self) -> Task<Message> {
        self.loading = false;
        Task::none()
    }

    fn folder_loaded(&mut self, directory: Directory, target_file: Option<PathBuf>) -> Task<Message> {
        self.loading = false;
        self.directory = Some(directory); // replace previous directory only on success
        let dir = self.directory.as_mut().expect("just set");
        let selected = target_file.as_deref().and_then(|p| dir.open_path(p));
        if let Some(file) = selected {
            Task::batch([
                Task::done(Message::FileOpened(file)),
                Task::done(Message::ScrollFolderListToSelected),
            ])
        } else {
            self.file_workspace.set_file(None);
            self.pending_file_updated = None;
            Task::none()
        }
    }

    fn apply_file_opened(&mut self, file: frename_core::File) -> Task<Message> {
        self.media_fullscreen = false;
        let snapshot = self.file_workspace.get_snapshot();
        self.file_workspace.set_file(Some(file.clone()));
        log::info!("Opening file: {}", file.file_path().display());
        let Some((id, snap)) = snapshot else {
            self.pending_file_updated = None;
            return self.media_viewer.open(&file).map(Message::MediaViewer);
        };
        if self.media_viewer.needs_unload_before_rename() {
            self.pending_file_updated = Some((id, snap));
            Task::done(Message::MediaViewer(media_viewer::Message::Unload))
        } else {
            Task::batch([
                Task::done(Message::FileUpdated { id, snapshot: snap }),
                self.media_viewer.open(&file).map(Message::MediaViewer),
            ])
        }
    }

    /// Called when media has unloaded. Persist pending snapshot (FileUpdated) then open the new media.
    fn on_media_unloaded(&mut self) -> Task<Message> {
        let Some((id, snapshot)) = self.pending_file_updated.take() else {
            return Task::none();
        };
        let Some(file) = self
            .directory
            .as_ref()
            .and_then(|d| d.selected_file())
            .cloned()
        else {
            return Task::none();
        };
        Task::batch([
            Task::done(Message::FileUpdated { id, snapshot }),
            self.media_viewer.open(&file).map(Message::MediaViewer),
        ])
    }

    fn apply_file_updated(&mut self, id: FileId, snapshot: frename_core::FileSnapshot) -> Task<Message> {
        // Resolve the current on-disk path via the stable file ID.
        let Some(current_path) = self
            .directory
            .as_ref()
            .and_then(|d| d.file_by_id(id))
            .map(|f| f.file_path().to_path_buf())
        else {
            log::warn!("apply_file_updated: file id not found in directory");
            return Task::none();
        };

        let path_before = current_path.clone();
        let (new_path, snapshot_after_save) = snapshot.save_and_reparse(&current_path);

        let _ = self
            .directory
            .as_mut()
            .map(|dir| dir.rename_file(id, &new_path, &snapshot_after_save));

        // Push NavigateFileCommand when we have a valid to_file_id (set by select_* after navigating).
        if let Some(to_file_id) = self.pending_to_file_id.take() {
            self.history.push(Box::new(NavigateFileCommand {
                file_id: id,
                to_file_id,
                path_before,
                path_after: new_path,
                snapshot_before: snapshot,
                snapshot_after: snapshot_after_save,
            }));
        }

        Task::none()
    }

    fn handle_folder_message(&mut self, msg: folder::Message) -> Task<Message> {
        match msg {
            folder::Message::SelectFile(index) => self.select_file_at(index),
            folder::Message::PreviousFile => self.select_previous(),
            folder::Message::NextFile => self.select_next(),
            folder::Message::Scrolled { scroll_y, viewport_height } => {
                self.folder_scroll_y = Some(scroll_y);
                self.folder_viewport_height = Some(viewport_height);
                Task::none()
            }
            folder::Message::ScrollToSelected => Task::done(Message::ScrollFolderListToSelected),
        }
    }

    fn select_file_at(&mut self, index: usize) -> Task<Message> {
        let Some(file) = self
            .directory
            .as_mut()
            .and_then(|dir| dir.select_index(index))
        else {
            self.pending_to_file_id = None;
            return Task::none();
        };
        self.pending_to_file_id = Some(file.id());
        Task::done(Message::FileOpened(file))
    }

    fn select_previous(&mut self) -> Task<Message> {
        let Some(file) = self
            .directory
            .as_mut()
            .and_then(|dir| dir.select_previous())
        else {
            self.pending_to_file_id = None;
            return Task::none();
        };
        self.pending_to_file_id = Some(file.id());
        Task::batch([
            Task::done(Message::FileOpened(file)),
            Task::done(Message::ScrollFolderListToSelected),
        ])
    }

    fn select_next(&mut self) -> Task<Message> {
        let Some(file) = self
            .directory
            .as_mut()
            .and_then(|dir| dir.select_next())
        else {
            self.pending_to_file_id = None;
            return Task::none();
        };
        self.pending_to_file_id = Some(file.id());
        Task::batch([
            Task::done(Message::FileOpened(file)),
            Task::done(Message::ScrollFolderListToSelected),
        ])
    }

    fn handle_tag_panel(
        &mut self,
        msg: tag_panel::Message,
    ) -> Task<Message> {
        // Capture drag state before update() clears it on DragEnded.
        let drag_on_end = if let tag_panel::Message::DragEnded = &msg {
            Some((self.tag_panel.dragging_tag_id(), self.tag_panel.drop_target_index()))
        } else {
            None
        };
        self.tag_panel
            .update(&msg, self.file_workspace.tag_list());
        match msg {
            tag_panel::Message::SetFilter(query) => {
                self.file_workspace.set_tag_filter(query);
                // Do not clamp selection: keep the selected tag even when it becomes hidden by the
                // filter. Space will then toggle the first visible tag and move selection there.
                Task::done(Message::ScrollTagListToSelection)
            }
            tag_panel::Message::CreateTag(name) => {
                let name = name.trim().to_string();
                if !name.is_empty() {
                    match self.file_workspace.create_and_save_new_tag(name.clone()) {
                        Ok(id) => {
                            let color_index = self.file_workspace.tag_list().get_tag(id).map_or(0, |t| t.color_index());
                            self.history.push(Box::new(CreateTagCommand { tag_id: id, tag_name: name, color_index }));
                            self.tag_panel.set_selected(Some(id));
                            self.clamp_selection_to_filtered();
                            return Task::done(Message::ScrollTagListToSelection);
                        }
                        Err(e) => log::error!("Failed to create tag: {}", e),
                    }
                }
                Task::none()
            }
            tag_panel::Message::TagListScrolled {
                scroll_y,
                viewport_height,
            } => {
                self.tag_list_scroll_y = Some(scroll_y);
                self.tag_list_viewport_height = Some(viewport_height);
                Task::none()
            }
            tag_panel::Message::ToggleTag(id) => {
                let was_checked = self.file_workspace.tag_list().get_tag(id).map_or(false, |t| t.is_checked());
                self.file_workspace.toggle_tag_by_id(id);
                self.tag_panel.set_selected(Some(id));
                self.history.push(Box::new(ToggleTagCommand { tag_id: id, was_checked }));
                Task::none()
            }
            tag_panel::Message::SelectLeft => {
                self.move_selection_left();
                Task::done(Message::ScrollTagListToSelection)
            }
            tag_panel::Message::SelectRight => {
                self.move_selection_right();
                Task::done(Message::ScrollTagListToSelection)
            }
            tag_panel::Message::SelectUp => {
                self.move_selection_up();
                Task::done(Message::ScrollTagListToSelection)
            }
            tag_panel::Message::SelectDown => {
                self.move_selection_down();
                Task::done(Message::ScrollTagListToSelection)
            }
            tag_panel::Message::ToggleSelectedTag => {
                // If search bar had focus, it may have inserted a space; strip it so Space doesn't add to the filter.
                let filter = self.file_workspace.tag_list().filter_query();
                if filter.ends_with(' ') {
                    let trimmed = filter.trim_end();
                    self.file_workspace.set_tag_filter(trimmed.to_string());
                }
                // Decide which tag to toggle:
                // - If selected tag is visible → toggle it (keep selection).
                // - Otherwise → toggle the first visible tag and move selection to it.
                let filtered = self.file_workspace.tag_list().filtered_display_tag_ids().to_vec();
                let selected_id = self.tag_panel.selected_tag_id();
                let selected_visible = selected_id.filter(|id| filtered.contains(id));
                let id_to_toggle = selected_visible.or_else(|| filtered.first().copied());
                if let Some(id) = id_to_toggle {
                    let was_checked = self.file_workspace.tag_list().get_tag(id).map_or(false, |t| t.is_checked());
                    self.file_workspace.toggle_tag_by_id(id);
                    self.history.push(Box::new(ToggleTagCommand { tag_id: id, was_checked }));
                    if selected_visible.is_none() {
                        self.tag_panel.set_selected(Some(id));
                    }
                }
                Task::none()
            }
            tag_panel::Message::DeleteTag(id) => {
                // Find the deleted tag's position in the filtered list before removal.
                let filtered_before = self.file_workspace.tag_list().filtered_display_tag_ids().to_vec();
                let deleted_pos = filtered_before.iter().position(|&fid| fid == id);

                let delete_data = self.file_workspace.tag_list().capture_delete_data(id);
                if let Err(e) = self.file_workspace.remove_stored_tag_by_id(id) {
                    log::error!("Failed to delete tag: {}", e);
                } else if let Some((name, color, was_stored, was_starred, was_checked, sort_order)) = delete_data {
                    self.history.push(Box::new(DeleteTagCommand {
                        tag_id: id, tag_name: name, color_index: color,
                        was_stored, was_starred, was_checked, sort_order,
                    }));
                }

                // Select the neighbor: left (or first if deleted was first).
                if let Some(pos) = deleted_pos {
                    let filtered_after = self.file_workspace.tag_list().filtered_display_tag_ids().to_vec();
                    let new_i = if pos == 0 { 0 } else { pos - 1 };
                    self.tag_panel.set_selected(filtered_after.get(new_i).copied());
                } else {
                    self.clamp_selection_to_filtered();
                }
                Task::none()
            }
            tag_panel::Message::DeleteSelectedTag => {
                if let Some(id) = self.tag_panel.selected_tag_id() {
                    self.handle_tag_panel(tag_panel::Message::DeleteTag(id))
                } else {
                    Task::none()
                }
            }
            tag_panel::Message::SaveTag(id) => {
                if let Err(e) = self.file_workspace.save_tag(id) {
                    log::error!("Failed to save tag to store: {}", e);
                } else {
                    let color_index = self.file_workspace.tag_list().get_tag(id).map_or(0, |t| t.color_index());
                    self.history.push(Box::new(SaveTagCommand { tag_id: id, color_index }));
                }
                Task::none()
            }
            tag_panel::Message::ToggleStar(id) => {
                let was_starred = self
                    .file_workspace
                    .tag_list()
                    .get_tag(id)
                    .map_or(false, |t| t.is_starred());
                let result = if was_starred {
                    self.file_workspace.unstar_tag(id)
                } else {
                    self.file_workspace.star_tag(id)
                };
                match result {
                    Ok(()) => self.history.push(Box::new(StarTagCommand { tag_id: id, was_starred })),
                    Err(e) => log::error!("Failed to toggle star: {}", e),
                }
                Task::none()
            }
            tag_panel::Message::DragStarted(_) | tag_panel::Message::DragHoverCursor { .. } => {
                Task::none()
            }
            tag_panel::Message::DragEnded => {
                if let Some((Some(did), Some(display_idx))) = drag_on_end {
                    let tag_list = self.file_workspace.tag_list();
                    let display_ids = tag_list.filtered_display_tag_ids().to_vec();
                    // Map display index → checked index: count checked tags before display_idx.
                    let checked_idx = display_ids
                        .iter()
                        .take(display_idx)
                        .filter(|&&id| tag_list.get_tag(id).map_or(false, |t| t.is_checked()))
                        .count();
                    let from_index = self.file_workspace.tag_list().checked_index_of(did);
                    self.file_workspace.reorder_tag_to_index(did, checked_idx);
                    if let Some(fi) = from_index {
                        self.history.push(Box::new(ReorderTagCommand {
                            moved_id: did, from_index: fi, to_index: checked_idx,
                        }));
                    }
                }
                Task::none()
            }
            tag_panel::Message::PanelBounds { .. } => Task::none(),
        }
    }

    fn handle_file_name_panel(&mut self, msg: file_name_panel::Message) -> Task<Message> {
        if let file_name_panel::Message::ClearSegmentStart = msg {
            let old_secs = self.file_workspace.segment_start_secs();
            self.file_workspace.set_segment_start_secs(None);
            self.history.push(Box::new(SetSegmentStartCommand { old_secs, new_secs: None }));
            return Task::none();
        }
        if let file_name_panel::Message::ClearSegmentEnd = msg {
            let old_secs = self.file_workspace.segment_end_secs();
            self.file_workspace.set_segment_end_secs(None);
            self.history.push(Box::new(SetSegmentEndCommand { old_secs, new_secs: None }));
            return Task::none();
        }
        let (dragged_id, drop_index) = if let file_name_panel::Message::DragEnded = &msg {
            (
                self.file_name_panel.dragging_tag_id(),
                self.file_name_panel.drop_target_index(),
            )
        } else {
            (None, None)
        };
        self.file_name_panel
            .update(msg, self.file_workspace.tag_list());
        if let Some(id) = self.file_name_panel.take_dropped_dragged_tag_id() {
            self.file_workspace.toggle_tag_by_id(id);
        } else if let (Some(did), Some(idx)) = (dragged_id, drop_index) {
            let from_index = self.file_workspace.tag_list().checked_index_of(did);
            self.file_workspace.reorder_tag_to_index(did, idx);
            if let Some(fi) = from_index {
                self.history.push(Box::new(ReorderTagCommand {
                    moved_id: did,
                    from_index: fi,
                    to_index: idx,
                }));
            }
            // When locked: immediately push the new order to DB as well.
            if self.sync_locked {
                if let Err(e) = self.file_workspace.sync_selected_to_display() {
                    log::error!("sync locked: failed to persist display order: {}", e);
                }
            }
        }
        Task::none()
    }

    fn handle_sync_panel(&mut self, msg: sync_panel::Message) -> Task<Message> {
        match msg {
            sync_panel::Message::SyncUp => {
                if let Err(e) = self.file_workspace.sync_selected_to_display() {
                    log::error!("SyncUp failed: {}", e);
                } else {
                    self.sync_locked = true;
                }
            }
            sync_panel::Message::SyncDown => {
                self.file_workspace.sync_display_to_selected();
                self.sync_locked = true;
            }
            sync_panel::Message::ToggleLock => {
                // Only toggle when sequences are equal (i.e. lock button is active).
                if self.file_workspace.is_selected_order_same_as_display_order() {
                    self.sync_locked = !self.sync_locked;
                }
            }
        }
        Task::none()
    }

    fn perform_undo(&mut self) -> Task<Message> {
        if !self.history.can_undo() || self.directory.is_none() {
            return Task::none();
        }
        // Inner block: limits the lifetime of dir/tl borrows so we can use self after.
        let result: Result<(), UndoError> = {
            let dir = self.directory.as_mut().expect("checked above");
            let tl = self.file_workspace.tag_list_mut();
            let mut ctx = UndoContext { directory: dir, tag_list: tl };
            self.history.undo(&mut ctx)
        };
        match result {
            Ok(()) => self.refresh_after_undo_redo(),
            Err(e) => {
                log::warn!("Undo failed: {}", e);
                Task::none()
            }
        }
    }

    fn perform_redo(&mut self) -> Task<Message> {
        if !self.history.can_redo() || self.directory.is_none() {
            return Task::none();
        }
        let result: Result<(), UndoError> = {
            let dir = self.directory.as_mut().expect("checked above");
            let tl = self.file_workspace.tag_list_mut();
            let mut ctx = UndoContext { directory: dir, tag_list: tl };
            self.history.redo(&mut ctx)
        };
        match result {
            Ok(()) => self.refresh_after_undo_redo(),
            Err(e) => {
                log::warn!("Redo failed: {}", e);
                Task::none()
            }
        }
    }

    fn refresh_after_undo_redo(&self) -> Task<Message> {
        let file = self
            .directory
            .as_ref()
            .and_then(|d| d.selected_file())
            .cloned();
        match file {
            Some(f) => Task::done(Message::FileOpened(f)),
            None => Task::none(),
        }
    }

    /// Clear selection if the selected tag is not in the current filtered list (selection must be visible).
    fn clamp_selection_to_filtered(&mut self) {
        let tag_list = self.file_workspace.tag_list();
        let visible = self
            .tag_panel
            .selected_tag_id()
            .filter(|id| tag_list.filtered_display_tag_ids().contains(id));
        if visible.is_none() && self.tag_panel.selected_tag_id().is_some() {
            self.tag_panel.set_selected(None);
        }
    }

    /// Left: move by -1, wrap last→first.
    fn move_selection_left(&mut self) {
        let filtered = self.file_workspace.tag_list().filtered_display_tag_ids().to_vec();
        if filtered.is_empty() { self.tag_panel.set_selected(None); return; }
        let cur = self.tag_panel.selected_tag_id()
            .and_then(|id| filtered.iter().position(|&fid| fid == id));
        let new_i = match cur {
            None | Some(0) => filtered.len() - 1,
            Some(i) => i - 1,
        };
        self.tag_panel.set_selected(filtered.get(new_i).copied());
    }

    /// Right: move by +1, wrap last→first.
    fn move_selection_right(&mut self) {
        let filtered = self.file_workspace.tag_list().filtered_display_tag_ids().to_vec();
        if filtered.is_empty() { self.tag_panel.set_selected(None); return; }
        let last = filtered.len() - 1;
        let cur = self.tag_panel.selected_tag_id()
            .and_then(|id| filtered.iter().position(|&fid| fid == id));
        let new_i = match cur {
            None => 0,
            Some(i) if i >= last => 0,
            Some(i) => i + 1,
        };
        self.tag_panel.set_selected(filtered.get(new_i).copied());
    }

    /// Up: move one visual row up; top row wraps to last row at same column.
    fn move_selection_up(&mut self) {
        let filtered = self.file_workspace.tag_list().filtered_display_tag_ids().to_vec();
        if filtered.is_empty() { self.tag_panel.set_selected(None); return; }
        let cols = self.tag_panel.cols().max(1) as usize;
        let count = filtered.len();
        let cur = self.tag_panel.selected_tag_id()
            .and_then(|id| filtered.iter().position(|&fid| fid == id));
        let new_i = match cur {
            None => count - 1,
            Some(i) if i < cols => {
                // top row → jump to last row, same column
                let x = i % cols;
                let last_row = (count - 1) / cols;
                let new_i = last_row * cols + x;
                if new_i >= count { new_i - cols } else { new_i }
            }
            Some(i) => i - cols,
        };
        self.tag_panel.set_selected(filtered.get(new_i).copied());
    }

    /// Down: move one visual row down; last row wraps to first row at same column.
    fn move_selection_down(&mut self) {
        let filtered = self.file_workspace.tag_list().filtered_display_tag_ids().to_vec();
        if filtered.is_empty() { self.tag_panel.set_selected(None); return; }
        let cols = self.tag_panel.cols().max(1) as usize;
        let count = filtered.len();
        let cur = self.tag_panel.selected_tag_id()
            .and_then(|id| filtered.iter().position(|&fid| fid == id));
        let new_i = match cur {
            None => 0,
            Some(i) => {
                let new_i = i + cols;
                if new_i >= count { i % cols } else { new_i }
            }
        };
        self.tag_panel.set_selected(filtered.get(new_i).copied());
    }

    /// Scroll the tag list so the selected row is in view (scroll-into-view: only when selection would leave viewport).
    /// Uses tag panel row_height and cols so list (1 col) and grid (N cols) both work.
    /// When we haven't received on_scroll yet, use panel bounds height as viewport and assume scroll_y = 0.
    fn scroll_tag_list_to_selection(&self) -> Task<Message> {
        let selected_id = match self.tag_panel.selected_tag_id() {
            Some(id) => id,
            None => return Task::none(),
        };
        let tag_list = self.file_workspace.tag_list();
        let filtered = tag_list.filtered_display_tag_ids();
        let flat_index = match filtered.iter().position(|&id| id == selected_id) {
            Some(i) => i,
            None => return Task::none(),
        };
        let cols = self.tag_panel.cols() as usize;
        let row_stride = self.tag_panel.row_height();
        let row_extent = self
            .tag_panel
            .row_content_height()
            .unwrap_or(row_stride);
        let visual_row = flat_index / cols;
        let row_top = (visual_row as f32) * row_stride;
        let row_bottom = row_top + row_extent;

        let (current, vh) = match (
            self.tag_list_scroll_y,
            self.tag_list_viewport_height,
            self.tag_panel.panel_bounds(),
        ) {
            (Some(y), Some(h), _) => (y, h),
            (y_opt, _, Some(bounds)) => {
                let vh = self.tag_list_viewport_height.unwrap_or(bounds.height);
                let current = y_opt.unwrap_or(0.0);
                (current, vh)
            }
            _ => return Task::none(),
        };

        let target_y = if row_top < current {
            row_top
        } else if row_bottom > current + vh {
            (row_bottom - vh).max(0.0)
        } else {
            return Task::none();
        };

        let offset = iced::widget::scrollable::AbsoluteOffset {
            x: None,
            y: Some(target_y),
        };
        let scroll_op = operation::scroll_to(
            iced::widget::Id::new(TAG_LIST_SCROLLABLE_ID),
            offset,
        )
        .map(|_: ()| Message::Noop);
        Task::batch([
            scroll_op,
            Task::done(Message::TagListScrollAdjusted(target_y)),
        ])
    }

    fn scroll_folder_list_to_selected(&self) -> Task<Message> {
        let Some(dir) = self.directory.as_ref() else {
            return Task::none();
        };
        let Some(index) = dir.selected_index() else {
            return Task::none();
        };
        let row_top = (index as f32) * folder::FOLDER_ROW_HEIGHT;
        let row_bottom = row_top + folder::FOLDER_ROW_HEIGHT;

        let current = self.folder_scroll_y.unwrap_or(0.0);
        let vh = self.folder_viewport_height.unwrap_or(f32::MAX);

        let target_y = if row_top < current {
            row_top
        } else if row_bottom > current + vh {
            (row_bottom - vh).max(0.0)
        } else {
            return Task::none();
        };

        let offset = iced::widget::scrollable::AbsoluteOffset {
            x: None,
            y: Some(target_y),
        };
        Task::batch([
            operation::scroll_to(iced::widget::Id::new(folder::FOLDER_LIST_SCROLLABLE_ID), offset)
                .map(|_: ()| Message::Noop),
            Task::done(Message::FolderListScrollAdjusted(target_y)),
        ])
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.media_viewer.subscription().map(Message::MediaViewer),
            self.file_name_panel.subscription().map(Message::FileNamePanel),
            self.tag_panel.subscription().map(Message::TagPanel),
        ])
    }

    /// Currently selected file (from directory selection).
    pub fn current_file(&self) -> Option<&File> {
        self.directory
            .as_ref()
            .and_then(|d| d.selected_file())
    }

    pub fn directory(&self) -> Option<&Directory> {
        self.directory.as_ref()
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// File workspace: current file and its tag selection (for rename panel). Use this for display and tag toggles.
    pub fn file_workspace(&self) -> &FileWorkspace<AppDatabase> {
        &self.file_workspace
    }

    pub fn has_previous_next(&self) -> (bool, bool) {
        self.directory
            .as_ref()
            .map(|d| d.has_previous_next())
            .unwrap_or((false, false))
    }

    pub fn media_viewer(&self) -> &MediaViewerState {
        &self.media_viewer
    }

    /// True when the media viewer is in fullscreen mode (F5).
    pub fn media_fullscreen(&self) -> bool {
        self.media_fullscreen
    }

    /// True when a video is active and must be unloaded before rename or close.
    pub fn needs_media_unload(&self) -> bool {
        self.media_viewer.needs_unload_before_rename()
    }

    /// Test helper: inject a pending deferred rename as if media is locked.
    /// Only available in test builds.
    #[cfg(test)]
    pub fn inject_pending_rename(&mut self, id: FileId, snapshot: FileSnapshot) {
        self.pending_file_updated = Some((id, snapshot));
    }

    /// Test helper: check whether a deferred rename is pending.
    #[cfg(test)]
    pub fn has_pending_rename(&self) -> bool {
        self.pending_file_updated.is_some()
    }

    pub fn tag_panel(&self) -> &TagPanelState {
        &self.tag_panel
    }

    pub fn file_name_panel(&self) -> &FileNamePanelState {
        &self.file_name_panel
    }

    pub fn sync_locked(&self) -> bool {
        self.sync_locked
    }

    pub fn left_width(&self) -> f32 {
        self.left_width
    }

    pub fn folder_width(&self) -> f32 {
        self.folder_width
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::SystemTime;

    use frename_core::{AppDatabase, File, FileId, FileSnapshot, Initializable, LoggingAppStateStore};

    use crate::features::{folder, tag_panel};

    use super::{Directory, FolderWorkspace, Message};

    /// Simulates the iced runtime processing a FileOpened task: directory already has selection, so send FileOpened(selected_file).
    fn flush_file_opened(workspace: &mut FolderWorkspace) {
        if let Some(file) = workspace
            .directory()
            .and_then(|d| d.selected_file())
            .cloned()
        {
            let _ = workspace.update(Message::FileOpened(file));
        }
    }

    /// Test fixture: a directory with a given number of files. Use in folder workspace tests.
    pub struct TestDirectory {
        pub directory: Directory,
        pub target_file: PathBuf,
    }

    impl TestDirectory {
        pub fn new(file_count: usize) -> Self {
            let dir = PathBuf::from("C:/test/folder");
            let files: Vec<File> = (0..file_count)
                .map(|i| {
                    File::from_path(
                        dir.join(format!("file_{}.mp4", i)),
                        SystemTime::UNIX_EPOCH,
                    )
                })
                .collect();
            let db = AppDatabase::new();
            let store = LoggingAppStateStore::new(db);
            store.initialize().unwrap();
            let directory = Directory::with_files(&dir, files, store);
            let target_file = dir.join("file_0.mp4");
            Self {
                directory,
                target_file,
            }
        }
    }

    #[test]
    fn open_folder_with_two_files_shows_two_in_folder_panel() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = FolderWorkspace::new();
        let _task = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory,
            target_file: Some(test_dir.target_file),
        });
        flush_file_opened(&mut workspace);

        assert_eq!(
            workspace.directory().map(|d| d.files_in_order().count()),
            Some(2),
            "folder panel should show two files"
        );
    }

    /// Helper: get the FileId of the file at the given directory index.
    fn file_id_at(workspace: &FolderWorkspace, index: usize) -> FileId {
        workspace
            .directory()
            .and_then(|d| d.files_in_order().nth(index))
            .map(|f| f.id())
            .expect("file at index should exist")
    }

    #[test]
    fn tags_are_saved_after_selecting_another_file() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory,
            target_file: Some(test_dir.target_file),
        });
        flush_file_opened(&mut workspace);

        // Capture file_0's stable ID before any navigation.
        let file_0_id = file_id_at(&workspace, 0);

        let tag_name = "Comedy";
        let tag_list = workspace.file_workspace().tag_list();
        let tag_id = tag_list
            .filtered_display_tag_ids()
            .iter()
            .find(|id| tag_list.get_tag(**id).map(|t| t.tag() == tag_name).unwrap_or(false))
            .copied()
            .expect("Comedy is a stored tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(tag_id)));

        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);

        // Manually drive the FileUpdated that would fire from the iced runtime.
        let snapshot = FileSnapshot::new(vec![tag_name.to_string()], "file_0", ".mp4", "file_0.mp4");
        let _ = workspace.update(Message::FileUpdated { id: file_0_id, snapshot });

        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        flush_file_opened(&mut workspace);

        assert!(
            workspace.file_workspace().file().unwrap().snapshot().has_tag(tag_name),
            "tags should be saved after selecting another file"
        );
    }

    /// When a video file is "loading" its GStreamer pipeline holds a file handle, so rename
    /// must be deferred.  Opening an .mp4 sets `video.loading = true`, which makes
    /// `needs_unload_before_rename()` return true on the next file switch — the rename is
    /// stored in `pending_file_updated` and only fires after `MediaViewer::Unloaded`.
    #[test]
    fn deferred_rename_fires_after_media_unloaded() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = FolderWorkspace::new();
        // Load folder; file_0 is selected and the video pipeline starts loading.
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory,
            target_file: Some(test_dir.target_file),
        });
        flush_file_opened(&mut workspace);
        // After opening file_0.mp4: video.loading = true → needs_unload_before_rename() = true.

        // Toggle a tag on file_0 so there is something to defer.
        let tag_list = workspace.file_workspace().tag_list();
        let tag_id = tag_list
            .filtered_display_tag_ids()
            .iter()
            .find(|id| tag_list.get_tag(**id).map(|t| t.tag() == "Comedy").unwrap_or(false))
            .copied()
            .expect("Comedy is a stored tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(tag_id)));

        // Navigate to file_1; because media is "loading", rename is deferred.
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);

        // The rename of file_0 must now be pending.
        assert!(
            workspace.has_pending_rename(),
            "rename must be deferred while video is loading/active"
        );

        // on_media_unloaded consumes the pending snapshot and emits FileUpdated + open next.
        // In tests we drive these manually since there is no iced runtime.
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));

        // Pending must now be consumed.
        assert!(
            !workspace.has_pending_rename(),
            "pending rename must be consumed after Unloaded"
        );

        // Drive the FileUpdated that on_media_unloaded emitted (manually re-emit here using the stable ID).
        let file_0_id = file_id_at(&workspace, 0);
        let _ = workspace.update(Message::FileUpdated {
            id: file_0_id,
            snapshot: frename_core::FileSnapshot::new(
                vec!["Comedy".to_string()],
                "file_0",
                ".mp4",
                "file_0.mp4",
            ),
        });

        // Navigate back to file_0 and verify its snapshot reflects the deferred rename.
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        flush_file_opened(&mut workspace);

        assert!(
            workspace
                .file_workspace()
                .file()
                .unwrap()
                .snapshot()
                .has_tag("Comedy"),
            "Comedy tag must survive the deferred rename on file_0"
        );
    }

    /// When the pending rename is consumed by `on_media_unloaded`, the pending field is cleared
    /// so a second `Unloaded` event does not cause a double rename.
    #[test]
    fn pending_rename_is_consumed_on_media_unloaded() {
        let test_dir = TestDirectory::new(2);

        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory,
            target_file: Some(test_dir.target_file),
        });
        flush_file_opened(&mut workspace);
        // file_0.mp4 is loading → needs_unload_before_rename() = true.

        // Navigate to file_1; the rename of file_0 is deferred.
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        assert!(workspace.has_pending_rename(), "rename must be pending");

        // First Unloaded: consumes the pending.
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        assert!(!workspace.has_pending_rename(), "pending must be None after Unloaded");

        // Second Unloaded: must be a no-op (nothing to consume).
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        assert!(!workspace.has_pending_rename(), "still None after second Unloaded");
    }

    /// Verify that `save_and_reparse` (called by FileUpdated) updates the file path in the
    /// directory when tags change the file name.
    #[test]
    fn file_updated_renames_file_path_in_directory() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory,
            target_file: Some(test_dir.target_file),
        });
        flush_file_opened(&mut workspace);

        // Capture file_0's stable ID before navigation.
        let file_0_id = file_id_at(&workspace, 0);

        // Navigate to file_1.
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);

        // Apply FileUpdated with a snapshot that changes the name: "Comedy.file_0.mp4".
        let snapshot = frename_core::FileSnapshot::new(
            vec!["Comedy".to_string()],
            "file_0",
            ".mp4",
            "file_0.mp4",
        );
        let _ = workspace.update(Message::FileUpdated { id: file_0_id, snapshot });

        // The directory should now track the file under its new path.
        let dir = workspace.directory().expect("directory should be loaded");
        let new_path = PathBuf::from("C:/test/folder/Comedy.file_0.mp4");
        let file_renamed = dir.files_in_order().any(|f| f.file_path() == new_path.as_path());
        assert!(
            file_renamed,
            "directory must track the renamed path Comedy.file_0.mp4"
        );
    }
}
