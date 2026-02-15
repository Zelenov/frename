//! State for the folder feature

use frename_core::FileInfo;
use iced::widget::scrollable;
use iced::Task;

use super::Message;

/// Scrollable widget ID for the folder file list.
const FOLDER_LIST_ID: &str = "folder-file-list";

/// Folder state - holds the scanned file list and selection
pub struct FolderState {
    /// Files in the current folder, sorted by creation date
    files: Vec<FileInfo>,
    /// Index of the currently selected file
    selected_index: Option<usize>,
    /// Whether a directory scan is in progress
    loading: bool,
    /// Stable scrollable ID for scroll-into-view
    scrollable_id: scrollable::Id,
}

impl Default for FolderState {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            selected_index: None,
            loading: false,
            scrollable_id: scrollable::Id::new(FOLDER_LIST_ID),
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
                self.files.clear();
                self.selected_index = None;

                Task::future(async move {
                    match frename_core::scan_directory(&directory) {
                        Ok(files) => {
                            log::info!("Directory scan complete: {} files found", files.len());
                            Message::FolderLoaded { files, target_file }
                        }
                        Err(e) => {
                            log::error!("Failed to scan directory: {e}");
                            Message::FolderLoaded {
                                files: Vec::new(),
                                target_file,
                            }
                        }
                    }
                })
            }
            Message::FolderLoaded { files, target_file } => {
                self.loading = false;

                // Find the target file in the scanned list
                let target_index = files
                    .iter()
                    .position(|f| f.file_path() == target_file.as_path());

                self.files = files;

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
                if index < self.files.len() {
                    self.selected_index = Some(index);
                    // Scroll the selected item into view
                    let fraction = if self.files.len() <= 1 {
                        0.0
                    } else {
                        index as f32 / (self.files.len() - 1) as f32
                    };
                    scrollable::snap_to(
                        self.scrollable_id.clone(),
                        scrollable::RelativeOffset { x: 0.0, y: fraction },
                    )
                } else {
                    Task::none()
                }
            }
        }
    }

    /// Get the list of files
    pub fn files(&self) -> &[FileInfo] {
        &self.files
    }

    /// Get the selected file index
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Get the currently selected file
    pub fn selected_file(&self) -> Option<&FileInfo> {
        self.selected_index.and_then(|i| self.files.get(i))
    }

    /// Whether a directory scan is in progress
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Get the scrollable widget ID
    pub fn scrollable_id(&self) -> &scrollable::Id {
        &self.scrollable_id
    }
}
