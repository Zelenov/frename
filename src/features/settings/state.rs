//! State for the settings feature.

use frename_core::{
    AppDatabase, AppSettings, AppStateStore, ConversionPlan, ConversionReport, MetadataStorage,
};

use super::Message;

/// Where the open folder stands against the comment and in/out storage settings.
#[derive(Debug, Clone, Default)]
pub enum FolderConversion {
    /// No folder is open.
    #[default]
    NoFolder,
    /// Looking at the folder's files.
    Checking,
    /// What converting the folder would move (empty when it already matches).
    Planned(ConversionPlan),
    /// Conversion is running.
    Converting,
    /// Conversion finished.
    Done(ConversionReport),
}

/// Current app settings, loaded from the app database and saved back on every change.
pub struct SettingsState {
    settings: AppSettings,
    conversion: FolderConversion,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            settings: AppDatabase::new().get_app_settings().unwrap_or_default(),
            conversion: FolderConversion::default(),
        }
    }
}

impl SettingsState {
    /// Apply a change and persist the result.
    pub fn update(&mut self, message: Message) {
        self.apply(message);
        AppDatabase::new().set_app_settings(self.settings);
    }

    /// Current settings.
    pub fn settings(&self) -> AppSettings {
        self.settings
    }

    /// The comment and in/out storage, as a conversion targets them.
    pub fn metadata_storage(&self) -> MetadataStorage {
        MetadataStorage {
            comment: self.settings.comment_storage,
            in_out: self.settings.in_out_storage,
        }
    }

    /// Where the open folder stands; shown in the settings window.
    pub fn conversion(&self) -> &FolderConversion {
        &self.conversion
    }

    pub fn set_conversion(&mut self, conversion: FolderConversion) {
        self.conversion = conversion;
    }

    /// Take a finished check. A check started before the storage changed again is stale:
    /// the newer one is still running and replaces it.
    pub fn set_plan(&mut self, plan: ConversionPlan) {
        if plan.storage == self.metadata_storage() {
            self.conversion = FolderConversion::Planned(plan);
        }
    }

    fn apply(&mut self, message: Message) {
        match message {
            Message::SetAutoplayVideo(autoplay) => self.settings.autoplay_video = autoplay,
            Message::SetMonochromeTags(monochrome) => self.settings.monochrome_tags = monochrome,
            Message::SetCommentStorage(storage) => self.settings.comment_storage = storage,
            Message::SetInOutStorage(storage) => self.settings.in_out_storage = storage,
            // Run by the app, which owns the folder.
            Message::ConvertFolder => {}
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
            conversion: FolderConversion::default(),
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

        state.apply(Message::SetInOutStorage(InOutStorage::Xmp));
        assert_eq!(state.settings().in_out_storage, InOutStorage::Xmp);
        assert_eq!(state.settings().comment_storage, CommentStorage::TextFile);
    }

    #[test]
    fn a_check_for_an_older_storage_choice_is_dropped() {
        let mut state = SettingsState {
            settings: AppSettings::default(),
            conversion: FolderConversion::Checking,
        };
        let stale = ConversionPlan {
            storage: MetadataStorage {
                comment: CommentStorage::TextFile,
                in_out: InOutStorage::Xmp,
            },
            ..ConversionPlan::default()
        };
        state.set_plan(stale);
        assert!(matches!(state.conversion(), FolderConversion::Checking));

        state.set_plan(ConversionPlan {
            storage: state.metadata_storage(),
            ..ConversionPlan::default()
        });
        assert!(matches!(state.conversion(), FolderConversion::Planned(_)));
    }
}
