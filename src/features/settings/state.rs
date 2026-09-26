//! State for the settings feature.

use std::path::PathBuf;

use frename_core::{old_settings, AppDatabase, AppSettings, AppStateStore};
use iced::Task;

use super::Message;
use crate::features::batch::Operation;
use crate::features::updates;

/// Where the import of an old frename's settings stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OldSettingsImport {
    None,
    /// Scheduled for the next start, from this folder.
    Scheduled(PathBuf),
    /// The picked folder has no old frename in it.
    NotFound(PathBuf),
    Failed(String),
}

/// Current app settings, loaded from the app database and saved back on every change.
pub struct SettingsState {
    settings: AppSettings,
    /// The comment storage changed since the window last offered moving the files' comments.
    comment_storage_changed: bool,
    /// The in/out storage changed since the window last offered moving the files' points.
    in_out_storage_changed: bool,
    /// The tag spacing changed since the window last offered renaming the files to it.
    tag_spacing_changed: bool,
    updates: updates::UpdatesState,
    import: OldSettingsImport,
}

impl Default for SettingsState {
    fn default() -> Self {
        let import = old_settings::scheduled_import(&frename_core::app_data_dir())
            .map_or(OldSettingsImport::None, OldSettingsImport::Scheduled);
        Self {
            settings: AppDatabase::new().get_app_settings().unwrap_or_default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            tag_spacing_changed: false,
            updates: updates::UpdatesState::default(),
            import,
        }
    }
}

impl SettingsState {
    /// Apply a change and persist the result.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Updates(msg) => self.updates.update(msg).map(Message::Updates),
            Message::ImportOldSettings => Task::perform(
                rfd::AsyncFileDialog::new()
                    .set_title("Folder of the old frename (with frename.exe and frename.db)")
                    .pick_folder(),
                |folder| Message::OldSettingsFolderPicked(folder.map(|f| f.path().to_path_buf())),
            ),
            Message::OldSettingsFolderPicked(folder) => {
                if let Some(folder) = folder {
                    self.import = schedule_import(folder);
                }
                Task::none()
            }
            message => {
                self.apply(message);
                AppDatabase::new().set_app_settings(self.settings.clone());
                Task::none()
            }
        }
    }

    /// The Updates section.
    pub fn updates(&self) -> &updates::UpdatesState {
        &self.updates
    }

    /// Where the import of an old frename's settings stands.
    pub fn old_settings_import(&self) -> &OldSettingsImport {
        &self.import
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
                Operation::MarkersComment(_)
                | Operation::TagCommented
                | Operation::FixTags
                | Operation::ReloadFiles,
            ) => {}
            // Handled by `update`.
            Message::Updates(_)
            | Message::ImportOldSettings
            | Message::OldSettingsFolderPicked(_) => {}
        }
    }
}

/// Schedule importing the settings in `folder` at the next start: the database is open now.
fn schedule_import(folder: PathBuf) -> OldSettingsImport {
    if !old_settings::is_old_frename_folder(&folder) {
        return OldSettingsImport::NotFound(folder);
    }
    match old_settings::schedule_import(&frename_core::app_data_dir(), &folder) {
        Ok(()) => OldSettingsImport::Scheduled(folder),
        Err(e) => OldSettingsImport::Failed(e.to_string()),
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
            updates: updates::UpdatesState::default(),
            import: OldSettingsImport::None,
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
            updates: updates::UpdatesState::default(),
            import: OldSettingsImport::None,
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
            updates: updates::UpdatesState::default(),
            import: OldSettingsImport::None,
        };
        state.apply(Message::SetCommentedTag("Has comment.v2:".to_string()));
        assert_eq!(state.settings().commented_tag, "Has commentv2");
    }
}
