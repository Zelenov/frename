//! Keyboard focus in the settings window (#167, `docs/design/design-system.md` §11, §14.1).
//!
//! iced 0.14 focuses only text fields, and its focus operations run over every open window, so
//! the window keeps its own focus: the control Tab and Shift+Tab move to, in page order, then
//! **Close**. Space presses the focused control the way a click would (a checkbox toggles, a
//! radio is picked, a dropdown moves to its next option, a button is pressed); Enter presses a
//! focused button. A text field takes real iced focus, so typing goes into it.

use clipscribe::MODELS;
use frename_core::ai::key::{ApiKey, KeyState};
use frename_core::ai::SummaryLanguage;
use frename_core::{CommentStorage, CueLength, InOutStorage, MarkerStorage};

use super::state::{KeySection, SettingsState};
use super::{KeyMessage, Message, Page};
use crate::features::batch::Operation;
use crate::features::updates;

/// The text fields of the settings window; only these are focused or unfocused by Tab.
pub const FIELD_IDS: &[&str] = &[ANTHROPIC_KEY_FIELD, SONIOX_KEY_FIELD, COMMENTED_TAG_FIELD];
pub const ANTHROPIC_KEY_FIELD: &str = "settings-key-anthropic";
pub const SONIOX_KEY_FIELD: &str = "settings-key-soniox";
pub const COMMENTED_TAG_FIELD: &str = "settings-commented-tag";

/// A control of the settings window that keyboard focus can reach.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Control {
    UiLanguage,
    MonochromeTags,
    AutoplayVideo,
    SpaceAfterTags,
    /// The offer to rename the files to the tag spacing just chosen.
    RespaceTags,
    CommentStorage(CommentStorage),
    CommentedTagEnabled,
    CommentedTagName,
    /// The offer to move the files' comments to the storage just chosen.
    MoveComments,
    MarkerStorage(MarkerStorage),
    MoveMarkers,
    InOutStorage(InOutStorage),
    MoveInOut,
    Key(ApiKey, KeyControl),
    AiModel,
    SummaryLanguage,
    CueLength(CueLength),
    /// A language spoken in the footage, by its code.
    SubtitleLanguage(String),
    Updates(updates::Control),
    ImportOldSettings,
    Close,
}

/// A control of an API key row (§14.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyControl {
    Field,
    Show,
    Save,
    Cancel,
    Replace,
    AskRemove,
    Remove,
    Keep,
}

/// The key that pressed a focused control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    Space,
    Enter,
}

impl Control {
    /// The id of the text field this control is, if it is one.
    pub fn field_id(&self) -> Option<&'static str> {
        match self {
            Control::Key(ApiKey::Anthropic, KeyControl::Field) => Some(ANTHROPIC_KEY_FIELD),
            Control::Key(ApiKey::Soniox, KeyControl::Field) => Some(SONIOX_KEY_FIELD),
            Control::CommentedTagName => Some(COMMENTED_TAG_FIELD),
            _ => None,
        }
    }

    fn is_button(&self) -> bool {
        match self {
            Control::RespaceTags
            | Control::MoveComments
            | Control::MoveMarkers
            | Control::MoveInOut
            | Control::ImportOldSettings
            | Control::Close
            | Control::Updates(updates::Control::Check | updates::Control::UpdateAndRestart) => {
                true
            }
            Control::Key(_, key) => *key != KeyControl::Field,
            _ => false,
        }
    }
}

/// The text field of the `which` key row.
pub fn key_field_id(which: ApiKey) -> &'static str {
    match which {
        ApiKey::Anthropic => ANTHROPIC_KEY_FIELD,
        ApiKey::Soniox => SONIOX_KEY_FIELD,
    }
}

/// The choices of the UI language dropdown, in its order: `System` (an empty code), then every
/// UI language. The view lists them and Space steps through them.
pub fn ui_languages() -> Vec<&'static str> {
    std::iter::once("")
        .chain(crate::i18n::LANGUAGES.iter().copied())
        .collect()
}

/// The text field `id` is, as a control.
pub fn control_of_field(id: &str) -> Option<Control> {
    match id {
        ANTHROPIC_KEY_FIELD => Some(Control::Key(ApiKey::Anthropic, KeyControl::Field)),
        SONIOX_KEY_FIELD => Some(Control::Key(ApiKey::Soniox, KeyControl::Field)),
        COMMENTED_TAG_FIELD => Some(Control::CommentedTagName),
        _ => None,
    }
}

/// What a key row shows (§14.3): the one place both the view and [`controls`] read it from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRow {
    /// The credential store cannot be used: a warning, no controls.
    Unavailable,
    /// "Remove the saved key?": **Remove key** and **Keep**.
    ConfirmRemove,
    /// A key is saved: **Replace…** and **Remove…**.
    Saved,
    /// The field to type a key into, **Show**, **Save key** (enabled once something is typed)
    /// and, while replacing a saved key, **Cancel**.
    Typing { can_save: bool, cancel: bool },
}

/// What the key row of `key` shows now.
pub fn key_row(key: &KeySection) -> KeyRow {
    match key.state {
        Some(KeyState::Unavailable) => KeyRow::Unavailable,
        Some(KeyState::Saved) if key.confirm_remove => KeyRow::ConfirmRemove,
        Some(KeyState::Saved) if !key.replacing => KeyRow::Saved,
        _ => KeyRow::Typing {
            can_save: !key.input.trim().is_empty(),
            cancel: key.replacing,
        },
    }
}

/// The controls of the page shown that take a click now, in the order the page shows them, and
/// **Close** last. `batch_running` holds back **Update and restart**, as the view does.
pub fn controls(state: &SettingsState, batch_running: bool) -> Vec<Control> {
    let settings = state.settings();
    let mut controls = Vec::new();
    match state.page() {
        Page::Interface => controls.extend([
            Control::UiLanguage,
            Control::MonochromeTags,
            Control::AutoplayVideo,
        ]),
        Page::Saving => {
            controls.push(Control::SpaceAfterTags);
            if state.tag_spacing_changed() {
                controls.push(Control::RespaceTags);
            }
            controls.push(Control::CommentStorage(CommentStorage::InVideo));
            if settings.comment_storage == CommentStorage::InVideo {
                controls.push(Control::CommentedTagEnabled);
                if settings.commented_tag_enabled {
                    controls.push(Control::CommentedTagName);
                }
            }
            controls.push(Control::CommentStorage(CommentStorage::TextFile));
            if state.comment_storage_changed() {
                controls.push(Control::MoveComments);
            }
            controls.extend([
                Control::MarkerStorage(MarkerStorage::InVideo),
                Control::MarkerStorage(MarkerStorage::Comment),
            ]);
            if state.marker_storage_changed() {
                controls.push(Control::MoveMarkers);
            }
            controls.extend([
                Control::InOutStorage(InOutStorage::InVideo),
                Control::InOutStorage(InOutStorage::Comment),
            ]);
            if state.in_out_storage_changed() {
                controls.push(Control::MoveInOut);
            }
        }
        Page::Ai => {
            controls.extend(key_controls(state, ApiKey::Anthropic));
            controls.extend([Control::AiModel, Control::SummaryLanguage]);
        }
        Page::Subtitles => {
            controls.extend(key_controls(state, ApiKey::Soniox));
            controls.extend([
                Control::CueLength(CueLength::Short),
                Control::CueLength(CueLength::Sentence),
            ]);
            controls.extend(
                state
                    .offered_subtitle_languages()
                    .into_iter()
                    .map(|(code, _)| Control::SubtitleLanguage(code)),
            );
        }
        Page::Updates => {
            controls.extend(
                state
                    .updates()
                    .controls(batch_running)
                    .into_iter()
                    .map(Control::Updates),
            );
            if state.updates().installed() {
                controls.push(Control::ImportOldSettings);
            }
        }
    }
    controls.push(Control::Close);
    controls
}

/// The controls of a key row that take a click now, in the order the row shows them.
pub fn key_controls(state: &SettingsState, which: ApiKey) -> Vec<Control> {
    let controls = match key_row(state.key(which)) {
        KeyRow::Unavailable => vec![],
        KeyRow::ConfirmRemove => vec![KeyControl::Remove, KeyControl::Keep],
        KeyRow::Saved => vec![KeyControl::Replace, KeyControl::AskRemove],
        KeyRow::Typing { can_save, cancel } => [
            Some(KeyControl::Field),
            Some(KeyControl::Show),
            can_save.then_some(KeyControl::Save),
            cancel.then_some(KeyControl::Cancel),
        ]
        .into_iter()
        .flatten()
        .collect(),
    };
    controls
        .into_iter()
        .map(|control| Control::Key(which, control))
        .collect()
}

/// The control `steps` away from `from` among `controls`, wrapping around; from none (or from
/// one no longer shown), Tab starts at the first and Shift+Tab at the last.
pub fn step(controls: &[Control], from: Option<&Control>, steps: isize) -> Option<Control> {
    let count = controls.len() as isize;
    if count == 0 {
        return None;
    }
    let at = from.and_then(|from| controls.iter().position(|c| c == from));
    let next = match at {
        Some(at) => (at as isize + steps).rem_euclid(count),
        None if steps < 0 => count - 1,
        None => 0,
    };
    Some(controls[next as usize].clone())
}

/// What pressing `control` with `press` sends: what a click on it would. `None` for a text field
/// (it takes the key itself) and for Enter on anything but a button.
pub fn press(state: &SettingsState, control: &Control, press: Press) -> Option<Message> {
    if press == Press::Enter && !control.is_button() {
        return None;
    }
    let settings = state.settings();
    let message = match control {
        Control::UiLanguage => Message::SetUiLanguage(
            next_of(&ui_languages(), &settings.ui_language.as_str()).to_string(),
        ),
        Control::MonochromeTags => Message::SetMonochromeTags(!settings.monochrome_tags),
        Control::AutoplayVideo => Message::SetAutoplayVideo(!settings.autoplay_video),
        Control::SpaceAfterTags => Message::SetSpaceAfterTags(!settings.space_after_tags),
        Control::RespaceTags => Message::OpenBatchAction(Operation::RespaceTags),
        Control::CommentStorage(storage) => Message::SetCommentStorage(*storage),
        Control::CommentedTagEnabled => {
            Message::SetCommentedTagEnabled(!settings.commented_tag_enabled)
        }
        Control::CommentedTagName => return None,
        Control::MoveComments => {
            Message::OpenBatchAction(Operation::MoveComments(settings.comment_storage))
        }
        Control::MarkerStorage(storage) => Message::SetMarkerStorage(*storage),
        Control::MoveMarkers => Message::OpenBatchAction(state.markers_offer()),
        Control::InOutStorage(storage) => Message::SetInOutStorage(*storage),
        Control::MoveInOut => {
            Message::OpenBatchAction(Operation::MoveInOut(settings.in_out_storage))
        }
        Control::Key(which, key) => {
            let message = match key {
                KeyControl::Field => return None,
                KeyControl::Show => KeyMessage::ToggleShow,
                KeyControl::Save => KeyMessage::Save,
                KeyControl::Cancel => KeyMessage::CancelReplace,
                KeyControl::Replace => KeyMessage::Replace,
                KeyControl::AskRemove => KeyMessage::AskRemove,
                KeyControl::Remove => KeyMessage::Remove,
                KeyControl::Keep => KeyMessage::CancelRemove,
            };
            Message::Key(*which, message)
        }
        Control::AiModel => {
            let ids: Vec<&str> = MODELS.iter().map(|model| model.id).collect();
            let current = clipscribe::Model::from_id(&settings.ai_model).id;
            Message::SetAiModel(clipscribe::Model::from_id(next_of(&ids, &current)))
        }
        Control::SummaryLanguage => {
            Message::SetSummaryLanguage(*next_of(&SummaryLanguage::ALL, &settings.summary_language))
        }
        Control::CueLength(length) => Message::SetSubtitleCueLength(*length),
        Control::SubtitleLanguage(code) => {
            Message::SetSubtitleLanguage(code.clone(), !settings.subtitle_languages.contains(code))
        }
        Control::Updates(control) => Message::Updates(state.updates().press(*control)),
        Control::ImportOldSettings => Message::ImportOldSettings,
        Control::Close => Message::Close,
    };
    Some(message)
}

/// Where the focus goes after `pressed` was pressed, when the press takes that control away
/// (§11): the question about removing a key keeps the key by default; giving up a replacement or
/// a removal goes back to the button that started it; **Replace…** goes into the new field;
/// **Remove key** closes the question, so the focus goes to the **Remove…** it shows again;
/// **Check for updates** (disabled while it checks) to the checkbox under it. **Save key** stays
/// focused while the store saves: where it goes then is settled by the store's answer
/// (`SettingsState::after_key_answer`). An offer goes away with its press: the focus goes to the
/// control after it (see `SettingsState::press`).
pub fn after_press(pressed: &Control) -> Option<Control> {
    match pressed {
        Control::Key(which, key) => {
            let next = match key {
                KeyControl::AskRemove => KeyControl::Keep,
                KeyControl::Keep => KeyControl::AskRemove,
                KeyControl::Replace => KeyControl::Field,
                KeyControl::Cancel => KeyControl::Replace,
                KeyControl::Remove => KeyControl::AskRemove,
                KeyControl::Field | KeyControl::Show | KeyControl::Save => return None,
            };
            Some(Control::Key(*which, next))
        }
        Control::Updates(updates::Control::Check) => {
            Some(Control::Updates(updates::Control::CheckOnStart))
        }
        _ => None,
    }
}

/// Whether pressing `control` makes it go away: an offer to run a batch action.
pub fn is_offer(control: &Control) -> bool {
    matches!(
        control,
        Control::RespaceTags | Control::MoveComments | Control::MoveMarkers | Control::MoveInOut
    )
}

/// The item after `current` in `items`, wrapping around; the first when `current` is not there.
fn next_of<'a, T: PartialEq>(items: &'a [T], current: &T) -> &'a T {
    let at = items.iter().position(|item| item == current);
    &items[at.map_or(0, |at| (at + 1) % items.len())]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controls_of(page: Page) -> Vec<Control> {
        let mut state = SettingsState::for_tests();
        state.show_page_for_tests(page);
        controls(&state, false)
    }

    #[test]
    fn tab_goes_through_the_page_in_order_and_close_comes_last() {
        assert_eq!(
            controls_of(Page::Interface),
            [
                Control::UiLanguage,
                Control::MonochromeTags,
                Control::AutoplayVideo,
                Control::Close
            ]
        );
        // A fresh install keeps comments inside the video with the tag on.
        assert_eq!(
            controls_of(Page::Saving),
            [
                Control::SpaceAfterTags,
                Control::CommentStorage(CommentStorage::InVideo),
                Control::CommentedTagEnabled,
                Control::CommentedTagName,
                Control::CommentStorage(CommentStorage::TextFile),
                Control::MarkerStorage(MarkerStorage::InVideo),
                Control::MarkerStorage(MarkerStorage::Comment),
                Control::InOutStorage(InOutStorage::InVideo),
                Control::InOutStorage(InOutStorage::Comment),
                Control::Close,
            ]
        );
    }

    #[test]
    fn controls_that_are_hidden_or_disabled_are_skipped() {
        let mut state = SettingsState::for_tests();
        state.show_page_for_tests(Page::Saving);
        state.apply_for_tests(Message::SetCommentedTagEnabled(false));
        assert!(!controls(&state, false).contains(&Control::CommentedTagName));
        state.apply_for_tests(Message::SetCommentStorage(CommentStorage::TextFile));
        let saving = controls(&state, false);
        assert!(!saving.contains(&Control::CommentedTagEnabled));
        assert_eq!(
            saving[saving
                .iter()
                .position(|c| *c == Control::CommentStorage(CommentStorage::TextFile))
                .unwrap()
                + 1],
            Control::MoveComments,
            "the offer follows the option just picked"
        );

        state.show_page_for_tests(Page::Ai);
        let key = |control| Control::Key(ApiKey::Anthropic, control);
        let ai = controls(&state, false);
        assert!(ai.contains(&key(KeyControl::Field)));
        assert!(
            !ai.contains(&key(KeyControl::Save)),
            "Save waits for a typed key"
        );
        state.apply_for_tests(Message::Key(
            ApiKey::Anthropic,
            KeyMessage::Input("sk".to_string()),
        ));
        assert!(controls(&state, false).contains(&key(KeyControl::Save)));
    }

    #[test]
    fn stepping_wraps_and_starts_at_either_end() {
        let all = controls_of(Page::Interface);
        assert_eq!(step(&all, None, 1), Some(Control::UiLanguage));
        assert_eq!(step(&all, None, -1), Some(Control::Close));
        assert_eq!(
            step(&all, Some(&Control::Close), 1),
            Some(Control::UiLanguage)
        );
        assert_eq!(
            step(&all, Some(&Control::UiLanguage), -1),
            Some(Control::Close)
        );
        assert_eq!(
            step(&all, Some(&Control::MoveComments), 1),
            Some(Control::UiLanguage),
            "a control no longer shown starts over"
        );
        assert_eq!(step(&[], None, 1), None);
    }

    #[test]
    fn space_presses_like_a_click_and_enter_only_presses_buttons() {
        let state = SettingsState::for_tests();
        assert!(matches!(
            press(&state, &Control::MonochromeTags, Press::Space),
            Some(Message::SetMonochromeTags(true))
        ));
        assert!(press(&state, &Control::MonochromeTags, Press::Enter).is_none());
        assert!(matches!(
            press(
                &state,
                &Control::CommentStorage(CommentStorage::TextFile),
                Press::Space
            ),
            Some(Message::SetCommentStorage(CommentStorage::TextFile))
        ));
        assert!(matches!(
            press(&state, &Control::Close, Press::Enter),
            Some(Message::Close)
        ));
        assert!(matches!(
            press(&state, &Control::Close, Press::Space),
            Some(Message::Close)
        ));
        assert!(press(&state, &Control::CommentedTagName, Press::Space).is_none());
    }

    #[test]
    fn space_on_a_dropdown_moves_to_its_next_option_and_wraps() {
        let mut state = SettingsState::for_tests();
        assert!(matches!(
            press(&state, &Control::UiLanguage, Press::Space),
            Some(Message::SetUiLanguage(language)) if language == "en"
        ));
        state.apply_for_tests(Message::SetUiLanguage("ru".to_string()));
        assert!(matches!(
            press(&state, &Control::UiLanguage, Press::Space),
            Some(Message::SetUiLanguage(language)) if language.is_empty()
        ));
        let last = *SummaryLanguage::ALL.last().unwrap();
        state.apply_for_tests(Message::SetSummaryLanguage(last));
        assert!(matches!(
            press(&state, &Control::SummaryLanguage, Press::Space),
            Some(Message::SetSummaryLanguage(language)) if language == SummaryLanguage::ALL[0]
        ));
        let Some(Message::SetAiModel(model)) = press(&state, &Control::AiModel, Press::Space)
        else {
            panic!("the model dropdown cycles");
        };
        assert_ne!(
            model.id,
            clipscribe::Model::from_id(&state.settings().ai_model).id
        );
    }

    #[test]
    fn the_saving_page_lists_what_each_choice_shows() {
        let mut state = SettingsState::for_tests();
        state.show_page_for_tests(Page::Saving);
        let comments_in_text_files = [
            Control::SpaceAfterTags,
            Control::RespaceTags,
            Control::CommentStorage(CommentStorage::InVideo),
            Control::CommentStorage(CommentStorage::TextFile),
            Control::MoveComments,
            Control::MarkerStorage(MarkerStorage::InVideo),
            Control::MarkerStorage(MarkerStorage::Comment),
            Control::MoveMarkers,
            Control::InOutStorage(InOutStorage::InVideo),
            Control::InOutStorage(InOutStorage::Comment),
            Control::MoveInOut,
            Control::Close,
        ];
        state.apply_for_tests(Message::SetSpaceAfterTags(true));
        state.apply_for_tests(Message::SetCommentStorage(CommentStorage::TextFile));
        state.apply_for_tests(Message::SetMarkerStorage(MarkerStorage::Comment));
        state.apply_for_tests(Message::SetInOutStorage(InOutStorage::Comment));
        assert_eq!(controls(&state, false), comments_in_text_files);

        // Back inside the video, with the tag turned off: its name field is not offered.
        state.apply_for_tests(Message::SetCommentStorage(CommentStorage::InVideo));
        state.apply_for_tests(Message::SetCommentedTagEnabled(false));
        let in_video = controls(&state, false);
        assert_eq!(
            in_video[2..5],
            [
                Control::CommentStorage(CommentStorage::InVideo),
                Control::CommentedTagEnabled,
                Control::CommentStorage(CommentStorage::TextFile),
            ]
        );
        // Taking an offer removes it.
        state.apply_for_tests(Message::OpenBatchAction(Operation::RespaceTags));
        assert!(!controls(&state, false).contains(&Control::RespaceTags));
    }

    #[test]
    fn every_field_id_names_its_control() {
        for id in FIELD_IDS {
            assert_eq!(control_of_field(id).and_then(|c| c.field_id()), Some(*id));
        }
        assert_eq!(control_of_field("search-bar-input"), None);
    }

    #[test]
    fn the_key_row_shows_what_its_state_asks_for() {
        let mut key = KeySection::default();
        assert_eq!(
            key_row(&key),
            KeyRow::Typing {
                can_save: false,
                cancel: false
            }
        );
        key.input = "  ".to_string();
        assert!(matches!(
            key_row(&key),
            KeyRow::Typing {
                can_save: false,
                ..
            }
        ));
        key.state = Some(KeyState::Saved);
        assert_eq!(key_row(&key), KeyRow::Saved);
        key.replacing = true;
        assert_eq!(
            key_row(&key),
            KeyRow::Typing {
                can_save: false,
                cancel: true
            }
        );
        key.confirm_remove = true;
        assert_eq!(key_row(&key), KeyRow::ConfirmRemove);
        key.state = Some(KeyState::Unavailable);
        assert_eq!(key_row(&key), KeyRow::Unavailable);
    }
}
