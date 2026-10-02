//! State for the settings feature.

use std::path::PathBuf;

use frename_core::ai::key::{ApiKey, KeyState};
use frename_core::{old_settings, AppDatabase, AppSettings, AppStateStore, MarkerStorage};
use iced::Task;

use super::focus::{self, Control, KeyControl, KeyRow, Press};
use super::{KeyMessage, Message, Page, SETTINGS_SCROLLABLE_ID};
use crate::features::batch::{MarkersDirection, Operation};
use crate::features::updates;
use crate::widgets::focus_ring;

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
    /// The languages Soniox offers as hints; asked for each time the window opens with a key.
    subtitle_languages: LanguageList,
    /// The control keyboard focus is on (#167); `None` until Tab is pressed, and after a click.
    focus: Option<Control>,
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
            subtitle_languages: LanguageList::default(),
            focus: None,
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
            Message::Close
            | Message::Escape
            | Message::FocusNext
            | Message::FocusPrevious
            | Message::Press(_) => Task::none(),
            // A click takes the ring away; a click into a field puts the focus there.
            Message::ClearFocus => {
                self.focus = None;
                focus_ring::focused_among(focus::FIELD_IDS).map(Message::FieldFocused)
            }
            Message::FieldFocused(field) => {
                if let Some(control) = field.and_then(focus::control_of_field) {
                    self.focus = Some(control);
                }
                Task::none()
            }
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
                // Typing into a field moves the focus there: Tab goes on from it.
                if matches!(message, KeyMessage::Input(_)) {
                    self.focus = Some(Control::Key(which, KeyControl::Field));
                }
                let answer = match &message {
                    KeyMessage::State { result, .. } => Some(result.is_ok()),
                    _ => None,
                };
                self.apply_key(which, message);
                match answer {
                    Some(ok) => self.after_key_answer(which, ok),
                    None => Task::none(),
                }
            }
            Message::SubtitleLanguagesListed(result) => {
                self.subtitle_languages = match result {
                    Ok(list) => LanguageList::Listed(list),
                    Err(why) => LanguageList::Failed(why),
                };
                Task::none()
            }
            message => {
                if matches!(message, Message::SetCommentedTag(_)) {
                    self.focus = Some(Control::CommentedTagName);
                }
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
        self.focus = None;
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
        let (cancel, focus_to) = if key.confirm_remove {
            (KeyMessage::CancelRemove, KeyControl::AskRemove)
        } else if key.replacing {
            (KeyMessage::CancelReplace, KeyControl::Replace)
        } else {
            return false;
        };
        self.apply_key(which, cancel);
        // As Keep and Cancel move it (§11): only while the keyboard focus is in this row.
        if matches!(self.focus, Some(Control::Key(row, _)) if row == which) {
            self.focus = Some(Control::Key(which, focus_to));
        }
        true
    }

    /// The control keyboard focus is on.
    pub fn focus(&self) -> Option<&Control> {
        self.focus.as_ref()
    }

    /// Tab (`steps` 1) or Shift+Tab (-1): move the focus to the next or previous control of the
    /// page, focus or leave its text fields, and scroll the control into view.
    pub fn move_focus(&mut self, steps: isize, batch_running: bool) -> Task<Message> {
        let controls = focus::controls(self, batch_running);
        self.focus = focus::step(&controls, self.focus.as_ref(), steps);
        self.focus_effects()
    }

    /// Space or Enter: press the focused control, as a click would. The control's own message
    /// comes back to the app, which handles it like a click's.
    pub fn press(&mut self, press: Press, batch_running: bool) -> Task<Message> {
        let Some(control) = self.focus.clone() else {
            return Task::none();
        };
        // A control disabled since it took the focus does nothing, as a click would not.
        if !focus::controls(self, batch_running).contains(&control) {
            return Task::none();
        }
        let Some(message) = focus::press(self, &control, press) else {
            return Task::none();
        };
        let controls = focus::controls(self, batch_running);
        let next = focus::after_press(&control).or_else(|| {
            // An offer goes away with its press: the control after it takes the focus.
            focus::is_offer(&control)
                .then(|| controls.iter().position(|c| *c == control))
                .flatten()
                .and_then(|at| controls.get(at + 1).cloned())
        });
        // A key row's own buttons only change this state: apply them now, so the control the
        // focus moves to (the new key field after Replace…) is drawn before the focus effects run
        // on it. Save and Remove go to the app, which talks to the credential store.
        let message = match message {
            Message::Key(which, key) if key.is_local() => {
                self.apply_key(which, key);
                None
            }
            message => Some(message),
        };
        let effects = match next {
            Some(next) => {
                self.focus = Some(next);
                self.focus_effects()
            }
            None => Task::none(),
        };
        Task::batch([message.map_or_else(Task::none, Task::done), effects])
    }

    /// Take the keyboard focus away: the window opens without it.
    pub fn clear_focus(&mut self) {
        self.focus = None;
    }

    /// The credential store answered for the `which` key row (after Save, Remove or a read), and
    /// the row may show other controls now. While the keyboard focus is in that row it stays on a
    /// control the row still shows (§11): after a save the focus is on **Replace…**, after a
    /// removal in the new key field (focused now that it is there); a failed save leaves it on
    /// **Save key**, a failed removal on **Remove…**.
    fn after_key_answer(&mut self, which: ApiKey, ok: bool) -> Task<Message> {
        let Some(focused @ Control::Key(row, _)) = self.focus.clone() else {
            return Task::none();
        };
        if row != which {
            return Task::none();
        }
        let shown = focus::key_controls(self, which);
        if !shown.contains(&focused) {
            let next = match focus::key_row(self.key(which)) {
                KeyRow::Unavailable => None,
                KeyRow::ConfirmRemove => Some(KeyControl::Keep),
                KeyRow::Saved if ok => Some(KeyControl::Replace),
                KeyRow::Saved => Some(KeyControl::AskRemove),
                KeyRow::Typing { can_save: true, .. } if !ok => Some(KeyControl::Save),
                KeyRow::Typing { .. } => Some(KeyControl::Field),
            };
            self.focus = next.map(|control| Control::Key(which, control));
        }
        self.focus_effects()
    }

    /// Focus the text field the focus is on, or none of the window's fields, and bring the
    /// focused control into view.
    fn focus_effects(&self) -> Task<Message> {
        let field = self.focus.as_ref().and_then(Control::field_id);
        Task::batch([
            focus_ring::focus_among(field, focus::FIELD_IDS),
            focus_ring::scroll_into_view(SETTINGS_SCROLLABLE_ID, field),
        ])
    }

    /// The batch action offered after the marker storage changed.
    pub fn markers_offer(&self) -> Operation {
        Operation::MarkersComment(match self.settings.marker_storage {
            MarkerStorage::InVideo => MarkersDirection::CommentToMarkers,
            MarkerStorage::Comment => MarkersDirection::MarkersToComment,
        })
    }

    /// The languages the Subtitles page offers, (code, name): Soniox's own list, or until it
    /// comes the codes checked already.
    pub fn offered_subtitle_languages(&self) -> Vec<(String, String)> {
        match &self.subtitle_languages {
            LanguageList::Listed(all) => all.clone(),
            _ => self
                .settings
                .subtitle_languages
                .iter()
                .map(|c| (c.clone(), c.clone()))
                .collect(),
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
            | Message::FocusNext
            | Message::FocusPrevious
            | Message::Press(_)
            | Message::ClearFocus
            | Message::FieldFocused(_)
            | Message::Updates(_)
            | Message::ImportOldSettings
            | Message::OldSettingsFolderPicked(_)
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
impl SettingsState {
    /// Settings as a fresh install has them, not read from any database.
    pub fn for_tests() -> Self {
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
            subtitle_languages: LanguageList::default(),
            focus: None,
        }
    }

    /// Show `page`, as `show_page` does without its scroll.
    pub fn show_page_for_tests(&mut self, page: Page) {
        self.page = page;
        self.focus = None;
    }

    /// A change, without saving the settings to the database.
    pub fn apply_for_tests(&mut self, message: Message) {
        self.apply(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frename_core::{CommentStorage, InOutStorage};

    fn test_state() -> SettingsState {
        SettingsState::for_tests()
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

    fn saved_key(state: &mut SettingsState, which: ApiKey) {
        let request = state.begin_key_request(which);
        state.apply(Message::Key(
            which,
            KeyMessage::State {
                request,
                result: Ok(KeyState::Saved),
            },
        ));
    }

    #[test]
    fn replace_shows_the_field_before_the_focus_moves_into_it() {
        let mut state = test_state();
        state.page = Page::Ai;
        saved_key(&mut state, ApiKey::Anthropic);
        state.focus = Some(Control::Key(ApiKey::Anthropic, KeyControl::Replace));
        let _ = state.press(Press::Space, false);
        let field = Control::Key(ApiKey::Anthropic, KeyControl::Field);
        assert!(state.key(ApiKey::Anthropic).replacing, "applied at once");
        assert_eq!(state.focus(), Some(&field));
        assert!(
            focus::controls(&state, false).contains(&field),
            "the field is on the page when the focus effects run"
        );
    }

    fn answer(state: &mut SettingsState, which: ApiKey, result: Result<KeyState, String>) -> usize {
        let request = state.begin_key_request(which);
        state
            .update(Message::Key(which, KeyMessage::State { request, result }))
            .units()
    }

    #[test]
    fn removing_a_key_focuses_the_new_field_once_the_store_answers() {
        let mut state = test_state();
        state.page = Page::Subtitles;
        saved_key(&mut state, ApiKey::Soniox);
        state.apply(Message::Key(ApiKey::Soniox, KeyMessage::AskRemove));
        state.focus = Some(Control::Key(ApiKey::Soniox, KeyControl::Remove));
        let _ = state.press(Press::Enter, false);
        let field = Control::Key(ApiKey::Soniox, KeyControl::Field);
        assert_eq!(state.focus(), Some(&field));
        // The app routes Remove back here, then asks the store.
        let _ = state.update(Message::Key(ApiKey::Soniox, KeyMessage::Remove));
        let effects = answer(&mut state, ApiKey::Soniox, Ok(KeyState::Missing));
        assert_eq!(state.focus(), Some(&field));
        assert!(effects > 0, "the field is focused now that it is shown");
    }

    #[test]
    fn a_failed_removal_puts_the_focus_back_on_remove() {
        let mut state = test_state();
        state.page = Page::Subtitles;
        saved_key(&mut state, ApiKey::Soniox);
        state.apply(Message::Key(ApiKey::Soniox, KeyMessage::AskRemove));
        state.focus = Some(Control::Key(ApiKey::Soniox, KeyControl::Remove));
        let _ = state.press(Press::Space, false);
        let _ = state.update(Message::Key(ApiKey::Soniox, KeyMessage::Remove));
        answer(&mut state, ApiKey::Soniox, Err("locked".to_string()));
        assert_eq!(
            state.focus(),
            Some(&Control::Key(ApiKey::Soniox, KeyControl::AskRemove))
        );
    }

    #[test]
    fn saving_a_key_moves_the_focus_to_replace_and_a_failed_save_back_to_save() {
        let mut state = test_state();
        state.page = Page::Ai;
        let _ = state.update(Message::Key(
            ApiKey::Anthropic,
            KeyMessage::Input("sk".to_string()),
        ));
        state.focus = Some(Control::Key(ApiKey::Anthropic, KeyControl::Save));
        let _ = state.press(Press::Enter, false);
        let _ = state.update(Message::Key(ApiKey::Anthropic, KeyMessage::Save));
        answer(&mut state, ApiKey::Anthropic, Err("locked".to_string()));
        assert_eq!(
            state.focus(),
            Some(&Control::Key(ApiKey::Anthropic, KeyControl::Save)),
            "the typed key is kept to try again"
        );

        // Enter in the field saves too.
        state.focus = Some(Control::Key(ApiKey::Anthropic, KeyControl::Field));
        let _ = state.update(Message::Key(ApiKey::Anthropic, KeyMessage::Save));
        let effects = answer(&mut state, ApiKey::Anthropic, Ok(KeyState::Saved));
        assert_eq!(
            state.focus(),
            Some(&Control::Key(ApiKey::Anthropic, KeyControl::Replace))
        );
        assert!(effects > 0, "the field is left");
    }

    #[test]
    fn a_store_answer_leaves_a_focus_outside_its_row_alone() {
        let mut state = test_state();
        state.page = Page::Ai;
        state.focus = Some(Control::AiModel);
        assert_eq!(
            answer(&mut state, ApiKey::Anthropic, Ok(KeyState::Saved)),
            0
        );
        assert_eq!(state.focus(), Some(&Control::AiModel));
    }

    #[test]
    fn asking_to_remove_then_esc_puts_the_focus_back_on_remove() {
        let mut state = test_state();
        state.page = Page::Subtitles;
        saved_key(&mut state, ApiKey::Soniox);
        state.focus = Some(Control::Key(ApiKey::Soniox, KeyControl::AskRemove));
        let _ = state.press(Press::Enter, false);
        assert!(state.key(ApiKey::Soniox).confirm_remove);
        assert_eq!(
            state.focus(),
            Some(&Control::Key(ApiKey::Soniox, KeyControl::Keep))
        );
        assert!(state.escape());
        assert_eq!(
            state.focus(),
            Some(&Control::Key(ApiKey::Soniox, KeyControl::AskRemove))
        );

        // Without keyboard focus, Esc leaves it that way.
        state.apply(Message::Key(ApiKey::Soniox, KeyMessage::AskRemove));
        state.focus = None;
        assert!(state.escape());
        assert_eq!(state.focus(), None);
    }

    #[test]
    fn a_pressed_offer_hands_the_focus_to_the_control_after_it() {
        let mut state = test_state();
        state.page = Page::Saving;
        state.apply(Message::SetSpaceAfterTags(true));
        state.focus = Some(Control::RespaceTags);
        let _ = state.press(Press::Space, false);
        assert_eq!(
            state.focus(),
            Some(&Control::CommentStorage(CommentStorage::InVideo))
        );
    }

    #[test]
    fn a_control_disabled_since_it_took_the_focus_does_nothing() {
        let mut state = test_state();
        state.page = Page::Ai;
        state.focus = Some(Control::Key(ApiKey::Anthropic, KeyControl::Save));
        let _ = state.press(Press::Enter, false);
        assert_eq!(
            state.focus(),
            Some(&Control::Key(ApiKey::Anthropic, KeyControl::Save)),
            "nothing typed: Save is disabled, the press is dropped"
        );
        assert!(state.key(ApiKey::Anthropic).error.is_none());

        state.page = Page::Updates;
        let update = Control::Updates(crate::features::updates::Control::UpdateAndRestart);
        assert!(!focus::controls(&state, true).contains(&update));
        state.focus = Some(update.clone());
        let _ = state.press(Press::Enter, true);
        assert_eq!(state.focus(), Some(&update));
    }

    #[test]
    fn a_page_change_or_a_click_takes_the_focus_away_and_typing_brings_it_to_the_field() {
        let mut state = test_state();
        state.focus = Some(Control::MonochromeTags);
        let _ = state.update(Message::ClearFocus);
        assert_eq!(state.focus(), None);
        let _ = state.update(Message::FieldFocused(Some(focus::COMMENTED_TAG_FIELD)));
        assert_eq!(state.focus(), Some(&Control::CommentedTagName));
        let _ = state.update(Message::FieldFocused(None));
        assert_eq!(
            state.focus(),
            Some(&Control::CommentedTagName),
            "no field focused: the focus stays"
        );

        let _ = state.show_page(Page::Subtitles);
        assert_eq!(state.focus(), None);
        let _ = state.update(Message::Key(
            ApiKey::Soniox,
            KeyMessage::Input("k".to_string()),
        ));
        assert_eq!(
            state.focus(),
            Some(&Control::Key(ApiKey::Soniox, KeyControl::Field))
        );
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
