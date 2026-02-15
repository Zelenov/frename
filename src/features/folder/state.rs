//! State for the folder feature

use frename_core::Directory;
use iced::widget::scrollable::RelativeOffset;
use iced::widget::{operation, Id};
use iced::Task;

use super::Message;

/// Scrollable widget ID for the folder file list.
const FOLDER_LIST_ID: &str = "folder-file-list";

/// Folder state - wraps a core Directory and adds UI concerns
pub struct FolderState {
    /// The scanned directory (None until first scan)
    directory: Option<Directory>,
    /// Whether a directory scan is in progress
    loading: bool,
    /// Index of the file currently being "applied" (show spinner next to it).
    applying_index: Option<usize>,
    /// Stable scrollable ID for scroll-into-view
    scrollable_id: Id,
}

impl Default for FolderState {
    fn default() -> Self {
        Self {
            directory: None,
            loading: false,
            applying_index: None,
            scrollable_id: Id::new(FOLDER_LIST_ID),
        }
    }
}

impl FolderState {
    /// Handle folder messages
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ScanFolder {
                directory,
                target_file,
            } => {
                log::info!("Scanning directory: {}", directory.display());
                self.loading = true;
                self.directory = None;

                Task::future(async move {
                    match Directory::open(&directory).await {
                        Ok(dir) => {
                            log::info!(
                                "Directory scan complete: {} files found",
                                dir.len()
                            );
                            Message::FolderLoaded {
                                directory: dir,
                                target_file,
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to scan directory: {e}");
                            // Create an empty directory on error — use a
                            // synchronous fallback since we already failed.
                            Message::FolderLoaded {
                                directory: Directory::empty(directory),
                                target_file,
                            }
                        }
                    }
                })
            }
            Message::FolderLoaded {
                directory,
                target_file,
            } => {
                self.loading = false;

                let target_index = directory.find_by_path(&target_file);
                self.directory = Some(directory);

                if let Some(index) = target_index {
                    log::info!("Auto-selecting dropped file at index {index}");
                    Task::done(Message::SelectFile(index))
                } else {
                    log::warn!(
                        "Target file not found in scanned directory: {}",
                        target_file.display()
                    );
                    Task::none()
                }
            }
            Message::SelectFile(index) => {
                if let Some(dir) = &mut self.directory {
                    if dir.select(index) {
                        // Scroll the selected item into view
                        let total = dir.len();
                        let fraction = if total <= 1 {
                            0.0
                        } else {
                            index as f32 / (total - 1) as f32
                        };
                        return operation::snap_to(
                            self.scrollable_id.clone(),
                            RelativeOffset::<Option<f32>> {
                                x: None,
                                y: Some(fraction),
                            },
                        );
                    }
                }
                Task::none()
            }
            Message::PreviousFile => {
                if let Some(dir) = &mut self.directory {
                    let current = dir.selected_index().unwrap_or(0);
                    if current > 0 {
                        let index = current - 1;
                        if dir.select(index) {
                            let total = dir.len();
                            let fraction = if total <= 1 {
                                0.0
                            } else {
                                index as f32 / (total - 1) as f32
                            };
                            return operation::snap_to(
                                self.scrollable_id.clone(),
                                RelativeOffset::<Option<f32>> {
                                    x: None,
                                    y: Some(fraction),
                                },
                            );
                        }
                    }
                }
                Task::none()
            }
            Message::NextFile => {
                if let Some(dir) = &mut self.directory {
                    let len = dir.len();
                    let current = dir.selected_index().unwrap_or(0);
                    if current + 1 < len {
                        let index = current + 1;
                        if dir.select(index) {
                            let total = dir.len();
                            let fraction = if total <= 1 {
                                0.0
                            } else {
                                index as f32 / (total - 1) as f32
                            };
                            return operation::snap_to(
                                self.scrollable_id.clone(),
                                RelativeOffset::<Option<f32>> {
                                    x: None,
                                    y: Some(fraction),
                                },
                            );
                        }
                    }
                }
                Task::none()
            }
        }
    }

    /// Get the directory (if loaded)
    pub fn directory(&self) -> Option<&Directory> {
        self.directory.as_ref()
    }

    /// Get a mutable reference to the directory (if loaded)
    pub fn directory_mut(&mut self) -> Option<&mut Directory> {
        self.directory.as_mut()
    }

    /// Whether a directory scan is in progress
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Index of the file row that should show the "applying" spinner.
    pub fn applying_index(&self) -> Option<usize> {
        self.applying_index
    }

    /// Set which file is currently being applied (show spinner). None to clear.
    pub fn set_applying(&mut self, index: Option<usize>) {
        self.applying_index = index;
    }

    /// Get the scrollable widget ID
    pub fn scrollable_id(&self) -> &Id {
        &self.scrollable_id
    }
}
