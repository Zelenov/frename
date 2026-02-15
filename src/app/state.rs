//! Root application state and coordination

use iced::{Element, Subscription, Task};

use crate::features::{drag_drop, folder, folder_workspace};

use super::Message;

/// Main application state. The app is folder-centric: once a folder is chosen,
/// the state is the folder workspace (folder + video + rename panels).
#[derive(Default)]
pub struct FrenameApp {
    drag_drop_state: drag_drop::DragDropState,
    folder_workspace: folder_workspace::FolderWorkspace,
}

impl FrenameApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DragDrop(drag_drop::Message::FileDropped(path)) => {
                self.drag_drop_state.handle_file_dropped(path.clone());

                let parent = path.parent().map(|p| p.to_path_buf());
                let current_dir = self
                    .folder_workspace
                    .folder()
                    .directory()
                    .map(|d| d.path());

                let same_directory = parent.as_ref().and_then(|p| {
                    current_dir.map(|cur| cur == p.as_path())
                }).unwrap_or(false);

                if same_directory {
                    let index = self
                        .folder_workspace
                        .folder()
                        .directory()
                        .and_then(|d| d.find_by_path(&path));

                    if let Some(idx) = index {
                        return Task::batch([
                            Task::done(Message::FolderWorkspace(folder_workspace::Message::Folder(
                                folder::Message::SelectFile(idx),
                            ))),
                            Task::done(Message::FolderWorkspace(folder_workspace::Message::OpenFile(path))),
                        ]);
                    }
                }

                let directory = parent.unwrap_or_else(|| path.clone());
                Task::done(Message::FolderWorkspace(folder_workspace::Message::Folder(
                    folder::Message::ScanFolder {
                        directory,
                        target_file: path,
                    },
                )))
            }
            Message::FolderWorkspace(msg) => {
                self.folder_workspace.update(msg).map(Message::FolderWorkspace)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        folder_workspace::view::view(&self.folder_workspace).map(Message::FolderWorkspace)
    }

    /// Feature subscriptions (file drop, keyboard, timers, etc.)
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.drag_drop_state.subscription().map(Message::DragDrop),
            self.folder_workspace
                .subscription()
                .map(Message::FolderWorkspace),
        ])
    }

    /// Get the window title based on the currently open file
    pub fn title(&self) -> String {
        if let Some(file) = self.folder_workspace.current_file() {
            file.display().to_string()
        } else {
            String::from("frename")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let _app = FrenameApp::default();
        // Basic smoke test - app can be created
    }
}
