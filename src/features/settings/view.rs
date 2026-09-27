//! UI for the settings window.

use clipscribe::{Model, MODELS};
use frename_core::ai::key::{ApiKey, KeyState};
use frename_core::ai::SummaryLanguage;
use frename_core::{CommentStorage, CueLength, InOutStorage, MarkerStorage};
use iced::widget::{
    button, checkbox, column, container, pick_list, radio, row, scrollable, text, text_input,
};
use iced::{Element, Length};

use crate::theme;

use super::state::{KeySection, LanguageList, OldSettingsImport};
use super::{KeyMessage, Message, SettingsState};

use crate::features::batch::{describe_ai, MarkersDirection, Operation};
use crate::features::updates;

/// The settings' scrollable content, which "Describe with AI" and "Generate subtitles" open
/// scrolled to its end, where their sections are.
pub const SETTINGS_SCROLLABLE_ID: &str = "settings-content";

/// Render the settings window: one titled section per area, one control per setting.
/// `batch_running` holds back **Update and restart** while a batch job writes files.
pub fn view(state: &SettingsState, batch_running: bool) -> Element<'_, Message> {
    let settings = state.settings();

    let language = section(
        fl!("settings-language"),
        row![pick_list(
            language_options(),
            Some(LanguageOption(settings.ui_language.clone())),
            |opt| Message::SetUiLanguage(opt.0),
        )
        .text_size(13)
        .padding([3, 8])]
        .into(),
    );

    let video = section(
        fl!("settings-video"),
        checkbox(settings.autoplay_video)
            .label(fl!("settings-video-autoplay"))
            .on_toggle(Message::SetAutoplayVideo)
            .into(),
    );
    let mut tag_options = column![
        checkbox(settings.monochrome_tags)
            .label(fl!("settings-tags-monochrome"))
            .on_toggle(Message::SetMonochromeTags),
        checkbox(settings.space_after_tags)
            .label(fl!("settings-tags-space-after"))
            .on_toggle(Message::SetSpaceAfterTags),
    ]
    .spacing(8);
    if state.tag_spacing_changed() {
        let label = if settings.space_after_tags {
            fl!("settings-tags-space-add")
        } else {
            fl!("settings-tags-space-remove")
        };
        tag_options = tag_options.push(move_offer(
            fl!("settings-tags-space-note"),
            label,
            Operation::RespaceTags,
        ));
    }
    let tags = section(fl!("settings-tags"), tag_options.into());

    // Each option's own settings sit right under it: the tag under "inside the video", and the
    // offer to move existing files under whichever option was just chosen.
    let selected_storage = Some(settings.comment_storage);
    let comment_offer = state.comment_storage_changed().then(|| {
        let label = match settings.comment_storage {
            CommentStorage::InVideo => fl!("settings-comments-move-into-videos"),
            CommentStorage::TextFile => fl!("settings-comments-move-into-text-files"),
        };
        move_offer(
            fl!("settings-comments-note"),
            label,
            Operation::MoveComments(settings.comment_storage),
        )
    });
    let mut comment_options = column![radio(
        fl!("settings-comments-in-video"),
        CommentStorage::InVideo,
        selected_storage,
        Message::SetCommentStorage,
    )]
    .spacing(8);
    let mut text_file_offer = None;
    match settings.comment_storage {
        CommentStorage::InVideo => {
            comment_options = comment_options
                .push(commented_tag(
                    settings.commented_tag_enabled,
                    &settings.commented_tag,
                ))
                .extend(comment_offer);
        }
        CommentStorage::TextFile => text_file_offer = comment_offer,
    }
    let comment_options = comment_options
        .push(radio(
            fl!("settings-comments-text-file"),
            CommentStorage::TextFile,
            selected_storage,
            Message::SetCommentStorage,
        ))
        .extend(text_file_offer);
    let comments = section(fl!("settings-comments"), comment_options.into());

    let selected_in_out = Some(settings.in_out_storage);
    let in_out_offer = |storage: InOutStorage| {
        (state.in_out_storage_changed() && settings.in_out_storage == storage).then(|| {
            let label = match storage {
                InOutStorage::InVideo => fl!("settings-in-out-move-into-videos"),
                InOutStorage::Comment => fl!("settings-in-out-move-into-comments"),
            };
            move_offer(
                fl!("settings-in-out-note"),
                label,
                Operation::MoveInOut(storage),
            )
        })
    };
    let in_out_options = column![radio(
        fl!("settings-in-out-in-video"),
        InOutStorage::InVideo,
        selected_in_out,
        Message::SetInOutStorage,
    )]
    .extend(in_out_offer(InOutStorage::InVideo))
    .push(radio(
        fl!("settings-in-out-comment"),
        InOutStorage::Comment,
        selected_in_out,
        Message::SetInOutStorage,
    ))
    .extend(in_out_offer(InOutStorage::Comment))
    .spacing(8);
    let in_out = section(fl!("settings-in-out"), in_out_options.into());

    let selected_markers = Some(settings.marker_storage);
    let marker_offer = |storage: MarkerStorage| {
        (state.marker_storage_changed() && settings.marker_storage == storage).then(|| {
            let (label, direction) = match storage {
                MarkerStorage::InVideo => (
                    fl!("settings-markers-move-into-videos"),
                    MarkersDirection::CommentToMarkers,
                ),
                MarkerStorage::Comment => (
                    fl!("settings-markers-copy-into-comment"),
                    MarkersDirection::MarkersToComment,
                ),
            };
            move_offer(
                fl!("settings-markers-note"),
                label,
                Operation::MarkersComment(direction),
            )
        })
    };
    let marker_options = column![radio(
        fl!("settings-markers-in-video"),
        MarkerStorage::InVideo,
        selected_markers,
        Message::SetMarkerStorage,
    )]
    .extend(marker_offer(MarkerStorage::InVideo))
    .push(radio(
        fl!("settings-markers-comment"),
        MarkerStorage::Comment,
        selected_markers,
        Message::SetMarkerStorage,
    ))
    .extend(marker_offer(MarkerStorage::Comment))
    .push(
        text(fl!("settings-markers-hint"))
            .size(12)
            .color(theme::TEXT_MUTED),
    )
    .spacing(8);
    let markers = section(fl!("settings-markers"), marker_options.into());

    let updates = section(
        fl!("settings-updates"),
        updates::view::view(state.updates(), batch_running).map(Message::Updates),
    );
    let mut sections =
        column![language, video, tags, comments, markers, in_out, updates].spacing(20);
    // Only a package keeps its settings away from the exe; elsewhere they are next to it.
    if state.updates().installed() {
        sections = sections.push(section(
            fl!("settings-old-title"),
            old_settings_import(state.old_settings_import()),
        ));
    }
    // Last, so "Describe with AI" and "Generate subtitles" can open the window scrolled to its
    // end, at these sections.
    sections = sections
        .push(section(
            fl!("settings-ai"),
            ai_options(
                state.key(ApiKey::Anthropic),
                settings.summary_language,
                Model::from_id(&settings.ai_model),
            ),
        ))
        .push(section(
            fl!("settings-subtitles"),
            subtitle_options(
                state.key(ApiKey::Soniox),
                state.subtitle_languages(),
                &settings.subtitle_languages,
                settings.subtitle_cue_length,
            ),
        ));

    // The window is not resizable: whatever does not fit scrolls.
    container(
        scrollable(container(sections).padding(20))
            .id(iced::widget::Id::new(SETTINGS_SCROLLABLE_ID))
            .style(theme::dark_scrollable_style),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::main_container_style)
    .into()
}

/// The AI section: the Anthropic API key, and the model and language of descriptions.
fn ai_options(key: &KeySection, language: SummaryLanguage, model: Model) -> Element<'_, Message> {
    let muted = |line: String| text(line).size(12).color(theme::TEXT_MUTED);
    key_block(
        ApiKey::Anthropic,
        key,
        fl!("settings-ai-key-label"),
        fl!("settings-ai-key-placeholder"),
        fl!("settings-ai-key-get"),
    )
    .push(
        row![
            text(fl!("settings-ai-model-label")).size(13),
            pick_list(MODELS, Some(model), Message::SetAiModel)
                .text_size(13)
                .padding([3, 8]),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    )
    .push(
        row![
            text(fl!("settings-ai-language-label")).size(13),
            pick_list(
                SummaryLanguage::ALL.map(SummaryLanguageOption),
                Some(SummaryLanguageOption(language)),
                |opt| Message::SetSummaryLanguage(opt.0)
            )
            .text_size(13)
            .padding([3, 8]),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    )
    .push(muted(fl!("settings-ai-hint")))
    .into()
}

/// The Subtitles section: the Soniox API key, the languages spoken in the footage, and how
/// long a cue may get.
fn subtitle_options<'a>(
    key: &'a KeySection,
    list: &'a LanguageList,
    languages: &'a [String],
    cue_length: CueLength,
) -> Element<'a, Message> {
    let muted = |line: String| text(line).size(12).color(theme::TEXT_MUTED);
    let checked = |code: &str| languages.iter().any(|l| l == code);
    // Soniox's own list; until it comes (or without a key) the checked codes stay uncheckable.
    let offered: Vec<(String, String)> = match list {
        LanguageList::Listed(all) => all.clone(),
        _ => languages.iter().map(|c| (c.clone(), c.clone())).collect(),
    };
    let language_rows: Vec<Element<'a, Message>> = offered
        .chunks(4)
        .map(|chunk| {
            row(chunk.iter().map(|(code, name)| {
                let code = code.clone();
                checkbox(checked(&code))
                    .label(name.clone())
                    .text_size(13)
                    .on_toggle(move |on| Message::SetSubtitleLanguage(code.clone(), on))
                    .width(Length::Fixed(118.0))
                    .into()
            }))
            .spacing(8)
            .into()
        })
        .collect();
    let status = match list {
        LanguageList::NotAsked if key.state == Some(KeyState::Missing) => {
            Some(fl!("settings-subtitles-languages-locked"))
        }
        LanguageList::NotAsked | LanguageList::Listed(_) => None,
        LanguageList::Loading => Some(fl!("settings-subtitles-languages-loading")),
        LanguageList::Failed(why) => Some(why.clone()),
    };
    let hint = if languages.is_empty() {
        fl!("settings-subtitles-languages-none")
    } else {
        fl!("settings-subtitles-languages-hint")
    };
    let selected = Some(cue_length);
    key_block(
        ApiKey::Soniox,
        key,
        fl!("settings-subtitles-key-label"),
        fl!("settings-subtitles-key-placeholder"),
        fl!("settings-subtitles-key-get"),
    )
    .push(text(fl!("settings-subtitles-languages-label")).size(13))
    .extend(language_rows)
    .extend(status.map(|s| text(s).size(12).color(theme::TEXT_MUTED).into()))
    .push(muted(hint))
    .push(
        row![
            text(fl!("settings-subtitles-cue-length-label")).size(13),
            radio(
                fl!("settings-subtitles-cue-short"),
                CueLength::Short,
                selected,
                Message::SetSubtitleCueLength,
            )
            .text_size(13),
            radio(
                fl!("settings-subtitles-cue-sentence"),
                CueLength::Sentence,
                selected,
                Message::SetSubtitleCueLength,
            )
            .text_size(13),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center),
    )
    .push(muted(fl!("settings-subtitles-hint")))
    .into()
}

/// An API key's field (or "Key saved" with Replace / Remove), where it is kept, and where to
/// get one.
fn key_block<'a>(
    which: ApiKey,
    key: &'a KeySection,
    label: String,
    placeholder: String,
    get_one: String,
) -> iced::widget::Column<'a, Message> {
    let muted = |line: String| text(line).size(12).color(theme::TEXT_MUTED);
    let message = move |m: KeyMessage| Message::Key(which, m);
    let store = if cfg!(windows) {
        fl!("settings-key-store-windows")
    } else if cfg!(target_os = "macos") {
        fl!("settings-key-store-macos")
    } else {
        fl!("settings-key-store-other")
    };
    let key_row: Element<'_, Message> = match key.state {
        Some(KeyState::Unavailable) => text(fl!("settings-key-unavailable"))
            .size(13)
            .color(theme::ERROR)
            .into(),
        // The question and its buttons on lines of their own, so they fit the window.
        Some(KeyState::Saved) if key.confirm_remove => column![
            text(fl!("settings-key-remove-confirm")).size(12),
            row![
                small_button(fl!("settings-key-remove"), message(KeyMessage::Remove)),
                small_button(fl!("settings-key-keep"), message(KeyMessage::CancelRemove)),
            ]
            .spacing(8),
        ]
        .spacing(6)
        .into(),
        Some(KeyState::Saved) if !key.replacing => row![
            text(fl!("settings-key-saved")).size(13),
            small_button(fl!("settings-key-replace"), message(KeyMessage::Replace)),
            small_button(fl!("settings-key-remove"), message(KeyMessage::AskRemove)),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into(),
        _ => {
            let can_save = !key.input.trim().is_empty();
            let save = can_save.then_some(message(KeyMessage::Save));
            let cancel = key
                .replacing
                .then(|| small_button(fl!("batch-cancel"), message(KeyMessage::CancelReplace)));
            // The buttons go under the field, so the row fits the window with Cancel too.
            column![
                text_input(&placeholder, &key.input)
                    .secure(!key.shown)
                    .on_input(move |input| message(KeyMessage::Input(input)))
                    .on_submit_maybe(save.clone())
                    .size(13)
                    .padding([3, 6])
                    .width(Length::Fixed(260.0)),
                row![
                    small_button(
                        if key.shown {
                            fl!("settings-key-hide")
                        } else {
                            fl!("settings-key-show")
                        },
                        message(KeyMessage::ToggleShow)
                    ),
                    button(text(fl!("settings-key-save")).size(12))
                        .on_press_maybe(save)
                        .padding([3, 10]),
                ]
                .extend(cancel)
                .spacing(8),
            ]
            .spacing(6)
            .into()
        }
    };
    let mut options = column![row![text(label).size(13), key_row]
        .spacing(12)
        .align_y(iced::Alignment::Center)]
    .spacing(6);
    options = match key.state {
        Some(KeyState::Unavailable) => options.push(muted(fl!("settings-key-unavailable-hint"))),
        Some(KeyState::Saved) if !key.replacing => options.push(
            text(fl!("settings-key-saved-in", store = store))
                .size(12)
                .color(theme::TEXT_MUTED),
        ),
        _ => options
            .push(
                text(fl!("settings-key-save-into", store = store))
                    .size(12)
                    .color(theme::TEXT_MUTED),
            )
            .push(muted(get_one)),
    };
    if let Some(error) = &key.error {
        options = options.push(text(error.as_str()).size(12).color(theme::ERROR));
    }
    options
}

fn small_button(label: String, message: Message) -> Element<'static, Message> {
    button(text(label).size(12))
        .on_press(message)
        .padding([3, 10])
        .style(theme::icon_button_style(true))
        .into()
}

/// The way back when the first-start search missed the zip version's folder.
fn old_settings_import(import: &OldSettingsImport) -> Element<'_, Message> {
    let note = match import {
        OldSettingsImport::None => String::new(),
        OldSettingsImport::Scheduled(_) => fl!("settings-old-scheduled"),
        OldSettingsImport::NotFound(folder) => {
            fl!(
                "settings-old-not-found",
                folder = folder.display().to_string()
            )
        }
        OldSettingsImport::Failed(reason) => {
            fl!("settings-old-failed", reason = reason.clone())
        }
    };
    row![
        button(text(fl!("settings-old-import-button")).size(13))
            .on_press(Message::ImportOldSettings),
        text(note).size(13).color(theme::TEXT_MUTED),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center)
    .into()
}

/// The tag for videos with a comment, shown while comments are inside the video: a comment inside the
/// file is not visible in Explorer or in the name, the tag is. The check box turns tagging off
/// altogether; the name is kept for when it is turned back on.
fn commented_tag(enabled: bool, tag: &str) -> Element<'_, Message> {
    let mut input = text_input(frename_core::DEFAULT_COMMENTED_TAG, tag)
        .size(13)
        .padding([3, 6])
        .width(Length::Fixed(160.0));
    if enabled {
        input = input.on_input(Message::SetCommentedTag);
    }
    let hint = match frename_core::clean_commented_tag(tag).filter(|_| enabled) {
        Some(tag) => fl!("settings-commented-tag-hint", tag = tag),
        None => fl!("settings-commented-tag-off"),
    };
    column![
        row![
            checkbox(enabled)
                .label(fl!("settings-commented-tag"))
                .text_size(13)
                .on_toggle(Message::SetCommentedTagEnabled),
            input,
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center),
        text(hint).size(12).color(theme::TEXT_MUTED),
    ]
    .spacing(4)
    .padding(iced::Padding {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 26.0,
    })
    .into()
}

fn section<'a>(title: String, content: Element<'a, Message>) -> Element<'a, Message> {
    column![text(title).size(14).color(theme::TEXT_MUTED), content]
        .spacing(8)
        .into()
}

/// Shown after a storage change: the setting only decides where things are saved from now
/// on, so moving what the files already have is a separate batch action, one click away.
fn move_offer(note: String, label: String, operation: Operation) -> Element<'static, Message> {
    column![
        text(note).size(12).color(theme::TEXT_MUTED),
        button(text(label).size(13)).on_press(Message::OpenBatchAction(operation)),
    ]
    .spacing(6)
    .padding(iced::Padding {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 26.0,
    })
    .into()
}

/// One item of the Language pick list: an empty code stands for `System`.
#[derive(Clone, PartialEq, Eq)]
struct LanguageOption(String);

impl std::fmt::Display for LanguageOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(
                f,
                "{}",
                fl!(
                    "settings-language-system",
                    language = crate::i18n::language_name(crate::i18n::system_language())
                )
            )
        } else {
            write!(f, "{}", crate::i18n::language_name(&self.0))
        }
    }
}

/// `System`, then every UI language, in the order Settings lists them.
fn language_options() -> Vec<LanguageOption> {
    std::iter::once(String::new())
        .chain(crate::i18n::LANGUAGES.iter().map(|l| l.to_string()))
        .map(LanguageOption)
        .collect()
}

/// One item of the description-language pick list: `SummaryLanguage`'s own `Display` is
/// English only (it is the value stored in the settings), so this wrapper renders it translated
/// via [`describe_ai::language_name`], the same name used in the batch panel's plan line.
#[derive(Clone, Copy, PartialEq, Eq)]
struct SummaryLanguageOption(SummaryLanguage);

impl std::fmt::Display for SummaryLanguageOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", describe_ai::language_name(self.0))
    }
}
