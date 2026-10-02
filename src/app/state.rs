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
    batch, drag_drop, drag_out, file_menu, folder, folder_workspace, media_viewer,
    media_viewer::video as media_viewer_video, settings, tag_panel, updates, video_controls,
};
use crate::ui::palette::TagPalette;
use frename_core::ai::key::{self as api_key, ApiKey};
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
        key,
        physical_key,
        modifiers,
        ..
    }) = ev
    {
        if latin_key(&key, physical_key) == Some('v') && modifiers.command() {
            return Some(Message::FolderWorkspace(
                folder_workspace::Message::PasteTags,
            ));
        }
    }
    None
}

/// The Latin letter a key stands for, so Ctrl+Z, Y, C and V work on any keyboard layout: with a
/// Russian layout the key is `я` but the physical key is still Z.
fn latin_key(key: &keyboard::Key, physical_key: keyboard::key::Physical) -> Option<char> {
    key.to_latin(physical_key).map(|c| c.to_ascii_lowercase())
}

/// What a restore holds, in words: `5 tags, 3 markers, comment`.
fn recovery_what(summary: frename_core::recovery::Summary) -> String {
    let mut parts = Vec::new();
    if summary.tags > 0 {
        parts.push(fl!("recovery-tags", count = summary.tags));
    }
    if summary.markers > 0 {
        parts.push(fl!("recovery-markers", count = summary.markers));
    }
    if summary.comment {
        parts.push(fl!("recovery-comment"));
    }
    if summary.in_out {
        parts.push(fl!("recovery-in-out"));
    }
    if parts.is_empty() {
        parts.push(fl!("recovery-edits"));
    }
    parts.join(", ")
}

/// Apply the edits an earlier crash left in the recovery journal, when this is the only frename
/// running; the messages to show and the first restored clip (to open it, so the message
/// appears over it). The restore is synchronous on purpose: it must run before any video is
/// open (Windows locks a playing clip), and it is a few small file operations.
fn restore_after_crash(
    lock: Option<&frename_core::recovery::InstanceLock>,
) -> (Vec<String>, Option<frename_core::FolderAndFile>) {
    use frename_core::recovery::{KeptWhy, Restored};
    let mut notes = Vec::new();
    let mut open = None;
    let Some(lock) = lock else {
        log::info!("recovery: another frename is running; leaving the journal to it");
        return (notes, open);
    };
    for report in frename_core::recovery::restore_all(lock) {
        let file_name = |path: &std::path::Path| {
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        };
        let folder_of = |path: &std::path::Path| {
            path.parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        };
        match report {
            Restored::Applied { path, summary } => {
                log::info!("recovery: restored {summary:?} on {path:?}");
                notes.push(fl!(
                    "recovery-restored",
                    clip = file_name(&path),
                    what = recovery_what(summary)
                ));
                if open.is_none() {
                    open = path
                        .parent()
                        .map(|folder| frename_core::FolderAndFile::new(folder, Some(&path)));
                }
            }
            Restored::Kept {
                clip,
                kept,
                why,
                summary,
            } => {
                log::warn!("recovery: edits on {clip:?} kept at {kept:?}: {why:?}");
                let why = match why {
                    KeptWhy::ClipGone => fl!("recovery-why-gone"),
                    KeptWhy::ClipChanged => fl!("recovery-why-changed"),
                    KeptWhy::NotWritten => fl!("recovery-why-not-written"),
                };
                notes.push(fl!(
                    "recovery-kept",
                    clip = file_name(&clip),
                    what = recovery_what(summary),
                    why = why,
                    folder = folder_of(&kept)
                ));
            }
            Restored::Ignored { kept } => {
                log::warn!("recovery: an unreadable journal file was set aside at {kept:?}");
                notes.push(fl!("recovery-unreadable", folder = folder_of(&kept)));
            }
        }
    }
    (notes, open)
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
        // The keyboard modifiers Ctrl/Shift+click on a file row reads (a mouse click carries
        // none of its own in iced). Cleared on losing focus, so a Ctrl released while the
        // window was unfocused cannot leave a stuck modifier behind.
        iced::Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => Some(
            Message::FolderWorkspace(folder_workspace::Message::ModifiersChanged(modifiers)),
        ),
        iced::Event::Window(window::Event::Unfocused) => Some(Message::FolderWorkspace(
            folder_workspace::Message::ModifiersChanged(keyboard::Modifiers::empty()),
        )),
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
        // F11 shows the open file in Explorer, Shift+F11 copies its path, Ctrl+F11 its name:
        // always, like the other F-keys. A held key acts once.
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modifiers,
            repeat,
            ..
        }) if file_menu::FileAction::from_key(&key, modifiers).is_some() => {
            if repeat {
                return Some(Message::Noop);
            }
            file_menu::FileAction::from_key(&key, modifiers).map(|action| {
                Message::FolderWorkspace(folder_workspace::Message::FileAction(action))
            })
        }
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
        // Ctrl+Alt+← / → turn the open video, also after typing in a search field (like the
        // F-keys), but not while writing a comment; plain arrows stay with the tag grid. A held
        // key turns it once: each turn rewrites the file and reopens the video.
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(arrow),
            modifiers,
            repeat,
            ..
        }) if modifiers.command()
            && modifiers.alt()
            && matches!(
                arrow,
                keyboard::key::Named::ArrowLeft | keyboard::key::Named::ArrowRight
            ) =>
        {
            if repeat {
                return Some(Message::Noop);
            }
            let quarter_turns = if arrow == keyboard::key::Named::ArrowLeft {
                -1
            } else {
                1
            };
            // A text field took the key: the workspace checks it is not the comment box, where
            // the keys belong to the text.
            let turn = if matches!(status, event::Status::Ignored) {
                folder_workspace::Message::RotateVideo(quarter_turns)
            } else {
                folder_workspace::Message::RotateVideoWhileTyping(quarter_turns)
            };
            Some(Message::FolderWorkspace(turn))
        }
        // Alt+← / → step one frame in the open video, also after typing in a search field (like
        // the F-keys), but not while writing a comment; a held key keeps stepping.
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(arrow),
            modifiers,
            ..
        }) if modifiers.alt()
            && !modifiers.command()
            && matches!(
                arrow,
                keyboard::key::Named::ArrowLeft | keyboard::key::Named::ArrowRight
            ) =>
        {
            let step = if arrow == keyboard::key::Named::ArrowLeft {
                video_controls::FrameStep::Back
            } else {
                video_controls::FrameStep::Forward
            };
            // A text field took the key: the workspace checks it is not the comment box, where
            // the keys belong to the text.
            let step = if matches!(status, event::Status::Ignored) {
                folder_workspace::Message::StepFrame(step)
            } else {
                folder_workspace::Message::StepFrameWhileTyping(step)
            };
            Some(Message::FolderWorkspace(step))
        }
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
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            ..
        }) if matches!(status, event::Status::Ignored) => {
            if modifiers.command() {
                return match latin_key(&key, physical_key) {
                    Some('c') => Some(Message::FolderWorkspace(
                        folder_workspace::Message::CopyTags,
                    )),
                    Some('z') if modifiers.shift() => {
                        Some(Message::FolderWorkspace(folder_workspace::Message::Redo))
                    }
                    Some('z') => Some(Message::FolderWorkspace(folder_workspace::Message::Undo)),
                    Some('y') => Some(Message::FolderWorkspace(folder_workspace::Message::Redo)),
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
fn only_from_window(
    (window, (window_id, message)): (window::Id, (window::Id, Message)),
) -> Option<Message> {
    (window_id == window).then_some(message)
}

/// Keys of the settings window: Esc, Ctrl+Tab / Ctrl+Shift+Tab between its pages, Tab /
/// Shift+Tab between the controls of a page, and Space / Enter on the focused control (#167). A
/// focused field takes the first Esc itself (it leaves the field), and Space and Enter (it types
/// or submits). A click takes the focus ring away.
fn settings_window_key(
    ev: iced::Event,
    status: event::Status,
    window_id: window::Id,
) -> Option<(window::Id, Message)> {
    use keyboard::key::Named;
    let ignored = status == event::Status::Ignored;
    let message = match ev {
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(_)) => settings::Message::ClearFocus,
        iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
            keyboard::Key::Named(Named::Escape) if ignored => settings::Message::Escape,
            keyboard::Key::Named(Named::Tab) => match (modifiers.control(), modifiers.shift()) {
                (true, true) => settings::Message::PreviousPage,
                (true, false) => settings::Message::NextPage,
                (false, true) => settings::Message::FocusPrevious,
                (false, false) => settings::Message::FocusNext,
            },
            keyboard::Key::Named(Named::Space) if ignored && modifiers.is_empty() => {
                settings::Message::Press(settings::focus::Press::Space)
            }
            keyboard::Key::Named(Named::Enter) if ignored && modifiers.is_empty() => {
                settings::Message::Press(settings::focus::Press::Enter)
            }
            _ => return None,
        },
        _ => return None,
    };
    Some((window_id, Message::Settings(message)))
}

/// Settings window size (logical px); its pages scroll when they do not fit.
const SETTINGS_WINDOW_SIZE: iced::Size = iced::Size::new(
    crate::ui::tokens::SETTINGS_WINDOW_WIDTH,
    crate::ui::tokens::SETTINGS_WINDOW_HEIGHT,
);

/// The smallest the settings window gets.
const SETTINGS_MIN_SIZE: iced::Size = iced::Size::new(
    crate::ui::tokens::SETTINGS_MIN_WIDTH,
    crate::ui::tokens::SETTINGS_MIN_HEIGHT,
);

/// How often a running frename looks whether the daily update check is due.
const UPDATE_TICK: std::time::Duration = std::time::Duration::from_secs(60 * 60);

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
    /// A folder or file given on the command line, opened instead of the last session.
    initial_path: Option<frename_core::FolderAndFile>,
    /// The clip whose edits were restored after a crash: opened first when no clip was asked for.
    restored_clip: Option<frename_core::FolderAndFile>,
    /// Held while frename runs: tells the next start (and a second frename) whose the journal is.
    _recovery_lock: Option<frename_core::recovery::InstanceLock>,
    /// A downloaded update to apply once the main window has closed.
    pending_update: Option<updates::Release>,
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
        frename_core::set_marker_storage(settings.settings().marker_storage);
        frename_core::set_commented_tag(settings.settings().effective_commented_tag());
        frename_core::set_space_after_tags(settings.settings().space_after_tags);
        crate::i18n::apply(&settings.settings().ui_language);
        // Edits left in the recovery journal mean the last run ended abnormally: apply them now,
        // before any clip is open (nothing locks the clips yet).
        let recovery_lock = frename_core::recovery::InstanceLock::acquire();
        let (recovery_notes, restored_clip) = restore_after_crash(recovery_lock.as_ref());
        let mut folder_workspace = folder_workspace::FolderWorkspace::new();
        for note in recovery_notes {
            folder_workspace.add_startup_note(note);
        }
        Self {
            drag_drop_state: drag_drop::DragDropState::default(),
            folder_workspace,
            restored_clip,
            _recovery_lock: recovery_lock,
            settings,
            pending_close: None,
            main_window,
            settings_window: None,
            window_icon,
            window_pos: saved.map(|g| (g.x, g.y)).unwrap_or((0.0, 0.0)),
            window_size: saved.map(|g| (g.width, g.height)).unwrap_or((
                crate::ui::tokens::WINDOW_WIDTH,
                crate::ui::tokens::WINDOW_HEIGHT,
            )),
            is_maximized: saved.map(|g| g.is_maximized).unwrap_or(false),
            monitor_size: saved
                .map(|g| (g.monitor_width, g.monitor_height))
                .unwrap_or((0.0, 0.0)),
            demo: None,
            initial_path: None,
            pending_update: None,
        }
    }

    /// Run `demo` instead of a normal session.
    pub fn with_demo(mut self, demo: Option<crate::demo::DemoRun>) -> Self {
        self.demo = demo;
        self
    }

    /// Open `path` (from the command line) instead of the last session.
    pub fn with_initial_path(mut self, path: Option<frename_core::FolderAndFile>) -> Self {
        self.initial_path = path;
        self
    }
}

impl FrenameApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Only the main window's events get through the subscription filter.
            Message::WindowReady => {
                let language = Task::done(describe_ai_message(
                    batch::describe_ai::Message::SetLanguage(
                        self.settings.settings().summary_language,
                    ),
                ));
                let model = Task::done(describe_ai_message(batch::describe_ai::Message::SetModel(
                    clipscribe::Model::from_id(&self.settings.settings().ai_model),
                )));
                let open = match self.initial_path.take().or(self.restored_clip.take()) {
                    Some(pair) => folder_workspace::Message::ScanFolder(pair),
                    None => folder_workspace::Message::LoadLastSession,
                };
                let load = Task::batch([
                    language,
                    model,
                    self.subtitle_config(),
                    Task::done(Message::FolderWorkspace(open)),
                ]);
                if self.demo.is_none() {
                    // The daily background update check, when it is due.
                    let check = Task::done(Message::Settings(settings::Message::Updates(
                        updates::Message::Tick,
                    )));
                    return Task::batch([load, check]);
                }
                Task::batch([load, crate::demo::DemoRun::start().map(Message::Demo)])
            }
            Message::Demo(msg) => match &self.demo {
                Some(demo) => demo.update(msg).map(Message::Demo),
                None => Task::none(),
            },
            Message::WindowClosed(id) => {
                if id == self.main_window {
                    // The open file is saved by now; Velopack waits for this process to end.
                    if let Some(release) = self.pending_update.take() {
                        if let Err(e) = updates::apply_on_exit(&release) {
                            log::error!("could not start the update: {e}");
                        }
                    }
                    return iced::exit();
                }
                if self.settings_window == Some(id) {
                    self.settings_window = None;
                }
                Task::none()
            }
            Message::OpenSettings(page) => self.open_settings_on(page),
            Message::Settings(settings::Message::Close) => self.close_settings(),
            // Keyboard focus in Settings (#167): a running batch job holds a button back.
            Message::Settings(settings::Message::FocusNext) => self
                .settings
                .move_focus(1, self.folder_workspace.is_batch_running())
                .map(Message::Settings),
            Message::Settings(settings::Message::FocusPrevious) => self
                .settings
                .move_focus(-1, self.folder_workspace.is_batch_running())
                .map(Message::Settings),
            Message::Settings(settings::Message::Press(press)) => self
                .settings
                .press(press, self.folder_workspace.is_batch_running())
                .map(Message::Settings),
            // Esc cancels a pending key removal or replacement first.
            Message::Settings(settings::Message::Escape) => {
                if self.settings.escape() {
                    Task::none()
                } else {
                    self.close_settings()
                }
            }
            // Downloaded: close the app the way the user would (the open file is saved), then
            // apply the update once the main window is gone.
            Message::Settings(settings::Message::Updates(updates::Message::ApplyAndRestart(
                release,
            ))) => {
                self.pending_update = Some(release);
                Task::done(Message::CloseRequested(self.main_window))
            }
            Message::Settings(msg) => {
                let task = self.settings.update(msg.clone()).map(Message::Settings);
                // Tag colors are read from the settings at view time; autoplay lives in the player;
                // comment and in/out storage live in core, which saves them. Moving what the
                // files already have is a batch action in the main window.
                let effect = match msg {
                    // Both windows redraw in the new language on the next frame.
                    settings::Message::SetUiLanguage(language) => {
                        crate::i18n::apply(&language);
                        Task::none()
                    }
                    settings::Message::SetCommentStorage(storage) => {
                        frename_core::set_comment_storage(storage);
                        Task::none()
                    }
                    settings::Message::SetInOutStorage(storage) => {
                        frename_core::set_in_out_storage(storage);
                        Task::none()
                    }
                    settings::Message::SetMarkerStorage(storage) => {
                        frename_core::set_marker_storage(storage);
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
                    settings::Message::SetSpaceAfterTags(space) => {
                        frename_core::set_space_after_tags(space);
                        Task::none()
                    }
                    settings::Message::SetSummaryLanguage(language) => Task::done(
                        describe_ai_message(batch::describe_ai::Message::SetLanguage(language)),
                    ),
                    settings::Message::SetAiModel(model) => Task::done(describe_ai_message(
                        batch::describe_ai::Message::SetModel(model),
                    )),
                    settings::Message::Key(which, settings::KeyMessage::Save) => {
                        match self.settings.typed_key(which) {
                            Some(key) => {
                                let saved = key_task(
                                    which,
                                    self.settings.begin_key_request(which),
                                    move || {
                                        api_key::save_key(which, &key).map_err(|e| {
                                            log::warn!("{which:?}: saving the key failed: {e}");
                                            "The key could not be saved. The system keyring may \
                                             be locked."
                                                .to_string()
                                        })
                                    },
                                );
                                // A saved Soniox key gets its price looked up again, even when
                                // it is the same key.
                                match which {
                                    ApiKey::Soniox => Task::batch([
                                        saved,
                                        Task::done(subtitles_message(
                                            batch::generate_subtitles::Message::KeySaved,
                                        )),
                                    ]),
                                    ApiKey::Anthropic => saved,
                                }
                            }
                            None => Task::none(),
                        }
                    }
                    settings::Message::Key(which, settings::KeyMessage::Remove) => {
                        key_task(which, self.settings.begin_key_request(which), move || {
                            api_key::delete_key(which).map_err(|e| {
                                log::warn!("{which:?}: removing the key failed: {e}");
                                "The key could not be removed. The system keyring may be locked."
                                    .to_string()
                            })
                        })
                    }
                    // The batch panel shows whether a key is saved too.
                    // Passed on as the settings took it (a stale answer changed nothing).
                    settings::Message::Key(which, settings::KeyMessage::State { .. }) => {
                        match (which, self.settings.key(which).state) {
                            (ApiKey::Anthropic, Some(state)) => Task::done(describe_ai_message(
                                batch::describe_ai::Message::KeyState(state),
                            )),
                            (ApiKey::Soniox, Some(state)) => {
                                let passed = Task::done(subtitles_message(
                                    batch::generate_subtitles::Message::KeyState(state),
                                ));
                                // With a key known, the window lists all of Soniox's languages.
                                let window_open = self.settings_window.is_some();
                                let list = if state == api_key::KeyState::Saved
                                    && window_open
                                    && self.settings.begin_subtitle_languages()
                                {
                                    subtitle_languages_task()
                                } else {
                                    Task::none()
                                };
                                Task::batch([passed, list])
                            }
                            (_, None) => Task::none(),
                        }
                    }
                    settings::Message::Key(..) => Task::none(),
                    settings::Message::SetSubtitleLanguage(..)
                    | settings::Message::SetSubtitleCueLength(_) => self.subtitle_config(),
                    settings::Message::SetMonochromeTags(_)
                    | settings::Message::ShowPage(_)
                    | settings::Message::NextPage
                    | settings::Message::PreviousPage
                    | settings::Message::Close
                    | settings::Message::Escape
                    | settings::Message::FocusNext
                    | settings::Message::FocusPrevious
                    | settings::Message::Press(_)
                    | settings::Message::ClearFocus
                    | settings::Message::Updates(_)
                    | settings::Message::ImportOldSettings
                    | settings::Message::OldSettingsFolderPicked(_)
                    | settings::Message::SubtitleLanguagesListed(_) => Task::none(),
                };
                Task::batch([task, effect])
            }
            Message::FolderWorkspace(folder_workspace::Message::Batch(batch::Message::Action(
                batch::ActionMessage::OpenAiSettings,
            ))) => self.open_settings_on(Some(settings::Page::Ai)),
            Message::FolderWorkspace(folder_workspace::Message::Batch(batch::Message::Action(
                batch::ActionMessage::OpenSubtitleSettings,
            ))) => self.open_settings_on(Some(settings::Page::Subtitles)),
            // One reader of a key's state: the settings, which pass it on to the batch panel.
            Message::FolderWorkspace(folder_workspace::Message::Batch(batch::Message::Action(
                batch::ActionMessage::ReadKeyState,
            ))) => key_task(
                ApiKey::Anthropic,
                self.settings.begin_key_request(ApiKey::Anthropic),
                || Ok(()),
            ),
            Message::FolderWorkspace(folder_workspace::Message::Batch(batch::Message::Action(
                batch::ActionMessage::ReadSonioxKeyState,
            ))) => key_task(
                ApiKey::Soniox,
                self.settings.begin_key_request(ApiKey::Soniox),
                || Ok(()),
            ),
            Message::FolderWorkspace(folder_workspace::Message::Folder(
                folder::Message::OpenSettings,
            )) => Task::done(Message::OpenSettings(None)),
            // Tag commented and Apply tag spacing: their settings are on the Saving page.
            Message::FolderWorkspace(folder_workspace::Message::Batch(batch::Message::Action(
                batch::ActionMessage::OpenSettings,
            ))) => Task::done(Message::OpenSettings(Some(settings::Page::Saving))),
            Message::CloseRequested(id) => {
                crate::crash_guard::mark_closing();
                // A batch job may be writing a file: stop it after that file, then close.
                if self.folder_workspace.is_batch_running() {
                    self.pending_close = Some(id);
                    return Task::done(Message::FolderWorkspace(folder_workspace::Message::Batch(
                        crate::features::batch::Message::Cancel,
                    )));
                }
                // Save the open file's tags, comment and in/out before closing: otherwise they
                // are lost silently (issue #21). Only for the main window: `CloseRequested`
                // also fires for the Settings window's own OS close button (the subscription
                // isn't scoped to one window), and that must not tell folder_workspace the app
                // itself is closing — it would wrongly drop a queued folder scan or skip
                // reopening the video once the (unrelated) unload this triggers finishes.
                let save = if id == self.main_window {
                    self.folder_workspace
                        .flush_open_file()
                        .map(Message::FolderWorkspace)
                } else {
                    Task::none()
                };
                if !self.folder_workspace.needs_media_unload() {
                    return Task::batch([save, window::close(id)]);
                }
                self.pending_close = Some(id);
                Task::batch([
                    save,
                    Task::done(Message::FolderWorkspace(
                        folder_workspace::Message::MediaViewer(media_viewer::Message::Unload),
                    )),
                ])
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
                self.drag_drop_state
                    .handle_file_dropped(path, std::time::Instant::now());
                Task::none()
            }
            Message::DragDrop(drag_drop::Message::Tick(now)) => {
                match self.drag_drop_state.handle_tick(now) {
                    Some(path) => Task::done(Message::FolderWorkspace(
                        folder_workspace::Message::OpenPath(path),
                    )),
                    None => Task::none(),
                }
            }
            Message::Noop => Task::none(),
            // The drag out of the window runs the system's drag loop on the window's thread.
            Message::FolderWorkspace(folder_workspace::Message::StartDragOut(paths)) => {
                self.drag_drop_state
                    .begin_own_drag(paths.clone(), std::time::Instant::now());
                window::run(self.main_window, move |window| {
                    drag_out::start(window, &paths)
                })
                .map(|()| Message::FolderWorkspace(folder_workspace::Message::DragOutFinished))
            }
            Message::FolderWorkspace(folder_workspace::Message::DragOutFinished) => {
                self.drag_drop_state.end_own_drag(std::time::Instant::now());
                self.folder_workspace
                    .update(folder_workspace::Message::DragOutFinished)
                    .map(Message::FolderWorkspace)
            }
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
                let task = self
                    .folder_workspace
                    .update(msg)
                    .map(Message::FolderWorkspace);
                let demo_steps = match self.demo.as_mut() {
                    Some(demo) if video_ready => demo.video_ready(),
                    _ => None,
                };
                if let Some(steps) = demo_steps {
                    let steps = steps
                        .into_iter()
                        .map(|step| Task::done(Message::FolderWorkspace(step)));
                    return Task::batch(
                        std::iter::once(task)
                            .chain(steps)
                            .chain([self.demo_capture()]),
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
            return settings::view::view(&self.settings, self.folder_workspace.is_batch_running())
                .map(Message::Settings);
        }
        let tag_palette = TagPalette::from_monochrome(self.settings.settings().monochrome_tags);
        let update_available = self.settings.updates().available_version();
        folder_workspace::view::view(&self.folder_workspace, tag_palette, update_available)
            .map(Message::FolderWorkspace)
    }

    /// Tell the subtitle action the settings it uses: the languages and the cue length.
    fn subtitle_config(&self) -> Task<Message> {
        let settings = self.settings.settings();
        Task::done(subtitles_message(
            batch::generate_subtitles::Message::SetConfig(batch::generate_subtitles::Config {
                languages: settings.subtitle_languages.clone(),
                cue_length: settings.subtitle_cue_length,
            }),
        ))
    }

    /// The demo's screenshot: of the main window, or of the settings window, opened for it.
    fn demo_capture(&mut self) -> Task<Message> {
        let settings_page = self.demo.as_ref().and_then(|d| d.settings_page());
        let (open, window, size) = match settings_page {
            Some(page) => {
                let tabs = self.demo.as_ref().map_or(0, |d| d.settings_tabs());
                let open = Task::batch(std::iter::once(self.open_settings_on(Some(page))).chain(
                    (0..tabs).map(|_| Task::done(Message::Settings(settings::Message::FocusNext))),
                ));
                let id = self.settings_window.unwrap_or(self.main_window);
                let size = SETTINGS_WINDOW_SIZE;
                (open, id, Some((size.width as u32, size.height as u32)))
            }
            None => (Task::none(), self.main_window, None),
        };
        match self.demo.as_ref() {
            Some(demo) => Task::batch([open, demo.capture(window, size).map(Message::Demo)]),
            None => open,
        }
    }

    /// Open the settings window on `page`, or bring it to the front there. Without a page it
    /// shows the page it showed last, or Updates when an update is ready (the dot on the
    /// settings button leads there).
    fn open_settings_on(&mut self, page: Option<settings::Page>) -> Task<Message> {
        let update_ready = self.settings.updates().available_version().is_some();
        let page = settings::Page::to_open(page, update_ready);
        let show = page.map_or_else(Task::none, |page| {
            self.settings.show_page(page).map(Message::Settings)
        });
        if let Some(id) = self.settings_window {
            return Task::batch([window::gain_focus(id), show]);
        }
        // Whether a key is saved is read each time the window opens, not at start-up: reading
        // may unlock a keyring, and a keyring locked before may be open now.
        let read_key = if self.demo.is_some() {
            let states =
                crate::demo::DemoRun::key_states(|which| self.settings.begin_key_request(which));
            Task::batch(states.into_iter().map(|(which, message)| {
                Task::done(Message::Settings(settings::Message::Key(which, message)))
            }))
        } else {
            Task::batch(
                [ApiKey::Anthropic, ApiKey::Soniox].map(|which| {
                    key_task(which, self.settings.begin_key_request(which), || Ok(()))
                }),
            )
        };
        let (id, open) = window::open(window::Settings {
            size: SETTINGS_WINDOW_SIZE,
            min_size: Some(SETTINGS_MIN_SIZE),
            position: window::Position::Centered,
            resizable: true,
            icon: self.window_icon.clone(),
            ..window::Settings::default()
        });
        self.settings_window = Some(id);
        Task::batch([open.discard(), read_key, show])
    }

    fn close_settings(&mut self) -> Task<Message> {
        self.settings_window.map_or_else(Task::none, window::close)
    }

    /// The theme of a window: the design system's in Settings; the main window keeps iced's dark
    /// theme until it moves onto the system (#59).
    /// Every window is on the design system.
    pub fn theme(&self, _window_id: window::Id) -> iced::Theme {
        crate::ui::theme()
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
            .filter_map(only_from_window)
        } else {
            Subscription::none()
        };
        let settings_keys = match self.settings_window {
            Some(id) => event::listen_with(settings_window_key)
                .with(id)
                .filter_map(only_from_window),
            None => Subscription::none(),
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
            .filter_map(only_from_window),
            settings_keys,
            window::close_events().map(Message::WindowClosed),
            iced::time::every(UPDATE_TICK)
                .map(|_| Message::Settings(settings::Message::Updates(updates::Message::Tick))),
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
            return fl!("settings-window-title");
        }
        self.folder_workspace.current_file().map_or_else(
            || String::from("frename"),
            |f| f.file_path().display().to_string(),
        )
    }
}

/// A message for the "Describe with AI" batch action, from the settings.
fn describe_ai_message(message: batch::describe_ai::Message) -> Message {
    Message::FolderWorkspace(folder_workspace::Message::Batch(batch::Message::Action(
        batch::ActionMessage::DescribeAi(message),
    )))
}

/// A message for the "Generate subtitles" batch action, from the settings.
fn subtitles_message(message: batch::generate_subtitles::Message) -> Message {
    Message::FolderWorkspace(folder_workspace::Message::Batch(batch::Message::Action(
        batch::ActionMessage::GenerateSubtitles(message),
    )))
}

/// Ask Soniox for the languages it recognises, on a worker thread.
fn subtitle_languages_task() -> Task<Message> {
    Task::future(async {
        let result = tokio::task::spawn_blocking(batch::generate_subtitles::supported_languages)
            .await
            .unwrap_or_else(|e| Err(e.to_string()));
        Message::Settings(settings::Message::SubtitleLanguagesListed(result))
    })
}

/// Run `change` on the credential store on a worker thread (it may wait on a keyring), then
/// read back whether the `which` key is saved.
fn key_task(
    which: ApiKey,
    request: u64,
    change: impl FnOnce() -> Result<(), String> + Send + 'static,
) -> Task<Message> {
    Task::future(async move {
        let result =
            tokio::task::spawn_blocking(move || change().map(|()| api_key::key_state(which)))
                .await
                .unwrap_or_else(|e| Err(e.to_string()));
        Message::Settings(settings::Message::Key(
            which,
            settings::KeyMessage::State { request, result },
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let _app = FrenameApp::new(window::Id::unique(), None);
    }

    fn ctrl_key(
        key: &str,
        code: keyboard::key::Code,
        shift: bool,
    ) -> (iced::Event, event::Status, window::Id) {
        let mut modifiers = keyboard::Modifiers::CTRL;
        modifiers.set(keyboard::Modifiers::SHIFT, shift);
        let key = keyboard::Key::Character(key.into());
        let event = iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Code(code),
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        });
        (event, event::Status::Ignored, window::Id::unique())
    }

    fn shortcut(
        key: &str,
        code: keyboard::key::Code,
        shift: bool,
    ) -> Option<folder_workspace::Message> {
        let (ev, status, id) = ctrl_key(key, code, shift);
        match main_window_event(ev, status, id) {
            Some(Message::FolderWorkspace(m)) => Some(m),
            _ => None,
        }
    }

    #[test]
    fn undo_redo_and_copy_work_on_a_russian_layout() {
        use keyboard::key::Code;
        assert!(matches!(
            shortcut("я", Code::KeyZ, false),
            Some(folder_workspace::Message::Undo)
        ));
        assert!(matches!(
            shortcut("я", Code::KeyZ, true),
            Some(folder_workspace::Message::Redo)
        ));
        assert!(matches!(
            shortcut("н", Code::KeyY, false),
            Some(folder_workspace::Message::Redo)
        ));
        assert!(matches!(
            shortcut("с", Code::KeyC, false),
            Some(folder_workspace::Message::CopyTags)
        ));
        assert!(matches!(
            shortcut("z", Code::KeyZ, false),
            Some(folder_workspace::Message::Undo)
        ));
    }

    fn arrow(
        named: keyboard::key::Named,
        modifiers: keyboard::Modifiers,
        status: event::Status,
    ) -> Option<folder_workspace::Message> {
        let key = keyboard::Key::Named(named);
        let event = iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::ArrowLeft),
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: true,
        });
        match main_window_event(event, status, window::Id::unique()) {
            Some(Message::FolderWorkspace(m)) => Some(m),
            _ => None,
        }
    }

    #[test]
    fn alt_arrows_step_frames_and_ctrl_alt_arrows_still_rotate() {
        use folder_workspace::Message as W;
        use keyboard::key::Named;
        use video_controls::FrameStep;
        let alt = keyboard::Modifiers::ALT;
        let idle = event::Status::Ignored;
        assert!(matches!(
            arrow(Named::ArrowLeft, alt, idle),
            Some(W::StepFrame(FrameStep::Back))
        ));
        // Held: auto-repeat keeps stepping.
        assert!(matches!(
            arrow(Named::ArrowRight, alt, idle),
            Some(W::StepFrame(FrameStep::Forward))
        ));
        // A text field had the keys: the workspace decides (not in the comment box).
        assert!(matches!(
            arrow(Named::ArrowRight, alt, event::Status::Captured),
            Some(W::StepFrameWhileTyping(FrameStep::Forward))
        ));
        let ctrl_alt = keyboard::Modifiers::CTRL | keyboard::Modifiers::ALT;
        assert!(!matches!(
            arrow(Named::ArrowLeft, ctrl_alt, idle),
            Some(W::StepFrame(_))
        ));
        assert!(matches!(
            arrow(Named::ArrowLeft, keyboard::Modifiers::empty(), idle),
            Some(W::TagPanel(_))
        ));
    }

    #[test]
    fn paste_works_on_a_russian_layout() {
        let (ev, status, id) = ctrl_key("м", keyboard::key::Code::KeyV, false);
        assert!(matches!(
            ctrl_v_paste_tags_handler(ev, status, id),
            Some(Message::FolderWorkspace(
                folder_workspace::Message::PasteTags
            ))
        ));
    }
}
