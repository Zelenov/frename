//! Root application state and coordination.
//!
//! The app only holds top-level features and uses their public API (update, view, subscription).
//! It does not know how any feature looks (scrollable, layout, panels) or what internals they have.
//! Only `folder_workspace.current_file()` is used for the window title.
//!
//! UI-level concern: when a typing key is pressed and no text field is focused, focus the search bar
//! and send the key so it can be emulated into the filter (Iced does not replay events to widgets).

use iced::{event, keyboard, window, Element, Subscription, Task};

use crate::features::{drag_drop, folder, folder_workspace, media_viewer, media_viewer::video as media_viewer_video, settings, tag_panel};
use crate::tag_colors::TagPalette;
use frename_core::{AppDatabase, AppStateStore, WindowGeometry};

use super::Message;

/// Ctrl+V handler fired when has_copied_tags is true: intercepts paste globally (even when the
/// search bar has focus) and pastes tags instead. paste_tags() clears the search filter so the
/// search bar's concurrent OS-clipboard paste gets overwritten.
fn ctrl_v_paste_tags_handler(
    ev: iced::Event,
    _status: event::Status,
    _window_id: window::Id,
) -> Option<Message> {
    if let iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Character(c),
        modifiers,
        ..
    }) = ev
    {
        if c.as_ref() == "v" && modifiers.command() {
            return Some(Message::FolderWorkspace(folder_workspace::Message::PasteTags));
        }
    }
    None
}

/// Global keyboard and window events of the main window: shortcuts, typing into the search bar,
/// and window geometry. The subscription filters them to the main window, so keys pressed in the
/// settings window do not reach the workspace.
fn main_window_event(
    ev: iced::Event,
    status: event::Status,
    _window_id: window::Id,
) -> Option<Message> {
    match ev {
        iced::Event::Window(window::Event::Opened { .. }) => Some(Message::WindowReady),
        iced::Event::Window(window::Event::Moved(point)) => {
            Some(Message::WindowMoved(point.x, point.y))
        }
        iced::Event::Window(window::Event::Resized(size)) => {
            Some(Message::WindowResized(size.width, size.height))
        }
        // Shift+Space toggles the selected tag.
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Space),
            modifiers,
            ..
        }) if modifiers.shift() => Some(Message::FolderWorkspace(
            folder_workspace::Message::TagPanel(tag_panel::Message::ToggleSelectedTag),
        )),
        // Space toggles video play/pause — only when no text widget has focus.
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Space),
            ..
        }) if matches!(status, event::Status::Ignored) => Some(Message::FolderWorkspace(
            folder_workspace::Message::MediaViewer(
                media_viewer::Message::Video(media_viewer_video::Message::TogglePause),
            ),
        )),
        // F5 toggles fullscreen for the media viewer.
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::F5),
            ..
        }) => Some(Message::FolderWorkspace(
            folder_workspace::Message::ToggleMediaFullscreen,
        )),
        // [ / ] always set segment IN/OUT (even when search bar has focus).
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Character(c),
            ..
        }) if c.as_ref() == "[" => Some(Message::FolderWorkspace(
            folder_workspace::Message::SetSegmentStart,
        )),
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Character(c),
            ..
        }) if c.as_ref() == "]" => Some(Message::FolderWorkspace(
            folder_workspace::Message::SetSegmentEnd,
        )),
        // Escape: handled by FolderWorkspace (exits fullscreen or clears search filter).
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Escape),
            ..
        }) => Some(Message::FolderWorkspace(
            folder_workspace::Message::EscapePressed,
        )),
        // Enter: always consume to prevent Windows Default Beep (WM_CHAR 0x0D reaching
        // DefWindowProc). Only trigger SaveSelectedTag when no widget captured the event.
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(keyboard::key::Named::Enter),
            ..
        }) => {
            if matches!(status, event::Status::Ignored) {
                Some(Message::FolderWorkspace(folder_workspace::Message::SaveSelectedTag))
            } else {
                Some(Message::Noop)
            }
        }
        iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
            if matches!(status, event::Status::Ignored) =>
        {
            if modifiers.command() {
                return match key.as_ref() {
                    keyboard::Key::Character("c") => Some(Message::FolderWorkspace(
                        folder_workspace::Message::CopyTags,
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
    }
}

/// Keeps a window-tagged event only when it came from the main window.
fn only_from_main_window(
    (main_window, (window_id, message)): (window::Id, (window::Id, Message)),
) -> Option<Message> {
    (window_id == main_window).then_some(message)
}

/// Settings window size (logical px). The window is not resizable, so this must fit every
/// section: a setting below the bottom edge is simply not seen.
const SETTINGS_WINDOW_SIZE: iced::Size = iced::Size::new(560.0, 560.0);

/// Application state: top-level features only. No knowledge of child UI or structure.
pub struct FrenameApp {
    drag_drop_state: drag_drop::DragDropState,
    folder_workspace: folder_workspace::FolderWorkspace,
    settings: settings::SettingsState,
    /// Window ID waiting for GStreamer unload before we close.
    pending_close: Option<window::Id>,
    /// The main window; closing it ends the app.
    main_window: window::Id,
    /// The settings window, while it is open.
    settings_window: Option<window::Id>,
    /// Icon shared by every window the app opens.
    window_icon: Option<window::Icon>,
    /// Last known window position (logical px); updated on Moved events when not maximized.
    window_pos: (f32, f32),
    /// Last known window size (logical px); updated on Resized events when not maximized.
    window_size: (f32, f32),
    /// Whether the window is currently maximized.
    is_maximized: bool,
    /// Last known monitor size (logical px); 0×0 if not yet fetched.
    monitor_size: (f32, f32),
}

impl FrenameApp {
    /// `main_window` is the id the main window was opened with; `window_icon` is reused for
    /// the windows the app opens later.
    pub fn new(main_window: window::Id, window_icon: Option<window::Icon>) -> Self {
        let saved = AppDatabase::new().get_window_state();
        let settings = settings::SettingsState::default();
        // Before the first folder scan, which already reads comments and in/out points.
        frename_core::set_comment_storage(settings.settings().comment_storage);
        frename_core::set_in_out_storage(settings.settings().in_out_storage);
        Self {
            drag_drop_state: drag_drop::DragDropState::default(),
            folder_workspace: folder_workspace::FolderWorkspace::new(),
            settings,
            pending_close: None,
            main_window,
            settings_window: None,
            window_icon,
            window_pos: saved.map(|g| (g.x, g.y)).unwrap_or((0.0, 0.0)),
            window_size: saved.map(|g| (g.width, g.height)).unwrap_or((1200.0, 600.0)),
            is_maximized: saved.map(|g| g.is_maximized).unwrap_or(false),
            monitor_size: saved.map(|g| (g.monitor_width, g.monitor_height)).unwrap_or((0.0, 0.0)),
        }
    }
}

impl FrenameApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Only the main window's events get through the subscription filter.
            Message::WindowReady => Task::done(Message::FolderWorkspace(
                folder_workspace::Message::LoadLastSession,
            )),
            Message::WindowClosed(id) => {
                if id == self.main_window {
                    return iced::exit();
                }
                if self.settings_window == Some(id) {
                    self.settings_window = None;
                }
                Task::none()
            }
            Message::OpenSettings => {
                let check = if self.settings_window.is_none() { self.check_folder_conversion() } else { Task::none() };
                Task::batch([self.open_settings_window(), check])
            }
            Message::Settings(msg) => {
                self.settings.update(msg.clone());
                // Tag colors are read from the settings at view time; autoplay lives in the player;
                // comment and in/out storage live in core, which saves them. A storage change
                // re-checks the open folder, which offers converting it.
                match msg {
                    settings::Message::SetCommentStorage(storage) => {
                        frename_core::set_comment_storage(storage);
                        self.check_folder_conversion()
                    }
                    settings::Message::SetInOutStorage(storage) => {
                        frename_core::set_in_out_storage(storage);
                        self.check_folder_conversion()
                    }
                    settings::Message::ConvertFolder => {
                        self.settings.set_conversion(settings::FolderConversion::Converting);
                        Task::done(Message::FolderWorkspace(folder_workspace::Message::ConvertMetadata(
                            self.settings.metadata_storage(),
                        )))
                    }
                    settings::Message::SetAutoplayVideo(autoplay) => {
                        Task::done(Message::FolderWorkspace(folder_workspace::Message::MediaViewer(
                            media_viewer::Message::Video(media_viewer_video::Message::SetAutoplay(autoplay)),
                        )))
                    }
                    settings::Message::SetMonochromeTags(_) => Task::none(),
                }
            }
            Message::FolderConversionPlanned(plan) => {
                self.settings.set_plan(plan);
                Task::none()
            }
            Message::FolderWorkspace(folder_workspace::Message::Folder(folder::Message::OpenSettings)) => {
                Task::done(Message::OpenSettings)
            }
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
                if !self.is_maximized {
                    self.window_pos = (x, y);
                }
                let fetch = self.fetch_window_extra();
                self.save_window_state();
                fetch
            }
            Message::WindowResized(width, height) => {
                if !self.is_maximized {
                    self.window_size = (width, height);
                }
                let fetch = self.fetch_window_extra();
                self.save_window_state();
                fetch
            }
            Message::WindowMaximizedFetched(maximized) => {
                self.is_maximized = maximized;
                self.save_window_state();
                Task::none()
            }
            Message::WindowMonitorSizeFetched(size) => {
                if let Some(s) = size {
                    self.monitor_size = (s.width, s.height);
                    self.save_window_state();
                }
                Task::none()
            }
            Message::DragDrop(drag_drop::Message::FileDropped(path)) => {
                self.drag_drop_state.handle_file_dropped(path.clone());
                Task::done(Message::FolderWorkspace(
                    folder_workspace::Message::OpenFile(path),
                ))
            }
            Message::Noop => Task::none(),
            Message::FolderWorkspace(msg) => {
                // The settings window reports the conversion it started, and re-checks when
                // another folder opens (but keeps a report the user has not seen replaced).
                if let folder_workspace::Message::MetadataConverted { report, .. } = &msg {
                    self.settings
                        .set_conversion(settings::FolderConversion::Done(report.clone()));
                }
                let recheck = matches!(msg, folder_workspace::Message::FolderLoaded { .. })
                    && self.settings_window.is_some()
                    && !matches!(
                        self.settings.conversion(),
                        settings::FolderConversion::Converting
                            | settings::FolderConversion::Done(_)
                    );
                let is_unloaded = matches!(
                    &msg,
                    folder_workspace::Message::MediaViewer(media_viewer::Message::Unloaded)
                );
                let task = self.folder_workspace.update(msg).map(Message::FolderWorkspace);
                let Some(id) = is_unloaded.then(|| self.pending_close.take()).flatten() else {
                    return task;
                };
                // After the update: the check reads the folder the workspace now holds.
                let task = if recheck {
                    Task::batch([task, self.check_folder_conversion()])
                } else {
                    task
                };
                Task::batch([task, window::close(id)])
            }
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.settings_window == Some(window_id) {
            return settings::view::view(&self.settings).map(Message::Settings);
        }
        let tag_palette = TagPalette::from_monochrome(self.settings.settings().monochrome_tags);
        folder_workspace::view::view(&self.folder_workspace, tag_palette).map(Message::FolderWorkspace)
    }

    /// Check the open folder against the storage settings on a blocking thread, for the
    /// "Current folder" part of the settings window.
    fn check_folder_conversion(&mut self) -> Task<Message> {
        let Some(dir) = self.folder_workspace.directory() else {
            self.settings
                .set_conversion(settings::FolderConversion::NoFolder);
            return Task::none();
        };
        let paths = dir.all_file_paths();
        let storage = self.settings.metadata_storage();
        self.settings
            .set_conversion(settings::FolderConversion::Checking);
        Task::future(async move {
            let plan =
                tokio::task::spawn_blocking(move || frename_core::plan_conversion(&paths, storage))
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("folder conversion check failed: {e}");
                        frename_core::ConversionPlan {
                            storage,
                            ..Default::default()
                        }
                    });
            Message::FolderConversionPlanned(plan)
        })
    }

    /// Open the settings window, or focus it when it is already open.
    fn open_settings_window(&mut self) -> Task<Message> {
        if let Some(id) = self.settings_window {
            return window::gain_focus(id);
        }
        let (id, open) = window::open(window::Settings {
            size: SETTINGS_WINDOW_SIZE,
            position: window::Position::Centered,
            resizable: false,
            icon: self.window_icon.clone(),
            ..window::Settings::default()
        });
        self.settings_window = Some(id);
        open.discard()
    }

    /// Feature subscriptions (file drop, window opened, global keyboard to search bar).
    pub fn subscription(&self) -> Subscription<Message> {
        // Conditional: when tags are on the internal clipboard, intercept Ctrl+V globally
        // (even when search bar has focus) so it pastes tags instead of text.
        let paste_tags_sub = if self.folder_workspace.has_copied_tags() {
            event::listen_with(|ev, status, window_id| {
                ctrl_v_paste_tags_handler(ev, status, window_id).map(|m| (window_id, m))
            })
            .with(self.main_window)
            .filter_map(only_from_main_window)
        } else {
            Subscription::none()
        };
        Subscription::batch([
            self.drag_drop_state.subscription().map(Message::DragDrop),
            self.folder_workspace
                .subscription()
                .map(Message::FolderWorkspace),
            window::close_requests().map(Message::CloseRequested),
            paste_tags_sub,
            event::listen_with(|ev, status, window_id| {
                main_window_event(ev, status, window_id).map(|m| (window_id, m))
            })
            .with(self.main_window)
            .filter_map(only_from_main_window),
            window::close_events().map(Message::WindowClosed),
        ])
    }

    fn save_window_state(&self) {
        AppDatabase::new().set_window_state(WindowGeometry {
            x: self.window_pos.0,
            y: self.window_pos.1,
            width: self.window_size.0,
            height: self.window_size.1,
            is_maximized: self.is_maximized,
            monitor_width: self.monitor_size.0,
            monitor_height: self.monitor_size.1,
            left_panel_width: self.folder_workspace.left_width(),
            folder_panel_width: self.folder_workspace.folder_width(),
        });
    }

    /// Spawn tasks to fetch is_maximized and monitor_size for the current window.
    fn fetch_window_extra(&self) -> Task<Message> {
        let id = self.main_window;
        Task::batch([
            window::is_maximized(id).map(Message::WindowMaximizedFetched),
            window::monitor_size(id).map(Message::WindowMonitorSizeFetched),
        ])
    }

    /// Window title: the settings window has a fixed one; the main window shows the open file.
    pub fn title(&self, window_id: window::Id) -> String {
        if self.settings_window == Some(window_id) {
            return String::from("Settings");
        }
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
        let _app = FrenameApp::new(window::Id::unique(), None);
    }
}
