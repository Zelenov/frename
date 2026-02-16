//! State for folder workspace: only data and selection logic.
//!
//! Owns: loaded folder (directory + files), currently selected file, loading flag.
//! Also holds child feature state (video, rename panel) and workspace layout (splitter positions).
//! No UI concepts (scrollable, shape, size of children)—only data passed to feature views.
//! Each feature view decides how it looks; the workspace view only arranges regions.

use std::path::PathBuf;

use frename_core::{Directory, File, FileTagSnapshot, SaveAndReparse};
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

/// Folder workspace: owns directory, selected file, loading. Handles selection; panels just show and send messages.
pub struct FolderWorkspace {
    directory: Option<Directory>,
    loading: bool,
    current_file: Option<File>,
    /// File currently being edited: copy of file + tag selection. Rename panel reads/updates this.
    file_workspace: FileWorkspace,
    video_player: VideoPlayerState,
    tag_panel: TagPanelState,
    /// Snapshot to persist after current video is unloaded (then we send FileUpdated and load next video).
    pending_file_updated: Option<FileTagSnapshot>,
    left_width: f32,
    folder_width: f32,
}

impl Default for FolderWorkspace {
    fn default() -> Self {
        Self {
            directory: None,
            loading: false,
            current_file: None,
            file_workspace: FileWorkspace::default(),
            video_player: VideoPlayerState::default(),
            tag_panel: TagPanelState::default(),
            pending_file_updated: None,
            left_width: DEFAULT_LEFT_WIDTH,
            folder_width: DEFAULT_FOLDER_WIDTH,
        }
    }
}

impl FolderWorkspace {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile(path) => self.open_file(path),
            Message::ScanFolder {
                directory,
                target_file,
            } => self.scan_folder(directory, target_file),
            Message::FolderLoaded {
                directory,
                target_file,
            } => self.folder_loaded(directory, target_file),
            Message::FileUpdated { path, new_tags } => self.apply_file_updated(path, new_tags),
            Message::Folder(folder_msg) => self.handle_folder_message(folder_msg),
            Message::VideoPlayer(msg) => match msg {
                video_player::Message::VideoUnloaded => self.on_video_unloaded(),
                other => self.video_player.update(other).map(Message::VideoPlayer),
            }
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

    fn open_file(&mut self, path: PathBuf) -> Task<Message> {
        if self
            .current_file
            .as_ref()
            .map(|f| f.file_path().as_ref())
            == Some(path.as_path())
        {
            return Task::none();
        }
        if let Some(target_file) = self
            .directory
            .as_ref()
            .and_then(|d| d.file_for_path(&path))
        {
            return self.open_file_in_folder(target_file);
        }
        self.current_file = None;
        self.file_workspace.set_file(None);
        self.pending_file_updated = None;
        let directory = path.parent().unwrap_or(&path).to_path_buf();
        Task::done(Message::ScanFolder {
            directory,
            target_file: path,
        })
    }

    fn scan_folder(&mut self, directory: PathBuf, target_file: PathBuf) -> Task<Message> {
        log::info!("Scanning directory: {}", directory.display());
        self.loading = true;
        self.directory = None;
        self.current_file = None;
        self.file_workspace.set_file(None);
        self.pending_file_updated = None;

        Task::future(async move {
            match Directory::open(&directory).await {
                Ok(dir) => {
                    log::info!("Directory scan complete: {} files found", dir.len());
                    Message::FolderLoaded {
                        directory: dir,
                        target_file,
                    }
                }
                Err(e) => {
                    log::error!("Failed to scan directory: {e}");
                    Message::FolderLoaded {
                        directory: Directory::empty(directory),
                        target_file,
                    }
                }
            }
        })
    }

    fn folder_loaded(&mut self, directory: Directory, target_file: PathBuf) -> Task<Message> {
        self.loading = false;
        self.directory = Some(directory);
        let dir = self.directory.as_mut().expect("just set");
        let Some(target_file) = dir.select_file_by_path(&target_file) else {
            log::warn!(
                "Target file not found in scanned directory: {}",
                target_file.display()
            );
            self.current_file = None;
            self.file_workspace.set_file(None);
            self.pending_file_updated = None;
            return Task::none();
        };
        self.open_file_in_folder(target_file)
    }

    fn open_file_in_folder(&mut self, target_file: frename_core::File) -> Task<Message> {
        let snapshot = self.file_workspace.get_snapshot();
        self.file_workspace.set_file(Some(target_file.clone()));
        if let Some(dir) = self.directory.as_mut() {
            dir.select_file(target_file.file_path());
        }
        log::info!("Opening file: {}", target_file.file_path().display());
        self.current_file = Some(target_file);
        if let Some(snapshot) = snapshot {
            self.pending_file_updated = Some(snapshot);
            Task::done(Message::VideoPlayer(video_player::Message::Unload))
        } else {
            self.pending_file_updated = None;
            let path = self.current_file.as_ref().expect("just set").file_path().to_path_buf();
            self.video_player.load_video(path, Message::VideoPlayer)
        }
    }

    /// Called when video player has unloaded. Persist pending snapshot (FileUpdated) then load the new video.
    fn on_video_unloaded(&mut self) -> Task<Message> {
        let pending = self.pending_file_updated.take();
        if let Some(s) = pending {
            let path = s.path;
            let new_tags = s.tags;
            let video_task = self
                .current_file
                .as_ref()
                .map(|f| {
                    self.video_player.load_video(
                        f.file_path().to_path_buf(),
                        Message::VideoPlayer,
                    )
                })
                .unwrap_or(Task::none());
            Task::batch([
                Task::done(Message::FileUpdated { path, new_tags }),
                video_task,
            ])
        } else {
            Task::none()
        }
    }

    fn apply_file_updated(&mut self, path: PathBuf, new_tags: Vec<frename_core::FileTag>) -> Task<Message> {
        let tag_values: Vec<&str> = new_tags.iter().map(frename_core::FileTag::value).collect();
        log::info!(
            "FolderWorkspace apply_file_updated saving {} <- {:?}",
            path.display(),
            tag_values
        );
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
        let Some(dir) = self.directory.as_mut() else {
            return Task::none();
        };
        if !dir.select(index) {
            return Task::none();
        }
        let Some(target_file) = dir.selected_file().cloned() else {
            return Task::none();
        };
        self.open_file_in_folder(target_file)
    }

    fn select_previous(&mut self) -> Task<Message> {
        let Some(dir) = self.directory.as_mut() else {
            return Task::none();
        };
        if !dir.select_previous() {
            return Task::none();
        }
        let Some(target_file) = dir.selected_file().cloned() else {
            return Task::none();
        };
        self.open_file_in_folder(target_file)
    }

    fn select_next(&mut self) -> Task<Message> {
        let Some(dir) = self.directory.as_mut() else {
            return Task::none();
        };
        if !dir.select_next() {
            return Task::none();
        }
        let Some(target_file) = dir.selected_file().cloned() else {
            return Task::none();
        };
        self.open_file_in_folder(target_file)
    }

    fn handle_tag_panel(
        &mut self,
        msg: crate::features::tag_panel::Message,
    ) -> Task<Message> {
        let crate::features::tag_panel::Message::ToggleTag(index) = msg;
        self.file_workspace.toggle_tag(index);
        self.tag_panel.update(&msg);
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        self.video_player.subscription().map(Message::VideoPlayer)
    }

    /// Currently selected file (immutable).
    pub fn current_file(&self) -> Option<&File> {
        self.current_file.as_ref()
    }

    pub fn directory(&self) -> Option<&Directory> {
        self.directory.as_ref()
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// File workspace: current file and its tag selection (for rename panel). Use this for display and tag toggles.
    pub fn file_workspace(&self) -> &FileWorkspace {
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

    use frename_core::{Directory, File, FileTag, TagStorage};

    use crate::features::{folder, tag_panel};

    use super::{FolderWorkspace, Message};

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
            let directory = Directory::with_files(&dir, files);
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
        let mut workspace = FolderWorkspace::default();
        let _task = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory,
            target_file: test_dir.target_file,
        });

        assert_eq!(
            workspace.directory().map(|d| d.len()),
            Some(2),
            "folder panel should show two files"
        );
    }

    #[test]
    fn tags_are_saved_after_selecting_another_file() {
        let test_dir = TestDirectory::new(2);
        let first_file_path = PathBuf::from("C:/test/folder/file_0.mp4");
        let mut workspace = FolderWorkspace::default();
        let _ = workspace.update(Message::FolderLoaded {
            directory: test_dir.directory,
            target_file: test_dir.target_file,
        });

        let tag_name = "Comedy";
        let tag_index = TagStorage::names()
            .iter()
            .position(|&n| n == tag_name)
            .expect("Comedy is a stored tag");
        let _ = workspace.update(Message::TagPanel(tag_panel::Message::ToggleTag(
            tag_index,
        )));

        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(1)));
        let _ = workspace.update(Message::FileUpdated {
            path: first_file_path,
            new_tags: vec![FileTag::new(tag_name)],
        });
        let _ = workspace.update(Message::Folder(folder::Message::SelectFile(0)));

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
