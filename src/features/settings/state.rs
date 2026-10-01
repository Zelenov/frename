//! State for the settings feature.

use std::path::PathBuf;

use frename_core::ai::key::{ApiKey, KeyState};
use frename_core::ai::ledger::SpendSummary;
use frename_core::{old_settings, AppDatabase, AppSettings, AppStateStore};
use iced::Task;

use super::{KeyMessage, Message, Page, TopUpMessage, SETTINGS_SCROLLABLE_ID};
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
    /// The page shown; kept while the app runs, so the window reopens where it was.
    page: Page,
    /// The comment storage changed since the window last offered moving the files' comments.
    comment_storage_changed: bool,
    /// The in/out storage changed since the window last offered moving the files' points.
    in_out_storage_changed: bool,
    /// The marker storage changed since the window last offered moving the files' markers.
    marker_storage_changed: bool,
    /// The tag spacing changed since the window last offered renaming the files to it.
    tag_spacing_changed: bool,
    updates: updates::UpdatesState,
    import: OldSettingsImport,
    /// The API key sections; never persisted here (the keys live in the credential store).
    keys: Keys,
    /// What each paid service cost and the top-up form (issue #121).
    spends: Spends,
    /// The languages Soniox offers as hints; asked for each time the window opens with a key.
    subtitle_languages: LanguageList,
}

/// Where the list of languages Soniox recognises stands.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LanguageList {
    #[default]
    NotAsked,
    Loading,
    /// (code, English name), sorted by name.
    Listed(Vec<(String, String)>),
    Failed(String),
}

/// One key section per service.
#[derive(Debug, Clone, Default)]
pub struct Keys {
    anthropic: KeySection,
    soniox: KeySection,
}

/// One spending section per service.
#[derive(Debug, Clone, Default)]
pub struct Spends {
    anthropic: SpendSection,
    soniox: SpendSection,
}

/// What a service cost (read from the spend ledger) and the top-up being typed.
#[derive(Debug, Clone, Default)]
pub struct SpendSection {
    pub summary: SpendSummary,
    /// The amount typed, in dollars.
    pub amount: String,
    /// The day typed (`YYYY-MM-DD`); empty means today.
    pub date: String,
    /// The last "Record" was refused: the amount or the day is not usable.
    pub refused: bool,
}

impl SpendSection {
    /// The amount typed as a number of dollars (a comma may stand for the point).
    pub fn parsed_amount(&self) -> Option<f64> {
        let amount = self.amount.trim().replace(',', ".");
        amount
            .parse::<f64>()
            .ok()
            .filter(|a| a.is_finite() && *a >= 0.0)
    }
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
        let import = old_settings::scheduled_import(&frename_core::app_data_dir())
            .map_or(OldSettingsImport::None, OldSettingsImport::Scheduled);
        Self {
            settings: AppDatabase::new().get_app_settings().unwrap_or_default(),
            page: Page::default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            marker_storage_changed: false,
            tag_spacing_changed: false,
            updates: updates::UpdatesState::default(),
            import,
            keys: Keys::default(),
            spends: Spends::default(),
            subtitle_languages: LanguageList::default(),
        }
    }
}

impl SettingsState {
    /// Apply a change and persist the result.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowPage(page) => self.show_page(page),
            Message::NextPage => self.show_page(self.page.step(1)),
            Message::PreviousPage => self.show_page(self.page.step(-1)),
            // Handled by the app.
            Message::Close | Message::Escape => Task::none(),
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
            // The key lives in the credential store, never in the saved settings.
            Message::Key(which, message) => {
                self.apply_key(which, message);
                Task::none()
            }
            Message::TopUp(which, message) => {
                self.apply_top_up(which, message);
                Task::none()
            }
            Message::SubtitleLanguagesListed(result) => {
                self.subtitle_languages = match result {
                    Ok(list) => LanguageList::Listed(list),
                    Err(why) => LanguageList::Failed(why),
                };
                Task::none()
            }
            message => {
                self.apply(message);
                AppDatabase::new().set_app_settings(self.settings.clone());
                Task::none()
            }
        }
    }

    /// The page shown.
    pub fn page(&self) -> Page {
        self.page
    }

    /// Show `page`, from its top.
    pub fn show_page(&mut self, page: Page) -> Task<Message> {
        self.page = page;
        self.refresh_spend();
        iced::widget::operation::snap_to(
            iced::widget::Id::new(SETTINGS_SCROLLABLE_ID),
            iced::widget::scrollable::RelativeOffset::START,
        )
    }

    /// Esc: cancel a pending key removal or replacement on the page shown, as its Keep or Cancel
    /// button would. `false` when there was none, and Esc closes the window instead.
    pub fn escape(&mut self) -> bool {
        let which = match self.page {
            Page::Ai => ApiKey::Anthropic,
            Page::Subtitles => ApiKey::Soniox,
            _ => return false,
        };
        let key = self.key(which);
        let cancel = if key.confirm_remove {
            KeyMessage::CancelRemove
        } else if key.replacing {
            KeyMessage::CancelReplace
        } else {
            return false;
        };
        self.apply_key(which, cancel);
        true
    }

    /// The Updates section.
    pub fn updates(&self) -> &updates::UpdatesState {
        &self.updates
    }

    /// Where the import of an old frename's settings stands.
    pub fn old_settings_import(&self) -> &OldSettingsImport {
        &self.import
    }

    /// The spending section of `which`.
    pub fn spend(&self, which: ApiKey) -> &SpendSection {
        match which {
            ApiKey::Anthropic => &self.spends.anthropic,
            ApiKey::Soniox => &self.spends.soniox,
        }
    }

    fn spend_mut(&mut self, which: ApiKey) -> &mut SpendSection {
        match which {
            ApiKey::Anthropic => &mut self.spends.anthropic,
            ApiKey::Soniox => &mut self.spends.soniox,
        }
    }

    /// Read what each service cost from the ledger again (the window opens, a page is shown, a
    /// top-up was recorded).
    pub fn refresh_spend(&mut self) {
        let db = AppDatabase::new();
        for which in [ApiKey::Anthropic, ApiKey::Soniox] {
            self.spend_mut(which).summary = db.ai_spend_summary(which);
        }
    }

    fn apply_top_up(&mut self, which: ApiKey, message: TopUpMessage) {
        let section = self.spend_mut(which);
        match message {
            TopUpMessage::Amount(amount) => {
                section.amount = amount;
                section.refused = false;
            }
            TopUpMessage::Date(date) => {
                section.date = date;
                section.refused = false;
            }
            TopUpMessage::Record => {
                let Some(amount) = section.parsed_amount() else {
                    section.refused = true;
                    return;
                };
                let date = section.date.trim();
                let date = (!date.is_empty()).then_some(date);
                if AppDatabase::new().set_ai_top_up(which, amount, date) {
                    *section = SpendSection::default();
                    self.refresh_spend();
                } else {
                    section.refused = true;
                }
            }
        }
    }

    /// The section of the `which` API key.
    pub fn key(&self, which: ApiKey) -> &KeySection {
        match which {
            ApiKey::Anthropic => &self.keys.anthropic,
            ApiKey::Soniox => &self.keys.soniox,
        }
    }

    fn key_mut(&mut self, which: ApiKey) -> &mut KeySection {
        match which {
            ApiKey::Anthropic => &mut self.keys.anthropic,
            ApiKey::Soniox => &mut self.keys.soniox,
        }
    }

    /// The languages Soniox offers as hints.
    pub fn subtitle_languages(&self) -> &LanguageList {
        &self.subtitle_languages
    }

    /// Whether to ask Soniox for its languages now: not while asking, nor once listed.
    pub fn begin_subtitle_languages(&mut self) -> bool {
        if matches!(
            self.subtitle_languages,
            LanguageList::Loading | LanguageList::Listed(_)
        ) {
            return false;
        }
        self.subtitle_languages = LanguageList::Loading;
        true
    }

    /// Number a new read (or change) of the key's store; its answer comes back with it.
    pub fn begin_key_request(&mut self, which: ApiKey) -> u64 {
        let key = self.key_mut(which);
        key.requested += 1;
        key.requested
    }

    /// The key typed to be saved (trimmed); `None` when the field is empty.
    pub fn typed_key(&self, which: ApiKey) -> Option<String> {
        Some(self.key(which).input.trim().to_string()).filter(|k| !k.is_empty())
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

    /// Whether to offer moving the files' markers to the storage just chosen.
    pub fn marker_storage_changed(&self) -> bool {
        self.marker_storage_changed
    }

    /// Whether to offer renaming the files to the tag spacing just chosen.
    pub fn tag_spacing_changed(&self) -> bool {
        self.tag_spacing_changed
    }

    fn apply_key(&mut self, which: ApiKey, message: KeyMessage) {
        let key = self.key_mut(which);
        match message {
            KeyMessage::Input(input) => {
                key.input = input;
                key.error = None;
            }
            KeyMessage::ToggleShow => key.shown = !key.shown,
            KeyMessage::Replace => key.replacing = true,
            KeyMessage::CancelReplace => {
                key.replacing = false;
                key.input.clear();
                key.error = None;
            }
            // Done by the app on a worker thread; the answer comes back as `KeyState`.
            KeyMessage::Save => key.error = None,
            KeyMessage::AskRemove => key.confirm_remove = true,
            KeyMessage::CancelRemove => key.confirm_remove = false,
            KeyMessage::Remove => {
                key.confirm_remove = false;
                key.error = None;
            }
            // An answer older than one already taken is stale.
            KeyMessage::State { request, .. } if request <= key.answered => {}
            KeyMessage::State {
                request,
                result: Ok(state),
            } => {
                key.answered = request;
                if state == KeyState::Saved {
                    key.input.clear();
                    key.shown = false;
                } else {
                    // No saved key: a question about removing one is gone with it.
                    key.confirm_remove = false;
                }
                key.replacing = false;
                key.state = Some(state);
            }
            // A failed save or removal leaves the store as it was: a read asked for before it
            // still tells the truth, so it does not become stale.
            KeyMessage::State {
                result: Err(error), ..
            } => key.error = Some(error),
        }
    }

    fn apply(&mut self, message: Message) {
        match message {
            Message::SetUiLanguage(language) => self.settings.ui_language = language,
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
            Message::SetMarkerStorage(storage) => {
                self.marker_storage_changed |= self.settings.marker_storage != storage;
                self.settings.marker_storage = storage;
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
            Message::OpenBatchAction(Operation::MarkersComment(_)) => {
                self.marker_storage_changed = false
            }
            Message::OpenBatchAction(Operation::RespaceTags) => self.tag_spacing_changed = false,
            Message::SetSummaryLanguage(language) => self.settings.summary_language = language,
            Message::SetAiModel(model) => self.settings.ai_model = model.id.to_string(),
            Message::SetSubtitleLanguage(code, on) => {
                let languages = &mut self.settings.subtitle_languages;
                languages.retain(|l| *l != code);
                if on {
                    languages.push(code);
                }
                languages.sort();
            }
            Message::SetSubtitleCueLength(length) => self.settings.subtitle_cue_length = length,
            Message::Key(which, message) => self.apply_key(which, message),
            Message::OpenBatchAction(
                Operation::TagCommented
                | Operation::FixTags
                | Operation::ReloadFiles
                | Operation::Rotate(_)
                | Operation::DescribeAi(_)
                | Operation::GenerateSubtitles(_),
            ) => {}
            // Handled by `update`, or by the app.
            Message::ShowPage(_)
            | Message::NextPage
            | Message::PreviousPage
            | Message::Close
            | Message::Escape
            | Message::Updates(_)
            | Message::ImportOldSettings
            | Message::OldSettingsFolderPicked(_)
            | Message::TopUp(..)
            | Message::SubtitleLanguagesListed(_) => {}
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

    /// Settings as a fresh install has them, not read from any database.
    fn test_state() -> SettingsState {
        SettingsState {
            settings: AppSettings::default(),
            page: Page::default(),
            comment_storage_changed: false,
            in_out_storage_changed: false,
            marker_storage_changed: false,
            tag_spacing_changed: false,
            updates: updates::UpdatesState::default(),
            import: OldSettingsImport::None,
            keys: Keys::default(),
            spends: Spends::default(),
            subtitle_languages: LanguageList::default(),
        }
    }

    #[test]
    fn the_top_up_amount_is_a_number_of_dollars_with_a_point_or_a_comma() {
        let section = |amount: &str| SpendSection {
            amount: amount.to_string(),
            ..SpendSection::default()
        };
        assert_eq!(section(" 20 ").parsed_amount(), Some(20.0));
        assert_eq!(section("12,50").parsed_amount(), Some(12.5));
        assert_eq!(section("").parsed_amount(), None);
        assert_eq!(section("twenty").parsed_amount(), None);
        assert_eq!(section("-5").parsed_amount(), None);
        assert_eq!(section("inf").parsed_amount(), None);
    }

    #[test]
    fn a_top_up_typed_wrong_is_refused_until_the_form_is_touched() {
        let mut state = test_state();
        state.apply_top_up(ApiKey::Soniox, TopUpMessage::Amount("oops".into()));
        state.apply_top_up(ApiKey::Soniox, TopUpMessage::Record);
        assert!(state.spend(ApiKey::Soniox).refused);
        assert!(!state.spend(ApiKey::Anthropic).refused, "per service");
        state.apply_top_up(ApiKey::Soniox, TopUpMessage::Amount("5".into()));
        assert!(!state.spend(ApiKey::Soniox).refused);
    }

    #[test]
    fn apply_changes_only_the_named_setting() {
        let mut state = test_state();
        state.apply(Message::SetMonochromeTags(true));
        assert!(state.settings().monochrome_tags);
        assert!(state.settings().autoplay_video);

        state.apply(Message::SetAutoplayVideo(false));
        assert!(!state.settings().autoplay_video);
        assert!(state.settings().monochrome_tags);

        state.apply(Message::SetCommentStorage(CommentStorage::TextFile));
        assert_eq!(state.settings().comment_storage, CommentStorage::TextFile);
        assert!(state.settings().monochrome_tags);

        state.apply(Message::SetInOutStorage(InOutStorage::Comment));
        assert_eq!(state.settings().in_out_storage, InOutStorage::Comment);
        assert_eq!(state.settings().comment_storage, CommentStorage::TextFile);

        state.apply(Message::SetUiLanguage("ru".to_string()));
        assert_eq!(state.settings().ui_language, "ru");
        assert_eq!(state.settings().in_out_storage, InOutStorage::Comment);
    }

    #[test]
    fn a_storage_change_offers_moving_the_files_until_taken() {
        let mut state = test_state();
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
        let mut state = test_state();
        state.apply(Message::SetCommentedTag("Has comment.v2:".to_string()));
        assert_eq!(state.settings().commented_tag, "Has commentv2");
    }

    #[test]
    fn a_saved_key_clears_the_field_and_is_never_persisted() {
        let mut state = test_state();
        state.apply(Message::Key(
            ApiKey::Anthropic,
            KeyMessage::Input(" sk-ant-123 ".to_string()),
        ));
        assert_eq!(
            state.typed_key(ApiKey::Anthropic).as_deref(),
            Some("sk-ant-123")
        );
        let request = state.begin_key_request(ApiKey::Anthropic);
        state.apply(Message::Key(
            ApiKey::Anthropic,
            KeyMessage::State {
                request,
                result: Err("locked".to_string()),
            },
        ));
        assert_eq!(
            state.key(ApiKey::Anthropic).error.as_deref(),
            Some("locked")
        );
        assert_eq!(
            state.key(ApiKey::Anthropic).input,
            " sk-ant-123 ",
            "kept to try again"
        );
        let slow_read = state.begin_key_request(ApiKey::Anthropic);
        let save = state.begin_key_request(ApiKey::Anthropic);
        state.apply(Message::Key(
            ApiKey::Anthropic,
            KeyMessage::State {
                request: save,
                result: Ok(KeyState::Saved),
            },
        ));
        assert!(state.key(ApiKey::Anthropic).input.is_empty());
        assert_eq!(state.key(ApiKey::Anthropic).state, Some(KeyState::Saved));
        state.apply(Message::Key(
            ApiKey::Anthropic,
            KeyMessage::State {
                request: slow_read,
                result: Ok(KeyState::Missing),
            },
        ));
        assert_eq!(
            state.key(ApiKey::Anthropic).state,
            Some(KeyState::Saved),
            "a read that answers late does not undo the save"
        );

        let read = state.begin_key_request(ApiKey::Anthropic);
        let failed_save = state.begin_key_request(ApiKey::Anthropic);
        state.apply(Message::Key(
            ApiKey::Anthropic,
            KeyMessage::State {
                request: failed_save,
                result: Err("locked".to_string()),
            },
        ));
        state.apply(Message::Key(
            ApiKey::Anthropic,
            KeyMessage::State {
                request: read,
                result: Ok(KeyState::Missing),
            },
        ));
        assert_eq!(
            state.key(ApiKey::Anthropic).state,
            Some(KeyState::Missing),
            "a read answered after a failed save still counts"
        );
        assert!(!format!("{:?}", state.settings()).contains("sk-ant"));
    }

    #[test]
    fn esc_cancels_a_pending_removal_on_the_page_shown_before_it_closes_the_window() {
        let mut state = test_state();
        let read = state.begin_key_request(ApiKey::Soniox);
        state.apply(Message::Key(
            ApiKey::Soniox,
            KeyMessage::State {
                request: read,
                result: Ok(KeyState::Saved),
            },
        ));
        state.apply(Message::Key(ApiKey::Soniox, KeyMessage::AskRemove));
        assert!(
            !state.escape(),
            "a removal asked on a page not shown: Esc closes the window"
        );
        state.page = Page::Subtitles;
        assert!(state.escape(), "the first Esc keeps the key");
        assert!(!state.key(ApiKey::Soniox).confirm_remove);
        assert!(!state.escape(), "nothing pending: Esc closes the window");

        state.page = Page::Ai;
        state.apply(Message::Key(ApiKey::Anthropic, KeyMessage::Replace));
        state.apply(Message::Key(
            ApiKey::Anthropic,
            KeyMessage::Input("half a key".to_string()),
        ));
        let save = state.begin_key_request(ApiKey::Anthropic);
        state.apply(Message::Key(
            ApiKey::Anthropic,
            KeyMessage::State {
                request: save,
                result: Err("locked".to_string()),
            },
        ));
        assert!(
            state.escape(),
            "Esc gives up replacing the key, as Cancel does"
        );
        let key = state.key(ApiKey::Anthropic);
        assert!(!key.replacing && key.input.is_empty() && key.error.is_none());
    }

    #[test]
    fn a_removal_question_left_for_a_key_that_is_gone_does_not_take_the_esc() {
        let mut state = test_state();
        state.page = Page::Subtitles;
        state.apply(Message::Key(ApiKey::Soniox, KeyMessage::AskRemove));
        let read = state.begin_key_request(ApiKey::Soniox);
        state.apply(Message::Key(
            ApiKey::Soniox,
            KeyMessage::State {
                request: read,
                result: Ok(KeyState::Missing),
            },
        ));
        assert!(
            !state.key(ApiKey::Soniox).confirm_remove,
            "the question went with the key"
        );
        assert!(!state.escape(), "nothing on screen to cancel: Esc closes");
    }

    #[test]
    fn the_two_keys_have_sections_of_their_own() {
        let mut state = test_state();
        state.apply(Message::Key(
            ApiKey::Soniox,
            KeyMessage::Input("soniox-key".to_string()),
        ));
        assert_eq!(
            state.typed_key(ApiKey::Soniox).as_deref(),
            Some("soniox-key")
        );
        assert_eq!(state.typed_key(ApiKey::Anthropic), None);
        let request = state.begin_key_request(ApiKey::Soniox);
        state.apply(Message::Key(
            ApiKey::Soniox,
            KeyMessage::State {
                request,
                result: Ok(KeyState::Saved),
            },
        ));
        assert_eq!(state.key(ApiKey::Soniox).state, Some(KeyState::Saved));
        assert_eq!(state.key(ApiKey::Anthropic).state, None);
    }

    #[test]
    fn subtitle_languages_are_sorted_and_none_means_detect() {
        let mut state = test_state();
        state.apply(Message::SetSubtitleLanguage("de".to_string(), true));
        state.apply(Message::SetSubtitleLanguage("en".to_string(), false));
        assert_eq!(state.settings().subtitle_languages, ["de", "ru"]);
        state.apply(Message::SetSubtitleLanguage("ru".to_string(), false));
        state.apply(Message::SetSubtitleLanguage("de".to_string(), false));
        assert!(state.settings().subtitle_languages.is_empty());
    }
}
