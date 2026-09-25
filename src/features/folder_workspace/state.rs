//! State for folder workspace: only data and selection logic.
//!
//! Owns: loaded folder (directory + files), currently selected file, loading flag.
//! Also holds child feature state (video, rename panel) and workspace layout (splitter positions).
//! No UI concepts (scrollable, shape, size of children)—only data passed to feature views.
//! Each feature view decides how it looks; the workspace view only arranges regions.

use std::path::PathBuf;

use arboard;
use frename_core::undo::History;
use frename_core::{
    AppDatabase, AppStateStore, CreateTagCommand, DeleteTagCommand, File, FileId, FileSnapshot,
    FolderAndFile, FolderTagStore, LoggingAppStateStore, NavigateFileCommand, PasteTagsCommand,
    ReorderTagCommand, SaveAndReparse, SaveTagCommand, SetSegmentEndCommand,
    SetSegmentStartCommand, StarTagCommand, ToggleTagCommand, UndoContext, UndoError,
};
use rfd;

use super::messages::GlobalSearchKey;
use super::Directory;
use iced::widget::operation;
use iced::{Subscription, Task};

use crate::features::batch::{self, BatchState, ItemResult, ItemStatus};
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
/// How many actions undo/redo keeps. Reset per folder, since tags are per folder.
const HISTORY_DEPTH: usize = 50;

/// Concrete history type for this workspace: Directory uses LoggingAppStateStore<AppDatabase>,
/// TagList uses the tag store of the open folder.
type WorkspaceHistory = History<LoggingAppStateStore<AppDatabase>, FolderTagStore>;

/// Folder workspace: owns directory, loading. Current file is the directory's selection.
pub struct FolderWorkspace {
    directory: Option<Directory>,
    loading: bool,
    /// File currently being edited: copy of file + tag selection. Rename panel reads/updates this.
    file_workspace: FileWorkspace<FolderTagStore>,
    media_viewer: MediaViewerState,
    tag_panel: TagPanelState,
    file_name_panel: FileNamePanelState,
    /// Deferred rename: set when media must unload before the previous file can be renamed.
    /// Stores the file's stable ID and the tag snapshot to save.
    pending_file_updated: Option<(FileId, FileSnapshot)>,
    /// Batch mode: checked files, the chosen action and its job.
    batch: BatchState,
    /// A batch job waits for a playing video to unload, since it may write into and rename
    /// that very file.
    batch_waits_for_unload: bool,
    /// The file being renamed in place in the folder list, if any.
    inline_rename: Option<folder::InlineRename>,
    /// Bumped per folder, so a comment batch for a folder no longer open is dropped.
    comment_load_generation: u64,
    /// Frame of the loading spinner shown in rows whose comment is still loading.
    spinner_frame: usize,
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
        let (left_width, folder_width) = AppDatabase::new()
            .get_window_state()
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
            file_workspace: FileWorkspace::<FolderTagStore>::default(),
            media_viewer: MediaViewerState::default(),
            tag_panel: TagPanelState::default(),
            file_name_panel: FileNamePanelState::default(),
            pending_file_updated: None,
            batch: BatchState::default(),
            batch_waits_for_unload: false,
            inline_rename: None,
            comment_load_generation: 0,
            spinner_frame: 0,
            pending_to_file_id: None,
            left_width,
            folder_width,
            tag_list_scroll_y: None,
            tag_list_viewport_height: None,
            folder_scroll_y: None,
            folder_viewport_height: None,
            copied_tags: None,
            history: WorkspaceHistory::new(HISTORY_DEPTH),
            media_fullscreen: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        if self.is_blocked(&message) {
            return Task::none();
        }
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
            Message::Batch(msg) => self.handle_batch(msg),
            Message::PrepareBatch(operation) => {
                let files = self.listed_ids();
                Task::done(Message::Batch(batch::Message::Prepare { operation, files }))
            }
            Message::BatchItemDone { id, result } => self.batch_item_done(id, result),
            Message::BatchFinished => self.batch_finished(),
            Message::CommentBatchLoaded {
                generation,
                results,
            } => self.comment_batch_loaded(generation, results),
            Message::SpinnerTick => {
                self.spinner_frame = self.spinner_frame.wrapping_add(1);
                Task::none()
            }
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
                media_viewer::Message::ScreenshotTaken(position_ms, jpeg) => {
                    Task::done(Message::ScreenshotTaken(position_ms, jpeg))
                }
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
            Message::SetSegmentStart | Message::SetSegmentEnd if self.inline_rename.is_some() => {
                Task::none()
            }
            Message::SetSegmentStart => Task::done(Message::MediaViewer(
                media_viewer::Message::Video(media_viewer_video::Message::CaptureSegmentStart),
            )),
            Message::SetSegmentEnd => Task::done(Message::MediaViewer(
                media_viewer::Message::Video(media_viewer_video::Message::CaptureSegmentEnd),
            )),
            Message::CommentAction(action) => {
                self.file_workspace.apply_comment_action(action);
                Task::none()
            }
            Message::ScreenshotTaken(position_ms, jpeg) => {
                let Some(file) = self.file_workspace.file() else {
                    return Task::none();
                };
                // Deduplicate: skip if any existing screenshot is within 100 ms.
                let too_close = self
                    .file_workspace
                    .screenshots()
                    .iter()
                    .any(|s| s.position_ms.abs_diff(position_ms) < 100);
                if too_close {
                    return Task::none();
                }
                let file_path = file.file_path().to_path_buf();
                self.file_workspace
                    .add_screenshot(frename_core::Screenshot::new(position_ms));
                // Append timestamp to comment.
                let time_str = frename_core::Screenshot::new(position_ms).format_time();
                let current = self.file_workspace.comment().to_string();
                let new_comment = if current.is_empty() {
                    format!("{}: ", time_str)
                } else {
                    format!("{}\n{}: ", current, time_str)
                };
                self.file_workspace.set_comment(new_comment);
                frename_core::FileTagger::save_screenshot(&file_path, position_ms, &jpeg);
                Task::none()
            }
            Message::EscapePressed => {
                if self.inline_rename.take().is_some() {
                    return Task::none();
                }
                if self.media_fullscreen {
                    self.media_fullscreen = false;
                    return Task::none();
                }
                // Both search bars label their clear button "Esc", so clear both.
                let clear_files = self.set_file_name_filter(String::new());
                Task::batch([
                    self.handle_tag_panel(tag_panel::Message::SetFilter(String::new())),
                    clear_files,
                ])
            }
            Message::OpenFilePicker => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_file()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                |opt| match opt {
                    Some(path) => Message::OpenFile(path),
                    None => Message::Noop,
                },
            ),
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
        operation::focus(iced::widget::Id::from(SEARCH_BAR_INPUT_ID)).map(|_: ()| Message::Noop)
    }

    fn set_segment_start(&mut self, secs: f32) -> Task<Message> {
        if self.file_workspace.file().is_none() {
            return Task::none();
        }
        let old_secs = self.file_workspace.segment_start_secs();
        self.file_workspace.set_segment_start_secs(Some(secs));
        self.history.push(Box::new(SetSegmentStartCommand {
            old_secs,
            new_secs: Some(secs),
        }));
        Task::none()
    }

    fn set_segment_end(&mut self, secs: f32) -> Task<Message> {
        if self.file_workspace.file().is_none() {
            return Task::none();
        }
        let old_secs = self.file_workspace.segment_end_secs();
        self.file_workspace.set_segment_end_secs(Some(secs));
        self.history.push(Box::new(SetSegmentEndCommand {
            old_secs,
            new_secs: Some(secs),
        }));
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
        self.file_workspace
            .reinitialize_tags_from_snapshot(snapshot_after.clone());
        self.file_workspace.set_tag_filter(String::new());
        self.clamp_selection_to_filtered();
        self.history.push(Box::new(PasteTagsCommand {
            snapshot_before,
            snapshot_after,
        }));
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
        let Some(file) = self.directory.as_mut().and_then(|dir| dir.open_path(&path)) else {
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
        self.inline_rename = None;
        // Batches for the folder being left must not land in the next one.
        self.comment_load_generation += 1;
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

    fn folder_loaded(
        &mut self,
        mut directory: Directory,
        target_file: Option<PathBuf>,
    ) -> Task<Message> {
        self.loading = false;
        // The list filters are user settings, not properties of the folder: carry them over.
        if let Some(previous) = self.directory.as_ref() {
            directory.set_untagged_only(previous.untagged_only());
            directory.set_subtitled_only(previous.subtitled_only());
            directory.set_commented_only(previous.commented_only());
        }
        self.directory = Some(directory); // replace previous directory only on success
        self.batch.reset_files();
        let folder = self
            .directory
            .as_ref()
            .expect("just set")
            .path()
            .to_path_buf();
        // Tags belong to the folder, so the workspace starts over on a new one. The history goes
        // with them: undoing "create tag" from the previous folder would delete it from this one.
        self.file_workspace = FileWorkspace::new(FolderTagStore::for_folder(&folder));
        self.history = WorkspaceHistory::new(HISTORY_DEPTH);
        let load_comments_task = self.load_next_comment_batch(true);
        let dir = self.directory.as_mut().expect("just set");
        let selected = target_file.as_deref().and_then(|p| dir.open_path(p));
        if let Some(file) = selected {
            Task::batch([
                Task::done(Message::FileOpened(file)),
                Task::done(Message::ScrollFolderListToSelected),
                load_comments_task,
            ])
        } else {
            self.file_workspace.set_file(None);
            self.pending_file_updated = None;
            load_comments_task
        }
    }

    /// Load the next batch of comments the folder scan deferred, on a blocking thread. `start`
    /// begins a new run for a newly opened folder, which drops any batch still in flight.
    fn load_next_comment_batch(&mut self, start: bool) -> Task<Message> {
        /// Files per batch: each costs about 20 ms cold, so a batch fills in the list about
        /// twice a second, and writes the folder's file list once.
        const BATCH: usize = 32;
        if start {
            self.comment_load_generation += 1;
        }
        let Some(dir) = self.directory.as_ref() else {
            return Task::none();
        };
        let batch: Vec<(FileId, PathBuf, FileSnapshot)> = dir
            .files_loading_comments()
            .into_iter()
            .take(BATCH)
            .collect();
        if batch.is_empty() {
            return Task::none();
        }
        let generation = self.comment_load_generation;
        Task::future(async move {
            let results = tokio::task::spawn_blocking(move || {
                let items: Vec<(PathBuf, FileSnapshot)> = batch
                    .iter()
                    .map(|(_, path, snapshot)| (path.clone(), snapshot.clone()))
                    .collect();
                let resolved = frename_core::FileTagger::load_comments(&items);
                batch
                    .into_iter()
                    .zip(resolved)
                    .map(|((id, path, _), snapshot)| (id, path, snapshot))
                    .collect()
            })
            .await
            .unwrap_or_else(|e| {
                log::error!("background comment load failed: {e}");
                Vec::new()
            });
            Message::CommentBatchLoaded {
                generation,
                results,
            }
        })
    }

    /// Take a background batch into the list, then start the next one.
    fn comment_batch_loaded(
        &mut self,
        generation: u64,
        results: Vec<(FileId, PathBuf, FileSnapshot)>,
    ) -> Task<Message> {
        if generation != self.comment_load_generation || results.is_empty() {
            return Task::none();
        }
        let Some(dir) = self.directory.as_mut() else {
            return Task::none();
        };
        for (id, path, snapshot) in &results {
            dir.apply_loaded_comment(*id, path, snapshot);
        }
        // A batch job may be writing the next files; loading goes on once it ends.
        if self.batch.is_running() {
            return Task::none();
        }
        self.load_next_comment_batch(false)
    }

    /// The file with its comment loaded, so a file opens with its comment and in/out already
    /// there instead of having them pop up after it was shown.
    fn with_comment_loaded(&mut self, file: frename_core::File) -> frename_core::File {
        if !file.snapshot().comment_loading() {
            return file;
        }
        let resolved = frename_core::FileTagger::load_comment(file.file_path(), file.snapshot());
        let Some(dir) = self.directory.as_mut() else {
            return file;
        };
        dir.apply_loaded_comment(file.id(), file.file_path(), &resolved);
        dir.file_by_id(file.id()).cloned().unwrap_or(file)
    }

    fn apply_file_opened(&mut self, file: frename_core::File) -> Task<Message> {
        let file = self.with_comment_loaded(file);
        // Opening another file closes the in-place rename editor, like leaving the row.
        if self
            .inline_rename
            .as_ref()
            .is_some_and(|r| r.id != file.id())
        {
            self.inline_rename = None;
        }
        // Detect same-file "refresh" (e.g. undo of a tag toggle on the current file).
        // In that case, skip media reload and fullscreen reset — only persist state.
        let same_file = self
            .file_workspace
            .file()
            .is_some_and(|f| f.file_path() == file.file_path());
        if !same_file {
            self.media_fullscreen = false;
        }
        let snapshot = self.file_workspace.get_snapshot();
        self.file_workspace.set_file(Some(file.clone()));
        log::info!("Opening file: {}", file.file_path().display());
        let Some((id, snap)) = snapshot else {
            self.pending_file_updated = None;
            if same_file {
                return Task::none();
            }
            return self.media_viewer.open(&file).map(Message::MediaViewer);
        };
        if self.media_viewer.needs_unload_before_rename() {
            self.pending_file_updated = Some((id, snap));
            Task::done(Message::MediaViewer(media_viewer::Message::Unload))
        } else {
            let media_task = if same_file {
                Task::none()
            } else {
                self.media_viewer.open(&file).map(Message::MediaViewer)
            };
            Task::batch([
                Task::done(Message::FileUpdated { id, snapshot: snap }),
                media_task,
            ])
        }
    }

    /// Messages to drop now. Batch mode shows batch actions instead of the open file, so what
    /// would edit that file is off. A running job also locks the folder: no file may open or
    /// change while the job writes its files.
    fn is_blocked(&self, message: &Message) -> bool {
        let edits_open_file = matches!(
            message,
            Message::TagPanel(_)
                | Message::FileNamePanel(_)
                | Message::SyncPanel(_)
                | Message::CommentAction(_)
                | Message::RemoveTag
                | Message::SaveSelectedTag
                | Message::CopyTags
                | Message::PasteTags
                | Message::Undo
                | Message::Redo
                | Message::SetSegmentStart
                | Message::SetSegmentEnd
                | Message::FocusSearchBarAndKey(_)
                | Message::ScreenshotTaken(..)
                | Message::Folder(
                    folder::Message::StartRename(_)
                        | folder::Message::RenameInput(_)
                        | folder::Message::SubmitRename
                )
        );
        if edits_open_file && self.batch.is_active() {
            return true;
        }
        let changes_files = matches!(
            message,
            Message::OpenFile(_)
                | Message::LoadLastSession
                | Message::ScanFolder(_)
                | Message::OpenFilePicker
                | Message::PrepareBatch(_)
                | Message::ToggleMediaFullscreen
                | Message::Folder(
                    folder::Message::SelectFile(_)
                        | folder::Message::PreviousFile
                        | folder::Message::NextFile
                        | folder::Message::OpenFolder
                        | folder::Message::SetBatchMode(_)
                        | folder::Message::ToggleChecked(_)
                        | folder::Message::ToggleAllChecked
                        | folder::Message::InvertChecks
                )
        );
        self.batch.is_running() && (edits_open_file || changes_files)
    }

    /// IDs of the files the folder list shows, in list order.
    fn listed_ids(&self) -> Vec<FileId> {
        self.directory.as_ref().map_or_else(Vec::new, |dir| {
            dir.files_in_order().map(|f| f.id()).collect()
        })
    }

    fn handle_batch(&mut self, msg: batch::Message) -> Task<Message> {
        if let batch::Message::Run = msg {
            return self.start_batch();
        }
        // Batch mode does not edit the open file, so its rename editor goes.
        self.inline_rename = None;
        self.batch.update(msg);
        Task::none()
    }

    /// Start the selected batch action on the checked files, in folder order. A playing video
    /// is unloaded first; see [`Self::run_batch`].
    fn start_batch(&mut self) -> Task<Message> {
        let Some(dir) = self.directory.as_ref() else {
            return Task::none();
        };
        let files: Vec<FileId> = dir
            .all_files()
            .map(|f| f.id())
            .filter(|id| self.batch.is_checked(*id))
            .collect();
        if !self.batch.start(files) {
            return Task::none();
        }
        self.media_fullscreen = false;
        if self.media_viewer.needs_unload_before_rename() {
            self.batch_waits_for_unload = true;
            return Task::done(Message::MediaViewer(media_viewer::Message::Unload));
        }
        self.run_batch()
    }

    /// Save the edits waiting for the disk (the open file may be one of the job's), close the
    /// open file until the job ends, and start the first file.
    fn run_batch(&mut self) -> Task<Message> {
        if let Some((id, snapshot)) = self.pending_file_updated.take() {
            let _ = self.apply_file_updated(id, snapshot);
        }
        if let Some((id, snapshot)) = self.file_workspace.get_snapshot() {
            let _ = self.apply_file_updated(id, snapshot);
        }
        self.file_workspace.set_file(None);
        self.next_batch_item()
    }

    /// Run the job on its next file on a blocking thread, or end the job when it has none
    /// left or was cancelled. One file per step: progress names the file in work, and a
    /// cancel stops after it, so no file is left half done.
    fn next_batch_item(&mut self) -> Task<Message> {
        let Some((id, operation)) = self.batch.begin_next() else {
            return Task::done(Message::BatchFinished);
        };
        let Some(path) = self
            .directory
            .as_ref()
            .and_then(|d| d.file_by_id(id))
            .map(|f| f.file_path().to_path_buf())
        else {
            self.batch.finish(id, ItemStatus::Failed);
            return self.next_batch_item();
        };
        Task::future(async move {
            let result = tokio::task::spawn_blocking(move || operation.run(&path))
                .await
                .unwrap_or_else(|e| {
                    log::error!("batch operation task failed: {e}");
                    ItemResult {
                        status: ItemStatus::Failed,
                        update: None,
                    }
                });
            Message::BatchItemDone { id, result }
        })
    }

    /// Take a finished file into the list (it may have been renamed), then start the next.
    fn batch_item_done(&mut self, id: FileId, result: ItemResult) -> Task<Message> {
        if let (Some(dir), Some((path, snapshot))) =
            (self.directory.as_mut(), result.update.as_ref())
        {
            dir.rename_file(id, path, snapshot);
        }
        self.batch.finish(id, result.status);
        self.next_batch_item()
    }

    /// The job ended: reopen the selected file, and resume loading comments.
    fn batch_finished(&mut self) -> Task<Message> {
        // The job renamed files behind the history: undoing across it would use stale paths.
        self.history = WorkspaceHistory::new(HISTORY_DEPTH);
        let load_comments = self.load_next_comment_batch(false);
        let Some(file) = self
            .directory
            .as_ref()
            .and_then(|d| d.selected_file())
            .cloned()
        else {
            return load_comments;
        };
        Task::batch([Task::done(Message::FileOpened(file)), load_comments])
    }

    /// Called when media has unloaded. Persist pending snapshot (FileUpdated) then open the new media.
    fn on_media_unloaded(&mut self) -> Task<Message> {
        if std::mem::take(&mut self.batch_waits_for_unload) {
            return self.run_batch();
        }
        let Some((id, snapshot)) = self.pending_file_updated.take() else {
            return Task::none();
        };
        // Save before opening media, not alongside it: when the selected file is the one being
        // saved (re-clicking it, renaming it in place), opening it first would lock it against
        // the rename, or point the player at the name it had before the save.
        let saved = self.apply_file_updated(id, snapshot);
        let Some(file) = self
            .directory
            .as_ref()
            .and_then(|d| d.selected_file())
            .cloned()
        else {
            return saved;
        };
        Task::batch([
            saved,
            self.media_viewer.open(&file).map(Message::MediaViewer),
        ])
    }

    fn apply_file_updated(
        &mut self,
        id: FileId,
        snapshot: frename_core::FileSnapshot,
    ) -> Task<Message> {
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

        // The saved file may have just dropped out of a filtered list, shifting every row
        // below it up by one; re-run scroll-into-view so the cursor stays where the user sees it.
        if self
            .directory
            .as_ref()
            .is_some_and(|d| d.has_content_filter())
        {
            return Task::done(Message::ScrollFolderListToSelected);
        }
        Task::none()
    }

    fn handle_folder_message(&mut self, msg: folder::Message) -> Task<Message> {
        match msg {
            folder::Message::SelectFile(index) => self.select_file_at(index),
            folder::Message::PreviousFile => self.select_previous(),
            folder::Message::NextFile => self.select_next(),
            folder::Message::Scrolled {
                scroll_y,
                viewport_height,
            } => {
                self.folder_scroll_y = Some(scroll_y);
                self.folder_viewport_height = Some(viewport_height);
                Task::none()
            }
            folder::Message::ScrollToSelected => Task::done(Message::ScrollFolderListToSelected),
            folder::Message::OpenFolder => Task::done(Message::OpenFilePicker),
            folder::Message::SetUntaggedOnly(untagged_only) => {
                self.set_list_filter(|dir| dir.set_untagged_only(untagged_only))
            }
            folder::Message::SetSubtitledOnly(subtitled_only) => {
                self.set_list_filter(|dir| dir.set_subtitled_only(subtitled_only))
            }
            folder::Message::SetCommentedOnly(commented_only) => {
                self.set_list_filter(|dir| dir.set_commented_only(commented_only))
            }
            folder::Message::SetNameFilter(query) => self.set_file_name_filter(query),
            // Intercepted by the app, which owns the windows; no-op here.
            folder::Message::OpenSettings => Task::none(),
            folder::Message::StartRename(index) => self.start_rename(index),
            folder::Message::RenameInput(text) => {
                if let Some(rename) = self.inline_rename.as_mut() {
                    rename.text = text;
                    rename.error = None;
                }
                Task::none()
            }
            folder::Message::SubmitRename => self.submit_rename(),
            folder::Message::SetBatchMode(on) => {
                Task::done(Message::Batch(batch::Message::SetActive(on)))
            }
            folder::Message::ToggleChecked(id) => {
                Task::done(Message::Batch(batch::Message::Toggle(id)))
            }
            folder::Message::ToggleAllChecked => {
                let listed = self.listed_ids();
                let all_checked =
                    !listed.is_empty() && listed.iter().all(|id| self.batch.is_checked(*id));
                let msg = if all_checked {
                    batch::Message::CheckNone
                } else {
                    batch::Message::CheckAll(listed)
                };
                Task::done(Message::Batch(msg))
            }
            folder::Message::InvertChecks => {
                Task::done(Message::Batch(batch::Message::Invert(self.listed_ids())))
            }
        }
    }

    /// Open the in-place rename editor on the row at `index` (selecting that file first when
    /// needed), with the name before the extension selected, as Windows Explorer does.
    fn start_rename(&mut self, index: usize) -> Task<Message> {
        let Some(dir) = self.directory.as_ref() else {
            return Task::none();
        };
        let Some(file) = dir.files_in_order().nth(index) else {
            return Task::none();
        };
        let id = file.id();
        let name = file
            .file_path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let stem_chars = name
            .rfind('.')
            .map_or(name.as_str(), |dot| &name[..dot])
            .chars()
            .count();
        let select = if dir.selected_index() == Some(index) {
            Task::none()
        } else {
            self.select_file_at(index)
        };
        self.inline_rename = Some(folder::InlineRename {
            id,
            text: name,
            error: None,
        });
        let input = iced::widget::Id::from(folder::FOLDER_RENAME_INPUT_ID);
        Task::batch([
            select,
            operation::focus(input.clone()),
            operation::select_range(input, 0, stem_chars),
        ])
    }

    /// Rename the file being edited in place to the typed name. A refused name keeps the
    /// editor open with the reason. The new name goes through the workspace like any other
    /// edit of the open file, so tags, in/out points, comment and sidecars stay consistent.
    fn submit_rename(&mut self) -> Task<Message> {
        let Some(rename) = self.inline_rename.clone() else {
            return Task::none();
        };
        let Some(file) = self
            .directory
            .as_ref()
            .and_then(|d| d.file_by_id(rename.id))
            .cloned()
        else {
            self.inline_rename = None;
            return Task::none();
        };
        // The editor only opens on the open file; anything else is a stale editor.
        let Some((_, current)) = self.file_workspace.get_snapshot().filter(|_| {
            self.file_workspace
                .file()
                .is_some_and(|f| f.id() == rename.id)
        }) else {
            self.inline_rename = None;
            return Task::none();
        };
        let current_name = file
            .file_path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let typed = rename.text.trim();
        if typed == current_name {
            self.inline_rename = None;
            return Task::none();
        }

        let mut snapshot = FileSnapshot::parse(typed);
        snapshot.set_comment(current.comment().to_string());
        snapshot.set_screenshots(current.screenshots().to_vec());
        // With in/out stored inside the video the name never shows them, so a typed name without them
        // does not mean "remove them".
        let name_has_in_out =
            snapshot.segment_start().is_some() || snapshot.segment_end().is_some();
        if !name_has_in_out
            && frename_core::metadata_storage().in_out == frename_core::InOutStorage::InVideo
        {
            snapshot.set_segment_start(current.segment_start());
            snapshot.set_segment_end(current.segment_end());
        }
        let folder = file
            .file_path()
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();
        let checked = check_new_file_name(&folder, current_name, typed)
            .and_then(|()| check_new_file_name(&folder, current_name, &snapshot.file_name()));
        if let Err(reason) = checked {
            if let Some(rename) = self.inline_rename.as_mut() {
                rename.error = Some(reason);
            }
            return Task::none();
        }

        self.inline_rename = None;
        self.file_workspace
            .reinitialize_tags_from_snapshot(snapshot);
        // Same-file refresh: persists the workspace snapshot (renaming on disk) without
        // reopening the media unless it must unload first.
        Task::done(Message::FileOpened(file))
    }

    /// Turn a list filter (untagged, subtitles, comments) on or off. The list changes shape,
    /// so bring the selected file back into view (it stays listed even when it does not match).
    fn set_list_filter(&mut self, apply: impl FnOnce(&mut Directory)) -> Task<Message> {
        let Some(dir) = self.directory.as_mut() else {
            return Task::none();
        };
        apply(dir);
        Task::done(Message::ScrollFolderListToSelected)
    }

    /// Narrow the file list by name. Like the untagged filter, the selected file stays listed,
    /// so the cursor keeps pointing at a real row while the query is being typed.
    fn set_file_name_filter(&mut self, query: String) -> Task<Message> {
        let Some(dir) = self.directory.as_mut() else {
            return Task::none();
        };
        dir.set_name_filter(query);
        Task::done(Message::ScrollFolderListToSelected)
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
        let Some(file) = self.directory.as_mut().and_then(|dir| dir.select_next()) else {
            self.pending_to_file_id = None;
            return Task::none();
        };
        self.pending_to_file_id = Some(file.id());
        Task::batch([
            Task::done(Message::FileOpened(file)),
            Task::done(Message::ScrollFolderListToSelected),
        ])
    }

    fn handle_tag_panel(&mut self, msg: tag_panel::Message) -> Task<Message> {
        // Capture drag state before update() clears it on DragEnded.
        let drag_on_end = if let tag_panel::Message::DragEnded = &msg {
            Some((
                self.tag_panel.dragging_tag_id(),
                self.tag_panel.drop_target_index(),
            ))
        } else {
            None
        };
        self.tag_panel.update(&msg, self.file_workspace.tag_list());
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
                            let color_index = self
                                .file_workspace
                                .tag_list()
                                .get_tag(id)
                                .map_or(0, |t| t.color_index());
                            self.history.push(Box::new(CreateTagCommand {
                                tag_id: id,
                                tag_name: name,
                                color_index,
                            }));
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
                let was_checked = self
                    .file_workspace
                    .tag_list()
                    .get_tag(id)
                    .is_some_and(|t| t.is_checked());
                self.file_workspace.toggle_tag_by_id(id);
                self.tag_panel.set_selected(Some(id));
                self.history.push(Box::new(ToggleTagCommand {
                    tag_id: id,
                    was_checked,
                }));
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
                let filtered = self
                    .file_workspace
                    .tag_list()
                    .filtered_display_tag_ids()
                    .to_vec();
                let selected_id = self.tag_panel.selected_tag_id();
                let selected_visible = selected_id.filter(|id| filtered.contains(id));
                let id_to_toggle = selected_visible.or_else(|| filtered.first().copied());
                if let Some(id) = id_to_toggle {
                    let was_checked = self
                        .file_workspace
                        .tag_list()
                        .get_tag(id)
                        .is_some_and(|t| t.is_checked());
                    self.file_workspace.toggle_tag_by_id(id);
                    self.history.push(Box::new(ToggleTagCommand {
                        tag_id: id,
                        was_checked,
                    }));
                    if selected_visible.is_none() {
                        self.tag_panel.set_selected(Some(id));
                    }
                }
                Task::none()
            }
            tag_panel::Message::DeleteTag(id) => {
                // Find the deleted tag's position in the filtered list before removal.
                let filtered_before = self
                    .file_workspace
                    .tag_list()
                    .filtered_display_tag_ids()
                    .to_vec();
                let deleted_pos = filtered_before.iter().position(|&fid| fid == id);

                let delete_data = self.file_workspace.tag_list().capture_delete_data(id);
                if let Err(e) = self.file_workspace.remove_stored_tag_by_id(id) {
                    log::error!("Failed to delete tag: {}", e);
                } else if let Some((
                    name,
                    color,
                    was_stored,
                    was_starred,
                    was_checked,
                    sort_order,
                )) = delete_data
                {
                    self.history.push(Box::new(DeleteTagCommand {
                        tag_id: id,
                        tag_name: name,
                        color_index: color,
                        was_stored,
                        was_starred,
                        was_checked,
                        sort_order,
                    }));
                }

                // Select the neighbor: left (or first if deleted was first).
                if let Some(pos) = deleted_pos {
                    let filtered_after = self
                        .file_workspace
                        .tag_list()
                        .filtered_display_tag_ids()
                        .to_vec();
                    let new_i = if pos == 0 { 0 } else { pos - 1 };
                    self.tag_panel
                        .set_selected(filtered_after.get(new_i).copied());
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
                    let color_index = self
                        .file_workspace
                        .tag_list()
                        .get_tag(id)
                        .map_or(0, |t| t.color_index());
                    self.history.push(Box::new(SaveTagCommand {
                        tag_id: id,
                        color_index,
                    }));
                }
                Task::none()
            }
            tag_panel::Message::ToggleStar(id) => {
                let was_starred = self
                    .file_workspace
                    .tag_list()
                    .get_tag(id)
                    .is_some_and(|t| t.is_starred());
                let result = if was_starred {
                    self.file_workspace.unstar_tag(id)
                } else {
                    self.file_workspace.star_tag(id)
                };
                match result {
                    Ok(()) => self.history.push(Box::new(StarTagCommand {
                        tag_id: id,
                        was_starred,
                    })),
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
                        .filter(|&&id| tag_list.get_tag(id).is_some_and(|t| t.is_checked()))
                        .count();
                    let from_index = self.file_workspace.tag_list().checked_index_of(did);
                    self.file_workspace.reorder_tag_to_index(did, checked_idx);
                    if let Some(fi) = from_index {
                        self.history.push(Box::new(ReorderTagCommand {
                            moved_id: did,
                            from_index: fi,
                            to_index: checked_idx,
                        }));
                    }
                }
                Task::none()
            }
            tag_panel::Message::PanelBounds { .. } => Task::none(),
        }
    }

    fn handle_file_name_panel(&mut self, msg: file_name_panel::Message) -> Task<Message> {
        if let file_name_panel::Message::RemoveTag(id) = msg {
            let was_checked = self
                .file_workspace
                .tag_list()
                .get_tag(id)
                .is_some_and(|t| t.is_checked());
            if was_checked {
                self.file_workspace.toggle_tag_by_id(id);
                self.history.push(Box::new(ToggleTagCommand {
                    tag_id: id,
                    was_checked,
                }));
            }
            return Task::none();
        }
        if let file_name_panel::Message::ClearSegmentStart = msg {
            let old_secs = self.file_workspace.segment_start_secs();
            self.file_workspace.set_segment_start_secs(None);
            self.history.push(Box::new(SetSegmentStartCommand {
                old_secs,
                new_secs: None,
            }));
            return Task::none();
        }
        if let file_name_panel::Message::ClearSegmentEnd = msg {
            let old_secs = self.file_workspace.segment_end_secs();
            self.file_workspace.set_segment_end_secs(None);
            self.history.push(Box::new(SetSegmentEndCommand {
                old_secs,
                new_secs: None,
            }));
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
        }
        Task::none()
    }

    fn handle_sync_panel(&mut self, msg: sync_panel::Message) -> Task<Message> {
        match msg {
            sync_panel::Message::SyncUp => {
                if let Err(e) = self.file_workspace.sync_selected_to_display() {
                    log::error!("SyncUp failed: {}", e);
                } else {
                    self.file_workspace.tag_list_mut().set_sync_locked(true);
                }
            }
            sync_panel::Message::SyncDown => {
                self.file_workspace.sync_display_to_selected();
                self.file_workspace.tag_list_mut().set_sync_locked(true);
            }
            sync_panel::Message::ToggleLock => {
                let tl = self.file_workspace.tag_list_mut();
                tl.set_sync_locked(!tl.sync_locked());
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
            let mut ctx = UndoContext {
                directory: dir,
                tag_list: tl,
            };
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
            let mut ctx = UndoContext {
                directory: dir,
                tag_list: tl,
            };
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
        let filtered = self
            .file_workspace
            .tag_list()
            .filtered_display_tag_ids()
            .to_vec();
        if filtered.is_empty() {
            self.tag_panel.set_selected(None);
            return;
        }
        let cur = self
            .tag_panel
            .selected_tag_id()
            .and_then(|id| filtered.iter().position(|&fid| fid == id));
        let new_i = match cur {
            None | Some(0) => filtered.len() - 1,
            Some(i) => i - 1,
        };
        self.tag_panel.set_selected(filtered.get(new_i).copied());
    }

    /// Right: move by +1, wrap last→first.
    fn move_selection_right(&mut self) {
        let filtered = self
            .file_workspace
            .tag_list()
            .filtered_display_tag_ids()
            .to_vec();
        if filtered.is_empty() {
            self.tag_panel.set_selected(None);
            return;
        }
        let last = filtered.len() - 1;
        let cur = self
            .tag_panel
            .selected_tag_id()
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
        let filtered = self
            .file_workspace
            .tag_list()
            .filtered_display_tag_ids()
            .to_vec();
        if filtered.is_empty() {
            self.tag_panel.set_selected(None);
            return;
        }
        let cols = self.tag_panel.cols().max(1) as usize;
        let count = filtered.len();
        let cur = self
            .tag_panel
            .selected_tag_id()
            .and_then(|id| filtered.iter().position(|&fid| fid == id));
        let new_i = match cur {
            None => count - 1,
            Some(i) if i < cols => {
                // top row → jump to last row, same column
                let x = i % cols;
                let last_row = (count - 1) / cols;
                let new_i = last_row * cols + x;
                if new_i >= count {
                    new_i - cols
                } else {
                    new_i
                }
            }
            Some(i) => i - cols,
        };
        self.tag_panel.set_selected(filtered.get(new_i).copied());
    }

    /// Down: move one visual row down; last row wraps to first row at same column.
    fn move_selection_down(&mut self) {
        let filtered = self
            .file_workspace
            .tag_list()
            .filtered_display_tag_ids()
            .to_vec();
        if filtered.is_empty() {
            self.tag_panel.set_selected(None);
            return;
        }
        let cols = self.tag_panel.cols().max(1) as usize;
        let count = filtered.len();
        let cur = self
            .tag_panel
            .selected_tag_id()
            .and_then(|id| filtered.iter().position(|&fid| fid == id));
        let new_i = match cur {
            None => 0,
            Some(i) => {
                let new_i = i + cols;
                if new_i >= count {
                    i % cols
                } else {
                    new_i
                }
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
        let row_extent = self.tag_panel.row_content_height().unwrap_or(row_stride);
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
        let scroll_op = operation::scroll_to(iced::widget::Id::new(TAG_LIST_SCROLLABLE_ID), offset)
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
            operation::scroll_to(
                iced::widget::Id::new(folder::FOLDER_LIST_SCROLLABLE_ID),
                offset,
            )
            .map(|_: ()| Message::Noop),
            Task::done(Message::FolderListScrollAdjusted(target_y)),
        ])
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // The spinner only turns while rows are loading, so an idle list does not redraw.
        let loading_comments = self
            .directory
            .as_ref()
            .is_some_and(|d| d.loading_comment_count() > 0);
        let spinner = if loading_comments {
            iced::time::every(std::time::Duration::from_millis(150)).map(|_| Message::SpinnerTick)
        } else {
            Subscription::none()
        };
        Subscription::batch([
            self.media_viewer.subscription().map(Message::MediaViewer),
            self.file_name_panel
                .subscription()
                .map(Message::FileNamePanel),
            self.tag_panel.subscription().map(Message::TagPanel),
            spinner,
        ])
    }

    /// Frame of the loading spinner in the folder list.
    pub fn spinner_frame(&self) -> usize {
        self.spinner_frame
    }

    /// Batch mode: checked files, the chosen action and its job.
    pub fn batch(&self) -> &BatchState {
        &self.batch
    }

    /// Whether a batch job is running or waiting to start; closing the app must wait for it.
    pub fn is_batch_running(&self) -> bool {
        self.batch.is_running()
    }

    /// Currently selected file (from directory selection).
    pub fn current_file(&self) -> Option<&File> {
        self.directory.as_ref().and_then(|d| d.selected_file())
    }

    /// The file being renamed in place in the folder list, if any.
    pub fn inline_rename(&self) -> Option<&folder::InlineRename> {
        self.inline_rename.as_ref()
    }

    pub fn directory(&self) -> Option<&Directory> {
        self.directory.as_ref()
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// File workspace: current file and its tag selection (for rename panel). Use this for display and tag toggles.
    pub fn file_workspace(&self) -> &FileWorkspace<FolderTagStore> {
        &self.file_workspace
    }

    /// True when tags have been copied (internal clipboard has data); used to decide whether
    /// Ctrl+V in the search bar should paste tags or pass through to text input.
    pub fn has_copied_tags(&self) -> bool {
        self.copied_tags.is_some()
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
        self.file_workspace.tag_list().sync_locked()
    }

    pub fn left_width(&self) -> f32 {
        self.left_width
    }

    pub fn folder_width(&self) -> f32 {
        self.folder_width
    }
}

/// Why `typed` cannot replace `current` as a file name in `folder`, if it cannot. A rename
/// on Windows replaces an existing file of the same name, so a clash must be refused here.
fn check_new_file_name(
    folder: &std::path::Path,
    current: &str,
    typed: &str,
) -> Result<(), &'static str> {
    if typed.is_empty() {
        return Err("Name is empty");
    }
    if typed.chars().any(|c| {
        matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control()
    }) {
        return Err("Not allowed: \\ / : * ? \" < > |");
    }
    if typed.ends_with('.') || typed.ends_with(' ') {
        return Err("Cannot end with a dot or space");
    }
    // A change of case only is the same file on Windows, not a clash.
    if !typed.eq_ignore_ascii_case(current) && folder.join(typed).exists() {
        return Err("A file with this name exists");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::SystemTime;

    use frename_core::{
        AppDatabase, File, FileId, FileSnapshot, Initializable, LoggingAppStateStore,
    };

    use crate::features::{batch, folder, tag_panel};

    use super::{Directory, FolderWorkspace, ItemResult, ItemStatus, Message};

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
    ///
    /// The folder is created on disk, because opening it writes the folder's tag file. Fields are
    /// cloned rather than moved out so the fixture stays alive to delete the folder afterwards.
    pub struct TestDirectory {
        directory: Directory,
        path: PathBuf,
    }

    impl TestDirectory {
        pub fn new(file_count: usize) -> Self {
            let path = unique_test_folder();
            std::fs::create_dir_all(&path).expect("create test folder");
            let files: Vec<File> = (0..file_count)
                .map(|i| {
                    File::from_path(path.join(format!("file_{}.mp4", i)), SystemTime::UNIX_EPOCH)
                })
                .collect();
            let db = AppDatabase::new();
            let store = LoggingAppStateStore::new(db);
            store.initialize().unwrap();
            let directory = Directory::with_files(&path, files, store);
            Self { directory, path }
        }

        /// The scanned directory, ready to hand to `Message::FolderLoaded`.
        pub fn directory(&self) -> Directory {
            self.directory.clone()
        }

        /// The file the workspace should open once the folder is loaded.
        pub fn target_file(&self) -> PathBuf {
            self.path.join("file_0.mp4")
        }

        /// Path of a file in this folder, for asserting on renames.
        pub fn file_path(&self, name: &str) -> PathBuf {
            self.path.join(name)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    /// A folder name no other test, and no concurrent test run, will pick.
    fn unique_test_folder() -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("frename-test-{}-{n}", std::process::id()))
    }

    #[test]
    fn open_folder_with_two_files_shows_two_in_folder_panel() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = FolderWorkspace::new();
        let _task = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
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
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);

        // Capture file_0's stable ID before any navigation.
        let file_0_id = file_id_at(&workspace, 0);

        let tag_name = "pick";
        let tag_list = workspace.file_workspace().tag_list();
        let tag_id = tag_list
            .filtered_display_tag_ids()
            .iter()
            .find(|id| {
                tag_list
                    .get_tag(**id)
                    .map(|t| t.tag() == tag_name)
                    .unwrap_or(false)
            })
            .copied()
            .expect("pick is a built-in tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(tag_id)));

        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);

        // Manually drive the FileUpdated that would fire from the iced runtime.
        let snapshot =
            FileSnapshot::new(vec![tag_name.to_string()], "file_0", ".mp4", "file_0.mp4");
        let _ = workspace.update(Message::FileUpdated {
            id: file_0_id,
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
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        // After opening file_0.mp4: video.loading = true → needs_unload_before_rename() = true.

        // Toggle a tag on file_0 so there is something to defer.
        let tag_list = workspace.file_workspace().tag_list();
        let tag_id = tag_list
            .filtered_display_tag_ids()
            .iter()
            .find(|id| {
                tag_list
                    .get_tag(**id)
                    .map(|t| t.tag() == "pick")
                    .unwrap_or(false)
            })
            .copied()
            .expect("pick is a built-in tag");
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
                vec!["pick".to_string()],
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
                .has_tag("pick"),
            "pick tag must survive the deferred rename on file_0"
        );
    }

    /// When the pending rename is consumed by `on_media_unloaded`, the pending field is cleared
    /// so a second `Unloaded` event does not cause a double rename.
    #[test]
    fn pending_rename_is_consumed_on_media_unloaded() {
        let test_dir = TestDirectory::new(2);

        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
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
        assert!(
            !workspace.has_pending_rename(),
            "pending must be None after Unloaded"
        );

        // Second Unloaded: must be a no-op (nothing to consume).
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        assert!(
            !workspace.has_pending_rename(),
            "still None after second Unloaded"
        );
    }

    /// A file whose comment the scan left loading is loaded when it is opened, before the
    /// workspace shows it: its comment must be there from the start, not pop up afterwards.
    #[test]
    fn a_file_is_opened_with_its_comment_already_loaded() {
        let test_dir = TestDirectory::new(2);
        let mut directory = test_dir.directory();
        let (id, path, mut snapshot) = {
            let file = directory.files_in_order().next().expect("a file");
            (
                file.id(),
                file.file_path().to_path_buf(),
                file.snapshot().clone(),
            )
        };
        snapshot.set_comment_loading(true);
        directory.rename_file(id, &path, &snapshot);
        assert_eq!(directory.loading_comment_count(), 1);

        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory,
            target_file: Some(path.clone()),
        });
        flush_file_opened(&mut workspace);

        let opened = workspace
            .file_workspace()
            .get_snapshot()
            .expect("open file")
            .1;
        assert!(!opened.comment_loading(), "read before it is shown");
        assert_eq!(
            workspace.directory().expect("dir").loading_comment_count(),
            0
        );
    }

    /// Loads a folder and turns batch mode on with every file checked.
    fn batch_workspace(test_dir: &TestDirectory) -> FolderWorkspace {
        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let ids: Vec<FileId> = workspace
            .directory()
            .expect("dir")
            .files_in_order()
            .map(|f| f.id())
            .collect();
        let _ = workspace.update(Message::Batch(batch::Message::SetActive(true)));
        let _ = workspace.update(Message::Batch(batch::Message::CheckAll(ids)));
        workspace
    }

    /// A job waits for the playing video to unload, runs file by file with the folder locked,
    /// and gives the open file back when it ends.
    #[test]
    fn a_batch_job_locks_the_folder_and_reopens_the_file_when_done() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = batch_workspace(&test_dir);

        let _ = workspace.update(Message::Batch(batch::Message::Run));
        assert!(workspace.is_batch_running());
        assert!(
            workspace.file_workspace().file().is_some(),
            "waits for the video to unload"
        );
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        assert!(
            workspace.file_workspace().file().is_none(),
            "the open file is closed during the job"
        );

        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        assert_eq!(
            workspace.directory().and_then(|d| d.selected_index()),
            Some(0),
            "navigation is locked"
        );

        let skipped = || ItemResult {
            status: ItemStatus::Skipped,
            update: None,
        };
        let _ = workspace.update(Message::BatchItemDone {
            id: file_id_at(&workspace, 0),
            result: skipped(),
        });
        assert!(workspace.is_batch_running());
        let _ = workspace.update(Message::BatchItemDone {
            id: file_id_at(&workspace, 1),
            result: skipped(),
        });
        assert!(!workspace.is_batch_running(), "ended after the last file");
        assert_eq!(workspace.batch().progress().map(|p| p.skipped), Some(2));

        let _ = workspace.update(Message::BatchFinished);
        flush_file_opened(&mut workspace);
        assert!(
            workspace.file_workspace().file().is_some(),
            "the file is open again"
        );
    }

    /// A cancelled job stops after the file in work and leaves the rest untouched.
    #[test]
    fn a_cancelled_batch_job_stops_after_the_file_in_work() {
        let test_dir = TestDirectory::new(3);
        let mut workspace = batch_workspace(&test_dir);
        let _ = workspace.update(Message::Batch(batch::Message::Run));
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));

        let _ = workspace.update(Message::Batch(batch::Message::Cancel));
        let first = file_id_at(&workspace, 0);
        let done = ItemResult {
            status: ItemStatus::Done,
            update: None,
        };
        let _ = workspace.update(Message::BatchItemDone {
            id: first,
            result: done,
        });
        assert!(!workspace.is_batch_running());
        let progress = workspace.batch().progress().expect("report");
        assert_eq!((progress.finished, progress.total), (1, 3));
    }

    /// Typing the first character of a comment checks the commented tag; clearing the comment
    /// unchecks it; unchecking it by hand while the comment stays is respected.
    #[test]
    fn the_commented_tag_follows_a_comment_appearing_and_disappearing() {
        use iced::widget::text_editor::{Action, Edit};
        let test_dir = TestDirectory::new(1);
        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let tags = |w: &FolderWorkspace| {
            w.file_workspace()
                .tag_list()
                .file_snapshot()
                .tags()
                .to_vec()
        };

        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Insert('g'))));
        assert_eq!(tags(&workspace), ["Commented"]);

        let id = workspace
            .file_workspace()
            .tag_list()
            .checked_tag_id_at(0)
            .expect("tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(id)));
        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Insert('o'))));
        assert!(
            tags(&workspace).is_empty(),
            "editing a comment leaves the tag to the user"
        );

        let _ = workspace.update(Message::CommentAction(Action::SelectAll));
        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Delete)));
        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Insert('x'))));
        assert_eq!(
            tags(&workspace),
            ["Commented"],
            "a new comment after clearing checks it again"
        );
    }

    /// Batch mode shows batch actions instead of the open file, so tag edits do not reach it.
    #[test]
    fn batch_mode_does_not_edit_the_open_file() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = batch_workspace(&test_dir);
        let tag_list = workspace.file_workspace().tag_list();
        let tag_id = tag_list
            .filtered_display_tag_ids()
            .iter()
            .find(|id| {
                tag_list
                    .get_tag(**id)
                    .map(|t| t.tag() == "pick")
                    .unwrap_or(false)
            })
            .copied()
            .expect("pick is a built-in tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(tag_id)));
        let checked = workspace
            .file_workspace()
            .tag_list()
            .get_tag(tag_id)
            .map(|t| t.is_checked());
        assert_eq!(checked, Some(false));
    }

    /// Re-saving the selected file after an unload (re-clicking it, renaming it in place) must
    /// rename it before its media is opened again, so the player gets the new name rather
    /// than the one the file no longer has.
    #[test]
    fn unload_saves_the_selected_file_before_reopening_it() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);

        let file = workspace.current_file().cloned().expect("a file is open");
        let mut snapshot = file.snapshot().clone();
        snapshot.set_tags(["Goat"]);
        workspace.inject_pending_rename(file.id(), snapshot);
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));

        let selected = workspace
            .directory()
            .and_then(|d| d.selected_file())
            .expect("still selected");
        let name = selected
            .file_path()
            .file_name()
            .and_then(|n| n.to_str())
            .expect("name");
        assert!(
            name.starts_with("Goat."),
            "saved before reopening, got {name}"
        );
    }

    /// With the untagged filter on, a file leaves the list only once the cursor has left it,
    /// and the row it frees is taken by the file the cursor moved to — the selection does not
    /// jump to another row while the list shrinks.
    #[test]
    fn untagged_filter_keeps_the_cursor_row_when_a_file_is_tagged() {
        let test_dir = TestDirectory::new(3);
        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::Folder(folder::Message::SetUntaggedOnly(true)));

        let file_0_id = file_id_at(&workspace, 0);

        // Move to file_1; the deferred save then writes file_0's tags.
        let _ = workspace.update(Message::Folder(folder::Message::NextFile));
        flush_file_opened(&mut workspace);
        let snapshot =
            FileSnapshot::new(vec!["Comedy".to_string()], "file_0", ".mp4", "file_0.mp4");
        let _ = workspace.update(Message::FileUpdated {
            id: file_0_id,
            snapshot,
        });

        let dir = workspace.directory().expect("directory should be loaded");
        assert_eq!(
            dir.files_in_order().count(),
            2,
            "the file that got tags must leave the untagged list"
        );
        assert_eq!(
            dir.selected_index(),
            Some(0),
            "the cursor must keep the row freed by the tagged file"
        );
    }

    /// The file search narrows the list, and the file under the cursor stays listed so the
    /// selection keeps pointing at a real row while the query is typed.
    #[test]
    fn file_search_narrows_the_list() {
        let test_dir = TestDirectory::new(3);
        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);

        let _ = workspace.update(Message::Folder(folder::Message::SetNameFilter(
            "file_2".to_string(),
        )));
        let dir = workspace.directory().expect("directory should be loaded");
        assert_eq!(
            dir.files_in_order().count(),
            2,
            "the match plus the selected file stay listed"
        );

        let _ = workspace.update(Message::Folder(folder::Message::SetNameFilter(
            String::new(),
        )));
        let dir = workspace.directory().expect("directory should be loaded");
        assert_eq!(
            dir.files_in_order().count(),
            3,
            "clearing the query restores the list"
        );
    }

    /// Verify that `save_and_reparse` (called by FileUpdated) updates the file path in the
    /// directory when tags change the file name.
    #[test]
    fn file_updated_renames_file_path_in_directory() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = FolderWorkspace::new();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
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
        let _ = workspace.update(Message::FileUpdated {
            id: file_0_id,
            snapshot,
        });

        // The directory should now track the file under its new path.
        let dir = workspace.directory().expect("directory should be loaded");
        let new_path = test_dir.file_path("Comedy.file_0.mp4");
        let file_renamed = dir
            .files_in_order()
            .any(|f| f.file_path() == new_path.as_path());
        assert!(
            file_renamed,
            "directory must track the renamed path Comedy.file_0.mp4"
        );
    }

    #[test]
    fn new_file_names_follow_windows_rules() {
        let folder = std::env::temp_dir().join(format!("frename-rename-{}", std::process::id()));
        std::fs::create_dir_all(&folder).expect("temp dir");
        std::fs::write(folder.join("taken.mp4"), b"").expect("write");

        assert_eq!(
            crate::features::folder_workspace::state::check_new_file_name(
                &folder, "a.mp4", "b.mp4"
            ),
            Ok(())
        );
        assert!(
            crate::features::folder_workspace::state::check_new_file_name(&folder, "a.mp4", "")
                .is_err()
        );
        assert!(
            crate::features::folder_workspace::state::check_new_file_name(
                &folder, "a.mp4", "a:b.mp4"
            )
            .is_err()
        );
        assert!(
            crate::features::folder_workspace::state::check_new_file_name(
                &folder, "a.mp4", "a.mp4."
            )
            .is_err()
        );
        assert!(
            crate::features::folder_workspace::state::check_new_file_name(
                &folder,
                "a.mp4",
                "taken.mp4"
            )
            .is_err(),
            "must not overwrite"
        );
        assert_eq!(
            crate::features::folder_workspace::state::check_new_file_name(
                &folder,
                "taken.mp4",
                "TAKEN.mp4"
            ),
            Ok(()),
            "case-only rename"
        );
    }
}
