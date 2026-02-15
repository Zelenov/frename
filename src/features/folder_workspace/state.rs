//! State for folder workspace: only data and selection logic.
//!
//! Owns: loaded folder (directory + files), currently selected file, loading flag.
//! Also holds child feature state (video, rename panel) and workspace layout (splitter positions).
//! No UI concepts (scrollable, shape, size of children)—only data passed to feature views.
//! Each feature view decides how it looks; the workspace view only arranges regions.

use std::path::{Path, PathBuf};

use frename_core::Directory;
use iced::{Subscription, Task};

use crate::features::folder;
use crate::features::rename_panel::RenamePanelState;
use crate::features::video_player::VideoPlayerState;
use crate::widgets::splitter::HIT_WIDTH;

use super::Message;

const DEFAULT_LEFT_WIDTH: f32 = 460.0;
const DEFAULT_FOLDER_WIDTH: f32 = 200.0;
const MIN_FOLDER_WIDTH: f32 = 120.0;

/// Folder workspace: owns directory, selected file, loading. Handles selection; panels just show and send messages.
pub struct FolderWorkspace {
    directory: Option<Directory>,
    loading: bool,
    current_file: Option<Box<Path>>,
    video_player: VideoPlayerState,
    rename_panel: RenamePanelState,
    left_width: f32,
    folder_width: f32,
}

impl Default for FolderWorkspace {
    fn default() -> Self {
        Self {
            directory: None,
            loading: false,
            current_file: None,
            video_player: VideoPlayerState::default(),
            rename_panel: RenamePanelState::default(),
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
            Message::Folder(folder_msg) => self.handle_folder_message(folder_msg),
            Message::VideoPlayer(msg) => {
                self.video_player.update(msg).map(Message::VideoPlayer)
            }
            Message::RenamePanel(msg) => self.handle_rename_panel(msg),
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
            .map(|p| p.as_ref())
            == Some(path.as_path())
        {
            return Task::none();
        }
        let parent = path.parent().map(|p| p.to_path_buf());
        let current_dir = self.directory.as_ref().map(|d| d.path());
        let same_directory = parent
            .as_ref()
            .and_then(|p| current_dir.map(|cur| cur == p.as_path()))
            .unwrap_or(false);
        let in_folder = same_directory
            && self
                .directory
                .as_ref()
                .and_then(|d| d.find_by_path(&path))
                .is_some();

        if in_folder {
            let Some(dir) = self.directory.as_mut() else {
                return Task::none();
            };
            let Some(index) = dir.find_by_path(&path) else {
                return Task::none();
            };
            if !dir.select(index) {
                return Task::none();
            }
            self.current_file = Some(path.clone().into_boxed_path());
            log::info!("Opening file: {}", path.display());
            self.video_player.load_video(path, Message::VideoPlayer)
        } else {
            let directory = parent.unwrap_or_else(|| path.clone());
            Task::done(Message::ScanFolder {
                directory,
                target_file: path,
            })
        }
    }

    fn scan_folder(&mut self, directory: PathBuf, target_file: PathBuf) -> Task<Message> {
        log::info!("Scanning directory: {}", directory.display());
        self.loading = true;
        self.directory = None;

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
        let target_index = directory.find_by_path(&target_file);
        self.directory = Some(directory);

        let Some(index) = target_index else {
            log::warn!(
                "Target file not found in scanned directory: {}",
                target_file.display()
            );
            return Task::none();
        };
        let dir = self.directory.as_mut().expect("just set");
        if !dir.select(index) {
            return Task::none();
        }
        let path = dir
            .selected_file()
            .map(|f| f.file_path().to_path_buf())
            .expect("we just selected");
        self.current_file = Some(path.clone().into_boxed_path());
        log::info!("Opening file: {}", path.display());
        self.video_player.load_video(path, Message::VideoPlayer)
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
        let Some(path) = dir.selected_file().map(|f| f.file_path().to_path_buf()) else {
            return Task::none();
        };
        self.current_file = Some(path.clone().into_boxed_path());
        log::info!("Opening file: {}", path.display());
        self.video_player.load_video(path, Message::VideoPlayer)
    }

    fn select_previous(&mut self) -> Task<Message> {
        let Some(dir) = self.directory.as_mut() else {
            return Task::none();
        };
        let current = dir.selected_index().unwrap_or(0);
        if current == 0 {
            return Task::none();
        }
        self.select_file_at(current - 1)
    }

    fn select_next(&mut self) -> Task<Message> {
        let Some(dir) = self.directory.as_ref() else {
            return Task::none();
        };
        let current = dir.selected_index().unwrap_or(0);
        if current + 1 >= dir.len() {
            return Task::none();
        }
        self.select_file_at(current + 1)
    }

    fn handle_rename_panel(
        &mut self,
        msg: crate::features::rename_panel::Message,
    ) -> Task<Message> {
        let crate::features::rename_panel::Message::ToggleTag(index) = msg;
        if let Some(file) = self
            .directory
            .as_mut()
            .and_then(|d| d.selected_file_mut())
        {
            file.toggle_tag(index);
        }
        self.rename_panel.update(&msg);
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        self.video_player.subscription().map(Message::VideoPlayer)
    }

    /// Currently selected file path (immutable).
    pub fn current_file(&self) -> Option<&Path> {
        self.current_file.as_deref()
    }

    pub fn directory(&self) -> Option<&Directory> {
        self.directory.as_ref()
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Currently selected file (for rename panel and others that need the File).
    pub fn selected_file(&self) -> Option<&frename_core::File> {
        self.directory.as_ref().and_then(|d| d.selected_file())
    }

    pub fn has_previous_next(&self) -> (bool, bool) {
        self.directory
            .as_ref()
            .and_then(|d| {
                self.current_file.as_ref().and_then(|path| d.find_by_path(path))
            })
            .map(|idx| {
                let len = self.directory.as_ref().map(|d| d.len()).unwrap_or(0);
                (idx > 0, idx + 1 < len)
            })
            .unwrap_or((false, false))
    }

    pub fn video_player(&self) -> &VideoPlayerState {
        &self.video_player
    }

    pub fn rename_panel(&self) -> &RenamePanelState {
        &self.rename_panel
    }

    pub fn left_width(&self) -> f32 {
        self.left_width
    }

    pub fn folder_width(&self) -> f32 {
        self.folder_width
    }
}
