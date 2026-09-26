//! State for the settings feature.

use frename_core::ai::key::KeyState;
use frename_core::{AppDatabase, AppSettings, AppStateStore};

use super::{KeyMessage, Message};
use crate::features::batch::Operation;

/// Current app settings, loaded from the app database and saved back on every change.
pub struct SettingsState {
    settings: AppSettings,
    /// The comment storage changed since the window last offered moving the files' comments.
    comment_storage_changed: bool,
    /// The in/out storage changed since the window last offered moving the files' points.
    in_out_storage_changed: bool,
    /// The tag spacing changed since the window last offered renaming the files to it.
    tag_spacing_changed: bool,
    /// The API key section; never persisted here (the key lives in the credential store).
    key: KeySection,
}

/// The API key field and what is known about the saved key.
#[derive(Debug, Clone, Default)]
pub struct KeySection {
    /// What is typed; cleared once saved.
    pub input: String,
    pub shown: bool,
    /// `None` until read: reading may unlock a keyring, so not at start-up.
    pub state: Option<KeyState>,
    /// Typing a new key over a saved one.
    pub replacing: bool,
    /// Whether "Remove the saved key?" is being asked.
    pub confirm_remove: bool,
    /// Why the last save or removal failed.
    pub error: Option<String>,
    /// The last read of the store asked for, and the newest one answered.
    requested: u64,
    answered: u64,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            settings: AppDatabase::new().get_app_settings().unwrap_or_default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
            key: KeySection::default(),
        }
    }
}

impl SettingsState {
    /// Apply a change and persist the result.
    pub fn update(&mut self, message: Message) {
        let persists = !matches!(message, Message::Key(_));
        self.apply(message);
        if persists {
            AppDatabase::new().set_app_settings(self.settings.clone());
        }
    }

    /// The API key section.
    pub fn key(&self) -> &KeySection {
        &self.key
    }

    /// Number a new read (or change) of the key's store; its answer comes back with it.
    pub fn begin_key_request(&mut self) -> u64 {
        self.key.requested += 1;
        self.key.requested
    }

    /// The key typed to be saved (trimmed); `None` when the field is empty.
    pub fn typed_key(&self) -> Option<String> {
        Some(self.key.input.trim().to_string()).filter(|k| !k.is_empty())
    }

    /// Current settings.
    pub fn settings(&self) -> &AppSettings {
        &self.settings
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

    fn apply_key(&mut self, message: KeyMessage) {
        match message {
            KeyMessage::Input(input) => {
                self.key.input = input;
                self.key.error = None;
            }
            KeyMessage::ToggleShow => self.key.shown = !self.key.shown,
            KeyMessage::Replace => self.key.replacing = true,
            KeyMessage::CancelReplace => {
                self.key.replacing = false;
                self.key.input.clear();
                self.key.error = None;
            }
            // Done by the app on a worker thread; the answer comes back as `KeyState`.
            KeyMessage::Save => self.key.error = None,
            KeyMessage::AskRemove => self.key.confirm_remove = true,
            KeyMessage::CancelRemove => self.key.confirm_remove = false,
            KeyMessage::Remove => {
                self.key.confirm_remove = false;
                self.key.error = None;
            }
            // An answer older than one already taken is stale.
            KeyMessage::State { request, .. } if request <= self.key.answered => {}
            KeyMessage::State {
                request,
                result: Ok(state),
            } => {
                self.key.answered = request;
                if state == KeyState::Saved {
                    self.key.input.clear();
                    self.key.shown = false;
                }
                self.key.replacing = false;
                self.key.state = Some(state);
            }
            // A failed save or removal leaves the store as it was: a read asked for before it
            // still tells the truth, so it does not become stale.
            KeyMessage::State {
                result: Err(error), ..
            } => self.key.error = Some(error),
        }
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
            Message::SetSummaryLanguage(language) => self.settings.summary_language = language,
            Message::Key(message) => self.apply_key(message),
            Message::OpenBatchAction(
                Operation::TagCommented
                | Operation::FixTags
                | Operation::ReloadFiles
                | Operation::DescribeAi(_),
            ) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frename_core::{CommentStorage, InOutStorage};

    #[test]
    fn apply_changes_only_the_named_setting() {
        let mut state = SettingsState {
            settings: AppSettings::default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
            key: KeySection::default(),
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
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
            key: KeySection::default(),
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
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
            key: KeySection::default(),
        };
        state.apply(Message::SetCommentedTag("Has comment.v2:".to_string()));
        assert_eq!(state.settings().commented_tag, "Has commentv2");
    }

    #[test]
    fn a_saved_key_clears_the_field_and_is_never_persisted() {
        let mut state = SettingsState {
            settings: AppSettings::default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
            key: KeySection::default(),
        };
        state.apply(Message::Key(KeyMessage::Input(" sk-ant-123 ".to_string())));
        assert_eq!(state.typed_key().as_deref(), Some("sk-ant-123"));
        let request = state.begin_key_request();
        state.apply(Message::Key(KeyMessage::State {
            request,
            result: Err("locked".to_string()),
        }));
        assert_eq!(state.key().error.as_deref(), Some("locked"));
        assert_eq!(state.key().input, " sk-ant-123 ", "kept to try again");
        let slow_read = state.begin_key_request();
        let save = state.begin_key_request();
        state.apply(Message::Key(KeyMessage::State {
            request: save,
            result: Ok(KeyState::Saved),
        }));
        assert!(state.key().input.is_empty());
        assert_eq!(state.key().state, Some(KeyState::Saved));
        state.apply(Message::Key(KeyMessage::State {
            request: slow_read,
            result: Ok(KeyState::Missing),
        }));
        assert_eq!(
            state.key().state,
            Some(KeyState::Saved),
            "a read that answers late does not undo the save"
        );

        let read = state.begin_key_request();
        let failed_save = state.begin_key_request();
        state.apply(Message::Key(KeyMessage::State {
            request: failed_save,
            result: Err("locked".to_string()),
        }));
        state.apply(Message::Key(KeyMessage::State {
            request: read,
            result: Ok(KeyState::Missing),
        }));
        assert_eq!(
            state.key().state,
            Some(KeyState::Missing),
            "a read answered after a failed save still counts"
        );
        assert!(!format!("{:?}", state.settings()).contains("sk-ant"));
    }
}
