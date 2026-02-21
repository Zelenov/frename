//! State for folder workspace: only data and selection logic.
//!
//! Owns: loaded folder (directory + files), currently selected file, loading flag.
//! Also holds child feature state (video, rename panel) and workspace layout (splitter positions).
//! No UI concepts (scrollable, shape, size of children)—only data passed to feature views.
//! Each feature view decides how it looks; the workspace view only arranges regions.

use std::path::PathBuf;

use frename_core::{
    AppDatabase, AppStateStore, File, FileSnapshot, FolderAndFile, LoggingAppStateStore,
    SaveAndReparse,
};

use super::messages::GlobalSearchKey;
use super::Directory;
use iced::{Subscription, Task};
use iced::widget::operation;

use crate::features::file_name_panel::{self, FileNamePanelState};
use crate::features::file_workspace::FileWorkspace;
use crate::features::folder;
use crate::features::tag_panel::{self, TagPanelState, TAG_LIST_SCROLLABLE_ID};
use crate::features::video_player::{self, VideoPlayerState};
use crate::widgets::search_bar::SEARCH_BAR_INPUT_ID;
use crate::widgets::splitter::HIT_WIDTH;

use super::Message;

const DEFAULT_LEFT_WIDTH: f32 = 460.0;
const DEFAULT_FOLDER_WIDTH: f32 = 200.0;
const MIN_FOLDER_WIDTH: f32 = 120.0;

/// Folder workspace: owns directory, loading. Current file is the directory's selection.
pub struct FolderWorkspace {
    directory: Option<Directory>,
    loading: bool,
    /// File currently being edited: copy of file + tag selection. Rename panel reads/updates this.
    file_workspace: FileWorkspace<AppDatabase>,
    video_player: VideoPlayerState,
    tag_panel: TagPanelState,
    file_name_panel: FileNamePanelState,
    /// Snapshot to persist after current video is unloaded (then we send FileUpdated and load next video).
    pending_file_updated: Option<(PathBuf, FileSnapshot)>,
    left_width: f32,
    folder_width: f32,
    /// Last reported tag list scroll offset and viewport height (for scroll-into-view).
    tag_list_scroll_y: Option<f32>,
    tag_list_viewport_height: Option<f32>,
}

impl FolderWorkspace {
    pub fn new() -> Self {
        Self {
            directory: None,
            loading: false,
            file_workspace: FileWorkspace::<AppDatabase>::default(),
            video_player: VideoPlayerState::default(),
            tag_panel: TagPanelState::default(),
            file_name_panel: FileNamePanelState::default(),
            pending_file_updated: None,
            left_width: DEFAULT_LEFT_WIDTH,
            folder_width: DEFAULT_FOLDER_WIDTH,
            tag_list_scroll_y: None,
            tag_list_viewport_height: None,
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
            Message::FileUpdated { path, snapshot } => self.apply_file_updated(path, snapshot),
            Message::Folder(folder_msg) => self.handle_folder_message(folder_msg),
            Message::VideoPlayer(msg) => match msg {
                video_player::Message::VideoUnloaded => self.on_video_unloaded(),
                other => self.video_player.update(other).map(Message::VideoPlayer),
            },
            Message::TagPanel(msg) => self.handle_tag_panel(msg),
            Message::FileNamePanel(msg) => self.handle_file_name_panel(msg),
            Message::LeftSplitterDragged(x) => {
                self.left_width = x;
                let folder_start = self.left_width + HIT_WIDTH;
                let folder_end = folder_start + self.folder_width;
                let new_folder_width = folder_end - x - HIT_WIDTH;
                self.folder_width = new_folder_width.max(MIN_FOLDER_WIDTH);
                Task::none()
            }
            Message::RightSplitterDragged(x) => {
                let new_folder_width = x - self.left_width - HIT_WIDTH;
                self.folder_width = new_folder_width.max(MIN_FOLDER_WIDTH);
                Task::none()
            }
            Message::FocusSearchBarAndKey(key) => self.focus_search_bar_and_key(key),
            Message::Noop => Task::none(),
            Message::ScrollTagListToSelection => self.scroll_tag_list_to_selection(),
            Message::TagListScrollAdjusted(scroll_y) => {
                self.tag_list_scroll_y = Some(scroll_y);
                Task::none()
            }
            Message::RemoveTag => self.handle_tag_panel(tag_panel::Message::DeleteSelectedTag),
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
        let selected = dir.set_selection(target_file.as_deref());
        if let Some(file) = selected {
            Task::done(Message::FileOpened(file))
        } else {
            self.file_workspace.set_file(None);
            self.pending_file_updated = None;
            Task::none()
        }
    }

    fn apply_file_opened(&mut self, file: frename_core::File) -> Task<Message> {
        let snapshot = self.file_workspace.get_snapshot();
        self.file_workspace.set_file(Some(file.clone()));
        log::info!("Opening file: {}", file.file_path().display());
        let Some(snapshot) = snapshot else {
            self.pending_file_updated = None;
            let path = file.file_path().to_path_buf();
            return self.video_player.load_video(path).map(Message::VideoPlayer);
        };
        self.pending_file_updated = Some(snapshot);
        Task::done(Message::VideoPlayer(video_player::Message::Unload))
    }

    /// Called when video player has unloaded. Persist pending snapshot (FileUpdated) then load the new video.
    fn on_video_unloaded(&mut self) -> Task<Message> {
        let pending = self.pending_file_updated.take();
        let Some((path, snapshot)) = pending else {
            return Task::none();
        };
        let video_task = self
            .directory
            .as_ref()
            .and_then(|d| d.selected_file())
            .map(|f| {
                self.video_player
                    .load_video(f.file_path().to_path_buf())
                    .map(Message::VideoPlayer)
            })
            .unwrap_or(Task::none());
        Task::batch([
            Task::done(Message::FileUpdated { path, snapshot }),
            video_task,
        ])
    }

    fn apply_file_updated(&mut self, path: PathBuf, snapshot: frename_core::FileSnapshot) -> Task<Message> {
        let (path_buf, snapshot_after_save) = snapshot.save_and_reparse(&path);
        let _ = self
            .directory
            .as_mut()
            .map(|dir| dir.update_file(&path_buf, &snapshot_after_save));
        Task::none()
    }

    fn handle_folder_message(&mut self, msg: folder::Message) -> Task<Message> {
        match msg {
            folder::Message::SelectFile(index) => self.select_file_at(index),
            folder::Message::PreviousFile => self.select_previous(),
            folder::Message::NextFile => self.select_next(),
        }
    }

    fn select_file_at(&mut self, index: usize) -> Task<Message> {
        let Some(file) = self
            .directory
            .as_mut()
            .and_then(|dir| dir.select_index(index))
        else {
            return Task::none();
        };
        Task::done(Message::FileOpened(file))
    }

    fn select_previous(&mut self) -> Task<Message> {
        let Some(file) = self
            .directory
            .as_mut()
            .and_then(|dir| dir.select_previous())
        else {
            return Task::none();
        };
        Task::done(Message::FileOpened(file))
    }

    fn select_next(&mut self) -> Task<Message> {
        let Some(file) = self
            .directory
            .as_mut()
            .and_then(|dir| dir.select_next())
        else {
            return Task::none();
        };
        Task::done(Message::FileOpened(file))
    }

    fn handle_tag_panel(
        &mut self,
        msg: tag_panel::Message,
    ) -> Task<Message> {
        self.tag_panel
            .update(&msg, self.file_workspace.tag_list());
        match msg {
            tag_panel::Message::SetFilter(query) => {
                self.file_workspace.set_tag_filter(query);
                self.clamp_selection_to_filtered();
                Task::done(Message::ScrollTagListToSelection)
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
                self.file_workspace.toggle_tag_by_id(id);
                self.tag_panel.set_selected(Some(id));
                Task::none()
            }
            tag_panel::Message::SelectUp => {
                let step = -(self.tag_panel.cols() as i32);
                self.move_tag_selection(step);
                Task::done(Message::ScrollTagListToSelection)
            }
            tag_panel::Message::SelectDown => {
                let step = self.tag_panel.cols() as i32;
                self.move_tag_selection(step);
                Task::done(Message::ScrollTagListToSelection)
            }
            tag_panel::Message::ToggleSelectedTag => {
                // If search bar had focus, it may have inserted a space; strip it so Space doesn't add to the filter.
                let filter = self.file_workspace.tag_list().filter_query();
                if filter.ends_with(' ') {
                    let trimmed = filter.trim_end();
                    self.file_workspace.set_tag_filter(trimmed.to_string());
                }
                self.clamp_selection_to_filtered();
                // Only toggle if the selected tag is visible (in the filtered list).
                if let Some(id) = self.tag_panel.selected_tag_id() {
                    self.file_workspace.toggle_tag_by_id(id);
                }
                Task::none()
            }
            tag_panel::Message::DeleteTag(id) => {
                if self.tag_panel.selected_tag_id() == Some(id) {
                    self.tag_panel.set_selected(None);
                }
                if let Err(e) = self.file_workspace.remove_stored_tag_by_id(id) {
                    log::error!("Failed to delete tag: {}", e);
                }
                self.clamp_selection_to_filtered();
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
                }
                Task::none()
            }
            tag_panel::Message::DragStarted(_)
            | tag_panel::Message::DragHoverCursor { .. }
            | tag_panel::Message::DragEnded
            | tag_panel::Message::PanelBounds { .. } => Task::none(),
        }
    }

    fn handle_file_name_panel(&mut self, msg: file_name_panel::Message) -> Task<Message> {
        if let file_name_panel::Message::UnselectTag(id) = &msg {
            self.file_workspace.toggle_tag_by_id(*id);
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
            self.file_workspace.reorder_tag_to_index(did, idx);
        }
        Task::none()
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

    /// Move tag list selection by delta (-1 = up, 1 = down). Uses filtered list.
    fn move_tag_selection(&mut self, delta: i32) {
        let tag_list = self.file_workspace.tag_list();
        let filtered = tag_list.filtered_display_tag_ids();
        if filtered.is_empty() {
            self.tag_panel.set_selected(None);
            return;
        }
        let current_pos = self
            .tag_panel
            .selected_tag_id()
            .and_then(|id| filtered.iter().position(|&fid| fid == id));
        let new_id = match current_pos {
            None if delta > 0 => filtered.first().copied(),
            None => None,
            Some(i) => {
                let next = i as i32 + delta;
                if next < 0 {
                    filtered.first().copied()
                } else {
                    let u = next as usize;
                    if u < filtered.len() {
                        filtered.get(u).copied()
                    } else {
                        filtered.get(i).copied()
                    }
                }
            }
        };
        self.tag_panel.set_selected(new_id);
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

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.video_player.subscription().map(Message::VideoPlayer),
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

    pub fn video_player(&self) -> &VideoPlayerState {
        &self.video_player
    }

    pub fn tag_panel(&self) -> &TagPanelState {
        &self.tag_panel
    }

    pub fn file_name_panel(&self) -> &FileNamePanelState {
        &self.file_name_panel
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

    use frename_core::{AppDatabase, File, FileSnapshot, Initializable, LoggingAppStateStore};

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
            workspace.directory().map(|d| d.files().len()),
            Some(2),
            "folder panel should show two files"
        );
    }

    #[test]
    fn tags_are_saved_after_selecting_another_file() {
        let test_dir = TestDirectory::new(2);
        let first_file_path = PathBuf::from("C:/test/folder/file_0.mp4");
        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory,
            target_file: Some(test_dir.target_file),
        });
        flush_file_opened(&mut workspace);

        let tag_name = "Comedy";
        let tag_list = workspace.file_workspace().tag_list();
        let tag_id = tag_list
            .filtered_display_tag_ids()
            .iter()
            .find(|id| tag_list.get_tag(**id).map(|t| t.tag() == tag_name).unwrap_or(false))
            .copied()
            .expect("Comedy is a stored tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(
            tag_id,
        )));

        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        let snapshot = FileSnapshot::new(vec![tag_name.to_string()], "file_0", "mp4", "file_0.mp4");
        let _ = workspace.update(Message::FileUpdated {
            path: first_file_path,
            snapshot,
        });
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        flush_file_opened(&mut workspace);

        assert!(
            workspace
                .file_workspace()
                .file()
                .unwrap()
                .snapshot()
                .has_tag(tag_name),
            "tags should be saved after selecting another file"
        );
    }
}
