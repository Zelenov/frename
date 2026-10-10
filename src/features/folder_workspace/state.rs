//! State for folder workspace: only data and selection logic.
//!
//! Owns: loaded folder (directory + files), currently selected file, loading flag.
//! Also holds child feature state (video, rename panel) and workspace layout (splitter positions).
//! No UI concepts (scrollable, shape, size of children)—only data passed to feature views.
//! Each feature view decides how it looks; the workspace view only arranges regions.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use frename_core::undo::History;
use frename_core::{
    AppDatabase, AppStateStore, BatchRun, CheckTagsCommand, CreateTagCommand, DeleteTagCommand,
    File, FileId, FileSnapshot, FolderAndFile, FolderTagStore, LoggingAppStateStore, Marker,
    NavigateFileCommand, PasteTagsCommand, RenameFileCommand, ReorderTagCommand, SaveAndReparse,
    SaveTagCommand, Segment, SetCommentCommand, SetSegmentCommand, SetSegmentEndCommand,
    SetSegmentStartCommand, StarTagCommand, SyncTagOrderCommand, ToggleTagCommand, UndoContext,
    UndoError,
};
use rfd;

use super::messages::GlobalSearchKey;
use super::Directory;
use iced::widget::operation;
use iced::{Subscription, Task};

use crate::features::batch::{self, BatchState, ItemResult, ItemStatus};
use crate::features::drag_out::{self, DragOutState};
use crate::features::file_menu::{self, FileAction, FileMenuState};
use crate::features::file_name_panel::{self, FileNamePanelState};
use crate::features::file_workspace::FileWorkspace;
use crate::features::folder;
use crate::features::markers::MarkersState;
use crate::features::media_viewer::{self, video as media_viewer_video, MediaViewerState};
use crate::features::recent_folders::{self, RecentFoldersState};
use crate::features::sync_panel;
use crate::features::tag_grid::groups::Shape;
use crate::features::tag_panel::{self, TagPanelState, TAG_LIST_SCROLLABLE_ID};
use crate::features::video_controls;
use crate::ui::tokens::{
    FILE_LIST_MAX_WIDTH, FILE_LIST_MIN_WIDTH, FILE_LIST_WIDTH, SPLITTER_HIT, VIDEO_MIN_WIDTH,
    VIDEO_WIDTH,
};
use crate::widgets::search_bar::SEARCH_BAR_INPUT_ID;

use super::Message;

mod drag_out_actions;
mod file_actions;
mod journal;
mod marker_actions;
mod marker_ai;
mod recent_list;
mod rotation;

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
    /// Deferred renames: set when media must unload before a file can be renamed. Usually one
    /// entry (the file just left); a second can queue up if another save (closing, opening
    /// another folder, switching files again) is requested before the first `Unloaded` fires,
    /// since only one video is ever loaded — pushed, never overwritten, so an earlier entry is
    /// never silently dropped. Applied in order once `Unloaded` fires.
    pending_file_updates: Vec<(FileId, FileSnapshot)>,
    /// A folder scan waiting for a playing video to unload first, since the open file's pending
    /// edits (in `pending_file_updates`) may rename it.
    pending_scan: Option<FolderAndFile>,
    /// The window close waiting for a playing video to unload first. Sets it apart from a plain
    /// file switch: once the pending saves are applied, the video must not be reopened — the
    /// app is closing, not moving to another file.
    closing: bool,
    /// Batch mode: checked files, the chosen action and its job.
    batch: BatchState,
    /// A batch job waits for a playing video to unload, since it may write into and rename
    /// that very file.
    batch_waits_for_unload: bool,
    /// Marker "Describe with AI" requests whose answer has not come back yet, stopped ones too:
    /// one may still be reading the clip's frames, which holds the file open.
    marker_requests: usize,
    /// The file being renamed in place in the folder list, if any.
    inline_rename: Option<folder::InlineRename>,
    /// The file list's filter menu is open.
    filter_menu_open: bool,
    /// The folders opened recently: the dropdown by the open button and the empty screen's list.
    recent_folders: RecentFoldersState,
    /// The app database every read and write of the workspace goes to: the recent folders, the
    /// last batch run, the panel widths, the last session. A test gives each workspace its own.
    app_db: AppDatabase,
    /// Bumped per folder, so a comment batch for a folder no longer open is dropped.
    comment_load_generation: u64,
    /// Frame of the loading spinner shown in rows whose comment is still loading.
    spinner_frame: usize,
    /// The ID of the file navigated to (captured after dir.select_*).
    /// Taken by apply_file_opened, which pairs it with the file left.
    pending_to_file_id: Option<FileId>,
    /// The navigations whose save has not run yet, as (file left, file navigated to), oldest
    /// first. apply_file_updated pushes the NavigateFileCommand from the first of its file. A
    /// list, not one value, so navigations before the video unloads each get their own step.
    navigation_targets: Vec<(FileId, FileId)>,
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
    /// The open file's tags and comment when the comment box was first typed in; the typing up
    /// to the box losing the keys (or the file changing) is one undo step.
    comment_session: Option<FileSnapshot>,
    /// Whether the media viewer is currently shown fullscreen (F5).
    media_fullscreen: bool,
    /// The marker list's own state: the row being edited and what `F2` did last.
    markers: MarkersState,
    /// Every marker GUID read from or written to a file this session. A save deletes a file
    /// marker whose GUID is here but no longer in the list: the user deleted it. Other markers
    /// frename never saw (added by Premiere while the file was open) are kept. GUIDs are unique
    /// across files, so one set serves them all.
    known_marker_guids: HashSet<String>,
    /// Markers whose write into their file failed (the file read-only or open in Premiere),
    /// kept until the file is saved again. By `FileId`, which the next folder scan renews.
    unsaved_markers: HashMap<FileId, Vec<Marker>>,
    /// Files shown with the markers their video holds, while markers are kept in the comment:
    /// as read. Untouched, they stay in the video; edited, they move into the comment.
    markers_in_video: HashMap<FileId, Vec<Marker>>,
    /// The open clip's state when it was opened or last saved: edits are journaled once the clip
    /// differs from this.
    journal_baseline: Option<(FileId, frename_core::recovery::Entry)>,
    /// What the recovery journal holds of the open clip.
    journal_written: Option<(FileId, frename_core::recovery::Entry)>,
    /// The open clip was reopened with edits still waiting to be saved: journal it as it is.
    journal_force: bool,
    /// The clip whose journal entry is being written (one write at a time).
    journal_in_flight: Option<FileId>,
    /// That clip was saved while its entry was being written: the entry is removed when the
    /// write lands.
    journal_saved_while_writing: bool,
    /// Messages from the start (edits restored after a crash), shown once a clip is open.
    startup_notes: Vec<String>,
    /// Files whose markers were shown from the video and then edited into the comment: the
    /// video's own are taken out once a save shows the comment holds them. Kept apart from
    /// `markers_in_video`, which is replaced whenever the clip is opened again: a save still
    /// waiting for the video to unload must find it after a reopen.
    clear_from_video: HashSet<FileId>,
    /// A press on a file row that may become a drag out of the window.
    drag_out: DragOutState,
    /// Ctrl/Shift held right now, as of the last `ModifiersChanged`/window-unfocus event: a
    /// mouse click carries no modifiers of its own in iced, so a file-row click reads this.
    modifiers: iced::keyboard::Modifiers,
    /// The file a Shift+click range runs from: the last Ctrl-clicked file, or, once multi-select
    /// starts without one (a first Shift+click), the file that was open before it. `None` outside
    /// multi-selection, so the next one starts fresh from whatever is open then.
    select_anchor: Option<FileId>,
    /// The file context menu (right-click on a file).
    file_menu: FileMenuState,
    /// A file menu action waiting for its file's pending edits to reach the disk, so it acts on
    /// the file's final name. Run by the save of that file.
    pending_file_action: Option<(FileId, FileAction)>,
    /// The system clipboard, kept open: on Linux what was copied lasts only while it is.
    clipboard: Option<arboard::Clipboard>,
}

/// The video pane's and the file list's widths, kept within their limits (§13.9).
fn column_widths(video: f32, file_list: f32) -> (f32, f32) {
    (
        video.max(VIDEO_MIN_WIDTH),
        file_list.clamp(FILE_LIST_MIN_WIDTH, FILE_LIST_MAX_WIDTH),
    )
}

impl FolderWorkspace {
    pub fn new() -> Self {
        Self::with_app_db(AppDatabase::new())
    }

    /// A workspace that reads and writes `app_db` instead of the global app database.
    pub fn with_app_db(app_db: AppDatabase) -> Self {
        let (left_width, folder_width) = app_db
            .get_window_state()
            .and_then(|w| {
                if w.left_panel_width > 0.0 && w.folder_panel_width > 0.0 {
                    Some((w.left_panel_width, w.folder_panel_width))
                } else {
                    None
                }
            })
            .unwrap_or((VIDEO_WIDTH, FILE_LIST_WIDTH));
        // A width saved by an older version or on a smaller screen is raised to the minimum.
        let (left_width, folder_width) = column_widths(left_width, folder_width);
        let mut recent_folders = RecentFoldersState::default();
        recent_folders.set_entries(
            app_db.get_recent_folders(),
            frename_core::recent_folders::now_ms(),
        );
        let mut batch = BatchState::default();
        if let Some(run) = app_db.get_batch_run() {
            batch.restore_last_run(run);
        }
        Self {
            directory: None,
            loading: false,
            file_workspace: FileWorkspace::<FolderTagStore>::default(),
            media_viewer: MediaViewerState::default(),
            tag_panel: TagPanelState::default(),
            file_name_panel: FileNamePanelState::default(),
            pending_file_updates: Vec::new(),
            pending_scan: None,
            closing: false,
            batch,
            batch_waits_for_unload: false,
            marker_requests: 0,
            inline_rename: None,
            filter_menu_open: false,
            recent_folders,
            app_db,
            comment_load_generation: 0,
            spinner_frame: 0,
            pending_to_file_id: None,
            navigation_targets: Vec::new(),
            left_width,
            folder_width,
            tag_list_scroll_y: None,
            tag_list_viewport_height: None,
            folder_scroll_y: None,
            folder_viewport_height: None,
            copied_tags: None,
            history: WorkspaceHistory::new(HISTORY_DEPTH),
            comment_session: None,
            media_fullscreen: false,
            markers: MarkersState::default(),
            known_marker_guids: HashSet::new(),
            unsaved_markers: HashMap::new(),
            markers_in_video: HashMap::new(),
            journal_baseline: None,
            journal_written: None,
            journal_force: false,
            journal_in_flight: None,
            journal_saved_while_writing: false,
            startup_notes: Vec::new(),
            clear_from_video: HashSet::new(),
            drag_out: DragOutState::default(),
            modifiers: iced::keyboard::Modifiers::empty(),
            select_anchor: None,
            file_menu: FileMenuState::default(),
            pending_file_action: None,
            clipboard: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        // The open recent folders list has the arrows, Enter and Esc, also in batch mode.
        if let Some(key) = self.recent_folders_key(&message) {
            return key.map_or_else(Task::none, |key| self.handle_recent_folders(key));
        }
        if self.is_blocked(&message) {
            return Task::none();
        }
        match message {
            // While a marker row is open, keys belong to its fields: `[b-roll]` is typed, not in
            // and out. Undo and redo are the exception: they close the row first (below), so
            // they never leave a row open on a marker they remove.
            Message::SetSegmentStart
            | Message::SetSegmentEnd
            | Message::TagPanel(tag_panel::Message::ToggleSelectedTag)
            | Message::CopyTags
            | Message::PasteTags
            | Message::RotateVideoWhileTyping(_)
            | Message::StepFrameWhileTyping(_)
            | Message::GoToStartWhileTyping
                if self.markers.is_editing() =>
            {
                Task::none()
            }
            Message::OpenPath(path) => self.open_path(path),
            Message::LoadLastSession => {
                Task::batch([self.load_last_session(), self.reload_recent_folders()])
            }
            Message::RecentFolders(msg) => self.handle_recent_folders(msg),
            Message::ScanFolder(pair) => self.begin_scan_folder(pair),
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
            // The tick doubles as the pump for the start-up notes: it runs while a clip is open,
            // which is when they can be shown.
            Message::JournalTick => Task::batch([self.journal_tick(), self.show_startup_notes()]),
            Message::ShowDescribing(name) => {
                let guid = self
                    .file_workspace
                    .markers()
                    .and_then(|markers| markers.iter().find(|m| m.name == name))
                    .and_then(|m| m.guid.clone());
                if let Some(guid) = guid {
                    let _ = self.markers.start_describing(&guid);
                }
                Task::none()
            }
            Message::ShowBatchRetry(seconds) => {
                let files: Vec<FileId> = self.directory.as_ref().map_or_else(Vec::new, |dir| {
                    dir.all_files()
                        .filter(|f| self.batch.is_checked(f.id()))
                        .map(|f| f.id())
                        .collect()
                });
                if !self.batch.start(files) {
                    // The clip lengths are still being read: try again shortly.
                    return Task::future(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        Message::ShowBatchRetry(seconds)
                    });
                }
                if self.batch.begin_next().is_some() {
                    if let Some(progress) = self.batch.item_progress() {
                        batch::describe_ai::show_retry(&progress, seconds);
                    }
                }
                Task::none()
            }
            Message::ShowDescribingUnnamed => {
                let guids = self.unnamed_marker_guids();
                self.markers.queue_describing(guids);
                // As `send_waiting_markers` starts them, but nothing is sent.
                while self
                    .markers
                    .start_next_waiting(self.markers.in_flight(), |_| true)
                    .is_some()
                {}
                Task::none()
            }
            Message::MarkerDescribed {
                file,
                guid,
                request,
                outcome,
            } => self.marker_described(file, guid, request, outcome),
            Message::JournalWritten(id, outcome) => {
                self.journal_written(id, outcome);
                Task::none()
            }
            Message::SpinnerTick => {
                self.spinner_frame = self.spinner_frame.wrapping_add(1);
                Task::none()
            }
            Message::DragOut(msg) => self.handle_drag_out(msg),
            // Intercepted by the app, which owns the window; no-op here.
            Message::StartDragOut(_) => Task::none(),
            Message::DragOutFinished => self.drag_out_finished(),
            Message::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
            }
            Message::Folder(folder_msg) => self.handle_folder_message(folder_msg),
            Message::FileMenu(msg) => self.handle_file_menu(msg),
            Message::FileAction(action) => self.file_action_on_open_file(action),
            Message::RunFileAction(id, action) => self.run_file_action(id, action),
            Message::MediaViewer(msg) => match msg {
                media_viewer::Message::Unloaded => self.on_media_unloaded(),
                media_viewer::Message::ToggleFullscreen => {
                    self.set_fullscreen(!self.media_fullscreen)
                }
                media_viewer::Message::SegmentStartMarked(secs) => self.set_segment_start(secs),
                media_viewer::Message::SegmentEndMarked(secs) => self.set_segment_end(secs),
                media_viewer::Message::ScreenshotTaken(position_ms, jpeg) => {
                    Task::done(Message::ScreenshotTaken(position_ms, jpeg))
                }
                // Straight to the markers in this update, not re-sent with `Task::done`: a
                // view built in between would hand the name field its old text, and fast
                // typing lost letters.
                media_viewer::Message::Video(media_viewer_video::Message::Markers(msg)) => {
                    let position_ms = self.media_viewer.video_position_ms().unwrap_or(0);
                    self.handle_marker(msg, position_ms)
                }
                media_viewer::Message::Video(media_viewer_video::Message::Controls(
                    video_controls::Message::Rotate(quarter_turns),
                )) => self.rotate_video(quarter_turns),
                other => {
                    let task = self.media_viewer.update(other).map(Message::MediaViewer);
                    self.grow_held_marker();
                    Task::batch([task, self.follow_marker_list(false)])
                }
            },
            Message::TagPanel(msg) => self.handle_tag_panel(msg),
            Message::FileNamePanel(msg) => self.handle_file_name_panel(msg),
            Message::SyncPanel(msg) => self.handle_sync_panel(msg),
            Message::LeftSplitterDragged(x) => {
                self.left_width = x;
                let folder_start = self.left_width + SPLITTER_HIT;
                let folder_end = folder_start + self.folder_width;
                let new_folder_width = folder_end - x - SPLITTER_HIT;
                self.folder_width =
                    new_folder_width.clamp(FILE_LIST_MIN_WIDTH, FILE_LIST_MAX_WIDTH);
                self.app_db
                    .set_panel_widths(self.left_width, self.folder_width);
                Task::none()
            }
            Message::RightSplitterDragged(x) => {
                let new_folder_width = x - self.left_width - SPLITTER_HIT;
                self.folder_width =
                    new_folder_width.clamp(FILE_LIST_MIN_WIDTH, FILE_LIST_MAX_WIDTH);
                self.app_db
                    .set_panel_widths(self.left_width, self.folder_width);
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
            // The comment box has the keys while it is focused: its text is not the app's to undo.
            Message::Undo | Message::Redo if self.file_workspace.comment_focused() => Task::none(),
            // What was typed or named is a step first, so it is the one undone.
            Message::Undo => {
                self.close_marker_row();
                self.end_comment_session();
                if self.history.can_undo() {
                    self.perform_undo()
                } else {
                    Task::none()
                }
            }
            Message::Redo => {
                self.close_marker_row();
                self.end_comment_session();
                if self.history.can_redo() {
                    self.perform_redo()
                } else {
                    Task::none()
                }
            }
            Message::RotateVideo(quarter_turns) => self.rotate_video(quarter_turns),
            Message::RotateVideoWhileTyping(quarter_turns) => {
                self.unless_writing(Message::RotateVideo(quarter_turns))
            }
            Message::StepFrame(step) => {
                Task::done(Message::MediaViewer(media_viewer::Message::Video(
                    media_viewer_video::Message::Controls(video_controls::Message::StepFrame(step)),
                )))
            }
            Message::StepFrameWhileTyping(step) => self.unless_writing(Message::StepFrame(step)),
            Message::GoToStart => {
                // Only a video shown has a start to go to: a clip hidden behind another file's
                // placeholder is not touched.
                if !self.media_viewer.is_previewable() {
                    return Task::none();
                }
                Task::done(Message::MediaViewer(media_viewer::Message::Video(
                    media_viewer_video::Message::GoToStart,
                )))
            }
            Message::GoToStartWhileTyping => {
                // Only the note's promise takes Home from a search field; a name being typed
                // keeps it. A rename open on another row does not hold the key: ask whether
                // its field is the one that has it.
                if !self.media_viewer.resume_note_shown() {
                    return Task::none();
                }
                if self.inline_rename.is_none() {
                    return self.unless_writing(Message::GoToStart);
                }
                iced::widget::operation::is_focused(iced::widget::Id::from(
                    folder::FOLDER_RENAME_INPUT_ID,
                ))
                .map(|renaming| {
                    if renaming {
                        Message::Noop
                    } else {
                        Message::GoToStartUnlessWriting
                    }
                })
            }
            Message::GoToStartUnlessWriting => self.unless_writing(Message::GoToStart),
            Message::ToggleMediaFullscreen => self.set_fullscreen(!self.media_fullscreen),
            Message::RestoreListScrolls { markers_y, cues_y } => {
                // Only a list on screen reports back; armed otherwise it would fire much later.
                if self.media_viewer.marker_list_shown() {
                    self.markers.restored(markers_y);
                }
                Task::batch([
                    marker_actions::scroll_marker_list_to(markers_y),
                    self.media_viewer
                        .update(media_viewer::Message::Video(
                            media_viewer_video::Message::RestoreCueScroll(cues_y),
                        ))
                        .map(Message::MediaViewer),
                ])
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
            Message::CheckCommentFocus => {
                // Asked only while the box is on screen: about a missing widget, no answer comes.
                if self.batch.is_active() || self.file_workspace.file().is_none() {
                    self.end_comment_session();
                    self.file_workspace.set_comment_focused(false);
                    return Task::none();
                }
                iced::widget::operation::is_focused(iced::widget::Id::new(
                    crate::features::file_workspace::view::COMMENT_EDITOR_ID,
                ))
                .map(Message::CommentFocused)
            }
            Message::CommentFocused(focused) => {
                if !focused {
                    self.end_comment_session();
                }
                self.file_workspace.set_comment_focused(focused);
                Task::none()
            }
            Message::CommentAction(action) => {
                // Anything done in the box (a click, typing) means it has the keys.
                self.file_workspace.set_comment_focused(true);
                let typed = matches!(action, iced::widget::text_editor::Action::Edit(_));
                if typed && self.comment_session.is_none() {
                    self.comment_session = self.file_workspace.get_snapshot().map(|(_, s)| s);
                }
                self.file_workspace.apply_comment_action(action);
                // The box grows with its text inside a scrollable: typing on the last line
                // keeps that line in view.
                if typed && self.file_workspace.comment_cursor_on_last_line() {
                    iced::widget::operation::snap_to_end(iced::widget::Id::new(
                        crate::features::file_workspace::view::COMMENT_SCROLLABLE_ID,
                    ))
                } else {
                    Task::none()
                }
            }
            Message::CommentLayout(layout) => {
                self.file_workspace.update_comment_layout(layout);
                Task::none()
            }
            // Save the frame next to the video, like VLC's snapshot, and forget it.
            Message::ScreenshotTaken(position_ms, jpeg) => {
                let Some(file) = self.file_workspace.file() else {
                    return Task::none();
                };
                let file_path = file.file_path().to_path_buf();
                frename_core::FileTagger::save_screenshot(&file_path, position_ms, &jpeg);
                Self::notice("Frame saved")
            }
            Message::EscapePressed => {
                if self.file_menu.is_open() {
                    return self.handle_file_menu(file_menu::Message::Close);
                }
                if self.markers.is_editing() {
                    self.close_marker_row();
                    return Task::none();
                }
                if self.inline_rename.take().is_some() {
                    return Task::none();
                }
                if self.media_fullscreen {
                    return self.set_fullscreen(false);
                }
                if self.batch.is_active() {
                    // A running job ignores `SetActive` (see `BatchState::update`), so batch mode
                    // stays on: keep the anchor too, or a Shift+click after the job finishes would
                    // silently range from the wrong file instead of the documented last-Ctrl-click.
                    if !self.batch.is_running() {
                        self.select_anchor = None;
                    }
                    return self.handle_batch(batch::Message::SetActive(false));
                }
                // Both search bars label their clear button "Esc", so clear both.
                let clear_files = self.set_file_name_filter(String::new());
                Task::batch([
                    self.handle_tag_panel(tag_panel::Message::SetFilter(String::new())),
                    clear_files,
                ])
            }
            Message::OpenFolderPicker => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                |opt| opt.map_or(Message::Noop, Message::OpenPath),
            ),
            Message::OpenFilePicker => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_file()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                |opt| opt.map_or(Message::Noop, Message::OpenPath),
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
        self.end_comment_session();
        self.file_workspace.set_comment_focused(false);
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

    /// Set the open clip's in and out points to the ones its AI description suggests, as one
    /// undo step. Like `[` and `]`, nothing happens while the clip's name is being edited.
    fn apply_suggested_in_out(&mut self) -> Task<Message> {
        if self.inline_rename.is_some() {
            return Task::none();
        }
        let Some(new) = file_name_panel::suggested_in_out(self.file_workspace.tag_list()) else {
            return Task::none();
        };
        let old = Segment {
            start: self.file_workspace.segment_start_secs(),
            end: self.file_workspace.segment_end_secs(),
        };
        self.file_workspace.set_segment_start_secs(new.start);
        self.file_workspace.set_segment_end_secs(new.end);
        self.history.push(Box::new(SetSegmentCommand { old, new }));
        Task::none()
    }

    /// Add the tags the open clip's AI description suggests, by name, as adding them by hand
    /// would: one undo step, whether one tag or all. Only the folder's tags the clip does not
    /// have yet; like `[` and `]`, nothing happens while the clip's name is being edited.
    fn add_suggested_tags(&mut self, names: &[String]) -> Task<Message> {
        if self.inline_rename.is_some() {
            return Task::none();
        }
        let tag_list = self.file_workspace.tag_list();
        let mut ids: Vec<frename_core::TagId> = Vec::new();
        for id in names
            .iter()
            .filter_map(|name| tag_list.tag_id_by_name(name))
        {
            let addable = tag_list
                .get_tag(id)
                .is_some_and(|tag| tag.is_stored() && !tag.is_checked());
            if addable && !ids.contains(&id) {
                ids.push(id);
            }
        }
        for id in &ids {
            self.file_workspace.toggle_tag_by_id(*id);
        }
        match ids.as_slice() {
            [] => {}
            [id] => self.history.push(Box::new(ToggleTagCommand {
                tag_id: *id,
                was_checked: false,
            })),
            _ => self
                .history
                .push(Box::new(CheckTagsCommand { tag_ids: ids })),
        }
        Task::none()
    }

    fn copy_tags(&mut self) -> Task<Message> {
        let Some((_, snapshot)) = self.file_workspace.get_snapshot() else {
            return Task::none();
        };
        self.copied_tags = Some(snapshot.tags().to_vec());
        if let Err(e) = self.set_clipboard_text(snapshot.file_name()) {
            log::warn!("could not copy the file name to the clipboard: {e}");
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
        let mut snapshot_after = FileSnapshot::new(
            names,
            snapshot_before.name_without_extension(),
            snapshot_before.extension(),
            snapshot_before.initial_file_name(),
        );
        // Pasting changes the tags only; the comment and in/out points are not in the name any
        // more and must be carried over, or they would show as wiped until the next save (#83).
        snapshot_after.set_comment(snapshot_before.comment().to_string());
        snapshot_after.set_segment(snapshot_before.segment());
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
        let store = LoggingAppStateStore::new(self.app_db.clone());
        let Some(session) = store.get_last_session() else {
            return Task::none();
        };
        Task::done(Message::ScanFolder(session))
    }

    fn open_path(&mut self, path: PathBuf) -> Task<Message> {
        let Some(pair) = FolderAndFile::from_path(&path) else {
            log::warn!("Path does not exist, not opening: {}", path.display());
            return Task::none();
        };
        match pair.file() {
            Some(file_path) => self.open_file(file_path.to_path_buf()),
            None => Task::done(Message::ScanFolder(pair)),
        }
    }

    fn open_file(&mut self, path: PathBuf) -> Task<Message> {
        let Some(file) = self.directory.as_mut().and_then(|dir| dir.open_path(&path)) else {
            // Capture the open file's pending edits before resetting file_workspace (issue #21).
            // The reset itself stays immediate and unconditional, as before: a reopen of the
            // very same path (`Directory::open_path` returns `None` for the already-selected
            // file too, which is what routes that case here — see
            // `a_white_ai_range_read_from_the_file_takes_a_new_color_and_keeps_it`, which reopens
            // the open file to pick up markers written externally) relies on file_workspace going
            // through `None` so the next `FileOpened` sees `same_file = false` and reloads. Do not
            // clear `pending_file_updates` here: an earlier deferred save (still waiting for the
            // same unload) must not be dropped by this one.
            let pending = self.file_workspace.get_snapshot();
            self.file_workspace.set_file(None);
            let pair = FolderAndFile::new(path.parent().unwrap_or(&path), Some(path.clone()));
            return self.save_then_scan(pending, pair);
        };
        Task::done(Message::FileOpened(file))
    }

    /// Save the open file's pending edits before scanning a new folder, so tags, comment and
    /// in/out set on the last clip of the previous folder are not lost when the user never
    /// switched files first (opening a different folder, `frename <folder>`, dropping a folder,
    /// "Open with", …). Mirrors the snapshot capture in `apply_file_opened`.
    fn begin_scan_folder(&mut self, pair: FolderAndFile) -> Task<Message> {
        let pending = self.file_workspace.get_snapshot();
        self.save_then_scan(pending, pair)
    }

    /// Apply an already-captured snapshot (deferred past a needed unload, same as a file
    /// switch) and only then scan the new folder, so the save resolves the old file's id
    /// against the still-current directory instead of racing the scan that replaces it.
    fn save_then_scan(
        &mut self,
        pending: Option<(FileId, FileSnapshot)>,
        pair: FolderAndFile,
    ) -> Task<Message> {
        let Some((id, snap)) = pending else {
            return self.scan_folder(pair);
        };
        if self.media_viewer.needs_unload_before_rename() {
            self.pending_file_updates.push((id, snap));
            self.pending_scan = Some(pair);
            Task::done(Message::MediaViewer(media_viewer::Message::Unload))
        } else {
            let saved = self.apply_file_updated(id, snap);
            Task::batch([saved, self.scan_folder(pair)])
        }
    }

    fn scan_folder(&mut self, pair: FolderAndFile) -> Task<Message> {
        self.batch.update(batch::Message::StopWaiting);
        self.inline_rename = None;
        self.markers.reset();
        // File ids are renewed by the scan, so markers kept for them cannot be matched again.
        self.unsaved_markers.clear();
        self.markers_in_video.clear();
        self.clear_from_video.clear();
        // Batches for the folder being left must not land in the next one.
        self.comment_load_generation += 1;
        let folder = pair.folder().to_path_buf();
        let target_file = pair.file().map(|p| p.to_path_buf());
        let store = LoggingAppStateStore::new(self.app_db.clone());
        self.loading = true;
        self.file_workspace.set_file(None);
        // Any deferred save must already be applied by now: `save_then_scan` only reaches this
        // call after applying (or, past a needed unload, queuing it for `on_media_unloaded` to
        // apply before this same call runs). This clear is just belt-and-braces against a stale
        // entry from a directory this scan is about to replace.
        self.pending_file_updates.clear();
        self.navigation_targets.clear();

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
        // A list left open (on the empty screen, where nothing shows it) must not pop up over
        // the folder that just opened.
        let _ = self.recent_folders.update(recent_folders::Message::Close);
        // The list filters are user settings, not properties of the folder: carry them over.
        if let Some(previous) = self.directory.as_ref() {
            directory.set_untagged_only(previous.untagged_only());
            directory.set_subtitled_only(previous.subtitled_only());
            directory.set_commented_only(previous.commented_only());
            directory.set_marked_only(previous.marked_only());
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
        // A specific file (a dropped file, "Open with", the last session) always wins when it is
        // still there; opening the folder itself, or a specific file that is gone since (a
        // rename, most likely), returns to the file this folder last had open (#98), by name and
        // then by name without tags, falling back to the first one listed.
        let selected = match target_file.as_deref().and_then(|p| dir.open_path(p)) {
            Some(file) => Some(file),
            None => {
                let last_viewed = target_file
                    .as_deref()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| FolderTagStore::get_last_viewed(&folder));
                dir.open_last_viewed(&last_viewed)
                    .or_else(|| dir.select_index(0))
            }
        };
        if let Some(file) = selected {
            Task::batch([
                Task::done(Message::FileOpened(file)),
                Task::done(Message::ScrollFolderListToSelected),
                load_comments_task,
            ])
        } else {
            self.file_workspace.set_file(None);
            self.pending_file_updates.clear();
            self.navigation_targets.clear();
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

    /// The file as its edits left it. A save waiting for the video to unload has not reached the
    /// directory yet, so a clip opened again in the meantime would show what it held before the
    /// edit (a cleared comment coming back), and leaving it again would save that over the edit.
    fn with_pending_edits(&self, file: &frename_core::File) -> frename_core::File {
        let mut file = file.clone();
        if let Some((_, edited)) = self
            .pending_file_updates
            .iter()
            .rev()
            .find(|(id, _)| *id == file.id())
        {
            file.set_file_snapshot(edited);
        }
        file
    }

    fn apply_file_opened(&mut self, file: frename_core::File) -> Task<Message> {
        // The typing in the comment box belongs to the file it was typed in.
        self.end_comment_session();
        let file = self.with_comment_loaded(file);
        // Opening another file closes the in-place rename editor, like leaving the row.
        if self
            .inline_rename
            .as_ref()
            .is_some_and(|r| r.id != file.id())
        {
            self.inline_rename = None;
        }
        // Detect same-file "refresh" (e.g. undo of a tag toggle on the current file, or an
        // in-place rename, whose path changed). In that case, skip media reload and fullscreen
        // reset — only persist state.
        let same_file = self
            .file_workspace
            .file()
            .is_some_and(|f| f.id() == file.id());
        if !same_file {
            self.media_fullscreen = false;
            self.close_marker_row();
            self.markers.reset();
        }
        let snapshot = self.file_workspace.get_snapshot();
        let navigated_to = self.pending_to_file_id.take();
        if let (Some((left, _)), Some(to)) = (snapshot.as_ref(), navigated_to) {
            self.navigation_targets.push((*left, to));
        }
        // A same-file refresh (undo, in-place rename) brings its own, newer state.
        let opened = if same_file {
            file.clone()
        } else {
            self.with_pending_edits(&file)
        };
        self.file_workspace.set_file(Some(opened));
        if !same_file {
            self.markers_loaded(file.id());
            self.journal_reset_baseline();
        }
        log::info!("Opening file: {}", file.file_path().display());
        let Some((id, snap)) = snapshot else {
            // Nothing new to save: leave any earlier deferred entry (still waiting for the same
            // unload) queued rather than dropping it here.
            if same_file {
                return Task::none();
            }
            return self.media_viewer.open(&file).map(Message::MediaViewer);
        };
        if self.media_viewer.needs_unload_before_rename() {
            self.pending_file_updates.push((id, snap));
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

    /// A video key (`Ctrl+Alt+←/→` turns, `Alt+←/→` frame steps) pressed while a text field had
    /// the keys: send `message`, unless the field is the comment box, where the keys belong to
    /// the text (the search fields hold nothing the keys would do, so there they still act). In
    /// batch mode the comment box is not shown, and asking about its focus would get no answer
    /// at all.
    fn unless_writing(&self, message: Message) -> Task<Message> {
        if self.batch.is_active() {
            return Task::done(message);
        }
        iced::widget::operation::is_focused(iced::widget::Id::new(
            crate::features::file_workspace::view::COMMENT_EDITOR_ID,
        ))
        .map(move |writing| {
            if writing {
                Message::Noop
            } else {
                message.clone()
            }
        })
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
        // Undo and redo follow what they step over: a marker or a turn of the open video is
        // undone in batch mode (when no job runs), a tag change is not.
        if self.batch.is_active() {
            let not_media = match message {
                Message::Undo => !self.history.undo_edits_open_video(),
                Message::Redo => !self.history.redo_edits_open_video(),
                _ => false,
            };
            if not_media && matches!(message, Message::Undo | Message::Redo) {
                return true;
            }
            if self.batch.is_running() && matches!(message, Message::Undo | Message::Redo) {
                return true;
            }
        }
        // The video and its marker list stay in batch mode, unlike the tags: markers and turns of
        // the open video are off only while a job runs (the job has closed the file then).
        let edits_open_media = matches!(
            message,
            Message::MediaViewer(media_viewer::Message::Video(
                media_viewer_video::Message::Markers(..)
                    | media_viewer_video::Message::Controls(video_controls::Message::Rotate(_))
            )) | Message::RotateVideo(_)
                | Message::RotateVideoWhileTyping(_)
        );
        if edits_open_media && self.batch.is_running() {
            return true;
        }
        let changes_files = matches!(
            message,
            Message::OpenPath(_)
                | Message::LoadLastSession
                | Message::ScanFolder(_)
                | Message::OpenFolderPicker
                | Message::OpenFilePicker
                | Message::RecentFolders(recent_folders::Message::Toggle)
                | Message::PrepareBatch(_)
                | Message::ToggleMediaFullscreen
                | Message::FileAction(_)
                | Message::RunFileAction(..)
                | Message::FileMenu(file_menu::Message::Open(_))
                | Message::Folder(
                    folder::Message::SelectFile(_)
                        | folder::Message::PreviousFile
                        | folder::Message::NextFile
                        | folder::Message::OpenFolder
                        | folder::Message::ToggleRecentFolders
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
        if let batch::Message::Retry = msg {
            if !self.batch.prepare_retry() {
                return Task::none();
            }
            return self.start_batch();
        }
        if let batch::Message::OpenLog = msg {
            open_in_default_app(crate::package::log_path_for_other_apps());
            return Task::none();
        }
        // From the job's report, when the service said its credit is used up.
        if let batch::Message::OpenBilling(service) = msg {
            open_in_default_app(service.billing_url());
            return Task::none();
        }
        self.handle_batch_many([msg])
    }

    /// What "Generate subtitles" shows before it runs is worked out in the background while
    /// its panel is shown: the plan of the checked videos (their headers), the price, and
    /// whether a Soniox key is saved.
    fn subtitle_reads(&mut self) -> Task<Message> {
        let Some(dir) = self.directory.as_ref() else {
            return Task::none();
        };
        let batch = &mut self.batch;
        // Every checked file, listed or hidden by a filter: the job runs them all.
        let checked: Vec<&frename_core::File> = dir
            .all_files()
            .filter(|f| batch.is_checked(f.id()))
            .collect();
        let reads = batch.subtitle_reads(&checked);
        let wrap = |msg: batch::generate_subtitles::Message| {
            Message::Batch(batch::Message::Action(
                batch::ActionMessage::GenerateSubtitles(msg),
            ))
        };
        let mut tasks = Vec::new();
        // The app reads it (the settings show it too) and passes the answer back.
        if reads.key_state {
            tasks.push(Task::done(Message::Batch(batch::Message::Action(
                batch::ActionMessage::ReadSonioxKeyState,
            ))));
        }
        if let Some((generation, files, replace, formats)) = reads.plan {
            tasks.push(Task::future(async move {
                let plan = tokio::task::spawn_blocking(move || {
                    batch::generate_subtitles::plan(&files, replace, formats)
                })
                .await
                .unwrap_or_else(|e| {
                    log::error!("subtitles: working out the plan failed: {e}");
                    batch::generate_subtitles::Plan::default()
                });
                wrap(batch::generate_subtitles::Message::PlanReady {
                    generation,
                    plan: Box::new(plan),
                })
            }));
        }
        if reads.price {
            tasks.push(Task::future(async move {
                let price = tokio::task::spawn_blocking(batch::generate_subtitles::price)
                    .await
                    .unwrap_or(batch::generate_subtitles::Price::Typical);
                wrap(batch::generate_subtitles::Message::PriceReady(price))
            }));
        }
        Task::batch(tasks)
    }

    /// What "Describe with AI" shows before it runs needs reading from disk: the length of
    /// each checked video and whether an API key is saved. Read in the background while its
    /// panel is shown.
    fn describe_ai_reads(&mut self) -> Task<Message> {
        let Some(dir) = self.directory.as_ref() else {
            return Task::none();
        };
        let batch = &mut self.batch;
        let checked: Vec<&frename_core::File> = dir
            .all_files()
            .filter(|f| batch.is_checked(f.id()))
            .collect();
        let (missing, read_key) = batch.describe_ai_reads(checked.into_iter());
        let wrap = |msg: batch::describe_ai::Message| {
            Message::Batch(batch::Message::Action(batch::ActionMessage::DescribeAi(
                msg,
            )))
        };
        // The app reads it (the settings show it too) and passes the answer back.
        let key = if read_key {
            Task::done(Message::Batch(batch::Message::Action(
                batch::ActionMessage::ReadKeyState,
            )))
        } else {
            Task::none()
        };
        if missing.is_empty() {
            return key;
        }
        let probes = Task::future(async move {
            let ids: Vec<FileId> = missing.iter().map(|(id, _)| *id).collect();
            let results =
                tokio::task::spawn_blocking(move || batch::describe_ai::probe_all(missing))
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("reading clip lengths failed: {e}");
                        // Unreadable rather than estimating forever.
                        ids.into_iter()
                            .map(|id| {
                                (
                                    id,
                                    batch::describe_ai::Probe {
                                        duration_s: None,
                                        subtitle_bytes: 0,
                                    },
                                )
                            })
                            .collect()
                    });
            wrap(batch::describe_ai::Message::Probed(results))
        });
        Task::batch([key, probes])
    }

    /// Start the selected batch action on the checked files, in folder order. A playing video
    /// is unloaded first; see [`Self::run_batch`].
    fn start_batch(&mut self) -> Task<Message> {
        // A list left open would swallow the keys with nothing to show for it.
        let _ = self.recent_folders.update(recent_folders::Message::Close);
        let Some(dir) = self.directory.as_ref() else {
            return Task::none();
        };
        let checked: Vec<&frename_core::File> = dir
            .all_files()
            .filter(|f| self.batch.is_checked(f.id()))
            .collect();
        let files = self
            .batch
            .actions()
            .job_files(self.batch.action(), &checked);
        // The job closes the clip: its marker requests stop, and the job starts once the last
        // one has let go of the clip (it may be reading its frames).
        if !files.is_empty() && !self.batch.is_running() && self.stop_marker_requests() {
            // The batch panel says so and offers Cancel.
            self.batch.update(batch::Message::WaitForMarkers);
            return Task::none();
        }
        self.share_folder_tags();
        if !self.batch.start(files) {
            return Task::none();
        }
        let action = self.batch.action();
        self.app_db.set_batch_run(BatchRun {
            action: action.id().to_string(),
            options: self.batch.actions().persist(action),
        });
        self.media_fullscreen = false;
        if self.media_viewer.needs_unload_before_rename() {
            self.batch_waits_for_unload = true;
            return Task::done(Message::MediaViewer(media_viewer::Message::Unload));
        }
        self.run_batch()
    }

    /// Tell "Describe with AI" the folder's own tags, so its panel and its next run suggest from
    /// the tags the folder has now.
    fn share_folder_tags(&mut self) {
        let tags = self.file_workspace.tag_list().saved_tag_names();
        self.batch.set_folder_tags(tags);
    }

    /// Save the edits waiting for the disk (the open file may be one of the job's), close the
    /// open file until the job ends, and start the first file.
    fn run_batch(&mut self) -> Task<Message> {
        let pending_saved = self.apply_pending_file_updates();
        let open_saved = self
            .file_workspace
            .get_snapshot()
            .map_or(Task::none(), |(id, snapshot)| {
                self.apply_file_updated(id, snapshot)
            });
        self.file_workspace.set_file(None);
        Task::batch([pending_saved, open_saved, self.next_batch_item()])
    }

    /// Run the job on its next file on a blocking thread, or end the job when it has none
    /// left or was cancelled. One file per step: progress names the file in work, and a
    /// cancel stops after it, so no file is left half done.
    fn next_batch_item(&mut self) -> Task<Message> {
        let Some((id, operation, cancel)) = self.batch.begin_next() else {
            return Task::done(Message::BatchFinished);
        };
        let progress = self.batch.item_progress().unwrap_or_default();
        let Some(path) = self
            .directory
            .as_ref()
            .and_then(|d| d.file_by_id(id))
            .map(|f| f.file_path().to_path_buf())
        else {
            self.batch
                .finish(id, &ItemResult::new(ItemStatus::Failed, None));
            return self.next_batch_item();
        };
        Task::future(async move {
            let result =
                tokio::task::spawn_blocking(move || operation.run(&path, &cancel, &progress))
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("batch operation task failed: {e}");
                        ItemResult::new(ItemStatus::Failed, None)
                    });
            Message::BatchItemDone {
                id,
                result: Box::new(result),
            }
        })
    }

    /// Take a finished file into the list (it may have been renamed), then start the next.
    fn batch_item_done(&mut self, id: FileId, result: Box<ItemResult>) -> Task<Message> {
        if let Some(dir) = self.directory.as_mut() {
            if let Some((path, snapshot)) = result.update.as_ref() {
                dir.rename_file(id, path, snapshot);
            }
            // Its subtitles marker, the "with subtitles" filter and its count, after a job
            // that may have written a `.srt`.
            let has_subtitles = dir.file_by_id(id).map(|f| {
                frename_core::existing_subtitle_path(&frename_core::FileTagger::disk_path(
                    f.file_path(),
                ))
                .is_some()
            });
            if let Some(has_subtitles) = has_subtitles {
                dir.set_has_subtitles(id, has_subtitles);
            }
        }
        self.batch.finish(id, &result);
        self.next_batch_item()
    }

    /// The job ended: reopen the selected file, and resume loading comments.
    fn batch_finished(&mut self) -> Task<Message> {
        // The job renamed files behind the history: undoing across it would use stale paths.
        self.history = WorkspaceHistory::new(HISTORY_DEPTH);
        // The job may have written subtitles: the plan is made again.
        let load_comments =
            Task::batch([self.load_next_comment_batch(false), self.subtitle_reads()]);
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

    /// Every queued deferred save, applied in order (there is normally one; see
    /// `pending_file_updates`'s doc comment for when a second can queue up).
    fn apply_pending_file_updates(&mut self) -> Task<Message> {
        let tasks: Vec<Task<Message>> = std::mem::take(&mut self.pending_file_updates)
            .into_iter()
            .map(|(id, snapshot)| self.apply_file_updated(id, snapshot))
            .collect();
        Task::batch(tasks)
    }

    /// Called when media has unloaded. Runs whichever action was waiting on it — a batch, a
    /// deferred close, a deferred folder scan, or a plain file-switch save — then, for the plain
    /// case, opens the newly selected file.
    fn on_media_unloaded(&mut self) -> Task<Message> {
        if std::mem::take(&mut self.batch_waits_for_unload) {
            return self.run_batch();
        }
        // Closing wins over any scan/switch also waiting on this same unload: just save, and
        // let the caller (CloseRequested's window::close) proceed. Reopening the video (the
        // plain branch below) or starting a scan would be wasted work on a window that is going
        // away, and racing a fresh GStreamer pipeline init against process shutdown besides.
        if std::mem::take(&mut self.closing) {
            self.pending_scan = None;
            return self.apply_pending_file_updates();
        }
        if let Some(pair) = self.pending_scan.take() {
            let saved = self.apply_pending_file_updates();
            return Task::batch([saved, self.scan_folder(pair)]);
        }
        if self.pending_file_updates.is_empty() {
            return Task::none();
        }
        // Save before opening media, not alongside it: when the selected file is the one being
        // saved (re-clicking it, renaming it in place), opening it first would lock it against
        // the rename, or point the player at the name it had before the save.
        let saved = self.apply_pending_file_updates();
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

        // Markers first, while the file still has the path they were read from; they travel
        // with it through the rename. Kept in the comment, they go back into it as lines.
        let saved_state = snapshot.clone();
        let mut snapshot = snapshot;
        let mut moved_markers = None;
        let markers_saved = match (snapshot.markers(), frename_core::marker_storage()) {
            // Shown from the video and not touched: they stay where they are.
            (Some(markers), frename_core::MarkerStorage::Comment)
                if self
                    .markers_in_video
                    .get(&id)
                    .is_some_and(|read| read.as_slice() == markers) =>
            {
                self.unsaved_markers.remove(&id);
                Task::none()
            }
            (Some(markers), frename_core::MarkerStorage::Comment) => {
                let wanted = markers.to_vec();
                let comment = frename_core::markers_into_comment(snapshot.comment(), markers);
                snapshot.set_comment(comment);
                moved_markers = Some(wanted);
                if self.markers_in_video.contains_key(&id) {
                    self.clear_from_video.insert(id);
                }
                Task::none()
            }
            (Some(markers), frename_core::MarkerStorage::InVideo) => {
                self.save_markers(id, &current_path, markers)
            }
            (None, _) => Task::none(),
        };
        let path_before = current_path.clone();
        let (new_path, snapshot_after_save) = snapshot.save_and_reparse(&current_path);
        // A drag out of the window waiting for this save learns whether it worked.
        self.drag_out_saved(id, &snapshot, &snapshot_after_save);
        // The save did not do what the tags/comment wanted: most likely a file with that name
        // already exists (issue #84), or the video is read-only or open elsewhere. The same
        // check the drag-out path already uses to detect this.
        let refused = drag_out::save_failed(&snapshot, &snapshot_after_save);
        let refused_notice = if refused {
            // The marker description's own reading of the clip may be what holds the file.
            if self.marker_requests > 0 {
                Self::notice(&fl!("folder-not-saved-marker-ai"))
            } else {
                Self::notice(
                    "Not saved: a file with that name already exists, or it is read-only or in use",
                )
            }
        } else {
            Task::none()
        };
        // Markers shown from the video are taken out of it only once the save shows the comment
        // holds all of them: a refused save, or a comment that did not get written, must not
        // leave them nowhere.
        let comment_holds_markers = moved_markers.as_ref().is_some_and(|wanted| {
            let lines = |markers: &[Marker]| {
                let mut lines: Vec<String> = markers
                    .iter()
                    .map(frename_core::format_marker_line)
                    .collect();
                lines.sort();
                lines
            };
            let saved = frename_core::markers_from_comment(snapshot_after_save.comment()).1;
            lines(wanted) == lines(&saved)
        });
        if comment_holds_markers {
            // Kept in the comment now: a failed write into the video no longer holds them.
            self.unsaved_markers.remove(&id);
            if !refused {
                self.move_markers_out_of_video(id, &new_path);
            }
        }
        // Applied and read back from the file: the journal's copy of these edits is no longer
        // needed. A refused save, markers that did not get written, or a comment or in/out
        // point that is not on disk keep it.
        if !refused
            && !self.unsaved_markers.contains_key(&id)
            && frename_core::recovery::saved_what_was_wanted(&snapshot, &snapshot_after_save)
        {
            self.journal_saved(id, &path_before, &new_path, &saved_state);
        }
        // A file menu action waiting for this save runs now, on the name the file has on disk.
        let file_action = self.file_action_after_save(id, refused);

        let _ = self
            .directory
            .as_mut()
            .map(|dir| dir.rename_file(id, &new_path, &snapshot_after_save));

        // Remember this as the folder's last viewed file (#98): every path here left a file that
        // was open, whether by switching to another one, closing frename, or switching folders.
        if let (Some(folder), Some(name)) = (
            self.directory.as_ref().map(|dir| dir.path().to_path_buf()),
            new_path.file_name().and_then(|n| n.to_str()),
        ) {
            FolderTagStore::set_last_viewed(&folder, name);
        }

        // Push NavigateFileCommand when we have a valid to_file_id (set by select_* after navigating).
        let target = self
            .navigation_targets
            .iter()
            .position(|(left, _)| *left == id)
            .map(|i| self.navigation_targets.remove(i).1);
        if let Some(to_file_id) = target {
            // A refused save never reached the file: undoing back to it must restore what is
            // really on disk (snapshot_after_save), not the picked-but-unsaved tags, or Ctrl+Z
            // would show them as if they had been saved.
            let snapshot_before = if refused {
                snapshot_after_save.clone()
            } else {
                snapshot
            };
            self.history.push(Box::new(NavigateFileCommand {
                file_id: id,
                to_file_id,
                path_before,
                path_after: new_path,
                snapshot_before,
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
            return Task::batch([
                markers_saved,
                refused_notice,
                file_action,
                Task::done(Message::ScrollFolderListToSelected),
            ]);
        }
        Task::batch([markers_saved, refused_notice, file_action])
    }

    fn handle_folder_message(&mut self, msg: folder::Message) -> Task<Message> {
        match msg {
            folder::Message::SelectFile(index) => self.select_file_on_click(index),
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
            folder::Message::OpenFolder => Task::done(Message::OpenFolderPicker),
            folder::Message::OpenFile => Task::done(Message::OpenFilePicker),
            folder::Message::ToggleRecentFolders => {
                self.handle_recent_folders(recent_folders::Message::Toggle)
            }
            folder::Message::SetUntaggedOnly(untagged_only) => {
                self.set_list_filter(|dir| dir.set_untagged_only(untagged_only))
            }
            folder::Message::SetSubtitledOnly(subtitled_only) => {
                self.set_list_filter(|dir| dir.set_subtitled_only(subtitled_only))
            }
            folder::Message::SetCommentedOnly(commented_only) => {
                self.set_list_filter(|dir| dir.set_commented_only(commented_only))
            }
            folder::Message::SetMarkedOnly(marked_only) => {
                self.set_list_filter(|dir| dir.set_marked_only(marked_only))
            }
            folder::Message::SetNameFilter(query) => self.set_file_name_filter(query),
            folder::Message::ToggleFilterMenu => {
                self.filter_menu_open = !self.filter_menu_open;
                Task::none()
            }
            folder::Message::CloseFilterMenu => {
                self.filter_menu_open = false;
                Task::none()
            }
            folder::Message::ShowAll => {
                self.filter_menu_open = false;
                Task::batch(
                    [
                        folder::Message::SetNameFilter(String::new()),
                        folder::Message::SetUntaggedOnly(false),
                        folder::Message::SetSubtitledOnly(false),
                        folder::Message::SetCommentedOnly(false),
                        folder::Message::SetMarkedOnly(false),
                    ]
                    .map(|m| Task::done(Message::Folder(m))),
                )
            }
            // Intercepted by the app, which owns the windows; no-op here.
            folder::Message::OpenSettings => Task::none(),
            folder::Message::StartRename(index) => self.start_rename(index),
            folder::Message::OpenFileMenu(index) => {
                let file = self
                    .directory
                    .as_ref()
                    .and_then(|dir| dir.files_in_order().nth(index))
                    .map(|file| file.id());
                file.map_or_else(Task::none, |id| {
                    Task::done(Message::FileMenu(file_menu::Message::Open(id)))
                })
            }
            folder::Message::RenameInput(text) => {
                if let Some(rename) = self.inline_rename.as_mut() {
                    rename.text = text;
                    rename.error = None;
                }
                Task::none()
            }
            folder::Message::SubmitRename => self.submit_rename(),
            folder::Message::SetBatchMode(on) => {
                // A running job ignores `SetActive` (see `BatchState::update`): the UI already
                // disables the ☑ button then, but keep the anchor too in case something else
                // sends this message while a job runs.
                if !on && !self.batch.is_running() {
                    self.select_anchor = None;
                }
                // The open file starts checked: it is usually the one to act on.
                let open = self
                    .directory
                    .as_ref()
                    .and_then(|d| d.selected_file())
                    .map(|f| f.id());
                let activated = self.handle_batch(batch::Message::SetActive(on));
                match open {
                    Some(id) if on => Task::batch([
                        activated,
                        self.handle_batch(batch::Message::CheckAll(vec![id])),
                    ]),
                    _ => activated,
                }
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

    /// Switch fullscreen on or off (only when a video is shown). The view builds the lists anew
    /// then, at offset 0, so the offsets they had are carried to a task that puts them back.
    fn set_fullscreen(&mut self, on: bool) -> Task<Message> {
        // Fullscreen shows no list to go with an open one.
        let _ = self.recent_folders.update(recent_folders::Message::Close);
        if on && !self.media_viewer.is_previewable() {
            return Task::none();
        }
        self.media_fullscreen = on;
        if !self.media_viewer.is_previewable() {
            return Task::none();
        }
        Task::done(self.restore_lists_message())
    }

    /// The task message that puts the lists back at the offsets they have now.
    fn restore_lists_message(&self) -> Message {
        Message::RestoreListScrolls {
            markers_y: self.markers.scroll_y(),
            cues_y: self.media_viewer.cue_scroll_y(),
        }
    }

    /// Open the in-place rename editor on the row at `index` (selecting that file first when
    /// needed), with the name before the extension selected, as Windows Explorer does.
    fn start_rename(&mut self, index: usize) -> Task<Message> {
        // The double-click's held button now belongs to the editor's text, not to a drag.
        self.drag_out.release();
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
        self.end_comment_session();
        self.file_workspace.set_comment_focused(false);
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

        // The markers stay in the tag list: `reinitialize_tags_from_snapshot` keeps them.
        let mut snapshot = FileSnapshot::parse(typed);
        snapshot.set_comment(current.comment().to_string());
        // In/out points are never part of the name: renaming keeps them.
        snapshot.set_segment(current.segment());
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
        self.history.push(Box::new(RenameFileCommand {
            snapshot_before: current,
            snapshot_after: snapshot.clone(),
        }));
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

    /// A file row was clicked: a plain click selects it alone and may start a drag out, as
    /// before; Ctrl+click or Shift+click instead build a multi-selection out of batch mode's
    /// checked set (see `ctrl_click_select`/`shift_click_select`). Either way the clicked file
    /// opens in the viewer, same as a plain click — it stays the file tags/comment edit.
    fn select_file_on_click(&mut self, index: usize) -> Task<Message> {
        if self.modifiers.shift() {
            return self.shift_click_select(index);
        }
        if self.modifiers.command() {
            return self.ctrl_click_select(index);
        }
        self.arm_drag_out(index);
        // The file open already stays as it is: opening it again reloads its video, so the two
        // clicks of a double-click (rename) would load it twice.
        if self.is_open_at(index) {
            return Task::none();
        }
        self.select_file_at(index)
    }

    /// Whether the file at `index` of the list is the one open in the file workspace.
    fn is_open_at(&self, index: usize) -> bool {
        let listed = self
            .directory
            .as_ref()
            .and_then(|dir| dir.files_in_order().nth(index))
            .map(|f| f.id());
        listed.is_some() && listed == self.file_workspace.file().map(|f| f.id())
    }

    /// Ctrl+click: with no multi-selection running yet, starts one with the previously open file
    /// plus the clicked one (just the clicked one when they are the same file); with one already
    /// running, toggles the clicked file in the checked set. Either way the clicked file becomes
    /// the anchor for the next Shift+click.
    fn ctrl_click_select(&mut self, index: usize) -> Task<Message> {
        let Some(dir) = self.directory.as_ref() else {
            return Task::none();
        };
        let Some(id) = dir.files_in_order().nth(index).map(|f| f.id()) else {
            return Task::none();
        };
        let previously_open = dir.selected_file().map(|f| f.id());
        let open = self.select_file_at(index);
        let check = if self.batch.is_active() {
            self.handle_batch(batch::Message::Toggle(id))
        } else {
            let ids: Vec<FileId> = previously_open.into_iter().chain([id]).collect();
            self.handle_batch_many([
                batch::Message::SetActive(true),
                batch::Message::CheckAll(ids),
            ])
        };
        self.select_anchor = Some(id);
        Task::batch([open, check])
    }

    /// Shift+click: checks every file from the anchor (the last Ctrl-clicked file, or, when
    /// multi-selection has not started yet, the file open before this click) to the clicked one,
    /// inclusive, in the list's current order — replacing whatever was checked before. Does not
    /// move the anchor, so repeated Shift+clicks keep ranging from the same file.
    fn shift_click_select(&mut self, index: usize) -> Task<Message> {
        let Some(dir) = self.directory.as_ref() else {
            return Task::none();
        };
        let listed: Vec<FileId> = dir.files_in_order().map(|f| f.id()).collect();
        let Some(&clicked_id) = listed.get(index) else {
            return Task::none();
        };
        let anchor = self
            .select_anchor
            .or_else(|| dir.selected_file().map(|f| f.id()));
        let range: Vec<FileId> = match anchor.and_then(|a| listed.iter().position(|id| *id == a)) {
            // The anchor may no longer be listed under the current filter: fall back to just
            // the clicked file rather than ranging from a position that no longer means it.
            Some(anchor_index) => {
                let (lo, hi) = if anchor_index <= index {
                    (anchor_index, index)
                } else {
                    (index, anchor_index)
                };
                listed[lo..=hi].to_vec()
            }
            None => vec![clicked_id],
        };
        if self.select_anchor.is_none() {
            self.select_anchor = anchor.or(Some(clicked_id));
        }
        let open = self.select_file_at(index);
        let check = self.handle_batch_many([
            batch::Message::SetActive(true),
            batch::Message::CheckNone,
            batch::Message::CheckAll(range),
        ]);
        Task::batch([open, check])
    }

    /// Apply several batch messages as one step, so the reads they trigger (AI/subtitle plans)
    /// run once for the result instead of once per message. `Run`/`Retry`/`OpenLog`/`OpenBilling`
    /// are `handle_batch`'s own special cases (starting a job, opening a file); never pass them
    /// here, or they reach `BatchState::update`, which does not handle them.
    fn handle_batch_many(
        &mut self,
        msgs: impl IntoIterator<Item = batch::Message>,
    ) -> Task<Message> {
        // Batch mode does not edit the open file, so its rename editor goes.
        self.inline_rename = None;
        for msg in msgs {
            self.batch.update(msg);
        }
        self.share_folder_tags();
        Task::batch([self.describe_ai_reads(), self.subtitle_reads()])
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
        if let file_name_panel::Message::OpenFileMenu = msg {
            return self.file_workspace.file().map_or_else(Task::none, |file| {
                Task::done(Message::FileMenu(file_menu::Message::Open(file.id())))
            });
        }
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
        if let file_name_panel::Message::ApplySuggestedInOut = msg {
            return self.apply_suggested_in_out();
        }
        let suggested = file_name_panel::pending_tag_suggestions(self.file_workspace.tag_list());
        let add = match &msg {
            file_name_panel::Message::AddSuggestedTag(name) => Some(vec![name.clone()]),
            file_name_panel::Message::AddFirstSuggestedTag => {
                Some(suggested.iter().take(1).map(|t| t.name.clone()).collect())
            }
            file_name_panel::Message::AddAllSuggestedTags => {
                Some(suggested.iter().map(|t| t.name.clone()).collect())
            }
            _ => None,
        };
        if let Some(names) = add {
            return self.add_suggested_tags(&names);
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

    /// The comment box lost the keys (or the file is about to change): what was typed in it
    /// since it got them is one undo step.
    fn end_comment_session(&mut self) {
        let Some(snapshot_before) = self.comment_session.take() else {
            return;
        };
        let Some((_, snapshot_after)) = self.file_workspace.get_snapshot() else {
            return;
        };
        if snapshot_before.comment() != snapshot_after.comment() {
            self.history.push(Box::new(SetCommentCommand {
                snapshot_before,
                snapshot_after,
            }));
        }
    }

    fn handle_sync_panel(&mut self, msg: sync_panel::Message) -> Task<Message> {
        let before = self.file_workspace.tag_list().order_state();
        let mut changed = true;
        match msg {
            sync_panel::Message::SyncUp => {
                if let Err(e) = self.file_workspace.sync_selected_to_display() {
                    log::error!("SyncUp failed: {}", e);
                    changed = false;
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
        let after = self.file_workspace.tag_list().order_state();
        if changed && !before.same_as(&after) {
            self.history
                .push(Box::new(SyncTagOrderCommand { before, after }));
        }
        Task::none()
    }

    fn perform_undo(&mut self) -> Task<Message> {
        self.end_comment_session();
        if !self.history.can_undo() || self.directory.is_none() {
            return Task::none();
        }
        let turns_a_video = self.history.undo_turns_a_video();
        let switches_file = self.history.undo_switches_file();
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
        self.after_undo_redo(
            result,
            turns_a_video,
            switches_file.then(|| fl!("undo-back-on-clip")),
        )
    }

    fn perform_redo(&mut self) -> Task<Message> {
        if !self.history.can_redo() || self.directory.is_none() {
            return Task::none();
        }
        let turns_a_video = self.history.redo_turns_a_video();
        let switches_file = self.history.redo_switches_file();
        let result: Result<(), UndoError> = {
            let dir = self.directory.as_mut().expect("checked above");
            let tl = self.file_workspace.tag_list_mut();
            let mut ctx = UndoContext {
                directory: dir,
                tag_list: tl,
            };
            self.history.redo(&mut ctx)
        };
        self.after_undo_redo(
            result,
            turns_a_video,
            switches_file.then(|| fl!("redo-on-clip")),
        )
    }

    /// Refresh after an undo or redo step, or say why it failed. A step that turned a video
    /// reopens it when it is the one shown. A step that moved to another file says so with
    /// `file_notice`, so a stack shared by all files does not look like an undo that did nothing.
    fn after_undo_redo(
        &mut self,
        result: Result<(), UndoError>,
        turns_a_video: bool,
        file_notice: Option<String>,
    ) -> Task<Message> {
        self.file_workspace.sync_comment_editor();
        let comment_write = if result.is_ok() {
            self.write_markers_to_comment_now()
        } else {
            Task::none()
        };
        let task = match result {
            Ok(()) if file_notice.is_some() => Task::batch([
                self.refresh_after_undo_redo(),
                Self::notice(&file_notice.unwrap_or_default()),
            ]),
            // A turn changes no tags, name or selection, so the open file needs no refresh; one
            // would unload the video to save it while the reopen below is still opening it.
            Ok(()) if turns_a_video => self.follow_rotation(),
            Ok(()) => self.refresh_after_undo_redo(),
            Err(e) => {
                log::warn!("Undo/redo failed: {}", e);
                Self::undo_failed_notice(&e)
            }
        };
        Task::batch([task, comment_write])
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

    /// Up: one row up in the grid as it is drawn (its groups start rows of their own); the top
    /// row wraps to the last.
    fn move_selection_up(&mut self) {
        self.move_selection_by_row(Shape::up);
    }

    /// Down: one row down; the last row wraps to the top.
    fn move_selection_down(&mut self) {
        self.move_selection_by_row(Shape::down);
    }

    /// Move the tag cursor to the tag `step` finds from it in the grid's shape; with no cursor,
    /// to the first tag.
    fn move_selection_by_row(&mut self, step: fn(&Shape, usize) -> Option<usize>) {
        let tag_list = self.file_workspace.tag_list();
        let filtered = tag_list.filtered_display_tag_ids().to_vec();
        if filtered.is_empty() {
            self.tag_panel.set_selected(None);
            return;
        }
        let shape = Shape::of(tag_list, self.tag_panel.cols() as usize);
        let cur = self
            .tag_panel
            .selected_tag_id()
            .and_then(|id| filtered.iter().position(|&fid| fid == id));
        let new_i = cur.and_then(|i| step(&shape, i)).unwrap_or(0);
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
        let shape = Shape::of(tag_list, self.tag_panel.cols() as usize);
        let row_extent = self
            .tag_panel
            .row_content_height()
            .unwrap_or(self.tag_panel.row_height());
        let row_top = shape.row_top(shape.position(flat_index).0);
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
        // The spinner only turns while something waits (rows loading their comments, a folder
        // opening, a job's file in work, a marker being described), so an idle list does not
        // redraw.
        let loading_comments = self
            .directory
            .as_ref()
            .is_some_and(|d| d.loading_comment_count() > 0);
        let spinner = if loading_comments
            || self.loading
            || self.batch.is_running()
            || self.markers.any_describing()
            || self.batch.is_waiting_for_markers()
        {
            iced::time::every(crate::ui::tokens::SPINNER_TICK).map(|_| Message::SpinnerTick)
        } else {
            Subscription::none()
        };
        // A running AI job tells how far its file is from another thread: redraw to show it.
        let job_progress =
            if self.batch.is_running() && self.batch.action() == batch::Action::DescribeAi {
                iced::time::every(std::time::Duration::from_millis(200)).map(|_| Message::Noop)
            } else {
                Subscription::none()
            };
        // The recovery journal follows the open clip's edits, a second after they change.
        let journal = if self.file_workspace.file().is_some() {
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::JournalTick)
        } else {
            Subscription::none()
        };
        Subscription::batch([
            journal,
            self.media_viewer.subscription().map(Message::MediaViewer),
            iced::event::listen_with(focus_may_move),
            job_progress,
            self.file_name_panel
                .subscription()
                .map(Message::FileNamePanel),
            self.tag_panel.subscription().map(Message::TagPanel),
            spinner,
            self.drag_out.subscription().map(Message::DragOut),
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
    /// Whether the file list's filter menu is open.
    pub fn filter_menu_open(&self) -> bool {
        self.filter_menu_open
    }

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

    /// Save the open file's live edits before the window closes, so tags, comment and in/out
    /// set on the last clip of the session are not lost when the user closes frename without
    /// switching files or folders first. If the video needs unloading before the resulting
    /// rename, the save is deferred: the caller must still trigger the unload (`needs_media_unload`
    /// says whether to) and `on_media_unloaded` flushes it once that finishes.
    pub fn flush_open_file(&mut self) -> Task<Message> {
        // Set regardless of whether there is anything new to save below: an earlier deferred
        // save (from a file switch that hasn't unloaded yet) must not reopen the video once
        // `on_media_unloaded` runs — the window is closing.
        self.closing = true;
        let Some((id, snap)) = self.file_workspace.get_snapshot() else {
            return Task::none();
        };
        if self.media_viewer.needs_unload_before_rename() {
            self.pending_file_updates.push((id, snap));
            Task::none()
        } else {
            self.apply_file_updated(id, snap)
        }
    }

    /// Test helper: inject a pending deferred rename as if media is locked.
    /// Only available in test builds.
    #[cfg(test)]
    pub fn inject_pending_rename(&mut self, id: FileId, snapshot: FileSnapshot) {
        self.pending_file_updates.push((id, snapshot));
    }

    /// Test helper: check whether a deferred rename is pending.
    #[cfg(test)]
    pub fn has_pending_rename(&self) -> bool {
        !self.pending_file_updates.is_empty()
    }

    /// Test helper: the media viewer, to stand in for a real video.
    #[cfg(test)]
    pub fn media_viewer_mut(&mut self) -> &mut MediaViewerState {
        &mut self.media_viewer
    }

    /// State of the marker list (open row, color picker).
    pub fn markers(&self) -> &MarkersState {
        &self.markers
    }

    /// Markers whose write failed, by file; the folder list marks those files.
    pub fn unsaved_markers(&self) -> &HashMap<FileId, Vec<Marker>> {
        &self.unsaved_markers
    }

    pub fn tag_panel(&self) -> &TagPanelState {
        &self.tag_panel
    }

    pub fn file_name_panel(&self) -> &FileNamePanelState {
        &self.file_name_panel
    }

    /// The folders opened recently.
    pub fn recent_folders(&self) -> &RecentFoldersState {
        &self.recent_folders
    }

    /// The file context menu.
    pub fn file_menu(&self) -> &FileMenuState {
        &self.file_menu
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
) -> Result<(), folder::RenameProblem> {
    if typed.is_empty() {
        return Err(folder::RenameProblem::Empty);
    }
    if typed.chars().any(|c| {
        matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control()
    }) {
        return Err(folder::RenameProblem::BadCharacter);
    }
    if typed.ends_with('.') || typed.ends_with(' ') {
        return Err(folder::RenameProblem::Trailing);
    }
    // A change of case only is the same file on Windows, not a clash.
    if !typed.eq_ignore_ascii_case(current) && folder.join(typed).exists() {
        return Err(folder::RenameProblem::Exists);
    }
    Ok(())
}

/// Open `target` with the app the system uses for it (a text editor for the log, the browser
/// for a web address).
fn open_in_default_app(target: impl AsRef<std::ffi::OsStr>) {
    let target = target.as_ref();
    #[cfg(windows)]
    let result = std::process::Command::new("explorer").arg(target).spawn();
    #[cfg(not(windows))]
    let result = std::process::Command::new("xdg-open").arg(target).spawn();
    if let Err(e) = result {
        log::warn!("could not open {target:?}: {e}");
    }
}

/// A mouse press, Tab or Esc may move the keys into or out of the comment box: ask where they
/// are then, so the box's edge shows its focus (the edge is drawn outside the editor, which
/// scrolls inside it).
fn focus_may_move(
    event: iced::Event,
    _status: iced::event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    use iced::keyboard::{key::Named, Key};
    match event {
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(_)) => {
            Some(Message::CheckCommentFocus)
        }
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: Key::Named(Named::Tab | Named::Escape),
            ..
        }) => Some(Message::CheckCommentFocus),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::SystemTime;

    use frename_core::{
        AppDatabase, AppStateStore, File, FileId, FileSnapshot, FolderTagStore, Initializable,
        LoggingAppStateStore,
    };
    use iced::keyboard::Modifiers;

    use super::{Directory, FolderWorkspace, ItemResult, ItemStatus, Message};
    use crate::features::{batch, folder, recent_folders, tag_panel};

    #[test]
    fn saved_column_widths_are_kept_within_their_limits() {
        use crate::ui::tokens::{FILE_LIST_MAX_WIDTH, FILE_LIST_MIN_WIDTH, VIDEO_MIN_WIDTH};
        assert_eq!(
            super::column_widths(150.0, 120.0),
            (VIDEO_MIN_WIDTH, FILE_LIST_MIN_WIDTH)
        );
        assert_eq!(
            super::column_widths(500.0, 900.0),
            (500.0, FILE_LIST_MAX_WIDTH)
        );
        assert_eq!(super::column_widths(640.0, 300.0), (640.0, 300.0));
    }

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
        /// The folder's own database, closed when the test ends; `None` when it shares the app
        /// database.
        own_db: Option<AppDatabase>,
    }

    impl TestDirectory {
        /// The folder's store is its own database file inside the test folder, so tests running
        /// at the same time never share session or tag-store rows (#107).
        pub fn new(file_count: usize) -> Self {
            let path = unique_test_folder();
            let db = AppDatabase::with_path(path.join("app.db"));
            Self::with_store(file_count, db.clone(), path, Some(db))
        }

        /// Like [`TestDirectory::new`], but the folder's store is the app database the player
        /// writes playback positions to, for a test that needs a rename to move one.
        pub fn sharing_app_database(file_count: usize) -> Self {
            Self::with_store(file_count, AppDatabase::new(), unique_test_folder(), None)
        }

        fn with_store(
            file_count: usize,
            db: AppDatabase,
            path: PathBuf,
            own_db: Option<AppDatabase>,
        ) -> Self {
            // Code under test also writes to the app database (playback positions, panel
            // widths); migrate it as `main` does at startup.
            LoggingAppStateStore::new(AppDatabase::new())
                .initialize()
                .unwrap();
            std::fs::create_dir_all(&path).expect("create test folder");
            let files: Vec<File> = (0..file_count)
                .map(|i| {
                    File::from_path(path.join(format!("file_{}.mp4", i)), SystemTime::UNIX_EPOCH)
                })
                .collect();
            let store = LoggingAppStateStore::new(db);
            store.initialize().unwrap();
            let directory = Directory::with_files(&path, files, store);
            Self {
                directory,
                path,
                own_db,
            }
        }

        /// The scanned directory, ready to hand to `Message::FolderLoaded`.
        pub fn directory(&self) -> Directory {
            self.directory.clone()
        }

        /// The file the workspace should open once the folder is loaded.
        pub fn target_file(&self) -> PathBuf {
            self.path.join("file_0.mp4")
        }

        /// The folder itself, for reading or seeding its `.frename` file directly.
        pub fn path(&self) -> &Path {
            &self.path
        }

        /// Path of a file in this folder, for asserting on renames.
        pub fn file_path(&self, name: &str) -> PathBuf {
            self.path.join(name)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            // Closed first, so SQLite lets go of the files and the folder can be removed whole.
            if let Some(db) = &self.own_db {
                db.close();
            }
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    /// A workspace with an app database of its own, so tests running at the same time never
    /// share the last batch run, the panel widths or the recent folders, and none of them reads
    /// or writes the developer's `frename.db`.
    fn test_workspace() -> FolderWorkspace {
        FolderWorkspace::with_app_db(fresh_app_db())
    }

    /// A migrated app database in a folder of its own (swept like the test folders).
    fn fresh_app_db() -> AppDatabase {
        let dir = unique_test_folder();
        std::fs::create_dir_all(&dir).expect("create test folder");
        let db = AppDatabase::with_path(dir.join("workspace.db"));
        db.initialize().expect("migrate");
        db
    }

    /// A folder name no other test, and no concurrent test run, will pick.
    fn unique_test_folder() -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        static SWEPT: std::sync::Once = std::sync::Once::new();
        SWEPT.call_once(sweep_old_test_folders);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("frename-test-{}-{n}", std::process::id()))
    }

    /// Removes `frename-test-*` leftovers of earlier test runs. Only ones untouched for an hour:
    /// another test run may be using newer ones right now.
    fn sweep_old_test_folders() {
        let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
            return;
        };
        let hour_ago = SystemTime::now() - std::time::Duration::from_secs(60 * 60);
        for entry in entries.flatten() {
            let is_ours = entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("frename-test-"));
            let old = entry
                .metadata()
                .and_then(|m| m.modified())
                .is_ok_and(|modified| modified < hour_ago);
            if is_ours && old {
                let path = entry.path();
                let _ = std::fs::remove_dir_all(&path).or_else(|_| std::fs::remove_file(&path));
            }
        }
    }

    /// Issue #107: a test's folder, its own database included, is gone when the test ends.
    #[test]
    fn a_test_folder_leaves_nothing_behind() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let path = test_dir.path().to_path_buf();
        assert!(path.join("app.db").exists(), "the folder's own database");

        drop(workspace);
        drop(test_dir);

        assert!(!path.exists(), "nothing left in temp");
    }

    #[test]
    fn open_folder_with_two_files_shows_two_in_folder_panel() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
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

    /// Issue #98: opening a folder without a specific target file returns to the clip it last
    /// had open, stored in its own `.frename` file.
    #[test]
    fn opening_a_folder_with_no_target_selects_the_remembered_file() {
        let test_dir = TestDirectory::new(2);
        FolderTagStore::set_last_viewed(test_dir.path(), "file_1.mp4");
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: None,
        });
        flush_file_opened(&mut workspace);
        assert_eq!(
            workspace
                .file_workspace()
                .file()
                .and_then(|f| f.file_path().file_name())
                .and_then(|n| n.to_str()),
            Some("file_1.mp4")
        );
    }

    /// Nothing remembered yet (a folder never opened before, or opened only before #98): the
    /// first file in the list opens, same as today's other folders.
    #[test]
    fn opening_a_folder_with_no_target_and_nothing_remembered_opens_the_first_file() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: None,
        });
        assert_eq!(
            workspace.directory().and_then(|d| d.selected_index()),
            Some(0)
        );
    }

    /// A specific file (a dropped file, "Open with") always wins over the remembered one.
    #[test]
    fn opening_a_specific_file_wins_over_the_remembered_one() {
        let test_dir = TestDirectory::new(2);
        FolderTagStore::set_last_viewed(test_dir.path(), "file_1.mp4");
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()), // file_0.mp4
        });
        assert_eq!(
            workspace.directory().and_then(|d| d.selected_index()),
            Some(0)
        );
    }

    /// A specific target file that is gone (e.g. renamed since, including in the app-wide last
    /// session) falls back the same way an unset target does: by name without tags, then to the
    /// first file, rather than opening nothing.
    #[test]
    fn a_gone_specific_target_falls_back_like_an_unset_one() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.file_path("gone.mp4")),
        });
        assert_eq!(
            workspace.directory().and_then(|d| d.selected_index()),
            Some(0),
            "falls back to the first file"
        );
    }

    /// Leaving a file (switching to another one) remembers it as this folder's last viewed file.
    #[test]
    fn leaving_a_file_remembers_it_as_the_folder_s_last_viewed_file() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()), // file_0.mp4
        });
        flush_file_opened(&mut workspace);
        assert_eq!(
            FolderTagStore::get_last_viewed(test_dir.path()),
            "",
            "nothing remembered yet"
        );
        let file_0_id = file_id_at(&workspace, 0);

        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        // Manually drive the FileUpdated that would fire from the iced runtime.
        let snapshot = FileSnapshot::new(Vec::<String>::new(), "file_0", ".mp4", "file_0.mp4");
        let _ = workspace.update(Message::FileUpdated {
            id: file_0_id,
            snapshot,
        });

        assert_eq!(
            FolderTagStore::get_last_viewed(test_dir.path()),
            "file_0.mp4"
        );
    }

    /// Issue #161: a clip left at 40 s and opened again looks up where it stopped, to continue
    /// there once it is open; one never played has nothing to continue.
    #[test]
    fn a_clip_opened_again_continues_where_playback_stopped() {
        use frename_core::AppStateStore;
        use std::time::Duration;
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()), // file_0.mp4
        });
        flush_file_opened(&mut workspace);
        assert_eq!(
            workspace.media_viewer().video_resume_lookup(),
            Some(None),
            "never played"
        );
        workspace
            .media_viewer_mut()
            .pretend_video_shown_at(Duration::from_secs(40));

        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        assert_eq!(
            AppDatabase::new().get_playback_position(&test_dir.file_path("file_0.mp4")),
            Some(Duration::from_secs(40))
        );
        // file_1 played for a moment only: nothing worth continuing in it.
        workspace
            .media_viewer_mut()
            .pretend_video_shown_at(Duration::from_secs(1));

        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        flush_file_opened(&mut workspace);
        assert_eq!(
            workspace.media_viewer().video_resume_lookup(),
            Some(Some(Duration::from_secs(40)))
        );
        assert_eq!(
            AppDatabase::new().get_playback_position(&test_dir.file_path("file_1.mp4")),
            None
        );
    }

    /// Issue #161: the open clip saved in place under a new name (a tag toggled, its save
    /// waiting for the video to unload) reopens at the very moment it showed: no lookup, no
    /// 2 s lead, no note, and the moment kept follows the rename.
    #[test]
    fn the_open_clip_saved_under_a_new_name_reopens_at_the_same_moment() {
        use frename_core::AppStateStore;
        use std::time::Duration;
        let test_dir = TestDirectory::sharing_app_database(1);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        workspace
            .media_viewer_mut()
            .pretend_video_shown_at(Duration::from_secs(40));
        let id = file_id_at(&workspace, 0);
        let tagged = FileSnapshot::new(vec!["pick".to_string()], "file_0", ".mp4", "file_0.mp4");
        workspace.inject_pending_rename(id, tagged);

        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unload,
        ));
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));

        let renamed = test_dir.file_path("pick.file_0.mp4");
        assert_eq!(
            workspace
                .directory()
                .and_then(|d| d.file_by_id(id))
                .map(|f| f.file_path().to_path_buf()),
            Some(renamed.clone()),
            "saved under its new name"
        );
        assert_eq!(
            workspace.media_viewer().video_resume_lookup(),
            None,
            "no lookup"
        );
        assert_eq!(
            workspace.media_viewer().video_reopens_at(),
            Some(Duration::from_secs(40))
        );
        assert_eq!(workspace.media_viewer().notice(), None);
        assert_eq!(
            AppDatabase::new().get_playback_position(&renamed),
            Some(Duration::from_secs(40)),
            "the kept moment followed the rename"
        );
    }

    /// Issue #161: `Home` acts only on a video shown; with nothing shown it does nothing.
    #[test]
    fn home_acts_only_on_a_video_shown() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = test_workspace();
        assert_eq!(
            workspace.update(Message::GoToStart).units(),
            0,
            "nothing shown"
        );
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        assert_eq!(workspace.update(Message::GoToStart).units(), 1);
    }

    /// Issue #161 end to end through the workspace, on a real clip: a clip left at 10 s opens
    /// at 8 s once its video has loaded, with the note; `Home` goes to the start.
    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn a_clip_opens_where_playback_stopped_once_its_video_loads() {
        use crate::features::media_viewer::{self, video};
        use frename_core::AppStateStore;
        use std::time::Duration;
        let test_dir = TestDirectory::new(1);
        AppDatabase::new().set_playback_position(&test_dir.target_file(), Duration::from_secs(10));
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);

        let (clip, _) = video::real_clip();
        let generation = workspace.media_viewer().video_loads_started();
        let _ = workspace.update(Message::MediaViewer(media_viewer::Message::Video(
            video::video_loaded(clip, generation),
        )));
        assert_eq!(workspace.media_viewer().video_position_ms(), Some(8_000));
        assert!(workspace.media_viewer().resume_note_shown());

        // The task of `Home` is one message to the video; a test cannot run a task, so it is
        // delivered here.
        assert_eq!(workspace.update(Message::GoToStart).units(), 1);
        let _ = workspace.update(Message::MediaViewer(media_viewer::Message::Video(
            video::Message::GoToStart,
        )));
        assert_eq!(workspace.media_viewer().video_position_ms(), Some(0));
        assert_eq!(workspace.media_viewer().notice(), None);
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
    fn a_click_on_the_open_file_does_not_open_it_again() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let open = workspace
            .directory()
            .and_then(|d| d.selected_index())
            .expect("a file is open");
        assert!(workspace.is_open_at(open));
        assert!(!workspace.is_open_at(1 - open));
    }

    #[test]
    fn tags_are_saved_after_selecting_another_file() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
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

    #[test]
    fn pasting_tags_keeps_the_in_out_points() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::SegmentStartMarked(3.0),
        ));
        let _ = workspace.update(Message::CopyTags);
        let _ = workspace.update(Message::PasteTags);
        assert_eq!(
            workspace.file_workspace().segment_start_secs(),
            Some(3.0),
            "in/out points are not tags: a paste keeps them"
        );
    }

    /// Issue #173: the In/Out the AI suggests is set only when the editor applies it, as one
    /// undo step, and is no longer offered once the points match it.
    #[test]
    fn the_ai_suggested_tags_are_added_only_on_request_and_undone_in_one_step() {
        use crate::features::file_name_panel::{self as panel, Message as P};
        let test_dir = TestDirectory::new(1);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let list = workspace.file_workspace.tag_list_mut();
        for name in ["night", "beach"] {
            // A saved tag of the folder, not on this clip.
            list.set_checked_by_name(name, true);
            list.set_checked_by_name(name, false);
        }
        list.set_comment(
            "Mine\n\nAI: A walk.\nSuggested tags: night (90%) · beach (70%) · gone (50%)\n\
             Tag ideas: kayak race · beach"
                .to_string(),
        );
        let checked = |w: &FolderWorkspace, name: &str| {
            let list = w.file_workspace().tag_list();
            list.tag_id_by_name(name)
                .and_then(|id| list.get_tag(id))
                .is_some_and(|tag| tag.is_checked())
        };
        let offered = |w: &FolderWorkspace| {
            panel::pending_tag_suggestions(w.file_workspace().tag_list())
                .into_iter()
                .map(|tag| tag.name)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            offered(&workspace),
            ["night", "beach"],
            "only the folder's tags, most likely first"
        );
        assert_eq!(
            panel::tag_ideas(workspace.file_workspace().tag_list()),
            ["kayak race"],
            "an idea the folder has as a tag is no idea any more"
        );
        assert!(!checked(&workspace, "night"), "nothing is added by itself");

        let _ = workspace.update(Message::FileNamePanel(P::AddFirstSuggestedTag));
        assert!(checked(&workspace, "night") && !checked(&workspace, "beach"));
        assert_eq!(
            offered(&workspace),
            ["beach"],
            "an added tag is not offered"
        );
        let _ = workspace.update(Message::Undo);
        assert!(!checked(&workspace, "night"));

        let _ = workspace.update(Message::FileNamePanel(P::AddAllSuggestedTags));
        assert!(checked(&workspace, "night") && checked(&workspace, "beach"));
        assert!(offered(&workspace).is_empty());
        let _ = workspace.update(Message::Undo);
        assert!(
            !checked(&workspace, "night") && !checked(&workspace, "beach"),
            "one undo takes both back"
        );
        let _ = workspace.update(Message::Redo);
        assert!(checked(&workspace, "night") && checked(&workspace, "beach"));
        let _ = workspace.update(Message::Undo);

        let _ = workspace.update(Message::FileNamePanel(P::AddSuggestedTag(
            "gone".to_string(),
        )));
        assert!(
            workspace
                .file_workspace()
                .tag_list()
                .tag_id_by_name("gone")
                .is_none(),
            "a tag the folder does not have is never made"
        );

        let _ = workspace.update(Message::Batch(batch::Message::SetActive(true)));
        let _ = workspace.update(Message::FileNamePanel(P::AddAllSuggestedTags));
        assert!(
            !checked(&workspace, "night"),
            "batch mode does not edit the open file"
        );
    }

    #[test]
    fn the_ai_suggested_in_out_is_applied_as_one_undo_step() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        workspace
            .file_workspace
            .tag_list_mut()
            .set_comment("AI: A walk.\nSuggested In/Out: 00:00:03.200 – 00:00:11.800".to_string());
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::SegmentStartMarked(1.0),
        ));
        let points = |w: &FolderWorkspace| {
            (
                w.file_workspace().segment_start_secs(),
                w.file_workspace().segment_end_secs(),
            )
        };
        assert_eq!(
            points(&workspace),
            (Some(1.0), None),
            "nothing set by itself"
        );
        assert!(crate::features::file_name_panel::suggested_in_out(
            workspace.file_workspace().tag_list()
        )
        .is_some());

        let _ = workspace.update(Message::FileNamePanel(
            crate::features::file_name_panel::Message::ApplySuggestedInOut,
        ));
        assert_eq!(points(&workspace), (Some(3.0), Some(12.0)));
        assert!(
            crate::features::file_name_panel::suggested_in_out(
                workspace.file_workspace().tag_list()
            )
            .is_none(),
            "applied: nothing left to offer"
        );

        let _ = workspace.update(Message::Undo);
        assert_eq!(
            points(&workspace),
            (Some(1.0), None),
            "one undo brings back both points"
        );
        let _ = workspace.update(Message::Redo);
        assert_eq!(points(&workspace), (Some(3.0), Some(12.0)));
    }

    /// Issue #83: pasting tags built a fresh snapshot and dropped the open file's comment,
    /// which then looked wiped in the comment box and the "commented" filter until the next save.
    #[test]
    fn pasting_tags_keeps_the_comment() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        workspace
            .file_workspace
            .tag_list_mut()
            .set_comment("keep this note".to_string());
        let _ = workspace.update(Message::CopyTags);
        let _ = workspace.update(Message::PasteTags);
        assert_eq!(
            workspace
                .file_workspace()
                .get_snapshot()
                .unwrap()
                .1
                .comment(),
            "keep this note",
            "the comment is not a tag: a paste keeps it"
        );
    }

    /// When a video file is "loading" its GStreamer pipeline holds a file handle, so rename
    /// must be deferred.  Opening an .mp4 sets `video.loading = true`, which makes
    /// `needs_unload_before_rename()` return true on the next file switch — the rename is
    /// stored in `pending_file_updates` and only fires after `MediaViewer::Unloaded`.
    #[test]
    fn deferred_rename_fires_after_media_unloaded() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
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

        let mut workspace = test_workspace();
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

    /// Regression test for issue #21: closing the window used to only unload the video and
    /// never save the open file's pending tags/comment/in-out, so the last clip of a session
    /// silently lost them. `flush_open_file` must capture them for save before the app decides
    /// whether to unload and close.
    #[test]
    fn closing_the_window_saves_the_open_files_pending_edits() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let file_0_id = file_id_at(&workspace, 0);
        // file_0.mp4 is "loading" → needs_unload_before_rename() = true, same as a real open file.

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

        // What the app does on CloseRequested, before deciding whether to unload and close.
        let _ = workspace.flush_open_file();
        assert!(
            workspace.has_pending_rename(),
            "the open file's edit must be captured for save, not dropped, when closing"
        );

        // on_media_unloaded (fired for real on MediaViewer::Unloaded) applies the save.
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));

        assert!(
            workspace
                .directory()
                .and_then(|d| d.file_by_id(file_0_id))
                .is_some_and(|f| f.snapshot().has_tag("pick")),
            "the tag toggled just before closing must be saved, not lost"
        );
    }

    /// Regression test for issue #21: opening a different folder without switching files first
    /// used to reset the file workspace and drop `pending_file_updates`, losing the open file's
    /// edits. `begin_scan_folder` must save them (deferred past a needed unload, same as a file
    /// switch) before the scan replaces the directory.
    #[test]
    fn opening_a_new_folder_saves_the_open_files_pending_edits() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let file_0_id = file_id_at(&workspace, 0);

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

        // Open a different folder directly (📂, drag-drop, `frename <folder>`, …) without
        // switching files first. The scan itself never resolves in this test (no async
        // executor), so `workspace.directory()` still refers to the original folder below —
        // exactly what lets us check the save happened before it would be replaced.
        let other_folder = std::env::temp_dir().join("frename-test-other-folder-issue-21");
        let _ = workspace.update(Message::ScanFolder(frename_core::FolderAndFile::new(
            other_folder,
            None::<PathBuf>,
        )));
        assert!(
            workspace.has_pending_rename(),
            "the open file's edit must be captured before the scan replaces the directory"
        );

        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));

        assert!(
            workspace
                .directory()
                .and_then(|d| d.file_by_id(file_0_id))
                .is_some_and(|f| f.snapshot().has_tag("pick")),
            "the tag toggled just before opening another folder must be saved, not lost"
        );
    }

    /// Regression test: a save deferred by a file switch (file_0's rename, waiting for its
    /// video to unload) used to be silently overwritten if closing (or opening another folder)
    /// captured and queued a second save before that same `Unloaded` fired — `flush_open_file`
    /// and `save_then_scan` used to assign `pending_file_updates` directly instead of pushing,
    /// so only the second, later entry survived. Both entries must now be applied once
    /// `Unloaded` fires, in the order they were queued.
    #[test]
    fn a_second_deferred_save_does_not_drop_an_earlier_one() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let file_0_id = file_id_at(&workspace, 0);
        // file_0.mp4 is "loading" → needs_unload_before_rename() = true.

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

        // Switch to file_1: file_0's rename (with "pick") is deferred into pending_file_updates,
        // since its video is still "loading". Its own Unload is dispatched but not driven here —
        // media_viewer still reflects file_0's video, so needs_unload_before_rename() stays true.
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        assert!(
            workspace.has_pending_rename(),
            "file_0's rename must be deferred while its video is still loading"
        );

        // Close right now, before that Unload's Unloaded ever fires (a fast double-action, or
        // simply a user who closes immediately after clicking another file).
        let _ = workspace.flush_open_file();

        // The single Unloaded that eventually fires must apply both queued saves, not just the
        // second one.
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));

        assert!(
            workspace
                .directory()
                .and_then(|d| d.file_by_id(file_0_id))
                .is_some_and(|f| f.snapshot().has_tag("pick")),
            "file_0's tag, deferred before the close queued its own save, must not be dropped"
        );
    }

    /// Regression test: closing the window while a video is loaded used to reopen it (a fresh
    /// GStreamer pipeline load) right after saving, because `on_media_unloaded`'s generic
    /// epilogue always reopens the directory's selected file. Closing must only save and let
    /// the window close, not reload media it is about to tear down anyway.
    #[test]
    fn closing_does_not_reopen_the_video() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let loads_before = workspace.media_viewer().video_loads_started();

        let _ = workspace.flush_open_file();
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));

        assert_eq!(
            workspace.media_viewer().video_loads_started(),
            loads_before,
            "closing must not reopen the video after saving"
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

        let mut workspace = test_workspace();
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
        let mut workspace = test_workspace();
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

    /// Turning batch mode on checks the open file, and only it.
    #[test]
    fn batch_mode_starts_with_the_open_file_checked() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let dir = workspace.directory().expect("dir");
        let open = dir.selected_file().expect("open file").id();
        let other = dir
            .files_in_order()
            .map(|f| f.id())
            .find(|id| *id != open)
            .expect("second file");

        let _ = workspace.update(Message::Folder(folder::Message::SetBatchMode(true)));
        assert!(workspace.batch.is_active());
        assert!(workspace.batch.is_checked(open));
        assert!(!workspace.batch.is_checked(other));
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

        let skipped = || ItemResult::new(ItemStatus::Skipped, None);
        let _ = workspace.update(Message::BatchItemDone {
            id: file_id_at(&workspace, 0),
            result: Box::new(skipped()),
        });
        assert!(workspace.is_batch_running());
        let _ = workspace.update(Message::BatchItemDone {
            id: file_id_at(&workspace, 1),
            result: Box::new(skipped()),
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
        let done = ItemResult::new(ItemStatus::Done, None);
        let _ = workspace.update(Message::BatchItemDone {
            id: first,
            result: Box::new(done),
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
        let mut workspace = test_workspace();
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

    /// Issue #125: a comment cleared in the box, then leaving the clip and coming back, showed
    /// the old comment again.
    #[test]
    fn a_cleared_comment_is_still_cleared_after_leaving_the_clip_and_coming_back() {
        use iced::widget::text_editor::{Action, Edit};
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let first = file_id_at(&workspace, 0);
        let leave_and_return = |workspace: &mut FolderWorkspace| {
            let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
            flush_file_opened(workspace);
            let _ = workspace.update(Message::MediaViewer(
                crate::features::media_viewer::Message::Unloaded,
            ));
            let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
            flush_file_opened(workspace);
            let _ = workspace.update(Message::MediaViewer(
                crate::features::media_viewer::Message::Unloaded,
            ));
        };
        let comment_of = |w: &FolderWorkspace| {
            w.directory()
                .and_then(|d| d.file_by_id(first))
                .map(|f| f.comment().to_string())
                .unwrap_or_default()
        };
        let shown = |w: &FolderWorkspace| w.file_workspace().tag_list().comment().to_string();

        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Insert('o'))));
        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Insert('k'))));
        leave_and_return(&mut workspace);
        assert_eq!(comment_of(&workspace), "ok", "the comment was saved");
        assert_eq!(
            shown(&workspace),
            "ok",
            "and shown when the clip is opened again"
        );

        let _ = workspace.update(Message::CommentAction(Action::SelectAll));
        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Delete)));
        leave_and_return(&mut workspace);
        assert_eq!(comment_of(&workspace), "", "the cleared comment was saved");
        assert_eq!(
            shown(&workspace),
            "",
            "and the box is empty when it is opened again"
        );
    }

    /// The same, when the way back comes before the first clip's video has unloaded: its save is
    /// still waiting, so the clip must not open with the comment it had before the edit.
    #[test]
    fn a_comment_cleared_just_before_leaving_is_not_shown_again_while_its_save_waits() {
        use iced::widget::text_editor::{Action, Edit};
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Insert('o'))));
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        assert_eq!(workspace.file_workspace().tag_list().comment(), "o");

        let _ = workspace.update(Message::CommentAction(Action::SelectAll));
        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Delete)));
        // Away and straight back: no Unloaded in between.
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        assert_eq!(
            workspace.file_workspace().tag_list().comment(),
            "",
            "the box shows the cleared comment"
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
        let mut workspace = test_workspace();
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
        let mut workspace = test_workspace();
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
        let mut workspace = test_workspace();
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
        let mut workspace = test_workspace();
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

    /// Issue #84: a save that would rename one file onto the name another already has must be
    /// refused, not silently overwrite it. `InMemoryFileTagger` mirrors `ProductionFileTagger`'s
    /// guard for exactly this (both check real files on disk for the target name).
    #[test]
    fn a_rename_onto_an_existing_files_name_is_refused() {
        let test_dir = TestDirectory::new(2);
        // TestDirectory's own files are virtual (never written to disk); write file_1 for real,
        // so the guard (which checks real files) has something on disk to find.
        std::fs::write(test_dir.file_path("file_1.mp4"), []).expect("write file_1");
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let file_0_id = file_id_at(&workspace, 0);

        // Computes to "file_1.mp4" — the name the other real file on disk already has.
        let snapshot = FileSnapshot::new(Vec::<String>::new(), "file_1", ".mp4", "file_0.mp4");
        let _ = workspace.update(Message::FileUpdated {
            id: file_0_id,
            snapshot,
        });

        let dir = workspace.directory().expect("directory should be loaded");
        let file_0 = dir.file_by_id(file_0_id).expect("file_0 still tracked");
        assert_eq!(
            file_0.file_path(),
            test_dir.file_path("file_0.mp4").as_path(),
            "refused: file_0 must not take file_1's name"
        );
        assert!(
            test_dir.file_path("file_1.mp4").exists(),
            "the other file must survive untouched"
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

    // --- Markers ---

    /// A marker key or list message with the playhead at `position_ms`.
    fn send_marker(
        workspace: &mut FolderWorkspace,
        msg: crate::features::markers::Message,
        position_ms: u64,
    ) {
        let _ = workspace.handle_marker(msg, position_ms);
    }

    /// A workspace on a folder whose files are real (tiny) videos, so they can hold markers.
    fn marker_workspace(test_dir: &TestDirectory, files: usize) -> FolderWorkspace {
        let clip = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("crates/frename-core/tests/fixtures/tiny.mov");
        for i in 0..files {
            std::fs::copy(&clip, test_dir.file_path(&format!("file_{i}.mp4"))).expect("clip");
        }
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        assert_eq!(workspace.file_workspace().markers(), Some(&[][..]));
        workspace
    }

    /// Ctrl+Alt+→ / ← turn the open video, each press one undo step; a file that has no
    /// rotation flag is left alone and nothing goes into the history.
    #[test]
    fn rotating_the_open_video_undoes_and_redoes() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let file = test_dir.target_file();
        let degrees = || {
            frename_core::FileTagger::video_rotation(&file)
                .expect("rotation")
                .degrees()
        };
        assert_eq!(degrees(), 0);
        let _ = workspace.update(Message::RotateVideo(1));
        let _ = workspace.update(Message::RotateVideo(1));
        let _ = workspace.update(Message::RotateVideo(-1));
        assert_eq!(degrees(), 90);
        let _ = workspace.update(Message::Undo);
        assert_eq!(degrees(), 180);
        let _ = workspace.update(Message::Undo);
        let _ = workspace.update(Message::Undo);
        assert_eq!(degrees(), 0);
        let _ = workspace.update(Message::Redo);
        assert_eq!(degrees(), 90);

        // A turn that fails pushes nothing: the redo step is still there.
        let _ = workspace.update(Message::Undo);
        assert!(workspace.history.can_redo());
        std::fs::write(&file, [0u8; 64]).expect("damage the file");
        let _ = workspace.update(Message::RotateVideo(1));
        assert!(workspace.history.can_redo());
    }

    /// Turn, then undo before the reopen from the turn has landed: the undo starts a reopen of
    /// its own (which reads the upright file), so the turn's reopen, now stale, is dropped
    /// instead of showing a turn the file no longer has.
    #[test]
    fn undoing_a_turn_before_its_reopen_lands_reopens_again() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let loads = |w: &FolderWorkspace| w.media_viewer.video_loads_started();
        let before = loads(&workspace);
        let _ = workspace.update(Message::RotateVideo(1));
        assert_eq!(loads(&workspace), before + 1, "the turn reopens the video");
        let _ = workspace.update(Message::Undo);
        assert_eq!(
            loads(&workspace),
            before + 2,
            "the undo reopens it again, once"
        );
        // No refresh of the open file: it would unload the video to save the file while the
        // reopen is still opening it.
        assert!(workspace.pending_file_updates.is_empty());
        let degrees = || {
            frename_core::FileTagger::video_rotation(&test_dir.target_file()).map(|r| r.degrees())
        };
        assert_eq!(degrees(), Ok(0));
        let _ = workspace.update(Message::Redo);
        assert_eq!(loads(&workspace), before + 3, "the redo reopens it once");
        assert!(workspace.pending_file_updates.is_empty());
        assert_eq!(degrees(), Ok(90));
    }

    /// Issue #161: `Home` taken by a text field goes to the start of the clip only while the
    /// note says Home starts it over, and never while a file name is being typed (the focus of
    /// its field decides, so with a rename open on another row Home still reaches the clip);
    /// otherwise the field keeps it.
    #[test]
    fn home_from_a_text_field_starts_the_clip_over_only_while_the_note_says_so() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        assert_eq!(
            workspace.update(Message::GoToStartWhileTyping).units(),
            0,
            "no note: the field keeps Home"
        );
        workspace.media_viewer_mut().pretend_resume_note();
        assert!(workspace.media_viewer().resume_note_shown());
        // The comment box check: a widget operation, then the message.
        assert_eq!(workspace.update(Message::GoToStartWhileTyping).units(), 1);
        let _ = workspace.update(Message::Folder(folder::Message::StartRename(0)));
        assert!(workspace.inline_rename.is_some(), "renaming by hand");
        // A rename open on another row does not hold the key: the focus of its field decides,
        // so the check is asked (a widget operation) instead of dropping Home.
        assert_eq!(
            workspace.update(Message::GoToStartWhileTyping).units(),
            1,
            "the rename field's focus decides"
        );
        assert_eq!(workspace.update(Message::GoToStartUnlessWriting).units(), 1);
    }

    /// In batch mode the comment box is not shown, so `Ctrl+Alt+→` after typing in the file
    /// filter turns the video at once instead of asking about a focus nobody can answer.
    #[test]
    fn a_turn_while_typing_in_batch_mode_goes_straight_through() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let _ = workspace.update(Message::Folder(folder::Message::SetBatchMode(true)));
        assert!(workspace.batch.is_active());
        let task = workspace.update(Message::RotateVideoWhileTyping(1));
        // `Task::done`: one message, known without running a widget operation.
        assert_eq!(task.units(), 1);
        let _ = workspace.update(Message::RotateVideo(1));
        let rotation = frename_core::FileTagger::video_rotation(&test_dir.target_file());
        assert_eq!(rotation.map(|r| r.degrees()), Ok(90));
    }

    #[test]
    fn a_turn_note_says_what_was_pressed_when_the_clip_was_already_turned() {
        use crate::features::rotation_text::turned;
        use frename_core::Rotation;
        let right = Rotation::UPRIGHT.turned(1);
        assert_eq!(turned(1, right), "Rotation: 90° right");
        assert_eq!(
            turned(1, right.turned(1)),
            "Turned 90° right · rotation now 180°"
        );
        assert_eq!(
            turned(-1, Rotation::UPRIGHT),
            "Turned 90° left · rotation now none"
        );
    }

    /// Typing `text` into the open marker row's name field.
    fn type_name(text: &str) -> crate::features::markers::Message {
        crate::features::markers::Message::NameAction(iced::widget::text_editor::Action::Edit(
            iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(text.to_string())),
        ))
    }

    fn marker_names(workspace: &FolderWorkspace) -> Vec<(u64, String)> {
        workspace
            .file_workspace()
            .markers()
            .unwrap_or_default()
            .iter()
            .map(|m| (m.start_ms, m.name.clone()))
            .collect()
    }

    // Issue #174: "Describe with AI" on a marker. The request's task is not run in tests: its
    // answer is handed in as the message the task would end with.

    /// The answer the request on its way for `guid` would end with.
    fn answer(
        workspace: &FolderWorkspace,
        guid: &str,
        outcome: crate::features::markers::MomentOutcome,
    ) -> Message {
        Message::MarkerDescribed {
            file: workspace.file_workspace().file().expect("open").id(),
            guid: guid.to_string(),
            request: workspace.markers().request_of(guid).expect("on its way"),
            outcome,
        }
    }

    fn described(
        workspace: &FolderWorkspace,
        guid: &str,
        name: &str,
        description: &str,
    ) -> Message {
        answer(
            workspace,
            guid,
            crate::features::markers::MomentOutcome::Described {
                name: name.to_string(),
                description: description.to_string(),
            },
        )
    }

    fn marker_texts(workspace: &FolderWorkspace) -> Vec<(String, String)> {
        workspace
            .file_workspace()
            .markers()
            .unwrap_or_default()
            .iter()
            .map(|m| (m.name.clone(), m.comment.clone()))
            .collect()
    }

    fn only_guid(workspace: &FolderWorkspace) -> String {
        let markers = workspace.file_workspace().markers().unwrap_or_default();
        assert_eq!(markers.len(), 1);
        markers[0].guid.clone().expect("editable")
    }

    #[test]
    fn a_described_marker_is_named_and_described_in_one_undo_step() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);

        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        assert!(workspace.markers().is_describing(&guid));
        let _ = workspace.update(described(&workspace, &guid, "Lion", "A lion walks past."));
        assert!(!workspace.markers().is_describing(&guid));
        assert_eq!(
            marker_texts(&workspace),
            [("Lion".to_string(), "A lion walks past.".to_string())]
        );

        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_texts(&workspace), [(String::new(), String::new())]);
        assert_eq!(
            marker_names(&workspace),
            [(1_000, String::new())],
            "only the AI's text was undone"
        );
        let _ = workspace.update(Message::Redo);
        assert_eq!(
            marker_texts(&workspace),
            [("Lion".to_string(), "A lion walks past.".to_string())]
        );
    }

    #[test]
    fn a_marker_name_typed_by_the_editor_is_kept() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        send_marker(&mut workspace, M::Add, 1_000);
        send_marker(&mut workspace, type_name("Mine"), 1_000);
        let guid = only_guid(&workspace);

        // Its row is still open while the answer comes: the typed name is a step of its own.
        send_marker(&mut workspace, M::Describe(guid.clone()), 1_000);
        let _ = workspace.update(described(&workspace, &guid, "Lion", "A lion walks past."));
        assert!(!workspace.markers().is_editing());
        assert_eq!(
            marker_texts(&workspace),
            [("Mine".to_string(), "A lion walks past.".to_string())]
        );
        let _ = workspace.update(Message::Undo);
        assert_eq!(
            marker_texts(&workspace),
            [("Mine".to_string(), String::new())]
        );
    }

    #[test]
    fn a_stopped_request_and_one_for_a_clip_left_are_dropped() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(2);
        let mut workspace = marker_workspace(&test_dir, 2);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);

        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let stopped = described(&workspace, &guid, "Lion", "A lion.");
        send_marker(&mut workspace, M::StopDescribing(guid.clone()), 0);
        assert!(!workspace.markers().any_describing());
        let _ = workspace.update(stopped);
        assert_eq!(marker_texts(&workspace), [(String::new(), String::new())]);

        // Asked again, then the clip is left before the answer comes.
        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let answer = described(&workspace, &guid, "Lion", "A lion.");
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        assert!(!workspace.markers().any_describing(), "leaving stopped it");
        let history = workspace.history.can_undo();
        let _ = workspace.update(answer);
        assert!(
            marker_texts(&workspace).is_empty(),
            "the other clip got nothing"
        );
        assert_eq!(workspace.history.can_undo(), history, "no step was pushed");
    }

    /// Review round 1: the late answer of a stopped request must not end the request sent after
    /// it for the same marker.
    #[test]
    fn a_stopped_requests_late_answer_does_not_take_the_next_ones_place() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);

        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let first = described(&workspace, &guid, "First", "The first answer.");
        send_marker(&mut workspace, M::StopDescribing(guid.clone()), 0);
        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let second = described(&workspace, &guid, "Second", "The second answer.");

        let _ = workspace.update(first);
        assert!(workspace.markers().is_describing(&guid), "still waiting");
        assert_eq!(marker_texts(&workspace), [(String::new(), String::new())]);
        let _ = workspace.update(second);
        assert!(!workspace.markers().any_describing());
        assert_eq!(
            marker_texts(&workspace),
            [("Second".to_string(), "The second answer.".to_string())]
        );
    }

    /// A batch job closes the clip: its marker requests stop, and the job waits until the last
    /// one has let go of the clip, then starts by itself.
    #[test]
    fn a_batch_job_stops_marker_requests_and_waits_for_them() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);
        let ids: Vec<FileId> = workspace
            .directory()
            .expect("dir")
            .files_in_order()
            .map(|f| f.id())
            .collect();
        let _ = workspace.update(Message::Batch(batch::Message::SetActive(true)));
        let _ = workspace.update(Message::Batch(batch::Message::CheckAll(ids)));

        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let late = described(&workspace, &guid, "Lion", "A lion.");
        let _ = workspace.update(Message::Batch(batch::Message::Run));
        assert!(!workspace.markers().any_describing(), "stopped");
        assert!(!workspace.is_batch_running(), "waits for the request");
        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        assert!(
            !workspace.markers().any_describing(),
            "none starts meanwhile"
        );

        let _ = workspace.update(late);
        assert!(workspace.is_batch_running(), "started once it let go");
        assert_eq!(marker_texts(&workspace), [(String::new(), String::new())]);
    }

    #[test]
    fn ctrl_f2_opens_the_marker_list_to_show_the_request() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        assert!(!workspace.media_viewer.marker_list_shown());
        send_marker(&mut workspace, M::DescribeAtPlayhead, 1_200);
        assert!(workspace.markers().is_describing(&only_guid(&workspace)));
        assert!(workspace.media_viewer.marker_list_shown());
    }

    #[test]
    fn no_key_sends_nothing_more_and_changes_nothing() {
        use crate::features::markers::{Message as M, MomentOutcome};
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);
        let steps = workspace.history.can_undo();

        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let _ = workspace.update(answer(&workspace, &guid, MomentOutcome::NoKey));
        assert!(
            !workspace.batch.actions().ai_key_missing(),
            "the settings read the key, not the request"
        );
        assert!(workspace.markers.no_key_said(), "said once");
        assert_eq!(marker_texts(&workspace), [(String::new(), String::new())]);
        assert_eq!(workspace.history.can_undo(), steps);
        // The settings' answer to the read the request asked for.
        let _ = workspace.update(Message::Batch(batch::Message::Action(
            batch::ActionMessage::DescribeAi(batch::describe_ai::Message::KeyState(
                frename_core::ai::key::KeyState::Missing,
            )),
        )));
        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        assert!(!workspace.markers().any_describing(), "nothing is sent");
    }

    /// Issue #208: a clip with `count` unnamed markers (and a named one at 40 s), their GUIDs in
    /// the list's order.
    fn unnamed_markers_workspace(
        test_dir: &TestDirectory,
        count: u64,
    ) -> (FolderWorkspace, Vec<String>) {
        let mut workspace = marker_workspace(test_dir, 1);
        for i in 0..count {
            let marker = frename_core::Marker::new(1_000 + i * 5_000);
            workspace.file_workspace.tag_list_mut().add_marker(marker);
        }
        let mut named = frename_core::Marker::new(40_000);
        named.name = "Mine".to_string();
        workspace.file_workspace.tag_list_mut().add_marker(named);
        let guids = workspace
            .file_workspace()
            .markers()
            .unwrap_or_default()
            .iter()
            .filter(|m| m.name.is_empty())
            .map(|m| m.guid.clone().expect("editable"))
            .collect::<Vec<_>>();
        assert_eq!(guids.len() as u64, count, "and one named");
        (workspace, guids)
    }

    #[test]
    fn describe_unnamed_sends_a_few_at_a_time_and_each_answer_is_its_own_undo_step() {
        use crate::features::markers::state_for_tests::MAX_DESCRIBING_AT_ONCE;
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let (mut workspace, guids) = unnamed_markers_workspace(&test_dir, 5);
        assert_eq!(
            workspace.unnamed_marker_guids().len(),
            5,
            "the named marker is not counted"
        );

        send_marker(&mut workspace, M::DescribeUnnamed, 0);
        assert_eq!(workspace.markers().in_flight(), MAX_DESCRIBING_AT_ONCE);
        assert_eq!(workspace.markers().waiting_count(), 2);
        assert_eq!(workspace.marker_requests, 3);
        for guid in &guids[..3] {
            assert!(
                workspace.markers().request_of(guid).is_some(),
                "in list order"
            );
        }
        assert!(workspace.markers().is_waiting(&guids[3]));
        assert_eq!(
            workspace.unnamed_marker_guids().len(),
            0,
            "all on their way or waiting: nothing to offer twice"
        );
        send_marker(&mut workspace, M::DescribeUnnamed, 0);
        assert_eq!(
            workspace.marker_requests, 3,
            "asking again sends nothing more"
        );

        let _ = workspace.update(described(&workspace, &guids[0], "One", "First."));
        assert_eq!(workspace.markers().in_flight(), 3, "the next one went out");
        assert_eq!(workspace.markers().waiting_count(), 1);
        assert!(workspace.markers().request_of(&guids[3]).is_some());
        for (i, guid) in guids.iter().enumerate().skip(1) {
            let _ = workspace.update(described(&workspace, guid, &format!("N{i}"), "Text."));
        }
        assert!(!workspace.markers().any_describing());
        assert_eq!(workspace.marker_requests, 0);
        let names: Vec<_> = marker_names(&workspace)
            .into_iter()
            .map(|(_, n)| n)
            .collect();
        assert_eq!(names, ["One", "N1", "N2", "N3", "N4", "Mine"]);

        // One undo step per marker: the last answer goes first, the others stay.
        let _ = workspace.update(Message::Undo);
        let names: Vec<_> = marker_names(&workspace)
            .into_iter()
            .map(|(_, n)| n)
            .collect();
        assert_eq!(names, ["One", "N1", "N2", "N3", "", "Mine"]);
    }

    #[test]
    fn describe_unnamed_keeps_a_name_typed_while_the_marker_waits() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let (mut workspace, guids) = unnamed_markers_workspace(&test_dir, 5);
        send_marker(&mut workspace, M::DescribeUnnamed, 0);
        // The fourth waits; the editor names it, and the fifth is in flight already (a slot
        // frees up), a name typed on one that is in flight is kept by the answer.
        workspace
            .file_workspace
            .tag_list_mut()
            .update_marker(&guids[3], |m| m.name = "Typed".to_string());
        let _ = workspace.update(described(&workspace, &guids[0], "One", "First."));
        assert!(
            !workspace.markers().is_describing(&guids[3]),
            "named meanwhile: not sent"
        );
        assert!(workspace.markers().request_of(&guids[4]).is_some());
        workspace
            .file_workspace
            .tag_list_mut()
            .update_marker(&guids[1], |m| m.name = "Mine too".to_string());
        let _ = workspace.update(described(&workspace, &guids[1], "Other", "Second."));
        let texts = marker_texts(&workspace);
        assert_eq!(texts[1], ("Mine too".to_string(), "Second.".to_string()));
        assert_eq!(texts[3].0, "Typed");
        assert_eq!(texts[3].1, "", "nothing was sent for it");
    }

    #[test]
    fn describe_unnamed_stops_one_waiting_all_or_the_rest_after_a_failure() {
        use crate::features::markers::{Message as M, MomentOutcome};
        let test_dir = TestDirectory::new(1);
        let (mut workspace, guids) = unnamed_markers_workspace(&test_dir, 6);

        // One waiting marker is stopped: its row is back to unnamed and it never goes out.
        send_marker(&mut workspace, M::DescribeUnnamed, 0);
        assert_eq!(workspace.markers().waiting_count(), 3);
        send_marker(&mut workspace, M::StopDescribing(guids[3].clone()), 0);
        assert!(!workspace.markers().is_describing(&guids[3]));
        assert_eq!(workspace.markers().waiting_count(), 2);

        // A failure ends the run: the rest stays unnamed, the ones on their way go on.
        let failed = MomentOutcome::Failed("Network error".to_string());
        let _ = workspace.update(answer(&workspace, &guids[0], failed));
        assert_eq!(workspace.markers().waiting_count(), 0);
        assert_eq!(workspace.markers().in_flight(), 2);
        assert_eq!(workspace.marker_requests, 2);

        // Stop all: the requests on their way too; their answers are dropped.
        let late = described(&workspace, &guids[1], "Lion", "A lion.");
        send_marker(&mut workspace, M::StopDescribingAll, 0);
        assert!(!workspace.markers().any_describing());
        let _ = workspace.update(late);
        assert!(marker_names(&workspace).iter().all(|(_, n)| n != "Lion"));
    }

    #[test]
    fn describe_unnamed_gives_one_failed_notice_per_run() {
        use crate::features::markers::{Message as M, MomentOutcome};
        let test_dir = TestDirectory::new(1);
        let (mut workspace, guids) = unnamed_markers_workspace(&test_dir, 3);
        send_marker(&mut workspace, M::DescribeUnnamed, 0);
        let failed = || MomentOutcome::Failed("Network error".to_string());
        let first = workspace.update(answer(&workspace, &guids[0], failed()));
        let second = workspace.update(answer(&workspace, &guids[1], failed()));
        let third = workspace.update(answer(&workspace, &guids[2], failed()));
        assert!(first.units() > second.units(), "the first says it");
        assert_eq!(second.units(), third.units(), "the others stay quiet");
        // A new request is a new run: it says its failure again.
        send_marker(&mut workspace, M::Describe(guids[0].clone()), 0);
        let again = workspace.update(answer(&workspace, &guids[0], failed()));
        assert_eq!(again.units(), first.units());
    }

    #[test]
    fn a_stopped_request_keeps_its_slot_until_its_answer_is_back() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let (mut workspace, guids) = unnamed_markers_workspace(&test_dir, 5);
        send_marker(&mut workspace, M::DescribeUnnamed, 0);
        let stopped = described(&workspace, &guids[0], "One", "First.");
        send_marker(&mut workspace, M::StopDescribing(guids[0].clone()), 0);
        assert_eq!(workspace.marker_requests, 3, "it still reads the clip");
        assert_eq!(workspace.markers().in_flight(), 2);
        assert_eq!(workspace.markers().waiting_count(), 2);
        // Another answer frees a real slot only once: no fourth request alive.
        let _ = workspace.update(described(&workspace, &guids[1], "Two", "Second."));
        assert_eq!(workspace.marker_requests, 3, "one back, one sent");
        assert_eq!(workspace.markers().waiting_count(), 1);
        // The stopped one comes back: its answer is dropped and the slot is used.
        let _ = workspace.update(stopped);
        assert_eq!(workspace.marker_requests, 3);
        assert_eq!(workspace.markers().waiting_count(), 0);
        assert!(marker_names(&workspace).iter().all(|(_, n)| n != "One"));
    }

    #[test]
    fn describe_unnamed_with_no_key_sends_nothing_and_a_missing_key_found_out_ends_the_run() {
        use crate::features::markers::{Message as M, MomentOutcome};
        let test_dir = TestDirectory::new(1);
        let (mut workspace, guids) = unnamed_markers_workspace(&test_dir, 5);

        // Found out by the first answer: the queue is dropped, the notice said once.
        send_marker(&mut workspace, M::DescribeUnnamed, 0);
        let _ = workspace.update(answer(&workspace, &guids[0], MomentOutcome::NoKey));
        assert_eq!(workspace.markers().waiting_count(), 0);
        assert_eq!(workspace.markers().in_flight(), 2);
        let _ = workspace.update(answer(&workspace, &guids[1], MomentOutcome::NoKey));
        assert!(workspace.markers.no_key_said(), "said already");
        let _ = workspace.update(Message::Batch(batch::Message::Action(
            batch::ActionMessage::DescribeAi(batch::describe_ai::Message::KeyState(
                frename_core::ai::key::KeyState::Missing,
            )),
        )));
        let _ = workspace.update(answer(&workspace, &guids[2], MomentOutcome::NoKey));
        assert!(!workspace.markers().any_describing());

        // Known to be missing: nothing is queued or sent, Settings opens.
        let task = workspace.handle_marker(M::DescribeUnnamed, 0);
        assert!(!workspace.markers().any_describing());
        assert_eq!(workspace.marker_requests, 0);
        assert_eq!(task.units(), 2, "the notice and the settings");
    }

    #[test]
    fn describe_unnamed_leaves_the_queue_with_the_clip() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(2);
        let (mut workspace, _guids) = unnamed_markers_workspace(&test_dir, 5);
        send_marker(&mut workspace, M::DescribeUnnamed, 0);
        assert!(workspace.markers().waiting_count() > 0);
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        assert!(
            !workspace.markers().any_describing(),
            "nothing waits for another clip"
        );
    }

    #[test]
    fn a_failed_request_or_nothing_new_leaves_the_marker_and_the_history() {
        use crate::features::markers::{Message as M, MomentOutcome};
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);

        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let failed = MomentOutcome::Failed("Network error".to_string());
        let _ = workspace.update(answer(&workspace, &guid, failed));
        assert!(!workspace.markers().any_describing());
        assert_eq!(marker_texts(&workspace), [(String::new(), String::new())]);

        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let _ = workspace.update(described(&workspace, &guid, "Lion", "A lion."));
        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let _ = workspace.update(described(&workspace, &guid, "Other", "A lion."));
        assert_eq!(
            marker_texts(&workspace),
            [("Lion".to_string(), "A lion.".to_string())]
        );
        let _ = workspace.update(Message::Undo);
        assert_eq!(
            marker_texts(&workspace),
            [(String::new(), String::new())],
            "nothing new pushed no step: one undo takes the first answer back"
        );
    }

    #[test]
    fn an_answer_for_a_marker_undone_meanwhile_is_dropped() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);
        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let late = described(&workspace, &guid, "Lion", "A lion.");
        let _ = workspace.update(Message::Undo);
        assert!(marker_texts(&workspace).is_empty(), "the add was undone");
        let _ = workspace.update(late);
        assert!(marker_texts(&workspace).is_empty());
        assert!(!workspace.markers().any_describing());
    }

    /// Review round 2: a run that waited for marker requests and was then given up on (batch
    /// mode left, Cancel, other files) does not start when the requests are back.
    #[test]
    fn a_run_given_up_while_waiting_for_marker_requests_does_not_start() {
        use crate::features::markers::Message as M;
        let give_ups = [
            Message::Batch(batch::Message::SetActive(false)),
            Message::Batch(batch::Message::Cancel),
            Message::Batch(batch::Message::CheckNone),
        ];
        for give_up in give_ups {
            let test_dir = TestDirectory::new(1);
            let mut workspace = marker_workspace(&test_dir, 1);
            send_marker(&mut workspace, M::Add, 1_000);
            let guid = only_guid(&workspace);
            let ids: Vec<FileId> = workspace
                .directory()
                .expect("dir")
                .files_in_order()
                .map(|f| f.id())
                .collect();
            let _ = workspace.update(Message::Batch(batch::Message::SetActive(true)));
            let _ = workspace.update(Message::Batch(batch::Message::CheckAll(ids)));
            send_marker(&mut workspace, M::Describe(guid.clone()), 0);
            let late = described(&workspace, &guid, "Lion", "A lion.");
            let _ = workspace.update(Message::Batch(batch::Message::Run));
            assert!(
                workspace.batch.is_waiting_for_markers(),
                "the panel says so"
            );

            let _ = workspace.update(give_up.clone());
            assert!(!workspace.batch.is_waiting_for_markers(), "{give_up:?}");
            let _ = workspace.update(late);
            assert!(!workspace.is_batch_running(), "{give_up:?}: not started");
        }
    }

    /// Review round 3: a save meeting a marker request does not wait for it; when the save fails
    /// (here: the new name is taken, as a file still held open fails on Windows), it goes the way
    /// of every refused save: the file keeps its name on disk, its markers are written, and the
    /// request's answer still lands.
    #[test]
    fn a_save_refused_while_a_marker_request_runs_is_a_normal_refused_save() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(2);
        let mut workspace = marker_workspace(&test_dir, 2);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);
        let id = workspace.file_workspace().file().expect("open").id();
        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let late = described(&workspace, &guid, "Lion", "A lion.");

        let markers = workspace.file_workspace().markers().map(<[_]>::to_vec);
        let mut snapshot = FileSnapshot::new(Vec::<String>::new(), "file_1", ".mp4", "file_0.mp4");
        snapshot.set_markers(markers);
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        let path = workspace
            .directory()
            .and_then(|d| d.file_by_id(id))
            .map(|f| f.file_path().to_path_buf());
        assert_eq!(path, Some(test_dir.file_path("file_0.mp4")), "refused");
        let saved = frename_core::FileTagger::load_markers(&test_dir.file_path("file_0.mp4"));
        assert_eq!(saved.map(|m| m.len()), Some(1), "markers written");

        assert!(workspace.markers().is_describing(&guid));
        let _ = workspace.update(late);
        assert_eq!(
            marker_texts(&workspace),
            [("Lion".to_string(), "A lion.".to_string())]
        );
    }

    #[test]
    fn ctrl_f2_describes_the_marker_the_playhead_is_on() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::DescribeAtPlayhead, 1_000);
        assert!(!workspace.markers().any_describing(), "no marker yet");
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);
        send_marker(&mut workspace, M::DescribeAtPlayhead, 9_000);
        assert!(
            !workspace.markers().any_describing(),
            "the playhead is past it"
        );
        send_marker(&mut workspace, M::DescribeAtPlayhead, 1_500);
        assert!(workspace.markers().is_describing(&guid));
    }

    #[test]
    fn a_deleted_marker_stops_its_request() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);
        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        send_marker(&mut workspace, M::Delete(guid), 0);
        assert!(!workspace.markers().any_describing());
    }

    #[test]
    fn a_described_marker_kept_in_a_comment_file_is_written_at_once() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let _storage = comment_file_storage(frename_core::MarkerStorage::Comment);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = only_guid(&workspace);
        send_marker(&mut workspace, M::Describe(guid.clone()), 0);
        let _ = workspace.update(described(&workspace, &guid, "Lion", "A lion walks past."));
        let written = comment_file_text(&test_dir);
        assert!(
            written.contains("Lion") && written.contains("A lion walks past."),
            "{written:?}"
        );
    }

    #[test]
    fn a_name_typed_in_the_marker_list_is_in_the_state_before_the_next_view() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        send_marker(&mut workspace, M::Add, 1_000);
        assert!(workspace.markers().is_editing());

        // As the name field sends it: through the video, with no task run afterwards.
        for letter in ['a', 's', 'a'] {
            let _ = workspace.update(Message::MediaViewer(
                crate::features::media_viewer::Message::Video(
                    crate::features::media_viewer::video::Message::Markers(M::NameAction(
                        iced::widget::text_editor::Action::Edit(
                            iced::widget::text_editor::Edit::Insert(letter),
                        ),
                    )),
                ),
            ));
        }
        assert_eq!(marker_names(&workspace), [(1_000, "asa".to_string())]);
    }

    #[test]
    fn f2_marks_f2_again_names_and_leaving_the_file_saves_the_markers() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);

        send_marker(&mut workspace, M::Add, 1_000);
        assert_eq!(marker_names(&workspace), [(1_000, String::new())]);
        // A second F2 right away opens the marker just added instead of adding one.
        send_marker(&mut workspace, M::Add, 1_300);
        assert!(workspace.markers().is_editing());
        send_marker(&mut workspace, type_name("Take 3"), 1_300);
        // Keys of the workspace are off while the row is open: `[` is typed, not an in point.
        let _ = workspace.update(Message::SetSegmentStart);
        assert_eq!(marker_names(&workspace), [(1_000, "Take 3".to_string())]);
        // F2 with the row open closes it and adds the next moment without opening it.
        send_marker(&mut workspace, M::Add, 5_000);
        assert!(!workspace.markers().is_editing());
        assert_eq!(marker_names(&workspace).len(), 2);

        let (id, snapshot) = workspace.file_workspace().get_snapshot().expect("open");
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        let saved = frename_core::FileTagger::load_markers(&test_dir.target_file()).expect("saved");
        let saved: Vec<_> = saved
            .iter()
            .map(|m| (m.start_ms, m.name.as_str()))
            .collect();
        assert_eq!(saved, [(1_000, "Take 3"), (5_000, "")]);
    }

    fn marker_spans(workspace: &FolderWorkspace) -> Vec<(u64, u64)> {
        workspace
            .file_workspace()
            .markers()
            .unwrap_or_default()
            .iter()
            .map(|m| (m.start_ms, m.duration_ms))
            .collect()
    }

    #[test]
    fn a_short_f2_press_stays_a_point_and_f2_again_names_it() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::KeyDown, 1_000);
        send_marker(&mut workspace, M::KeyUp, 1_300);
        assert_eq!(marker_spans(&workspace), [(1_000, 0)]);
        send_marker(&mut workspace, M::KeyDown, 1_400);
        send_marker(&mut workspace, M::KeyUp, 1_500);
        assert!(workspace.markers().is_editing());
        assert_eq!(marker_spans(&workspace), [(1_000, 0)]);
    }

    #[test]
    fn a_held_f2_draws_a_range_that_undoes_and_redoes_whole() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::KeyDown, 41_000);
        workspace
            .markers
            .backdate_recording(crate::features::markers::state_for_tests::RANGE_HOLD);
        send_marker(&mut workspace, M::KeyUp, 47_000);
        assert_eq!(marker_spans(&workspace), [(41_000, 6_000)]);
        // Right after the release, F2 names the range.
        send_marker(&mut workspace, M::KeyDown, 47_100);
        send_marker(&mut workspace, M::KeyUp, 47_100);
        assert!(workspace.markers().is_editing());
        send_marker(&mut workspace, M::Close, 47_100);

        let _ = workspace.update(Message::Undo);
        assert!(marker_spans(&workspace).is_empty());
        let _ = workspace.update(Message::Redo);
        assert_eq!(marker_spans(&workspace), [(41_000, 6_000)]);
    }

    #[test]
    fn a_long_press_without_the_playhead_moving_stays_a_point() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::KeyDown, 2_000);
        workspace
            .markers
            .backdate_recording(crate::features::markers::state_for_tests::RANGE_HOLD);
        send_marker(&mut workspace, M::KeyUp, 2_000);
        assert_eq!(marker_spans(&workspace), [(2_000, 0)]);
    }

    #[test]
    fn dragging_a_range_s_ends_swaps_them_and_together_makes_a_point() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::AddRange(47_000, 41_000), 0);
        assert_eq!(marker_spans(&workspace), [(41_000, 6_000)]);
        let guid = workspace.file_workspace().markers().unwrap()[0]
            .guid
            .clone()
            .unwrap();
        // The start dragged past the end: the ends swap.
        send_marker(&mut workspace, M::SetSpan(guid.clone(), 50_000, 47_000), 0);
        assert_eq!(marker_spans(&workspace), [(47_000, 3_000)]);
        // Ends within 100 ms of each other: a point.
        send_marker(&mut workspace, M::SetSpan(guid.clone(), 47_000, 47_050), 0);
        assert_eq!(marker_spans(&workspace), [(47_000, 0)]);
        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_spans(&workspace), [(47_000, 3_000)]);
        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_spans(&workspace), [(41_000, 6_000)]);
        let _ = workspace.update(Message::Undo);
        assert!(marker_spans(&workspace).is_empty());
    }

    #[test]
    fn a_white_ai_range_read_from_the_file_takes_a_new_color_and_keeps_it() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let mut ai = frename_core::Marker::new(3_000);
        ai.duration_ms = 3_000;
        ai.color = frename_core::AI_MARKER_COLOR;
        ai.name = "Close-up of a sign".to_string();
        let guid = ai.guid.clone().unwrap();
        frename_core::FileTagger::save_markers(
            &test_dir.target_file(),
            &[ai],
            &std::collections::HashSet::new(),
        )
        .expect("written");
        let _ = workspace.update(Message::OpenPath(test_dir.file_path("file_0.mp4")));
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::OpenPath(test_dir.target_file()));
        flush_file_opened(&mut workspace);
        assert_eq!(marker_spans(&workspace), [(3_000, 3_000)]);

        send_marker(&mut workspace, M::ToggleColorPicker(guid.clone()), 0);
        send_marker(
            &mut workspace,
            M::SetColor(guid.clone(), frename_core::MarkerColor::Red),
            0,
        );
        let (id, snapshot) = workspace.file_workspace().get_snapshot().expect("open");
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        let saved = frename_core::FileTagger::load_markers(&test_dir.target_file()).expect("saved");
        assert_eq!(saved[0].color, frename_core::MarkerColor::Red);
    }

    #[test]
    fn markers_can_be_edited_in_batch_mode_when_no_job_runs() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let _ = workspace.update(Message::Folder(folder::Message::SetBatchMode(true)));
        assert!(workspace.batch.is_active());
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Video(
                crate::features::media_viewer::video::Message::Markers(M::Add),
            ),
        ));
        assert_eq!(marker_names(&workspace).len(), 1);
    }

    fn first_marker_guid(workspace: &FolderWorkspace) -> String {
        workspace.file_workspace().markers().unwrap()[0]
            .guid
            .clone()
            .unwrap()
    }

    /// Issue #138: Undo was dropped in batch mode, though markers are edited there.
    #[test]
    fn a_deleted_marker_is_undone_and_redone_in_batch_mode() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let _ = workspace.update(Message::Folder(folder::Message::SetBatchMode(true)));
        assert!(workspace.batch.is_active());
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = first_marker_guid(&workspace);
        send_marker(&mut workspace, M::Delete(guid), 1_000);
        assert!(marker_names(&workspace).is_empty());
        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_names(&workspace).len(), 1);
        let _ = workspace.update(Message::Redo);
        assert!(marker_names(&workspace).is_empty());
    }

    /// In batch mode the tags are hidden, so their undo stays off.
    #[test]
    fn a_tag_change_is_not_undone_in_batch_mode() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let tag_list = workspace.file_workspace().tag_list();
        let tag_id = tag_list
            .filtered_display_tag_ids()
            .iter()
            .find(|id| tag_list.get_tag(**id).is_some_and(|t| t.tag() == "pick"))
            .copied()
            .expect("pick is a built-in tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(tag_id)));
        let checked = |w: &FolderWorkspace| {
            w.file_workspace()
                .tag_list()
                .get_tag(tag_id)
                .map(|t| t.is_checked())
        };
        assert_eq!(checked(&workspace), Some(true));
        let _ = workspace.update(Message::Folder(folder::Message::SetBatchMode(true)));
        let _ = workspace.update(Message::Undo);
        assert_eq!(checked(&workspace), Some(true), "batch mode keeps tags out");
        let _ = workspace.update(Message::Folder(folder::Message::SetBatchMode(false)));
        let _ = workspace.update(Message::Undo);
        assert_eq!(checked(&workspace), Some(false));
    }

    #[test]
    fn a_turn_is_undone_in_batch_mode() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let _ = workspace.update(Message::Folder(folder::Message::SetBatchMode(true)));
        let _ = workspace.update(Message::RotateVideo(1));
        let degrees = || {
            frename_core::FileTagger::video_rotation(&test_dir.target_file()).map(|r| r.degrees())
        };
        assert_eq!(degrees(), Ok(90));
        let _ = workspace.update(Message::Undo);
        assert_eq!(degrees(), Ok(0));
    }

    /// Issue #138: undo was dropped while a marker row was open, though the ✕ of the other rows
    /// still deletes. It closes the row and undoes.
    #[test]
    fn undo_with_a_marker_row_open_closes_the_row_and_undoes() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        send_marker(&mut workspace, M::AddRange(5_000, 7_000), 5_000);
        let guids: Vec<String> = workspace
            .file_workspace()
            .markers()
            .unwrap()
            .iter()
            .map(|m| m.guid.clone().unwrap())
            .collect();
        assert_eq!(guids.len(), 2);
        // Row A is open; row B's ✕ deletes B.
        send_marker(&mut workspace, M::Open(guids[0].clone()), 1_000);
        send_marker(&mut workspace, M::Delete(guids[1].clone()), 1_000);
        assert!(workspace.markers().is_editing());
        assert_eq!(marker_names(&workspace).len(), 1);
        let _ = workspace.update(Message::Undo);
        assert!(!workspace.markers().is_editing(), "the row is closed");
        assert_eq!(marker_names(&workspace).len(), 2, "B is back");
        let _ = workspace.update(Message::Redo);
        assert_eq!(marker_names(&workspace).len(), 1);
    }

    /// Issue #138: the history is one stack for the folder, so after leaving the file the first
    /// Undo goes back to it (and says so); the second brings the marker back.
    #[test]
    fn a_marker_deleted_before_leaving_the_file_is_back_after_two_undos() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(2);
        let mut workspace = marker_workspace(&test_dir, 2);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = first_marker_guid(&workspace);
        send_marker(&mut workspace, M::Delete(guid), 1_000);
        let (id, snapshot) = workspace.file_workspace().get_snapshot().expect("open");

        let _ = workspace.update(Message::Folder(folder::Message::NextFile));
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        assert_eq!(workspace.directory().unwrap().selected_index(), Some(1));

        let task = workspace.update(Message::Undo);
        assert_eq!(workspace.directory().unwrap().selected_index(), Some(0));
        // The file is reopened, and a note says the step was a move, not an edit.
        assert_eq!(task.units(), 2);
        flush_file_opened(&mut workspace);
        assert!(marker_names(&workspace).is_empty());

        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_names(&workspace).len(), 1);

        let (id, snapshot) = workspace.file_workspace().get_snapshot().expect("open");
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        let saved = frename_core::FileTagger::load_markers(&test_dir.target_file()).expect("saved");
        assert_eq!(saved.len(), 1, "the file holds the marker");

        // Redo goes forward the same way: the delete, then the move to the next file.
        let _ = workspace.update(Message::Redo);
        assert!(marker_names(&workspace).is_empty());
        let task = workspace.update(Message::Redo);
        assert_eq!(workspace.directory().unwrap().selected_index(), Some(1));
        assert_eq!(task.units(), 2);
    }

    /// A marker row opened by code takes the keys from the comment box, so Undo is not dropped
    /// for a focus the box no longer has.
    #[test]
    fn opening_a_marker_row_ends_the_comment_boxs_hold_on_undo() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        workspace.file_workspace.set_comment_focused(true);
        let guid = first_marker_guid(&workspace);
        send_marker(&mut workspace, M::Open(guid), 1_000);
        let _ = workspace.update(Message::Undo);
        assert!(marker_names(&workspace).is_empty());
    }

    /// Issue #138: while the comment box has the keys, Ctrl+Z is the box's, not the app's.
    #[test]
    fn undo_is_not_the_apps_while_the_comment_box_has_the_keys() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        workspace.file_workspace.set_comment_focused(true);
        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_names(&workspace).len(), 1);
        workspace.file_workspace.set_comment_focused(false);
        let _ = workspace.update(Message::Undo);
        assert!(marker_names(&workspace).is_empty());
    }

    // Issue #143: markers kept in the comment (a `.comment.txt`) are in the file as soon as they
    // are edited, without leaving the clip.

    /// Comments in text files, markers kept as `markers` says, for this test only.
    fn comment_file_storage(markers: frename_core::MarkerStorage) -> frename_core::StorageGuard {
        frename_core::use_storage_on_this_thread(
            frename_core::MetadataStorage {
                comment: frename_core::CommentStorage::TextFile,
                in_out: frename_core::InOutStorage::InVideo,
            },
            markers,
        )
    }

    fn comment_file_text(test_dir: &TestDirectory) -> String {
        let path = format!("{}.comment.txt", test_dir.target_file().display());
        std::fs::read_to_string(path).unwrap_or_default()
    }

    #[test]
    fn a_marker_edit_reaches_the_comment_file_without_leaving_the_clip() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let _storage = comment_file_storage(frename_core::MarkerStorage::Comment);
        let mut workspace = marker_workspace(&test_dir, 1);
        assert_eq!(comment_file_text(&test_dir), "");

        send_marker(&mut workspace, M::Add, 65_000);
        assert!(comment_file_text(&test_dir).contains("1:05"), "added");

        // Named, recolored and moved: the line follows each step.
        let guid = first_marker_guid(&workspace);
        send_marker(&mut workspace, M::Open(guid.clone()), 65_000);
        send_marker(&mut workspace, type_name("Lion"), 65_000);
        send_marker(&mut workspace, M::Close, 65_000);
        assert!(comment_file_text(&test_dir).contains("Lion"), "named");
        send_marker(
            &mut workspace,
            M::SetColor(guid.clone(), frename_core::MarkerColor::Red),
            65_000,
        );
        assert!(comment_file_text(&test_dir).contains("[red]"), "recolored");
        send_marker(&mut workspace, M::SetSpan(guid.clone(), 70_000, 70_000), 0);
        let text = comment_file_text(&test_dir);
        assert!(text.contains("1:10") && !text.contains("1:05"), "{text}");

        // Undo and delete update it too.
        let _ = workspace.update(Message::Undo);
        assert!(comment_file_text(&test_dir).contains("1:05"));
        send_marker(&mut workspace, M::Delete(guid), 0);
        assert!(!comment_file_text(&test_dir).contains("Lion"), "deleted");
    }

    #[test]
    fn the_comments_own_text_is_untouched_by_a_marker_edit() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let _storage = comment_file_storage(frename_core::MarkerStorage::Comment);
        let mut workspace = marker_workspace(&test_dir, 1);
        let path = format!("{}.comment.txt", test_dir.target_file().display());
        std::fs::write(&path, "A note\nAI: Two lions.").expect("comment");
        send_marker(&mut workspace, M::Add, 1_000);
        let text = comment_file_text(&test_dir);
        assert!(
            text.contains("A note") && text.contains("AI: Two lions."),
            "{text}"
        );
    }

    /// A clip whose markers are still in the video, while markers are kept in the comment: they
    /// are shown, left alone until one is edited, then all move into the comment and out of the
    /// video.
    #[test]
    fn markers_held_by_the_video_move_into_the_comment_on_the_first_edit() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        // Written in the video with the other storage.
        let in_video = comment_file_storage(frename_core::MarkerStorage::InVideo);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        send_marker(&mut workspace, M::AddRange(5_000, 8_000), 0);
        let (id, snapshot) = workspace.file_workspace().get_snapshot().expect("open");
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        let file = test_dir.target_file();
        assert_eq!(
            frename_core::FileTagger::load_markers(&file).map(|m| m.len()),
            Some(2)
        );

        // Now markers are kept in the comment: the clip shows its two, and opening wrote nothing.
        drop(in_video);
        let _storage = comment_file_storage(frename_core::MarkerStorage::Comment);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(file.clone()),
        });
        flush_file_opened(&mut workspace);
        assert_eq!(marker_names(&workspace).len(), 2);
        assert_eq!(comment_file_text(&test_dir), "");

        // Leaving it unedited changes nothing either.
        let (id, snapshot) = workspace.file_workspace().get_snapshot().expect("open");
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        assert_eq!(comment_file_text(&test_dir), "");
        assert_eq!(
            frename_core::FileTagger::load_markers(&file).map(|m| m.len()),
            Some(2)
        );

        // One edit: both are in the comment, and the video's own are gone once it is saved.
        let guid = first_marker_guid(&workspace);
        send_marker(&mut workspace, M::Open(guid), 1_000);
        send_marker(&mut workspace, type_name("Edited"), 1_000);
        let text = comment_file_text(&test_dir);
        assert!(text.contains("Edited") && text.contains("0:05"), "{text}");
        let (id, snapshot) = workspace.file_workspace().get_snapshot().expect("open");
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        assert!(frename_core::FileTagger::load_markers(&file)
            .unwrap_or_default()
            .is_empty());
        assert_eq!(
            comment_file_text(&test_dir),
            text,
            "saving again changes nothing"
        );
    }

    /// Issue #143 (d): a comment file that cannot be written keeps the markers and marks the
    /// file, like a failed write into the video does; the next edit tries again.
    #[test]
    fn a_comment_file_that_cannot_be_written_keeps_the_markers_and_marks_the_file() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let _storage = comment_file_storage(frename_core::MarkerStorage::Comment);
        let mut workspace = marker_workspace(&test_dir, 1);
        let path = format!("{}.comment.txt", test_dir.target_file().display());
        std::fs::create_dir_all(&path).expect("a folder where the comment file goes");
        let id = file_id_at(&workspace, 0);
        send_marker(&mut workspace, M::Add, 1_000);
        assert!(workspace.unsaved_markers().contains_key(&id));
        assert_eq!(marker_names(&workspace).len(), 1, "still shown");

        std::fs::remove_dir_all(&path).expect("free the path");
        send_marker(&mut workspace, M::AddRange(5_000, 8_000), 0);
        assert!(
            !workspace.unsaved_markers().contains_key(&id),
            "written now"
        );
        assert!(comment_file_text(&test_dir).contains("0:05"));
    }

    /// A held `F2` grows its marker with the playhead; the release writes the range.
    #[test]
    fn a_held_f2_range_reaches_the_comment_file_on_release() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let _storage = comment_file_storage(frename_core::MarkerStorage::Comment);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::KeyDown, 41_000);
        assert!(comment_file_text(&test_dir).contains("0:41"), "the press");
        workspace
            .markers
            .backdate_recording(crate::features::markers::state_for_tests::RANGE_HOLD);
        send_marker(&mut workspace, M::KeyUp, 47_000);
        assert!(comment_file_text(&test_dir).contains("0:41–0:47"));
    }

    /// An edit undone back to what the video holds leaves no lines behind in the comment.
    #[test]
    fn markers_edited_back_to_the_videos_own_leave_no_comment_lines() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let in_video = comment_file_storage(frename_core::MarkerStorage::InVideo);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let (id, snapshot) = workspace.file_workspace().get_snapshot().expect("open");
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        drop(in_video);
        let _storage = comment_file_storage(frename_core::MarkerStorage::Comment);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        send_marker(&mut workspace, M::AddRange(5_000, 8_000), 0);
        assert!(comment_file_text(&test_dir).contains("0:05"));
        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_names(&workspace).len(), 1);
        assert!(
            !comment_file_text(&test_dir).contains("0:0"),
            "{}",
            comment_file_text(&test_dir)
        );
    }

    /// Issue #171: a click on a row that is in view opens it without scrolling the list; with the
    /// viewport unknown, or the row out of view, the list scrolls to it as before.
    #[test]
    fn opening_a_marker_row_in_view_does_not_scroll_the_list() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        send_marker(&mut workspace, M::Close, 1_000);
        let guid = workspace.file_workspace().markers().unwrap()[0]
            .guid
            .clone()
            .expect("a guid");
        let unknown = workspace.handle_marker(M::Open(guid.clone()), 1_000);
        send_marker(&mut workspace, M::Close, 1_000);
        let _ = workspace.handle_marker(M::Scrolled(0.0, 600.0), 1_000);
        let in_view = workspace.handle_marker(M::Open(guid), 1_000);
        assert_eq!(unknown.units(), in_view.units() + 1);
    }

    /// Issue #145: F2 on a frame that already has a marker opens that marker; it adds no second.
    #[test]
    fn f2_on_a_frame_with_a_marker_opens_it_instead_of_adding_another() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 63_558);
        send_marker(&mut workspace, M::Close, 63_558);
        assert!(!workspace.markers().is_editing());
        send_marker(&mut workspace, M::Add, 63_558);
        assert_eq!(marker_names(&workspace).len(), 1);
        assert!(workspace.markers().is_editing());
        send_marker(&mut workspace, M::Close, 63_558);
        send_marker(&mut workspace, M::Add, 63_558 + 100);
        assert_eq!(
            marker_names(&workspace).len(),
            1,
            "within the snap is the same marker"
        );
    }

    /// Issue #140: the open clip's unsaved edits are in the recovery journal a second after they
    /// change, and the entry goes once they are saved or undone.
    fn journal_workspace(test_dir: &TestDirectory) -> FolderWorkspace {
        frename_core::recovery::use_dir_on_this_thread(test_dir.file_path("recovery"));
        // The clips are real files: a journal entry says which file it was made on.
        marker_workspace(test_dir, test_dir.directory().files_in_order().count())
    }

    /// One second passes: what the tick would write is written (here, at once).
    fn tick(workspace: &mut FolderWorkspace) {
        if let Some((id, entry)) = workspace.journal_next_write() {
            let result = frename_core::recovery::write(&entry).map_err(|e| e.to_string());
            workspace.journal_written(id, Some((entry, result)));
        }
    }

    fn journal_files(test_dir: &TestDirectory) -> usize {
        std::fs::read_dir(test_dir.file_path("recovery"))
            .map(|dir| {
                dir.filter_map(Result::ok)
                    .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
                    .count()
            })
            .unwrap_or(0)
    }

    fn pick_tag(workspace: &mut FolderWorkspace) {
        let tag_list = workspace.file_workspace().tag_list();
        let id = tag_list
            .filtered_display_tag_ids()
            .iter()
            .find(|id| tag_list.get_tag(**id).is_some_and(|t| t.tag() == "pick"))
            .copied()
            .expect("pick is a built-in tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(id)));
    }

    #[test]
    fn unsaved_edits_reach_the_journal_on_the_tick_and_go_when_saved() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = journal_workspace(&test_dir);
        tick(&mut workspace);
        assert_eq!(journal_files(&test_dir), 0, "nothing changed yet");

        pick_tag(&mut workspace);
        tick(&mut workspace);
        assert_eq!(journal_files(&test_dir), 1, "a tag is journaled");

        // Saving (leaving the clip) applies the edits: the entry goes.
        let (id, snapshot) = workspace.file_workspace().get_snapshot().expect("open");
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        assert_eq!(journal_files(&test_dir), 0, "saved");
    }

    #[test]
    fn the_save_status_follows_the_edits_and_the_journal() {
        use crate::features::file_name_panel::SaveStatus;
        let test_dir = TestDirectory::new(2);
        let mut workspace = journal_workspace(&test_dir);
        assert_eq!(
            workspace.save_status(),
            Some(SaveStatus::Saved),
            "just opened"
        );

        // An edit the journal does not hold yet: nothing is claimed.
        pick_tag(&mut workspace);
        assert_eq!(workspace.save_status(), None, "before the tick");
        tick(&mut workspace);
        assert_eq!(workspace.save_status(), Some(SaveStatus::InRecovery));

        // Undone to what the file holds, with the old entry still on disk for a tick.
        let _ = workspace.update(Message::Undo);
        assert_eq!(workspace.save_status(), Some(SaveStatus::Saved), "undone");

        // Another kind of edit gets the same treatment.
        use iced::widget::text_editor::{Action, Edit};
        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Insert('g'))));
        assert_eq!(workspace.save_status(), None, "before the tick");
        tick(&mut workspace);
        assert_eq!(workspace.save_status(), Some(SaveStatus::InRecovery));
    }

    /// The journal holds the edits: a clip with one pick, ticked once.
    fn in_recovery(test_dir: &TestDirectory) -> FolderWorkspace {
        use crate::features::file_name_panel::SaveStatus;
        let mut workspace = journal_workspace(test_dir);
        pick_tag(&mut workspace);
        tick(&mut workspace);
        assert_eq!(workspace.save_status(), Some(SaveStatus::InRecovery));
        workspace
    }

    fn other_file(workspace: &FolderWorkspace) -> FileId {
        let dir = workspace.directory().expect("dir");
        let open = workspace.file_workspace().file().expect("open").id();
        dir.files_in_order()
            .map(|f| f.id())
            .find(|id| *id != open)
            .expect("second file")
    }

    #[test]
    fn a_clip_whose_save_is_still_pending_is_not_called_saved() {
        use crate::features::file_name_panel::SaveStatus;
        let test_dir = TestDirectory::new(2);
        let mut workspace = journal_workspace(&test_dir);
        // Reopened with edits that are not on disk yet: unsaved though it equals the baseline.
        workspace.journal_force = true;
        assert_eq!(workspace.save_status(), None, "not journaled yet");
        tick(&mut workspace);
        assert_eq!(workspace.save_status(), Some(SaveStatus::InRecovery));
    }

    #[test]
    fn markers_that_are_not_in_the_file_are_not_called_saved() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = journal_workspace(&test_dir);
        let id = workspace.file_workspace().file().expect("open").id();
        workspace.unsaved_markers.insert(id, Vec::new());
        assert_eq!(workspace.save_status(), None);
    }

    #[test]
    fn an_entry_of_another_clip_does_not_make_this_one_safe() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = in_recovery(&test_dir);
        let other = other_file(&workspace);
        let (_, entry) = workspace.journal_written.take().expect("written");
        workspace.journal_written = Some((other, entry));
        assert_eq!(workspace.save_status(), None);
    }

    #[test]
    fn a_failed_journal_write_is_not_called_saved_to_recovery() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = journal_workspace(&test_dir);
        pick_tag(&mut workspace);
        let (id, entry) = workspace.journal_next_write().expect("a write");
        workspace.journal_written(id, Some((entry, Err("disk full".to_string()))));
        assert_eq!(workspace.save_status(), None);
    }

    #[test]
    fn edits_whose_write_is_still_running_are_not_called_saved_to_recovery() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = journal_workspace(&test_dir);
        pick_tag(&mut workspace);
        // The write starts and has not come back.
        let _ = workspace.journal_next_write().expect("a write");
        assert_eq!(workspace.save_status(), None);
    }

    #[test]
    fn a_clip_other_than_the_baseline_says_nothing() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = in_recovery(&test_dir);
        let other = other_file(&workspace);
        let (_, baseline) = workspace.journal_baseline.take().expect("baseline");
        workspace.journal_baseline = Some((other, baseline));
        assert_eq!(workspace.save_status(), None);
    }

    #[test]
    fn a_running_job_says_nothing() {
        use crate::features::file_name_panel::SaveStatus;
        let test_dir = TestDirectory::new(2);
        let mut workspace = journal_workspace(&test_dir);
        let _ = workspace.update(Message::Folder(folder::Message::SetBatchMode(true)));
        assert_eq!(workspace.save_status(), Some(SaveStatus::Saved));
        let _ = workspace.update(Message::Batch(batch::Message::Run));
        assert!(workspace.is_batch_running());
        assert!(workspace.file_workspace().file().is_some());
        assert_eq!(workspace.save_status(), None);
    }

    #[test]
    fn a_clip_renamed_since_its_write_has_no_entry_under_its_name() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = in_recovery(&test_dir);
        let id = workspace.file_workspace().file().expect("open").id();
        let snapshot = workspace.file_workspace().tag_list().file_snapshot();
        let new_path = test_dir.file_path("renamed.mp4");
        let dir = workspace.directory.as_mut().expect("dir");
        assert!(dir.rename_file(id, &new_path, &snapshot));
        assert_eq!(
            workspace.save_status(),
            None,
            "the entry is under the old path"
        );
    }

    #[test]
    fn undoing_everything_takes_the_entry_out_again() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = journal_workspace(&test_dir);
        pick_tag(&mut workspace);
        tick(&mut workspace);
        assert_eq!(journal_files(&test_dir), 1);
        let _ = workspace.update(Message::Undo);
        tick(&mut workspace);
        assert_eq!(journal_files(&test_dir), 0, "back to what is on disk");
    }

    #[test]
    fn a_crash_after_a_tick_is_restored_at_the_next_start() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = journal_workspace(&test_dir);
        pick_tag(&mut workspace);
        tick(&mut workspace);
        drop(workspace); // the process dies here: no save, no close
        let lock = frename_core::recovery::InstanceLock::acquire().expect("the only instance");
        // A second frename must not treat this one's live entries as leftovers.
        let reports = frename_core::recovery::restore_all(&lock);
        match reports.as_slice() {
            [frename_core::recovery::Restored::Applied { summary, .. }] => {
                assert_eq!(summary.tags, 1);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(journal_files(&test_dir), 0, "a restored entry is gone");
    }

    #[test]
    fn a_write_that_lands_after_the_save_does_not_bring_the_entry_back() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = journal_workspace(&test_dir);
        pick_tag(&mut workspace);
        // The tick starts a write...
        let (id, entry) = workspace.journal_next_write().expect("a write is due");
        workspace.journal_in_flight = Some(id);
        let result = frename_core::recovery::write(&entry).map_err(|e| e.to_string());
        // ...the clip is left and saved before the result is delivered...
        let (saved_id, snapshot) = workspace.file_workspace().get_snapshot().expect("open");
        let _ = workspace.update(Message::FileUpdated {
            id: saved_id,
            snapshot,
        });
        // ...then the write's result arrives: the entry it wrote must go.
        workspace.journal_written(id, Some((entry, result)));
        assert_eq!(
            journal_files(&test_dir),
            0,
            "saved edits leave no entry behind"
        );
    }

    #[test]
    fn a_write_that_lands_after_another_clip_opened_keeps_the_entry_of_the_left_one() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = journal_workspace(&test_dir);
        pick_tag(&mut workspace);
        let (id, entry) = workspace.journal_next_write().expect("a write is due");
        workspace.journal_in_flight = Some(id);
        let result = frename_core::recovery::write(&entry).map_err(|e| e.to_string());
        // The user opens the next clip; the left one's save waits for its video to unload.
        let _ = workspace.update(Message::Folder(folder::Message::NextFile));
        flush_file_opened(&mut workspace);
        workspace.journal_written(id, Some((entry, result)));
        assert_eq!(
            journal_files(&test_dir),
            1,
            "unsaved edits keep their entry"
        );
    }

    #[test]
    fn journaling_goes_on_after_the_open_clip_is_renamed_in_place() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = journal_workspace(&test_dir);
        // The clip is renamed on disk and in the directory; the open file keeps its old path
        // until it is opened again.
        let old = test_dir.target_file();
        let new = test_dir.file_path("renamed.mp4");
        std::fs::rename(&old, &new).expect("rename");
        let id = file_id_at(&workspace, 0);
        let snapshot = frename_core::FileTagger::parse(&new, &frename_core::FolderInfo::default());
        workspace
            .directory
            .as_mut()
            .expect("directory")
            .rename_file(id, &new, &snapshot);
        pick_tag(&mut workspace);
        tick(&mut workspace);
        assert_eq!(
            journal_files(&test_dir),
            1,
            "the edits are journaled at the new path"
        );
    }

    #[test]
    fn only_one_journal_write_runs_at_a_time() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = journal_workspace(&test_dir);
        pick_tag(&mut workspace);
        let _first = workspace.journal_tick();
        assert!(workspace.journal_in_flight.is_some());
        let second = workspace.journal_tick();
        assert_eq!(second.units(), 0, "no second write while one runs");
    }

    #[test]
    fn the_start_up_messages_come_as_one_long_note() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = journal_workspace(&test_dir);
        workspace.add_startup_note("Restored one.".to_string());
        workspace.add_startup_note("Kept two.".to_string());
        let task = workspace.show_startup_notes();
        assert_eq!(task.units(), 1, "one note, not one per message");
        assert!(workspace.startup_notes.is_empty());
    }

    #[test]
    fn a_normal_close_leaves_no_journal_behind() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = journal_workspace(&test_dir);
        pick_tag(&mut workspace);
        tick(&mut workspace);
        assert_eq!(journal_files(&test_dir), 1);
        // The window closes: the open clip's edits are saved once its video has unloaded.
        let _ = workspace.flush_open_file();
        let _ = workspace.on_media_unloaded();
        assert_eq!(journal_files(&test_dir), 0);
    }

    #[test]
    fn escape_closes_an_open_marker_row_first() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = workspace.file_workspace().markers().unwrap()[0]
            .guid
            .clone()
            .unwrap();
        send_marker(&mut workspace, M::Open(guid), 1_000);
        assert!(workspace.markers().is_editing());
        let _ = workspace.update(Message::EscapePressed);
        assert!(!workspace.markers().is_editing());
    }

    #[test]
    fn shift_f2_deletes_the_marker_under_the_playhead_and_undo_brings_it_back() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        send_marker(&mut workspace, M::DeleteAtPlayhead, 3_000);
        assert_eq!(marker_names(&workspace).len(), 1, "too far away");
        send_marker(&mut workspace, M::DeleteAtPlayhead, 1_400);
        assert!(marker_names(&workspace).is_empty());
        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_names(&workspace), [(1_000, String::new())]);
    }

    /// A drag out of the window waits for the open file's edits to be on disk: a toggled tag is
    /// not, until the file is saved.
    #[test]
    fn the_open_file_is_unsaved_for_a_drag_until_its_edits_are_on_disk() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let id = file_id_at(&workspace, 0);
        assert!(!workspace.open_file_unsaved(id), "just opened");

        let tag_list = workspace.file_workspace().tag_list();
        let tag_id = tag_list
            .filtered_display_tag_ids()
            .iter()
            .find(|t| tag_list.get_tag(**t).is_some_and(|t| t.tag() == "pick"))
            .copied()
            .expect("pick is a built-in tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(tag_id)));
        assert!(workspace.open_file_unsaved(id), "tag toggled");

        let (_, snapshot) = workspace
            .file_workspace()
            .get_snapshot()
            .expect("a file is open");
        let _ = workspace.update(Message::FileUpdated { id, snapshot });
        assert!(!workspace.open_file_unsaved(id), "saved");
    }

    /// A folder with the open file's tags changed (not saved yet), for the drag-out tests.
    fn workspace_with_unsaved_tag(test_dir: &TestDirectory) -> FolderWorkspace {
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        let tag_list = workspace.file_workspace().tag_list();
        let tag_id = tag_list
            .filtered_display_tag_ids()
            .iter()
            .find(|t| tag_list.get_tag(**t).is_some_and(|t| t.tag() == "pick"))
            .copied()
            .expect("pick is a built-in tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(tag_id)));
        workspace
    }

    fn drag_move(workspace: &mut FolderWorkspace, x: f32) {
        let _ = workspace.update(Message::DragOut(crate::features::drag_out::Message::Moved(
            iced::Point::new(x, 0.0),
        )));
    }

    /// The save a drag asks for counts once it ran, not when it was asked for: a move while its
    /// message is still queued (or its video unloads) waits instead of refusing the drag, and
    /// the drag starts once the file is on disk.
    #[test]
    fn a_drag_waits_for_the_save_it_asked_for_then_starts() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = workspace_with_unsaved_tag(&test_dir);
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        drag_move(&mut workspace, 0.0);
        drag_move(&mut workspace, 10.0);
        assert_eq!(
            workspace.drag_out.save_of(file_id_at(&workspace, 0)),
            Some(crate::features::drag_out::Save::Pending),
            "a save is asked for"
        );
        // No task delivered yet: the save has not run.
        drag_move(&mut workspace, 11.0);
        assert!(workspace.drag_out.is_pressed(), "waits, not refused");

        // The save runs: the same-file refresh unloads the video, then saves.
        flush_file_opened(&mut workspace);
        drag_move(&mut workspace, 12.0);
        assert!(workspace.drag_out.is_pressed(), "waits for the unload");
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        drag_move(&mut workspace, 13.0);
        assert!(
            workspace.drag_out.is_dragging(),
            "saved, so the drag starts"
        );
    }

    /// When the save the drag asked for ran and the file still is not on disk as edited (here:
    /// its markers could not be written), the drag is refused and nothing stays armed.
    /// Batch mode: pressing checked row B opens B and saves A, which is checked too. A's save
    /// failing (here its markers could not be written) refuses the drag of both, instead of
    /// dragging A under its old name.
    #[test]
    fn a_batch_drag_is_refused_when_the_file_the_press_left_failed_to_save() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = workspace_with_unsaved_tag(&test_dir);
        let (a, b) = (file_id_at(&workspace, 0), file_id_at(&workspace, 1));
        let _ = workspace.update(Message::Folder(folder::Message::SetBatchMode(true)));
        let _ = workspace.update(Message::Batch(batch::Message::CheckAll(vec![a])));
        let _ = workspace.update(Message::Batch(batch::Message::Toggle(b)));
        assert!(workspace.batch().is_checked(a) && workspace.batch().is_checked(b));
        workspace.unsaved_markers.insert(a, Vec::new());
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        drag_move(&mut workspace, 0.0);
        drag_move(&mut workspace, 10.0);
        assert!(
            !workspace.drag_out.is_dragging(),
            "A did not save: no drag of it under its old name"
        );
    }

    #[test]
    fn a_drag_is_refused_when_its_save_failed() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = workspace_with_unsaved_tag(&test_dir);
        let id = file_id_at(&workspace, 0);
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        workspace.unsaved_markers.insert(id, Vec::new());
        drag_move(&mut workspace, 0.0);
        drag_move(&mut workspace, 10.0);
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        drag_move(&mut workspace, 11.0);
        assert!(
            !workspace.drag_out.is_pressed() && !workspace.drag_out.is_dragging(),
            "refused: idle again"
        );
    }

    /// A double-click opens the rename editor: the held button then selects its text, it does
    /// not drag the file.
    #[test]
    fn opening_the_rename_editor_disarms_the_drag() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = workspace_with_unsaved_tag(&test_dir);
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        assert!(workspace.drag_out.is_pressed());
        let _ = workspace.update(Message::Folder(folder::Message::StartRename(0)));
        assert!(!workspace.drag_out.is_pressed());
        // A second press of the double-click arms again; its moves still do not drag.
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        drag_move(&mut workspace, 0.0);
        drag_move(&mut workspace, 10.0);
        assert!(!workspace.drag_out.is_pressed() && !workspace.drag_out.is_dragging());
    }

    #[test]
    fn a_renamed_open_file_keeps_the_markers_edited_after_the_rename() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let (id, mut snapshot) = workspace.file_workspace().get_snapshot().expect("open");
        snapshot.set_tags(["Goat"]);
        let _ = workspace.update(Message::FileUpdated { id, snapshot });

        let guid = workspace.file_workspace().markers().unwrap()[0]
            .guid
            .clone()
            .unwrap();
        send_marker(&mut workspace, M::Open(guid), 1_000);
        send_marker(&mut workspace, type_name("after"), 1_000);
        // The refresh opens the same file under its new name: nothing is read from disk again.
        flush_file_opened(&mut workspace);
        assert!(workspace
            .current_file()
            .is_some_and(|f| f.file_path().ends_with("Goat.file_0.mp4")));
        assert_eq!(marker_names(&workspace), [(1_000, "after".to_string())]);
    }

    #[test]
    fn opening_a_path_that_does_not_exist_keeps_the_open_file() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let _ = workspace.update(Message::OpenPath(test_dir.file_path("gone.mp4")));
        assert!(workspace.file_workspace().file().is_some());
        assert!(!workspace.is_loading());
    }

    // --- Multi-select (Ctrl+click / Shift+click), issue #60 ---

    /// Opens a fresh folder of `file_count` files with `file_0` open, ready for `SelectFile`.
    fn open_folder(file_count: usize) -> (TestDirectory, FolderWorkspace) {
        let test_dir = TestDirectory::new(file_count);
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        flush_file_opened(&mut workspace);
        (test_dir, workspace)
    }

    fn ctrl_click(workspace: &mut FolderWorkspace, index: usize) {
        let _ = workspace.update(Message::ModifiersChanged(Modifiers::COMMAND));
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(index)));
        flush_file_opened(workspace);
        let _ = workspace.update(Message::ModifiersChanged(Modifiers::empty()));
    }

    fn shift_click(workspace: &mut FolderWorkspace, index: usize) {
        let _ = workspace.update(Message::ModifiersChanged(Modifiers::SHIFT));
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(index)));
        flush_file_opened(workspace);
        let _ = workspace.update(Message::ModifiersChanged(Modifiers::empty()));
    }

    #[test]
    fn the_modifiers_held_are_tracked_from_modifiers_changed_and_cleared_on_unfocus() {
        let mut workspace = test_workspace();
        let _ = workspace.update(Message::ModifiersChanged(Modifiers::COMMAND));
        assert!(workspace.modifiers.command());
        let _ = workspace.update(Message::ModifiersChanged(Modifiers::empty()));
        assert!(!workspace.modifiers.command());
    }

    #[test]
    fn ctrl_click_starts_multi_select_with_the_open_file_and_the_clicked_one() {
        let (_test_dir, mut workspace) = open_folder(3);
        let (a, b) = (file_id_at(&workspace, 0), file_id_at(&workspace, 2));
        ctrl_click(&mut workspace, 2);
        assert!(
            workspace.batch().is_active(),
            "ctrl+click starts multi-select"
        );
        assert!(workspace.batch().is_checked(a) && workspace.batch().is_checked(b));
        assert_eq!(workspace.batch().checked_count(), 2);
        assert!(
            workspace.current_file().is_some_and(|f| f.id() == b),
            "the clicked file becomes the open one"
        );
    }

    #[test]
    fn ctrl_click_on_the_open_file_itself_starts_multi_select_with_just_it() {
        let (_test_dir, mut workspace) = open_folder(3);
        let a = file_id_at(&workspace, 0);
        ctrl_click(&mut workspace, 0);
        assert!(workspace.batch().is_active());
        assert_eq!(workspace.batch().checked_count(), 1);
        assert!(workspace.batch().is_checked(a));
    }

    #[test]
    fn a_further_ctrl_click_toggles_a_single_file_in_and_then_out() {
        let (_test_dir, mut workspace) = open_folder(3);
        let c = file_id_at(&workspace, 2);
        ctrl_click(&mut workspace, 1);
        ctrl_click(&mut workspace, 2);
        assert!(workspace.batch().is_checked(c), "toggled in");
        assert_eq!(workspace.batch().checked_count(), 3);
        ctrl_click(&mut workspace, 2);
        assert!(!workspace.batch().is_checked(c), "toggled back out");
        assert_eq!(workspace.batch().checked_count(), 2);
    }

    #[test]
    fn shift_click_checks_every_file_from_the_anchor_to_the_clicked_one_inclusive() {
        let (_test_dir, mut workspace) = open_folder(5);
        let ids: Vec<FileId> = (0..5).map(|i| file_id_at(&workspace, i)).collect();
        ctrl_click(&mut workspace, 3); // anchor = file_3, checked = {file_0, file_3}
        shift_click(&mut workspace, 1); // range [1, 3]
        assert_eq!(workspace.batch().checked_count(), 3);
        for (i, id) in ids.iter().enumerate().take(4).skip(1) {
            assert!(workspace.batch().is_checked(*id), "file {i} in range");
        }
        assert!(
            !workspace.batch().is_checked(ids[0]),
            "outside the range now"
        );
        assert!(
            !workspace.batch().is_checked(ids[4]),
            "outside the range now"
        );
    }

    #[test]
    fn repeated_shift_clicks_range_from_the_same_anchor() {
        let (_test_dir, mut workspace) = open_folder(5);
        let ids: Vec<FileId> = (0..5).map(|i| file_id_at(&workspace, i)).collect();
        ctrl_click(&mut workspace, 3); // anchor = file_3
        shift_click(&mut workspace, 1); // checked = {1, 2, 3}
        shift_click(&mut workspace, 4); // anchor is still file_3: checked = {3, 4}
        assert_eq!(workspace.batch().checked_count(), 2);
        assert!(workspace.batch().is_checked(ids[3]) && workspace.batch().is_checked(ids[4]));
        assert!(!workspace.batch().is_checked(ids[1]) && !workspace.batch().is_checked(ids[2]));
    }

    #[test]
    fn ctrl_click_after_shift_click_moves_the_anchor() {
        let (_test_dir, mut workspace) = open_folder(5);
        let ids: Vec<FileId> = (0..5).map(|i| file_id_at(&workspace, i)).collect();
        ctrl_click(&mut workspace, 3); // anchor = file_3
        shift_click(&mut workspace, 1); // checked = {1, 2, 3}, anchor unchanged
        ctrl_click(&mut workspace, 0); // toggles file_0 in; anchor moves to file_0
        assert_eq!(workspace.select_anchor, Some(ids[0]));
        shift_click(&mut workspace, 2); // ranges from the new anchor: [0, 2]
        assert_eq!(workspace.batch().checked_count(), 3);
        for id in ids.iter().take(3) {
            assert!(workspace.batch().is_checked(*id));
        }
    }

    #[test]
    fn a_shift_click_with_an_anchor_no_longer_listed_checks_only_the_clicked_file() {
        let (_test_dir, mut workspace) = open_folder(3);
        // An anchor from a file that has since left the list under a filter (or, as here,
        // simply does not exist any more) cannot define a range: fall back to just the click.
        workspace.select_anchor = Some(FileId::new());
        let clicked = file_id_at(&workspace, 2);
        shift_click(&mut workspace, 2);
        assert_eq!(workspace.batch().checked_count(), 1);
        assert!(workspace.batch().is_checked(clicked));
    }

    #[test]
    fn a_plain_click_during_multi_select_only_changes_the_open_file() {
        let (_test_dir, mut workspace) = open_folder(3);
        let (a, b) = (file_id_at(&workspace, 0), file_id_at(&workspace, 1));
        ctrl_click(&mut workspace, 1); // checked = {a, b}, multi-select active
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        flush_file_opened(&mut workspace);
        assert!(
            workspace.batch().is_active(),
            "a plain click does not leave multi-select"
        );
        assert!(
            workspace.batch().is_checked(a) && workspace.batch().is_checked(b),
            "checks are unaffected, as clicking any row already did before this feature"
        );
        assert!(workspace.current_file().is_some_and(|f| f.id() == a));
    }

    #[test]
    fn escape_leaves_multi_select_and_clears_the_anchor() {
        let (_test_dir, mut workspace) = open_folder(3);
        ctrl_click(&mut workspace, 1);
        assert!(workspace.batch().is_active());
        let _ = workspace.update(Message::EscapePressed);
        assert!(!workspace.batch().is_active());
        assert_eq!(workspace.select_anchor, None);
    }

    #[test]
    fn turning_batch_mode_off_clears_the_anchor() {
        let (_test_dir, mut workspace) = open_folder(3);
        ctrl_click(&mut workspace, 1);
        let _ = workspace.update(Message::Folder(folder::Message::SetBatchMode(false)));
        assert_eq!(workspace.select_anchor, None);
    }

    /// A running job ignores `SetActive(false)` (`BatchState::update`'s own guard), so batch
    /// mode stays on: the anchor must stay too, or a Shift+click once the job ends would range
    /// from the wrong file with no sign anything changed.
    #[test]
    fn escape_during_a_running_batch_job_keeps_batch_mode_and_the_anchor() {
        let (_test_dir, mut workspace) = open_folder(3);
        ctrl_click(&mut workspace, 1);
        ctrl_click(&mut workspace, 2); // anchor = file_2
        let anchor = workspace.select_anchor;
        let _ = workspace.update(Message::Batch(batch::Message::Run));
        assert!(
            workspace.is_batch_running(),
            "the job must actually be running"
        );
        let _ = workspace.update(Message::EscapePressed);
        assert!(
            workspace.batch().is_active(),
            "SetActive(false) had no effect while the job runs"
        );
        assert_eq!(
            workspace.select_anchor, anchor,
            "the anchor must not be dropped either"
        );
    }

    /// Ctrl/Shift+click must index the *filtered* list (as the issue's own "in the list's
    /// current order (with the current filter/search applied)" says), not the unfiltered one.
    #[test]
    fn multi_select_indexes_the_filtered_list_not_the_unfiltered_one() {
        let (_test_dir, mut workspace) = open_folder(5);
        let ids: Vec<FileId> = (0..5).map(|i| file_id_at(&workspace, i)).collect();
        let _ = workspace.update(Message::Folder(folder::Message::SetUntaggedOnly(true)));
        // Tags file_2, dropping it out of the untagged-only list: filtered order becomes
        // [file_0, file_1, file_3, file_4] (file_2 is not the selected file, so it is hidden).
        let snapshot =
            FileSnapshot::new(vec!["Comedy".to_string()], "file_2", ".mp4", "file_2.mp4");
        let _ = workspace.update(Message::FileUpdated {
            id: ids[2],
            snapshot,
        });
        assert_eq!(
            workspace
                .directory()
                .unwrap()
                .files_in_order()
                .map(|f| f.id())
                .collect::<Vec<_>>(),
            vec![ids[0], ids[1], ids[3], ids[4]],
            "file_2 must be filtered out"
        );

        ctrl_click(&mut workspace, 2); // filtered index 2 = file_3; anchor = file_3
        shift_click(&mut workspace, 3); // filtered index 3 = file_4: range [file_3, file_4]

        assert_eq!(workspace.batch().checked_count(), 2);
        assert!(workspace.batch().is_checked(ids[3]) && workspace.batch().is_checked(ids[4]));
        assert!(!workspace.batch().is_checked(ids[0]));
        assert!(
            !workspace.batch().is_checked(ids[2]),
            "file_2 was never a real position in the filtered range"
        );
    }

    /// The anchor is stored by `FileId`, not by the index it had when set — it must keep
    /// pointing at the right file once a filter that changed the list's shape is turned off.
    #[test]
    fn the_anchor_keeps_working_after_the_filter_that_hid_other_files_is_turned_off() {
        let (_test_dir, mut workspace) = open_folder(5);
        let ids: Vec<FileId> = (0..5).map(|i| file_id_at(&workspace, i)).collect();
        let _ = workspace.update(Message::Folder(folder::Message::SetUntaggedOnly(true)));
        let snapshot =
            FileSnapshot::new(vec!["Comedy".to_string()], "file_2", ".mp4", "file_2.mp4");
        let _ = workspace.update(Message::FileUpdated {
            id: ids[2],
            snapshot,
        });
        // Filtered order: [0, 1, 3, 4]; ctrl-click filtered index 2 = file_3, becoming the anchor.
        ctrl_click(&mut workspace, 2);
        assert_eq!(workspace.select_anchor, Some(ids[3]));

        let _ = workspace.update(Message::Folder(folder::Message::SetUntaggedOnly(false)));
        // Unfiltered order is [0,1,2,3,4] again; file_3 (the anchor) is now at index 3.
        shift_click(&mut workspace, 1); // range [1, 3] in the now-unfiltered order
        assert_eq!(workspace.batch().checked_count(), 3);
        for id in [ids[1], ids[2], ids[3]] {
            assert!(workspace.batch().is_checked(id));
        }
        assert!(!workspace.batch().is_checked(ids[0]) && !workspace.batch().is_checked(ids[4]));
    }

    /// A file menu action on the open file waits for its pending edits to be saved (here a
    /// tag, which renames it), and then sees the file's new name.
    #[test]
    fn a_file_action_saves_the_open_files_edits_first_and_sees_the_new_name() {
        use crate::features::file_menu::{system, FileAction};
        let test_dir = TestDirectory::new(2);
        let mut workspace = workspace_with_unsaved_tag(&test_dir);
        let id = file_id_at(&workspace, 0);
        let path_of = |workspace: &FolderWorkspace| {
            workspace
                .directory()
                .and_then(|d| d.file_by_id(id))
                .map(|f| f.file_path().to_path_buf())
                .expect("the file is listed")
        };

        let _ = workspace.update(Message::FileAction(FileAction::CopyName));
        assert_eq!(
            workspace.pending_file_action,
            Some((id, FileAction::CopyName)),
            "waits for the save it asked for"
        );
        assert_eq!(path_of(&workspace), test_dir.file_path("file_0.mp4"));

        // The save runs: the same-file refresh unloads the video, then saves.
        flush_file_opened(&mut workspace);
        assert!(workspace.pending_file_action.is_some(), "still unloading");
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Unloaded,
        ));
        assert_eq!(workspace.pending_file_action, None, "ran after the save");
        let name = system::clipboard_text(FileAction::CopyName, &path_of(&workspace));
        assert_eq!(name.as_deref(), Some("pick.file_0.mp4"));
    }

    /// A file with nothing to save (another file in the list, or the open one already saved)
    /// runs the action at once.
    #[test]
    fn a_file_action_on_a_file_with_nothing_to_save_does_not_wait() {
        use crate::features::file_menu::{self, FileAction};
        let (_test_dir, mut workspace) = open_folder(2);
        let _ = workspace.update(Message::FileAction(FileAction::ShowInFileManager));
        assert_eq!(workspace.pending_file_action, None);
        let other = file_id_at(&workspace, 1);
        let _ = workspace.update(Message::FileMenu(file_menu::Message::Choose(
            other,
            FileAction::CopyPath,
        )));
        assert_eq!(workspace.pending_file_action, None);
    }

    /// The waiting action runs on its own file's save only, and not when that save was refused
    /// (the notice of the refused save says why).
    #[test]
    fn a_waiting_file_action_runs_on_its_files_save_unless_it_was_refused() {
        use crate::features::file_menu::FileAction;
        let (_test_dir, mut workspace) = open_folder(2);
        let (open, other) = (file_id_at(&workspace, 0), file_id_at(&workspace, 1));
        workspace.pending_file_action = Some((open, FileAction::CopyPath));
        assert_eq!(workspace.file_action_after_save(other, false).units(), 0);
        assert!(
            workspace.pending_file_action.is_some(),
            "another file's save"
        );
        assert_eq!(workspace.file_action_after_save(open, false).units(), 1);
        assert_eq!(workspace.pending_file_action, None);

        workspace.pending_file_action = Some((open, FileAction::CopyPath));
        assert_eq!(workspace.file_action_after_save(open, true).units(), 0);
        assert_eq!(workspace.pending_file_action, None, "dropped");
    }

    /// Esc closes an open file menu and does nothing else.
    #[test]
    fn escape_closes_the_file_menu_first() {
        use crate::features::file_menu;
        let (_test_dir, mut workspace) = open_folder(2);
        let _ = workspace.update(Message::FileNamePanel(
            crate::features::file_name_panel::Message::OpenFileMenu,
        ));
        assert!(!workspace.file_menu().is_open(), "no right press: no menu");
        let _ = workspace.update(Message::FileMenu(file_menu::Message::RightPressed(
            iced::Point::new(10.0, 10.0),
        )));
        let _ = workspace.update(Message::Folder(folder::Message::OpenFileMenu(1)));
        // The row's message becomes the menu's own `Open` (a task in the app).
        let _ = workspace.update(Message::FileMenu(file_menu::Message::Open(file_id_at(
            &workspace, 1,
        ))));
        assert!(workspace.file_menu().is_open());
        workspace.file_workspace.set_tag_filter("pi".to_string());
        let _ = workspace.update(Message::EscapePressed);
        assert!(!workspace.file_menu().is_open());
        assert_eq!(
            workspace.file_workspace().tag_list().filter_query(),
            "pi",
            "the search is cleared by the next Esc, not this one"
        );
    }

    // --- Issue #139: every single-file edit is undoable ---

    fn open_file_name(workspace: &FolderWorkspace) -> String {
        workspace
            .file_workspace()
            .get_snapshot()
            .expect("a file is open")
            .1
            .file_name()
    }

    fn comment_of(workspace: &FolderWorkspace) -> (String, String) {
        (
            workspace.file_workspace().tag_list().comment().to_string(),
            workspace
                .file_workspace()
                .comment_content
                .text()
                .trim_end_matches('\n')
                .to_string(),
        )
    }

    #[test]
    fn an_inline_rename_undoes_and_redoes() {
        let (_test_dir, mut workspace) = open_folder(1);
        let _ = workspace.update(Message::Folder(folder::Message::StartRename(0)));
        let _ = workspace.update(Message::Folder(folder::Message::RenameInput(
            "renamed.mp4".to_string(),
        )));
        let _ = workspace.update(Message::Folder(folder::Message::SubmitRename));
        assert_eq!(open_file_name(&workspace), "renamed.mp4");

        let _ = workspace.update(Message::Undo);
        assert_eq!(open_file_name(&workspace), "file_0.mp4");
        let _ = workspace.update(Message::Redo);
        assert_eq!(open_file_name(&workspace), "renamed.mp4");
    }

    #[test]
    fn typing_in_the_comment_box_is_one_undo_step_per_focus_session() {
        use iced::widget::text_editor::{Action, Edit};
        let (_test_dir, mut workspace) = open_folder(1);
        for c in ['h', 'i'] {
            let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Insert(c))));
        }
        // Ctrl+Z belongs to the box while it has the keys.
        let _ = workspace.update(Message::Undo);
        assert_eq!(comment_of(&workspace).0, "hi");

        let _ = workspace.update(Message::CommentFocused(false));
        let _ = workspace.update(Message::Undo);
        assert_eq!(comment_of(&workspace), (String::new(), String::new()));
        let _ = workspace.update(Message::Redo);
        assert_eq!(comment_of(&workspace), ("hi".to_string(), "hi".to_string()));
    }

    #[test]
    fn a_comment_typed_before_leaving_the_file_is_a_step_of_that_file() {
        use iced::widget::text_editor::{Action, Edit};
        let (_test_dir, mut workspace) = open_folder(2);
        let _ = workspace.update(Message::CommentAction(Action::Edit(Edit::Insert('x'))));
        assert!(!workspace.history.can_undo(), "still being typed");
        let _ = workspace.update(Message::Folder(folder::Message::NextFile));
        flush_file_opened(&mut workspace);
        assert!(workspace.history.can_undo(), "the typing is a step now");
        assert_eq!(comment_of(&workspace).0, "", "the next file has its own");
    }

    #[test]
    fn a_marker_name_is_one_undo_step_per_row() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = first_marker_guid(&workspace);
        send_marker(&mut workspace, M::Open(guid), 1_000);
        send_marker(&mut workspace, type_name("hel"), 1_000);
        send_marker(&mut workspace, type_name("lo"), 1_000);
        send_marker(&mut workspace, M::Close, 1_000);
        assert_eq!(marker_names(&workspace), [(1_000, "hello".to_string())]);

        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_names(&workspace), [(1_000, String::new())]);
        let _ = workspace.update(Message::Redo);
        assert_eq!(marker_names(&workspace), [(1_000, "hello".to_string())]);
    }

    #[test]
    fn a_marker_name_typed_in_an_open_row_is_undone_by_ctrl_z() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = first_marker_guid(&workspace);
        send_marker(&mut workspace, M::Open(guid), 1_000);
        send_marker(&mut workspace, type_name("name"), 1_000);
        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_names(&workspace), [(1_000, String::new())]);
    }

    #[test]
    fn sync_up_and_the_lock_undo_and_redo() {
        use crate::features::sync_panel;
        let (_test_dir, mut workspace) = open_folder(1);
        let locked = |w: &FolderWorkspace| w.file_workspace().tag_list().sync_locked();
        let _ = workspace.update(Message::SyncPanel(sync_panel::Message::ToggleLock));
        assert!(!locked(&workspace));
        let _ = workspace.update(Message::SyncPanel(sync_panel::Message::SyncUp));
        assert!(locked(&workspace));
        let _ = workspace.update(Message::SyncPanel(sync_panel::Message::ToggleLock));
        assert!(!locked(&workspace));

        let _ = workspace.update(Message::Undo);
        assert!(locked(&workspace), "the unlock is undone");
        let _ = workspace.update(Message::Undo);
        assert!(!locked(&workspace), "Sync up and its lock are undone");
        let _ = workspace.update(Message::Undo);
        assert!(locked(&workspace), "the first unlock is undone");
        let _ = workspace.update(Message::Redo);
        assert!(!locked(&workspace));
        let _ = workspace.update(Message::Redo);
        assert!(locked(&workspace), "redo of Sync up locks again");
    }

    #[test]
    fn an_edit_after_an_undo_clears_redo() {
        let (_test_dir, mut workspace) = open_folder(1);
        let _ = workspace.update(Message::Folder(folder::Message::StartRename(0)));
        let _ = workspace.update(Message::Folder(folder::Message::RenameInput(
            "a.mp4".to_string(),
        )));
        let _ = workspace.update(Message::Folder(folder::Message::SubmitRename));
        let _ = workspace.update(Message::Undo);
        assert!(workspace.history.can_redo());

        let _ = workspace.update(Message::Folder(folder::Message::StartRename(0)));
        let _ = workspace.update(Message::Folder(folder::Message::RenameInput(
            "b.mp4".to_string(),
        )));
        let _ = workspace.update(Message::Folder(folder::Message::SubmitRename));
        assert!(
            !workspace.history.can_redo(),
            "a redo of `a` would be stale"
        );
    }

    /// Two navigations before the video unloads: each leaves a file with a save still waiting,
    /// and each gets its own step, from the file left to the file reached.
    #[test]
    fn two_quick_navigations_are_two_undo_steps() {
        let (_test_dir, mut workspace) = open_folder(3);
        let first = file_id_at(&workspace, 0);
        let second = file_id_at(&workspace, 1);
        let snapshot_of = |w: &FolderWorkspace| w.file_workspace().get_snapshot().expect("open");

        let (id0, snap0) = snapshot_of(&workspace);
        assert_eq!(id0, first);
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        let (id1, snap1) = snapshot_of(&workspace);
        assert_eq!(id1, second);
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(2)));
        flush_file_opened(&mut workspace);

        let _ = workspace.update(Message::FileUpdated {
            id: id0,
            snapshot: snap0,
        });
        let _ = workspace.update(Message::FileUpdated {
            id: id1,
            snapshot: snap1,
        });
        assert_eq!(workspace.directory().unwrap().selected_index(), Some(2));
        let _ = workspace.update(Message::Undo);
        assert_eq!(workspace.directory().unwrap().selected_index(), Some(1));
        let _ = workspace.update(Message::Undo);
        assert_eq!(workspace.directory().unwrap().selected_index(), Some(0));
    }

    /// Shift+F2 on a marker without a GUID (written by another tool) says it is read-only.
    #[test]
    fn shift_f2_on_a_read_only_marker_says_so() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let mut marker = frename_core::Marker::new(1_000);
        marker.guid = None;
        workspace.file_workspace.tag_list_mut().add_marker(marker);
        let before = marker_names(&workspace);
        assert_eq!(before.len(), 1);
        let _ = workspace.handle_marker(crate::features::markers::Message::DeleteAtPlayhead, 1_000);
        assert_eq!(marker_names(&workspace), before, "it is not deleted");
    }

    #[test]
    fn undo_closes_a_renamed_marker_row_even_with_nothing_else_to_undo() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        let guid = first_marker_guid(&workspace);
        // Nothing earlier in the history: as if the marker came from the file.
        workspace.history = super::WorkspaceHistory::new(super::HISTORY_DEPTH);
        send_marker(&mut workspace, M::Open(guid), 1_000);
        send_marker(&mut workspace, type_name("x"), 1_000);
        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_names(&workspace), [(1_000, String::new())]);
    }

    #[test]
    fn opening_another_marker_row_makes_the_first_rows_name_a_step() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        send_marker(&mut workspace, M::Add, 1_000);
        send_marker(&mut workspace, M::AddRange(5_000, 7_000), 5_000);
        let guids: Vec<String> = workspace
            .file_workspace()
            .markers()
            .unwrap()
            .iter()
            .map(|m| m.guid.clone().unwrap())
            .collect();
        send_marker(&mut workspace, M::Open(guids[0].clone()), 1_000);
        send_marker(&mut workspace, type_name("first"), 1_000);
        send_marker(&mut workspace, M::Open(guids[1].clone()), 5_000);
        send_marker(&mut workspace, M::Close, 5_000);
        let _ = workspace.update(Message::Undo);
        assert_eq!(marker_names(&workspace)[0], (1_000, String::new()));
        assert_eq!(marker_names(&workspace).len(), 2, "no marker was undone");
    }

    /// Issue #142: fullscreen builds the lists anew, so their scroll offsets are kept in the
    /// state, carried by the toggle's task and put back, both ways.
    #[test]
    fn the_list_scrolls_are_kept_across_fullscreen() {
        use crate::features::markers::Message as M;
        let test_dir = TestDirectory::new(1);
        let mut workspace = marker_workspace(&test_dir, 1);
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Video(
                crate::features::media_viewer::video::Message::Markers(M::Scrolled(120.0, 400.0)),
            ),
        ));
        assert_eq!(workspace.markers.scroll_y(), 120.0);

        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Video(
                crate::features::media_viewer::video::Message::CueListScrolled(80.0, 300.0),
            ),
        ));
        assert!(matches!(
            workspace.restore_lists_message(),
            Message::RestoreListScrolls { markers_y, cues_y } if markers_y == 120.0 && cues_y == 80.0
        ));
        for expect_fullscreen in [true, false] {
            let task = workspace.update(Message::ToggleMediaFullscreen);
            assert_eq!(workspace.media_fullscreen, expect_fullscreen);
            assert_eq!(
                task.units(),
                1,
                "the toggle asks for the lists to be put back"
            );
        }
        // The fresh list reports offset 0 before the restore runs: the offset carried by the
        // task is what is put back, and the report of the restore ends the wait.
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Video(
                crate::features::media_viewer::video::Message::Markers(M::Scrolled(0.0, 400.0)),
            ),
        ));
        let task = workspace.update(Message::RestoreListScrolls {
            markers_y: 120.0,
            cues_y: 80.0,
        });
        assert_eq!(
            task.units(),
            2,
            "marker list and subtitle list are scrolled back"
        );
        assert_eq!(workspace.media_viewer.cue_scroll_y(), 80.0);
        let _ = workspace.update(Message::MediaViewer(
            crate::features::media_viewer::Message::Video(
                crate::features::media_viewer::video::Message::Markers(M::Scrolled(120.0, 300.0)),
            ),
        ));
        assert_eq!(workspace.markers.scroll_y(), 120.0);
    }

    /// A workspace whose recent folders are `names` under `/shoots`, newest first, kept in a
    /// database of its own: tests running at the same time never share the list.
    fn workspace_with_recent_folders(names: &[&str]) -> FolderWorkspace {
        let dir = unique_test_folder();
        std::fs::create_dir_all(&dir).expect("create test folder");
        let store = AppDatabase::with_path(dir.join("recent.db"));
        store.initialize().expect("migrate");
        for (age, name) in names.iter().enumerate().rev() {
            store.record_recent_folder(
                &frename_core::FolderAndFile::new(format!("/shoots/{name}"), None::<PathBuf>),
                2_000 - age as i64,
            );
        }
        let mut workspace = FolderWorkspace::with_app_db(store);
        workspace
            .recent_folders
            .set_entries(workspace.app_db.get_recent_folders(), 0);
        workspace
    }

    #[test]
    fn the_recent_folders_list_takes_the_arrows_enter_and_escape_only_while_open() {
        let mut workspace = workspace_with_recent_folders(&["a", "b"]);
        let down = Message::TagPanel(tag_panel::Message::SelectDown);
        assert!(workspace.recent_folders_key(&down).is_none(), "closed");

        let _ = workspace.update(Message::RecentFolders(recent_folders::Message::Toggle));
        assert!(workspace.recent_folders.is_open());
        assert!(matches!(
            workspace.recent_folders_key(&down),
            Some(Some(recent_folders::Message::Move(1)))
        ));
        assert!(matches!(
            workspace.recent_folders_key(&Message::TagPanel(tag_panel::Message::SelectUp)),
            Some(Some(recent_folders::Message::Move(-1)))
        ));
        assert!(matches!(
            workspace.recent_folders_key(&Message::SaveSelectedTag),
            Some(Some(recent_folders::Message::ChooseHighlighted))
        ));
        assert!(matches!(
            workspace.recent_folders_key(&Message::EscapePressed),
            Some(Some(recent_folders::Message::Escape))
        ));
        // The keys it has no use for do nothing behind it.
        assert!(matches!(
            workspace.recent_folders_key(&Message::TagPanel(tag_panel::Message::SelectLeft)),
            Some(None)
        ));
        assert!(matches!(
            workspace.recent_folders_key(&Message::Folder(folder::Message::NextFile)),
            Some(None)
        ));
        assert!(workspace.recent_folders_key(&Message::Noop).is_none());
    }

    #[test]
    fn walking_the_list_and_escape_work_through_the_workspace() {
        let mut workspace = workspace_with_recent_folders(&["a", "b"]);
        let _ = workspace.update(Message::RecentFolders(recent_folders::Message::Toggle));
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::SelectDown));
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::SelectDown));
        assert_eq!(workspace.recent_folders.highlight(), Some(1));
        let _ = workspace.update(Message::EscapePressed);
        assert!(!workspace.recent_folders.is_open());
    }

    #[test]
    fn a_folder_that_opens_closes_a_list_left_open() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = workspace_with_recent_folders(&["a"]);
        let _ = workspace.update(Message::RecentFolders(recent_folders::Message::Toggle));
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        assert!(!workspace.recent_folders.is_open());
    }

    #[test]
    fn the_list_cannot_be_opened_in_fullscreen() {
        let mut workspace = workspace_with_recent_folders(&["a"]);
        workspace.media_fullscreen = true;
        let _ = workspace.update(Message::RecentFolders(recent_folders::Message::Toggle));
        assert!(!workspace.recent_folders.is_open());
    }

    #[test]
    fn fullscreen_closes_the_open_recent_folders_list() {
        let test_dir = TestDirectory::new(1);
        let mut workspace = workspace_with_recent_folders(&["a"]);
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        let _ = workspace.update(Message::RecentFolders(recent_folders::Message::Toggle));
        assert!(workspace.recent_folders.is_open());
        let _ = workspace.set_fullscreen(true);
        assert!(!workspace.recent_folders.is_open());
    }

    #[test]
    fn starting_a_batch_closes_the_open_recent_folders_list() {
        let test_dir = TestDirectory::new(2);
        let mut workspace = workspace_with_recent_folders(&["a"]);
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        let _ = workspace.update(Message::RecentFolders(recent_folders::Message::Toggle));
        assert!(workspace.recent_folders.is_open());
        let _ = workspace.start_batch();
        assert!(!workspace.recent_folders.is_open());
    }

    /// Issue #213: a workspace reads and writes only the app database it was given. What one
    /// saves (the panel widths, the last batch run) a second one with a database of its own
    /// does not see, and the global `frename.db` is not touched.
    #[test]
    fn workspaces_with_their_own_app_database_share_no_panel_widths_or_batch_run() {
        use crate::features::batch;
        let global = AppDatabase::new();
        let widths_of = |db: &AppDatabase| {
            db.get_window_state()
                .map(|w| (w.left_panel_width, w.folder_panel_width))
        };
        let widths_before = widths_of(&global);
        let run_before = global.get_batch_run();

        let test_dir = TestDirectory::new(2);
        let db_a = fresh_app_db();
        // `main` saves the window at startup; the panel widths update that row.
        db_a.set_window_state(frename_core::WindowGeometry {
            x: 0.0,
            y: 0.0,
            width: 1600.0,
            height: 900.0,
            is_maximized: false,
            monitor_width: 0.0,
            monitor_height: 0.0,
            left_panel_width: 0.0,
            folder_panel_width: 0.0,
        });
        let mut a = FolderWorkspace::with_app_db(db_a.clone());
        let _ = a.update(Message::FolderLoaded {
            directory: test_dir.directory(),
            target_file: Some(test_dir.target_file()),
        });
        let _ = a.update(Message::LeftSplitterDragged(777.0));
        assert_eq!(
            db_a.get_window_state().map(|w| w.left_panel_width),
            Some(777.0)
        );
        let ids: Vec<_> = a
            .directory
            .as_ref()
            .unwrap()
            .all_files()
            .map(|f| f.id())
            .collect();
        let _ = a.update(Message::Batch(batch::Message::SelectAction(
            batch::Action::FixTags,
        )));
        let _ = a.update(Message::Batch(batch::Message::CheckAll(ids)));
        let _ = a.start_batch();
        assert_eq!(
            db_a.get_batch_run().map(|r| r.action),
            Some(batch::Action::FixTags.id().to_string()),
            "the batch run went to the workspace's own database"
        );

        // A second workspace with a database of its own starts from the defaults.
        let b = test_workspace();
        assert_ne!(b.left_width, 777.0);
        assert_eq!(b.batch.action(), super::BatchState::default().action());
        // One built on A's database sees what A saved.
        let c = FolderWorkspace::with_app_db(db_a);
        assert_eq!(c.left_width, 777.0);
        assert_eq!(c.batch.action(), batch::Action::FixTags);

        let global = AppDatabase::new();
        assert_eq!(widths_of(&global), widths_before);
        assert_eq!(global.get_batch_run(), run_before);
    }
}
