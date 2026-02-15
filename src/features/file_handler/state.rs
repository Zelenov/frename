//! State for file handler feature

use std::path::PathBuf;
use std::time::Duration;

use iced::{Subscription, Task};

use crate::features::folder::{self, FolderState};
use crate::features::rename_panel::RenamePanelState;
use crate::features::video_player::VideoPlayerState;
use crate::widgets::splitter::HIT_WIDTH;

use super::Message;

/// Default width of the left (video) panel in pixels.
const DEFAULT_LEFT_WIDTH: f32 = 460.0;

/// Default width of the middle (folder) panel in pixels.
const DEFAULT_FOLDER_WIDTH: f32 = 200.0;

/// Minimum width of the folder panel.
const MIN_FOLDER_WIDTH: f32 = 120.0;

/// Central file handling state - the core of the application
pub struct FileHandlerState {
    /// Currently open file path
    current_file: Option<PathBuf>,
    /// Folder listing (scanned directory, file selection)
    folder: FolderState,
    /// Video player (activated when the opened file is a video)
    video_player: VideoPlayerState,
    /// Rename panel (right side: file name display + tag list)
    rename_panel: RenamePanelState,
    /// Width of the left (video) panel in pixels.
    left_width: f32,
    /// Width of the middle (folder) panel in pixels.
    folder_width: f32,
}

impl Default for FileHandlerState {
    fn default() -> Self {
        Self {
            current_file: None,
            folder: FolderState::default(),
            video_player: VideoPlayerState::default(),
            rename_panel: RenamePanelState::default(),
            left_width: DEFAULT_LEFT_WIDTH,
            folder_width: DEFAULT_FOLDER_WIDTH,
        }
    }
}

impl FileHandlerState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile(path) => {
                log::info!("Opening file: {}", path.display());
                self.current_file = Some(path.clone());

                // If this file is in the current directory, select and scroll to it
                let in_current_dir = self
                    .folder
                    .directory()
                    .filter(|d| path.parent().map(|p| p == d.path()).unwrap_or(false))
                    .and_then(|d| d.find_by_path(&path));

                if let Some(index) = in_current_dir {
                    let scroll_task = self
                        .folder
                        .update(folder::Message::SelectFile(index))
                        .map(Message::Folder);
                    Task::batch([
                        scroll_task,
                        self.video_player.load_video(path, Message::VideoPlayer),
                    ])
                } else {
                    self.video_player.load_video(path, Message::VideoPlayer)
                }
            }
            Message::Folder(folder_msg) => {
                // If user selected another file and current file is dirty: switch immediately
                // and run apply in background (spinner shows on the previous file).
                if let folder::Message::SelectFile(new_index) = &folder_msg {
                    let current_dirty = self
                        .folder
                        .directory()
                        .and_then(|d| d.selected_file())
                        .map(|f| f.is_dirty())
                        .unwrap_or(false);
                    if current_dirty {
                        let applying_idx =
                            self.folder.directory().and_then(|d| d.selected_index());
                        self.folder.set_applying(applying_idx);

                        let folder_task =
                            self.folder
                                .update(folder::Message::SelectFile(*new_index))
                                .map(Message::Folder);
                        let path = self
                            .folder
                            .directory()
                            .and_then(|d| d.selected_file())
                            .map(|f| f.file_path().to_path_buf());
                        let apply_task = Task::future(async move {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            Message::ApplyChangesCompleted
                        });

                        let mut tasks: Vec<Task<Message>> =
                            vec![folder_task, apply_task];
                        if let Some(p) = path {
                            tasks.push(Task::done(Message::OpenFile(p)));
                        }
                        return Task::batch(tasks);
                    }
                }

                let is_file_select = matches!(&folder_msg, folder::Message::SelectFile(_));
                let task = self.folder.update(folder_msg).map(Message::Folder);

                if is_file_select {
                    let selected = self
                        .folder
                        .directory()
                        .and_then(|dir| dir.selected_file());
                    if let Some(file) = selected {
                        let path = file.file_path().to_path_buf();
                        Task::batch([task, Task::done(Message::OpenFile(path))])
                    } else {
                        task
                    }
                } else {
                    task
                }
            }
            Message::ApplyChanges => {
                // Start async apply (e.g. for "apply on close"); spinner shows on current selection
                let applying_idx = self.folder.directory().and_then(|d| d.selected_index());
                self.folder.set_applying(applying_idx);
                Task::future(async move {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    Message::ApplyChangesCompleted
                })
            }
            Message::ApplyChangesCompleted => {
                let applying_idx = self.folder.applying_index();
                self.folder.set_applying(None);
                if let Some(idx) = applying_idx {
                    if let Some(file) = self
                        .folder
                        .directory_mut()
                        .and_then(|d| d.file_at_mut(idx))
                    {
                        file.clear_dirty();
                    }
                }
                Task::none()
            }
            Message::VideoPlayer(msg) => {
                self.video_player.update(msg).map(Message::VideoPlayer)
            }
            Message::RenamePanel(msg) => {
                match msg {
                    crate::features::rename_panel::Message::ToggleTag(index) => {
                        if let Some(file) = self
                            .folder
                            .directory_mut()
                            .and_then(|d| d.selected_file_mut())
                        {
                            file.toggle_tag(index);
                        }
                    }
                }
                self.rename_panel.update(&msg);
                Task::none()
            }
            Message::LeftSplitterDragged(x) => {
                self.left_width = x;
                // Ensure folder panel doesn't shrink below minimum
                let folder_start = self.left_width + HIT_WIDTH;
                let folder_end = folder_start + self.folder_width;
                let new_folder_width = folder_end - x - HIT_WIDTH;
                if new_folder_width < MIN_FOLDER_WIDTH {
                    self.folder_width = MIN_FOLDER_WIDTH;
                }
                Task::none()
            }
            Message::RightSplitterDragged(x) => {
                let new_folder_width = x - self.left_width - HIT_WIDTH;
                self.folder_width = new_folder_width.max(MIN_FOLDER_WIDTH);
                Task::none()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        self.video_player.subscription().map(Message::VideoPlayer)
    }

    /// Currently open file path
    pub fn current_file(&self) -> Option<&PathBuf> {
        self.current_file.as_ref()
    }

    /// Get reference to the folder state
    pub fn folder(&self) -> &FolderState {
        &self.folder
    }

    /// Get reference to the video player state
    pub fn video_player(&self) -> &VideoPlayerState {
        &self.video_player
    }

    /// Get reference to the rename panel state
    pub fn rename_panel(&self) -> &RenamePanelState {
        &self.rename_panel
    }

    /// Width of the left (video) panel in pixels.
    pub fn left_width(&self) -> f32 {
        self.left_width
    }

    /// Width of the middle (folder) panel in pixels.
    pub fn folder_width(&self) -> f32 {
        self.folder_width
    }
}
