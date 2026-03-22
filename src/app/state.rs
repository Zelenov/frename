//! Root application state and coordination.
//!
//! The app only holds top-level features and uses their public API (update, view, subscription).
//! It does not know how any feature looks (scrollable, layout, panels) or what internals they have.
//! Only `folder_workspace.current_file()` is used for the window title.
//!
//! UI-level concern: when a typing key is pressed and no text field is focused, focus the search bar
//! and send the key so it can be emulated into the filter (Iced does not replay events to widgets).

use iced::{event, keyboard, window, Element, Subscription, Task};

use crate::features::{drag_drop, folder, folder_workspace, media_viewer, tag_panel};
use frename_core::{AppDatabase, AppStateStore, WindowGeometry};

use super::Message;

/// Application state: top-level features only. No knowledge of child UI or structure.
pub struct FrenameApp {
    drag_drop_state: drag_drop::DragDropState,
    folder_workspace: folder_workspace::FolderWorkspace,
    /// Window ID waiting for GStreamer unload before we close.
    pending_close: Option<window::Id>,
    /// Last known window position (logical px); updated on Moved events.
    window_pos: (f32, f32),
    /// Last known window size (logical px); updated on Resized events.
    window_size: (f32, f32),
}

impl FrenameApp {
    pub fn new() -> Self {
        let saved = AppDatabase::new().get_window_state();
        Self {
            drag_drop_state: drag_drop::DragDropState::default(),
            folder_workspace: folder_workspace::FolderWorkspace::new(),
            pending_close: None,
            window_pos: saved.map(|g| (g.x, g.y)).unwrap_or((0.0, 0.0)),
            window_size: saved.map(|g| (g.width, g.height)).unwrap_or((1200.0, 600.0)),
        }
    }
}

impl FrenameApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowReady => Task::done(Message::FolderWorkspace(
                folder_workspace::Message::LoadLastSession,
            )),
            Message::CloseRequested(id) => {
                crate::crash_guard::mark_closing();
                if !self.folder_workspace.needs_media_unload() {
                    return window::close(id);
                }
                self.pending_close = Some(id);
                Task::done(Message::FolderWorkspace(
                    folder_workspace::Message::MediaViewer(media_viewer::Message::Unload),
                ))
            }
            Message::WindowMoved(x, y) => {
                AppDatabase::new().set_window_state(WindowGeometry { x, y, width: self.window_size.0, height: self.window_size.1 });
                Task::none()
            }
            Message::WindowResized(width, height) => {
                self.window_size = (width, height);
                AppDatabase::new().set_window_state(WindowGeometry { x: self.window_pos.0, y: self.window_pos.1, width, height });
                Task::none()
            }
            Message::DragDrop(drag_drop::Message::FileDropped(path)) => {
                self.drag_drop_state.handle_file_dropped(path.clone());
                Task::done(Message::FolderWorkspace(
                    folder_workspace::Message::OpenFile(path),
                ))
            }
            Message::FolderWorkspace(msg) => {
                let is_unloaded = matches!(
                    &msg,
                    folder_workspace::Message::MediaViewer(media_viewer::Message::Unloaded)
                );
                let task = self.folder_workspace.update(msg).map(Message::FolderWorkspace);
                let Some(id) = is_unloaded.then(|| self.pending_close.take()).flatten() else {
                    return task;
                };
                Task::batch([task, window::close(id)])
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        folder_workspace::view::view(&self.folder_workspace).map(Message::FolderWorkspace)
    }

    /// Feature subscriptions (file drop, window opened, global keyboard to search bar).
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.drag_drop_state.subscription().map(Message::DragDrop),
            self.folder_workspace
                .subscription()
                .map(Message::FolderWorkspace),
            window::close_requests().map(Message::CloseRequested),
            event::listen_with(|ev, status, _| match ev {
                iced::Event::Window(window::Event::Opened { .. }) => Some(Message::WindowReady),
                iced::Event::Window(window::Event::Moved(point)) => {
                    Some(Message::WindowMoved(point.x, point.y))
                }
                iced::Event::Window(window::Event::Resized(size)) => {
                    Some(Message::WindowResized(size.width, size.height))
                }
                // Space always toggles the selected tag (even when focus is in the search bar; we strip the space from filter in the handler).
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Space),
                    ..
                }) => Some(Message::FolderWorkspace(
                    folder_workspace::Message::TagPanel(tag_panel::Message::ToggleSelectedTag),
                )),
                // F5 toggles fullscreen for the media viewer.
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::F5),
                    ..
                }) => Some(Message::FolderWorkspace(
                    folder_workspace::Message::ToggleMediaFullscreen,
                )),
                // Escape: handled by FolderWorkspace (exits fullscreen or clears search filter).
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(keyboard::key::Named::Escape),
                    ..
                }) => Some(Message::FolderWorkspace(
                    folder_workspace::Message::EscapePressed,
                )),
                iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                    if matches!(status, event::Status::Ignored) =>
                {
                    if modifiers.command() {
                        return match key.as_ref() {
                            keyboard::Key::Character("c") => Some(Message::FolderWorkspace(
                                folder_workspace::Message::CopyTags,
                            )),
                            keyboard::Key::Character("v") => Some(Message::FolderWorkspace(
                                folder_workspace::Message::PasteTags,
                            )),
                            keyboard::Key::Character("z") if modifiers.shift() => {
                                Some(Message::FolderWorkspace(folder_workspace::Message::Redo))
                            }
                            keyboard::Key::Character("z") => {
                                Some(Message::FolderWorkspace(folder_workspace::Message::Undo))
                            }
                            keyboard::Key::Character("y") => {
                                Some(Message::FolderWorkspace(folder_workspace::Message::Redo))
                            }
                            _ => None,
                        };
                    }
                    if let keyboard::Key::Named(name) = key.as_ref() {
                        let msg = match name {
                            keyboard::key::Named::ArrowLeft => {
                                Some(Message::FolderWorkspace(
                                    folder_workspace::Message::TagPanel(
                                        tag_panel::Message::SelectLeft,
                                    ),
                                ))
                            }
                            keyboard::key::Named::ArrowRight => {
                                Some(Message::FolderWorkspace(
                                    folder_workspace::Message::TagPanel(
                                        tag_panel::Message::SelectRight,
                                    ),
                                ))
                            }
                            keyboard::key::Named::ArrowUp => {
                                Some(Message::FolderWorkspace(
                                    folder_workspace::Message::TagPanel(
                                        tag_panel::Message::SelectUp,
                                    ),
                                ))
                            }
                            keyboard::key::Named::ArrowDown => {
                                Some(Message::FolderWorkspace(
                                    folder_workspace::Message::TagPanel(
                                        tag_panel::Message::SelectDown,
                                    ),
                                ))
                            }
                            keyboard::key::Named::PageUp => {
                                Some(Message::FolderWorkspace(
                                    folder_workspace::Message::Folder(folder::Message::PreviousFile),
                                ))
                            }
                            keyboard::key::Named::PageDown => {
                                Some(Message::FolderWorkspace(
                                    folder_workspace::Message::Folder(folder::Message::NextFile),
                                ))
                            }
                            keyboard::key::Named::Delete => {
                                Some(Message::FolderWorkspace(
                                    folder_workspace::Message::RemoveTag,
                                ))
                            }
                            keyboard::key::Named::Enter => {
                                Some(Message::FolderWorkspace(
                                    folder_workspace::Message::SaveSelectedTag,
                                ))
                            }
                            _ => None,
                        };
                        if msg.is_some() {
                            return msg;
                        }
                    }
                    let k = match key.as_ref() {
                        keyboard::Key::Character(c) => {
                            c.chars().next().map(folder_workspace::GlobalSearchKey::Char)
                        }
                        keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                            Some(folder_workspace::GlobalSearchKey::Backspace)
                        }
                        keyboard::Key::Named(keyboard::key::Named::Delete) => {
                            Some(folder_workspace::GlobalSearchKey::Delete)
                        }
                        _ => None,
                    };
                    k.map(|key| {
                        Message::FolderWorkspace(
                            folder_workspace::Message::FocusSearchBarAndKey(key),
                        )
                    })
                }
                _ => None,
            }),
        ])
    }

    /// Get the window title based on the currently open file
    pub fn title(&self) -> String {
        self.folder_workspace
            .current_file()
            .map_or_else(|| String::from("frename"), |f| f.file_path().display().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let _app = FrenameApp::new();
    }
}
