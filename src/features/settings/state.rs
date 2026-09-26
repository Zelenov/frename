//! State for the settings feature.

use frename_core::{AppDatabase, AppSettings, AppStateStore};
use iced::Task;

use super::Message;
use crate::features::batch::generate_subtitles::{Config, KeyStatus};
use crate::features::batch::Operation;
use crate::soniox_key::{self, KeyInfo, KeySource, SonioxKey, TypedKey};

/// The Soniox API key as the settings window shows it. The key itself lives in the credential
/// store, read once per session, when first needed.
#[derive(Debug, Default)]
pub struct SonioxKeyState {
    /// What was read; `None` until then.
    info: Option<KeyInfo>,
    /// A read was started.
    requested: bool,
    /// What is typed in the key field.
    input: TypedKey,
    /// Show the typed key instead of dots.
    show: bool,
    /// Replace was pressed: the field is shown over the saved key.
    replacing: bool,
    /// A save or remove is under way.
    busy: bool,
    /// Why the last save or remove failed.
    error: Option<String>,
}

impl SonioxKeyState {
    pub fn info(&self) -> Option<&KeyInfo> {
        self.info.as_ref()
    }

    pub fn input(&self) -> &str {
        &self.input.0
    }

    pub fn show(&self) -> bool {
        self.show
    }

    pub fn busy(&self) -> bool {
        self.busy
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Whether the key field is shown: nothing saved, or Replace pressed.
    pub fn editing(&self) -> bool {
        self.replacing
            || !matches!(
                self.info.as_ref().and_then(|i| i.key.as_ref()),
                Some((_, KeySource::Stored))
            )
    }
}

/// Current app settings, loaded from the app database and saved back on every change.
pub struct SettingsState {
    settings: AppSettings,
    soniox_key: SonioxKeyState,
    /// The comment storage changed since the window last offered moving the files' comments.
    comment_storage_changed: bool,
    /// The in/out storage changed since the window last offered moving the files' points.
    in_out_storage_changed: bool,
    /// The tag spacing changed since the window last offered renaming the files to it.
    tag_spacing_changed: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            settings: AppDatabase::new().get_app_settings().unwrap_or_default(),
            soniox_key: SonioxKeyState::default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
        }
    }
}

impl SettingsState {
    /// Apply a change and persist the result. The Soniox key goes to the credential store
    /// instead, through the returned task.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        if let Some(task) = self.update_key(&message) {
            return task;
        }
        self.apply(message);
        AppDatabase::new().set_app_settings(self.settings.clone());
        Task::none()
    }

    /// Current settings.
    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// The Soniox key as the window shows it.
    pub fn soniox_key(&self) -> &SonioxKeyState {
        &self.soniox_key
    }

    /// Whether the key was read or is being read.
    pub fn soniox_key_requested(&self) -> bool {
        self.soniox_key.requested
    }

    /// What the subtitle action needs from the settings.
    pub fn subtitle_config(&self) -> Config {
        let key = match &self.soniox_key.info {
            None => KeyStatus::Unknown,
            Some(info) => info
                .key()
                .cloned()
                .map_or(KeyStatus::Missing, KeyStatus::Present),
        };
        Config {
            key,
            languages: self.settings.subtitle_languages.clone(),
            cue_length: self.settings.subtitle_cue_length,
        }
    }

    /// Handle a message about the Soniox key; `None` for any other message.
    fn update_key(&mut self, message: &Message) -> Option<Task<Message>> {
        let key = &mut self.soniox_key;
        Some(match message {
            Message::LoadSonioxKey => {
                if std::mem::replace(&mut key.requested, true) {
                    return Some(Task::none());
                }
                blocking(soniox_key::load, Message::SonioxKeyLoaded, |_| {
                    // Read as "no key, nowhere to store one" rather than "Reading…" for good.
                    Message::SonioxKeyLoaded(KeyInfo::default())
                })
            }
            Message::SonioxKeyLoaded(info) => {
                key.info = Some(info.clone());
                Task::none()
            }
            Message::SonioxKeyInput(input) => {
                key.input = input.clone();
                key.error = None;
                Task::none()
            }
            Message::ToggleShowSonioxKey => {
                key.show = !key.show;
                Task::none()
            }
            Message::SaveSonioxKey => {
                let Some(new_key) = SonioxKey::new(&key.input.0).filter(|_| !key.busy) else {
                    return Some(Task::none());
                };
                key.busy = true;
                blocking(
                    move || soniox_key::save(&new_key).map(|()| new_key),
                    Message::SonioxKeySaved,
                    |e| Message::SonioxKeySaved(Err(e)),
                )
            }
            Message::SonioxKeySaved(result) => {
                key.busy = false;
                match result {
                    Ok(saved) => {
                        key.info = Some(KeyInfo {
                            key: Some((saved.clone(), KeySource::Stored)),
                            store_available: true,
                        });
                        key.input = TypedKey::default();
                        key.show = false;
                        key.replacing = false;
                        key.error = None;
                    }
                    Err(e) => key.error = Some(e.clone()),
                }
                Task::none()
            }
            Message::ReplaceSonioxKey => {
                key.replacing = true;
                key.input = TypedKey::default();
                Task::none()
            }
            Message::RemoveSonioxKey => {
                if key.busy {
                    return Some(Task::none());
                }
                key.busy = true;
                blocking(soniox_key::remove, Message::SonioxKeyRemoved, |e| {
                    Message::SonioxKeyRemoved(Err(e))
                })
            }
            Message::SonioxKeyRemoved(result) => {
                key.busy = false;
                match result {
                    // What is left: the environment variable, if set.
                    Ok(()) => {
                        return Some(blocking(soniox_key::load, Message::SonioxKeyLoaded, |_| {
                            Message::SonioxKeyLoaded(KeyInfo::default())
                        }))
                    }
                    Err(e) => key.error = Some(e.clone()),
                }
                Task::none()
            }
            _ => return None,
        })
    }

    /// Whether to offer moving the files' comments to the storage just chosen: files keep
    /// them where they were until moved.
    pub fn comment_storage_changed(&self) -> bool {
        self.comment_storage_changed
    }

    /// Whether to offer moving the files' in/out points to the storage just chosen.
    pub fn in_out_storage_changed(&self) -> bool {
        self.in_out_storage_changed
    }

    /// Whether to offer renaming the files to the tag spacing just chosen.
    pub fn tag_spacing_changed(&self) -> bool {
        self.tag_spacing_changed
    }

    fn apply(&mut self, message: Message) {
        match message {
            Message::SetAutoplayVideo(autoplay) => self.settings.autoplay_video = autoplay,
            Message::SetMonochromeTags(monochrome) => self.settings.monochrome_tags = monochrome,
            Message::SetSpaceAfterTags(space) => {
                self.tag_spacing_changed |= self.settings.space_after_tags != space;
                self.settings.space_after_tags = space;
            }
            Message::SetCommentStorage(storage) => {
                self.comment_storage_changed |= self.settings.comment_storage != storage;
                self.settings.comment_storage = storage;
            }
            Message::SetInOutStorage(storage) => {
                self.in_out_storage_changed |= self.settings.in_out_storage != storage;
                self.settings.in_out_storage = storage;
            }
            // Characters a tag cannot hold never reach the field, so what it shows is the tag.
            Message::SetCommentedTag(tag) => {
                self.settings.commented_tag = tag
                    .chars()
                    .filter(|c| {
                        *c == ' ' || frename_core::clean_commented_tag(&c.to_string()).is_some()
                    })
                    .collect();
            }
            Message::SetCommentedTagEnabled(enabled) => {
                self.settings.commented_tag_enabled = enabled
            }
            // Opened by the app, which owns the folder; the offer is taken.
            Message::OpenBatchAction(Operation::MoveComments(_)) => {
                self.comment_storage_changed = false
            }
            Message::OpenBatchAction(Operation::MoveInOut(_)) => {
                self.in_out_storage_changed = false
            }
            Message::OpenBatchAction(Operation::RespaceTags) => self.tag_spacing_changed = false,
            Message::OpenBatchAction(
                Operation::TagCommented
                | Operation::FixTags
                | Operation::ReloadFiles
                | Operation::GenerateSubtitles(_),
            ) => {}
            Message::SetSubtitleLanguage(code, on) => {
                let languages = &mut self.settings.subtitle_languages;
                languages.retain(|l| *l != code);
                if on {
                    languages.push(code);
                }
                // Kept in the order the window lists them.
                languages.sort_by_key(|l| {
                    frename_core::SUBTITLE_LANGUAGES
                        .iter()
                        .position(|(c, _)| c == l)
                });
            }
            Message::SetSubtitleCueLength(length) => self.settings.subtitle_cue_length = length,
            // Handled by `update_key`.
            Message::LoadSonioxKey
            | Message::SonioxKeyLoaded(_)
            | Message::SonioxKeyInput(_)
            | Message::ToggleShowSonioxKey
            | Message::SaveSonioxKey
            | Message::SonioxKeySaved(_)
            | Message::ReplaceSonioxKey
            | Message::RemoveSonioxKey
            | Message::SonioxKeyRemoved(_) => {}
        }
    }
}

/// Run `work` on a blocking thread (the credential store may be a D-Bus round trip) and turn
/// its answer into a message; `failed` makes one if the thread itself fails.
fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
    message: impl FnOnce(T) -> Message + Send + 'static,
    failed: impl FnOnce(String) -> Message + Send + 'static,
) -> Task<Message> {
    Task::future(async move {
        match tokio::task::spawn_blocking(work).await {
            Ok(result) => message(result),
            Err(e) => {
                log::error!("settings: credential store task failed: {e}");
                failed("The key could not be read or saved (see the log).".to_string())
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use frename_core::{CommentStorage, InOutStorage};

    #[test]
    fn apply_changes_only_the_named_setting() {
        let mut state = SettingsState {
            settings: AppSettings::default(),
            soniox_key: SonioxKeyState::default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
        };
        state.apply(Message::SetMonochromeTags(true));
        assert!(state.settings().monochrome_tags);
        assert!(state.settings().autoplay_video);

        state.apply(Message::SetAutoplayVideo(false));
        assert!(!state.settings().autoplay_video);
        assert!(state.settings().monochrome_tags);

        state.apply(Message::SetCommentStorage(CommentStorage::TextFile));
        assert_eq!(state.settings().comment_storage, CommentStorage::TextFile);
        assert!(state.settings().monochrome_tags);

        state.apply(Message::SetInOutStorage(InOutStorage::InVideo));
        assert_eq!(state.settings().in_out_storage, InOutStorage::InVideo);
        assert_eq!(state.settings().comment_storage, CommentStorage::TextFile);
    }

    #[test]
    fn a_storage_change_offers_moving_the_files_until_taken() {
        let mut state = SettingsState {
            settings: AppSettings::default(),
            soniox_key: SonioxKeyState::default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
        };
        state.apply(Message::SetCommentStorage(state.settings().comment_storage));
        assert!(
            !state.comment_storage_changed(),
            "picking the same storage is no change"
        );

        state.apply(Message::SetCommentStorage(CommentStorage::TextFile));
        assert!(state.comment_storage_changed());
        assert!(!state.in_out_storage_changed());
        state.apply(Message::OpenBatchAction(Operation::MoveComments(
            CommentStorage::TextFile,
        )));
        assert!(!state.comment_storage_changed());

        state.apply(Message::SetSpaceAfterTags(true));
        assert!(state.tag_spacing_changed() && state.settings().space_after_tags);
        state.apply(Message::OpenBatchAction(Operation::RespaceTags));
        assert!(!state.tag_spacing_changed());
    }

    #[test]
    fn the_commented_tag_field_drops_characters_a_tag_cannot_hold() {
        let mut state = SettingsState {
            settings: AppSettings::default(),
            soniox_key: SonioxKeyState::default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
        };
        state.apply(Message::SetCommentedTag("Has comment.v2:".to_string()));
        assert_eq!(state.settings().commented_tag, "Has commentv2");
    }

    #[test]
    fn languages_keep_the_window_order_and_none_means_detect() {
        let mut state = SettingsState {
            settings: AppSettings::default(),
            soniox_key: SonioxKeyState::default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
        };
        state.apply(Message::SetSubtitleLanguage("de".to_string(), true));
        state.apply(Message::SetSubtitleLanguage("en".to_string(), false));
        assert_eq!(state.settings().subtitle_languages, ["ru", "de"]);
        state.apply(Message::SetSubtitleLanguage("ru".to_string(), false));
        state.apply(Message::SetSubtitleLanguage("de".to_string(), false));
        assert!(state.subtitle_config().languages.is_empty(), "no hints");
    }

    #[test]
    fn the_key_status_follows_what_was_read_and_saved() {
        let mut state = SettingsState {
            settings: AppSettings::default(),
            soniox_key: SonioxKeyState::default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
        };
        assert_eq!(state.subtitle_config().key, KeyStatus::Unknown);
        let _ = state.update(Message::SonioxKeyLoaded(KeyInfo {
            key: None,
            store_available: true,
        }));
        assert_eq!(state.subtitle_config().key, KeyStatus::Missing);
        assert!(state.soniox_key().editing());

        let key = SonioxKey::new("k").expect("key");
        let _ = state.update(Message::SonioxKeySaved(Ok(key.clone())));
        assert_eq!(state.subtitle_config().key, KeyStatus::Present(key));
        assert!(
            !state.soniox_key().editing(),
            "Key saved, with Replace / Remove"
        );
        let _ = state.update(Message::ReplaceSonioxKey);
        assert!(state.soniox_key().editing());
    }
}
