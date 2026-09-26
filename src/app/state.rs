//! Root application state and coordination.
//!
//! The app only holds top-level features and uses their public API (update, view, subscription).
//! It does not know how any feature looks (scrollable, layout, panels) or what internals they have.
//! Only `folder_workspace.current_file()` is used for the window title.
//!
//! UI-level concern: when a typing key is pressed and no text field is focused, focus the search bar
//! and send the key so it can be emulated into the filter (Iced does not replay events to widgets).

use iced::{event, keyboard, window, Element, Subscription, Task};

use crate::features::{
    batch, drag_drop, folder, folder_workspace, media_viewer,
    media_viewer::video as media_viewer_video, settings, tag_panel,
};
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
            return Some(Message::FolderWorkspace(
                folder_workspace::Message::PasteTags,
            ));
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
            folder_workspace::Message::MediaViewer(media_viewer::Message::Video(
                media_viewer_video::Message::TogglePause,
            )),
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
                Some(Message::FolderWorkspace(
                    folder_workspace::Message::SaveSelectedTag,
                ))
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
                    keyboard::key::Named::ArrowLeft => Some(Message::FolderWorkspace(
                        folder_workspace::Message::TagPanel(tag_panel::Message::SelectLeft),
                    )),
                    keyboard::key::Named::ArrowRight => Some(Message::FolderWorkspace(
                        folder_workspace::Message::TagPanel(tag_panel::Message::SelectRight),
                    )),
                    keyboard::key::Named::ArrowUp => Some(Message::FolderWorkspace(
                        folder_workspace::Message::TagPanel(tag_panel::Message::SelectUp),
                    )),
                    keyboard::key::Named::ArrowDown => Some(Message::FolderWorkspace(
                        folder_workspace::Message::TagPanel(tag_panel::Message::SelectDown),
                    )),
                    keyboard::key::Named::PageUp => Some(Message::FolderWorkspace(
                        folder_workspace::Message::Folder(folder::Message::PreviousFile),
                    )),
                    keyboard::key::Named::PageDown => Some(Message::FolderWorkspace(
                        folder_workspace::Message::Folder(folder::Message::NextFile),
                    )),
                    keyboard::key::Named::Delete => Some(Message::FolderWorkspace(
                        folder_workspace::Message::RemoveTag,
                    )),
                    _ => None,
                };
                if msg.is_some() {
                    return msg;
                }
            }
            let k = match key.as_ref() {
                keyboard::Key::Character(c) => c
                    .chars()
                    .next()
                    .map(folder_workspace::GlobalSearchKey::Char),
                keyboard::Key::Named(keyboard::key::Named::Backspace) => {
                    Some(folder_workspace::GlobalSearchKey::Backspace)
                }
                keyboard::Key::Named(keyboard::key::Named::Delete) => {
                    Some(folder_workspace::GlobalSearchKey::Delete)
                }
                _ => None,
            };
            k.map(|key| {
                Message::FolderWorkspace(folder_workspace::Message::FocusSearchBarAndKey(key))
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
const SETTINGS_WINDOW_SIZE: iced::Size = iced::Size::new(560.0, 660.0);

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
    /// The demo being run (`--demo`), if any.
    demo: Option<crate::demo::DemoRun>,
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
        frename_core::set_commented_tag(settings.settings().effective_commented_tag());
        frename_core::set_space_after_tags(settings.settings().space_after_tags);
        Self {
            drag_drop_state: drag_drop::DragDropState::default(),
            folder_workspace: folder_workspace::FolderWorkspace::new(),
            settings,
            pending_close: None,
            main_window,
            settings_window: None,
            window_icon,
            window_pos: saved.map(|g| (g.x, g.y)).unwrap_or((0.0, 0.0)),
            window_size: saved
                .map(|g| (g.width, g.height))
                .unwrap_or((1200.0, 600.0)),
            is_maximized: saved.map(|g| g.is_maximized).unwrap_or(false),
            monitor_size: saved
                .map(|g| (g.monitor_width, g.monitor_height))
                .unwrap_or((0.0, 0.0)),
            demo: None,
        }
    }

    /// Run `demo` instead of a normal session.
    pub fn with_demo(mut self, demo: Option<crate::demo::DemoRun>) -> Self {
        self.demo = demo;
        self
    }
}

impl FrenameApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Only the main window's events get through the subscription filter.
            Message::WindowReady => {
                let load = Task::batch([
                    Task::done(Message::FolderWorkspace(
                        folder_workspace::Message::LoadLastSession,
                    )),
                    self.send_subtitle_config(),
                ]);
                if self.demo.is_none() {
                    return load;
                }
                Task::batch([load, crate::demo::DemoRun::start().map(Message::Demo)])
            }
            Message::Demo(msg) => match &self.demo {
                Some(demo) => demo.update(msg).map(Message::Demo),
                None => Task::none(),
            },
            Message::WindowClosed(id) => {
                if id == self.main_window {
                    return iced::exit();
                }
                if self.settings_window == Some(id) {
                    self.settings_window = None;
                }
                Task::none()
            }
            Message::OpenSettings => self.open_settings_window(),
            Message::Settings(msg) => {
                let task = self.settings.update(msg.clone()).map(Message::Settings);
                // Tag colors are read from the settings at view time; autoplay lives in the player;
                // comment and in/out storage live in core, which saves them. Moving what the
                // files already have is a batch action in the main window.
                let effect = match msg {
                    settings::Message::SetCommentStorage(storage) => {
                        frename_core::set_comment_storage(storage);
                        Task::none()
                    }
                    settings::Message::SetInOutStorage(storage) => {
                        frename_core::set_in_out_storage(storage);
                        Task::none()
                    }
                    settings::Message::SetCommentedTag(_)
                    | settings::Message::SetCommentedTagEnabled(_) => {
                        // The field holds the cleaned tag by now.
                        frename_core::set_commented_tag(
                            self.settings.settings().effective_commented_tag(),
                        );
                        Task::none()
                    }
                    settings::Message::OpenBatchAction(operation) => Task::batch([
                        Task::done(Message::FolderWorkspace(
                            folder_workspace::Message::PrepareBatch(operation),
                        )),
                        window::gain_focus(self.main_window),
                    ]),
                    settings::Message::SetAutoplayVideo(autoplay) => {
                        Task::done(Message::FolderWorkspace(
                            folder_workspace::Message::MediaViewer(media_viewer::Message::Video(
                                media_viewer_video::Message::SetAutoplay(autoplay),
                            )),
                        ))
                    }
                    settings::Message::SetMonochromeTags(_) => Task::none(),
                    settings::Message::SetSpaceAfterTags(space) => {
                        frename_core::set_space_after_tags(space);
                        Task::none()
                    }
                    // A saved key gets its price looked up again, even when it is the same key.
                    settings::Message::SonioxKeySaved(Ok(_)) => Task::batch([
                        Task::done(Message::FolderWorkspace(folder_workspace::Message::Batch(
                            batch::Message::Action(batch::ActionMessage::GenerateSubtitles(
                                batch::generate_subtitles::Message::KeySaved,
                            )),
                        ))),
                        self.send_subtitle_config(),
                    ]),
                    // The subtitle action follows the key and the subtitle settings.
                    settings::Message::SonioxKeyLoaded(_)
                    | settings::Message::SonioxKeySaved(Err(_))
                    | settings::Message::SetSubtitleLanguage(..)
                    | settings::Message::SetSubtitleCueLength(_) => self.send_subtitle_config(),
                    settings::Message::LoadSonioxKey
                    | settings::Message::SonioxKeyInput(_)
                    | settings::Message::ToggleShowSonioxKey
                    | settings::Message::SaveSonioxKey
                    | settings::Message::ReplaceSonioxKey
                    | settings::Message::RemoveSonioxKey
                    | settings::Message::SonioxKeyRemoved(_) => Task::none(),
                };
                Task::batch([task, effect])
            }
            Message::FolderWorkspace(folder_workspace::Message::Folder(
                folder::Message::OpenSettings,
            ))
            | Message::FolderWorkspace(folder_workspace::Message::Batch(batch::Message::Action(
                batch::ActionMessage::OpenSettings,
            ))) => Task::done(Message::OpenSettings),
            // The key is set in the Subtitles section, the last one.
            Message::FolderWorkspace(folder_workspace::Message::Batch(batch::Message::Action(
                batch::ActionMessage::OpenSubtitleSettings,
            ))) => self
                .open_settings_window()
                .chain(iced::widget::operation::snap_to_end(iced::widget::Id::new(
                    settings::view::SETTINGS_SCROLLABLE_ID,
                ))),
            Message::CloseRequested(id) => {
                crate::crash_guard::mark_closing();
                // A batch job may be writing a file: stop it after that file, then close.
                if self.folder_workspace.is_batch_running() {
                    self.pending_close = Some(id);
                    return Task::done(Message::FolderWorkspace(folder_workspace::Message::Batch(
                        crate::features::batch::Message::Cancel,
                    )));
                }
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
                let is_unloaded = matches!(
                    &msg,
                    folder_workspace::Message::MediaViewer(media_viewer::Message::Unloaded)
                );
                let batch_finished = matches!(&msg, folder_workspace::Message::BatchFinished);
                let video_ready = matches!(
                    &msg,
                    folder_workspace::Message::MediaViewer(media_viewer::Message::Video(
                        media_viewer_video::Message::VideoReady { .. }
                    ))
                );
                let mut task = self
                    .folder_workspace
                    .update(msg)
                    .map(Message::FolderWorkspace);
                // The subtitle action was chosen: it needs the key, read once when first needed.
                if !self.settings.soniox_key_requested()
                    && self.folder_workspace.batch_needs_subtitle_key()
                {
                    task = Task::batch([
                        task,
                        Task::done(Message::Settings(settings::Message::LoadSonioxKey)),
                    ]);
                }
                let main_window = self.main_window;
                let demo_steps = match self.demo.as_mut() {
                    Some(demo) if video_ready => demo.video_ready(main_window),
                    _ => None,
                };
                if let Some((steps, capture)) = demo_steps {
                    let steps = steps
                        .into_iter()
                        .map(|step| Task::done(Message::FolderWorkspace(step)));
                    return Task::batch(
                        std::iter::once(task)
                            .chain(steps)
                            .chain([capture.map(Message::Demo)]),
                    );
                }
                // Closing waits for a batch job to stop; the file it reopens is unloaded then.
                if batch_finished && self.pending_close.is_some() {
                    return Task::batch([task, self.close_pending()]);
                }
                let Some(id) = is_unloaded.then(|| self.pending_close.take()).flatten() else {
                    return task;
                };
                Task::batch([task, window::close(id)])
            }
        }
    }

    /// Close the window waiting to close, unloading a video first (it closes on `Unloaded`).
    fn close_pending(&mut self) -> Task<Message> {
        if self.folder_workspace.needs_media_unload() {
            return Task::done(Message::FolderWorkspace(
                folder_workspace::Message::MediaViewer(media_viewer::Message::Unload),
            ));
        }
        self.pending_close
            .take()
            .map_or_else(Task::none, window::close)
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if self.settings_window == Some(window_id) {
            return settings::view::view(&self.settings).map(Message::Settings);
        }
        let tag_palette = TagPalette::from_monochrome(self.settings.settings().monochrome_tags);
        folder_workspace::view::view(&self.folder_workspace, tag_palette)
            .map(Message::FolderWorkspace)
    }

    /// Tell the subtitle action what it needs from the settings: the key, the languages and
    /// the cue length.
    fn send_subtitle_config(&self) -> Task<Message> {
        let config = self.settings.subtitle_config();
        Task::done(Message::FolderWorkspace(folder_workspace::Message::Batch(
            batch::Message::Action(batch::ActionMessage::GenerateSubtitles(
                batch::generate_subtitles::Message::SetConfig(config),
            )),
        )))
    }

    /// Open the settings window, or focus it when it is already open. The Soniox key is read
    /// the first time, for its section.
    fn open_settings_window(&mut self) -> Task<Message> {
        let load_key = Task::done(Message::Settings(settings::Message::LoadSonioxKey));
        if let Some(id) = self.settings_window {
            return Task::batch([window::gain_focus(id), load_key]);
        }
        let (id, open) = window::open(window::Settings {
            size: SETTINGS_WINDOW_SIZE,
            position: window::Position::Centered,
            resizable: false,
            icon: self.window_icon.clone(),
            ..window::Settings::default()
        });
        self.settings_window = Some(id);
        Task::batch([open.discard(), load_key])
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
        self.folder_workspace.current_file().map_or_else(
            || String::from("frename"),
            |f| f.file_path().display().to_string(),
        )
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
