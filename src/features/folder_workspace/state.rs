//! State for folder workspace: only data and selection logic.
//!
//! Owns: loaded folder (directory + files), currently selected file, loading flag.
//! Also holds child feature state (video, rename panel) and workspace layout (splitter positions).
//! No UI concepts (scrollable, shape, size of children)—only data passed to feature views.
//! Each feature view decides how it looks; the workspace view only arranges regions.

use std::path::PathBuf;

use frename_core::{
    AppDatabase, AppStateStore, File, FolderAndFile, FileTagSnapshot, LoggingAppStateStore,
    SaveAndReparse,
};

use super::Directory;
use iced::{Subscription, Task};

use crate::features::file_workspace::FileWorkspace;
use crate::features::folder;
use crate::features::tag_panel::TagPanelState;
use crate::features::video_player::{self, VideoPlayerState};
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
    /// Snapshot to persist after current video is unloaded (then we send FileUpdated and load next video).
    pending_file_updated: Option<FileTagSnapshot>,
    left_width: f32,
    folder_width: f32,
}

impl FolderWorkspace {
    pub fn new() -> Self {
        Self {
            directory: None,
            loading: false,
            file_workspace: FileWorkspace::<AppDatabase>::default(),
            video_player: VideoPlayerState::default(),
            tag_panel: TagPanelState::default(),
            pending_file_updated: None,
            left_width: DEFAULT_LEFT_WIDTH,
            folder_width: DEFAULT_FOLDER_WIDTH,
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
            Message::FileUpdated { path, new_tags } => self.apply_file_updated(path, new_tags),
            Message::Folder(folder_msg) => self.handle_folder_message(folder_msg),
            Message::VideoPlayer(msg) => match msg {
                video_player::Message::VideoUnloaded => self.on_video_unloaded(),
                other => self.video_player.update(other).map(Message::VideoPlayer),
            },
            Message::TagPanel(msg) => self.handle_tag_panel(msg),
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
        }
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
        let Some(s) = pending else {
            return Task::none();
        };
        let path = s.path;
        let new_tags = s.tags;
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
            Task::done(Message::FileUpdated { path, new_tags }),
            video_task,
        ])
    }

    fn apply_file_updated(&mut self, path: PathBuf, new_tags: Vec<frename_core::FileTag>) -> Task<Message> {
        let (path_buf, tags_after_save) = new_tags.as_slice().save_and_reparse(&path);
        let _ = self
            .directory
            .as_mut()
            .map(|dir| dir.update_file(&path_buf, &tags_after_save));
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
        msg: crate::features::tag_panel::Message,
    ) -> Task<Message> {
        match msg {
            crate::features::tag_panel::Message::SetFilter(query) => {
                self.file_workspace.set_tag_filter(query);
                Task::none()
            }
            crate::features::tag_panel::Message::ToggleTag(index) => {
                self.file_workspace.toggle_tag(index);
                self.tag_panel.update(&msg);
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        self.video_player.subscription().map(Message::VideoPlayer)
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

    use frename_core::{AppDatabase, File, FileTag, Initializable, LoggingAppStateStore};

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
        let tag_index = workspace
            .file_workspace()
            .tag_list()
            .tags()
            .iter()
            .position(|t| t.tag() == tag_name)
            .expect("Comedy is a stored tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(
            tag_index,
        )));

        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        flush_file_opened(&mut workspace);
        let _ = workspace.update(Message::FileUpdated {
            path: first_file_path,
            new_tags: vec![FileTag::new(tag_name)],
        });
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));
        flush_file_opened(&mut workspace);

        assert!(
            workspace
                .file_workspace()
                .file()
                .unwrap()
                .tag_list()
                .has_tag(tag_name),
            "tags should be saved after selecting another file"
        );
    }
}
