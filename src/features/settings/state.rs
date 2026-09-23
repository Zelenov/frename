//! State for the settings feature.

use frename_core::{AppDatabase, AppSettings, AppStateStore};

use super::Message;

/// Current app settings, loaded from the app database and saved back on every change.
pub struct SettingsState {
    settings: AppSettings,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            settings: AppDatabase::new().get_app_settings().unwrap_or_default(),
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

    fn apply(&mut self, message: Message) {
        match message {
            Message::SetAutoplayVideo(autoplay) => self.settings.autoplay_video = autoplay,
            Message::SetMonochromeTags(monochrome) => self.settings.monochrome_tags = monochrome,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_changes_only_the_named_setting() {
        let mut state = SettingsState {
            settings: AppSettings::default(),
        };
        state.apply(Message::SetMonochromeTags(true));
        assert!(state.settings().monochrome_tags);
        assert!(state.settings().autoplay_video);

        state.apply(Message::SetAutoplayVideo(false));
        assert!(!state.settings().autoplay_video);
        assert!(state.settings().monochrome_tags);
    }
}
