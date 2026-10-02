//! UI for the settings window (`docs/design/design-system.md` §14): a list of pages on the left,
//! the page on the right, one option per row, and a button bar with Close.

use clipscribe::{Model, MODELS};
use frename_core::ai::key::{ApiKey, KeyState};
use frename_core::ai::{MomentsMode, SummaryLanguage};
use frename_core::{CommentStorage, CueLength, InOutStorage, MarkerStorage};
use iced::widget::{column, row, Row};
use iced::{Alignment, Element, Length};

use crate::ui::layout::{self, NoticeKind};
use crate::ui::tokens::*;
use crate::ui::{button, form, scroll, text};

use super::state::{KeySection, LanguageList, OldSettingsImport};
use super::{KeyMessage, Message, Page, SettingsState, SETTINGS_SCROLLABLE_ID};

use crate::features::batch::{describe_ai, MarkersDirection, Operation};
use crate::features::updates;

/// Render the settings window. `batch_running` holds back **Update and restart** while a batch
/// job writes files.
pub fn view(state: &SettingsState, batch_running: bool) -> Element<'_, Message> {
    let update_ready = state.updates().available_version().is_some();
    let navigation = Page::ALL.map(|page| {
        layout::nav_item(
            page.icon(),
            page.label(),
            page == state.page(),
            page == Page::Updates && update_ready,
            Message::ShowPage(page),
        )
    });
    let page = match state.page() {
        Page::Interface => interface(state),
        Page::Saving => saving(state),
        Page::Ai => ai(state),
        Page::Subtitles => subtitles(state),
        Page::Updates => updates_page(state, batch_running),
    };
    layout::window_with_navigation(
        layout::sidebar(navigation),
        scroll::vertical_with_id(SETTINGS_SCROLLABLE_ID, page),
        layout::button_bar(
            fl!("settings-apply-note"),
            [button::secondary(fl!("settings-close"))
                .on_press(Message::Close)
                .into()],
        ),
    )
}

fn interface(state: &SettingsState) -> Element<'_, Message> {
    let settings = state.settings();
    layout::page(
        Page::Interface.heading(),
        [
            layout::setting_row(
                fl!("settings-language"),
                form::dropdown(
                    language_options(),
                    Some(LanguageOption(settings.ui_language.clone())),
                    |option| Message::SetUiLanguage(option.0),
                )
                .width(FIELD_WIDTH_M),
            ),
            layout::setting_row(
                fl!("settings-tags"),
                layout::aligned([form::checkbox_with_hint(
                    form::checkbox(fl!("settings-tags-monochrome"), settings.monochrome_tags)
                        .on_toggle(Message::SetMonochromeTags),
                    text::secondary(fl!("settings-tags-monochrome-hint")),
                )]),
            ),
            layout::setting_row(
                fl!("settings-video"),
                layout::aligned([form::checkbox(
                    fl!("settings-video-autoplay"),
                    settings.autoplay_video,
                )
                .on_toggle(Message::SetAutoplayVideo)
                .into()]),
            ),
        ],
    )
}

fn saving(state: &SettingsState) -> Element<'_, Message> {
    let settings = state.settings();

    let spacing_offer = state.tag_spacing_changed().then(|| {
        let label = if settings.space_after_tags {
            fl!("settings-tags-space-add")
        } else {
            fl!("settings-tags-space-remove")
        };
        move_offer(
            fl!("settings-tags-space-note"),
            label,
            Operation::RespaceTags,
        )
    });
    let file_names = layout::aligned(
        std::iter::once(form::checkbox_with_hint(
            form::checkbox(fl!("settings-tags-space-after"), settings.space_after_tags)
                .on_toggle(Message::SetSpaceAfterTags),
            text::mono(fl!("settings-tags-space-example")),
        ))
        .chain(spacing_offer),
    );

    let comment_storage = Some(settings.comment_storage);
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
    // The tag for commented videos only matters while comments are inside the video, as before.
    let commented = (settings.comment_storage == CommentStorage::InVideo).then(|| {
        layout::indented(commented_tag(
            settings.commented_tag_enabled,
            &settings.commented_tag,
        ))
    });
    let comments = layout::aligned(
        [form::radio_option(
            fl!("settings-comments-in-video"),
            Some(form::description(fl!("settings-comments-in-video-hint"))),
            CommentStorage::InVideo,
            comment_storage,
            Message::SetCommentStorage,
        )]
        .into_iter()
        .chain(commented)
        .chain([form::radio_option(
            fl!("settings-comments-text-file"),
            Some(form::example(fl!("settings-comments-text-file-example"))),
            CommentStorage::TextFile,
            comment_storage,
            Message::SetCommentStorage,
        )])
        .chain(comment_offer),
    );

    let marker_storage = Some(settings.marker_storage);
    let marker_offer = state.marker_storage_changed().then(|| {
        let (label, direction) = match settings.marker_storage {
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
    });
    let markers = layout::aligned(
        [
            form::radio_option(
                fl!("settings-markers-in-video"),
                Some(form::description(fl!("settings-markers-in-video-hint"))),
                MarkerStorage::InVideo,
                marker_storage,
                Message::SetMarkerStorage,
            ),
            form::radio_option(
                fl!("settings-markers-comment"),
                Some(form::example(fl!("settings-markers-comment-example"))),
                MarkerStorage::Comment,
                marker_storage,
                Message::SetMarkerStorage,
            ),
            text::secondary(fl!("settings-markers-hint")).into(),
        ]
        .into_iter()
        .chain(marker_offer),
    );

    let in_out_storage = Some(settings.in_out_storage);
    let in_out_offer = state.in_out_storage_changed().then(|| {
        let label = match settings.in_out_storage {
            InOutStorage::InVideo => fl!("settings-in-out-move-into-videos"),
            InOutStorage::Comment => fl!("settings-in-out-move-into-comments"),
        };
        move_offer(
            fl!("settings-in-out-note"),
            label,
            Operation::MoveInOut(settings.in_out_storage),
        )
    });
    let in_out = layout::aligned(
        [
            form::radio_option(
                fl!("settings-in-out-in-video"),
                Some(form::description(fl!("settings-in-out-in-video-hint"))),
                InOutStorage::InVideo,
                in_out_storage,
                Message::SetInOutStorage,
            ),
            form::radio_option(
                fl!("settings-in-out-comment"),
                Some(form::example(fl!("settings-in-out-comment-example"))),
                InOutStorage::Comment,
                in_out_storage,
                Message::SetInOutStorage,
            ),
        ]
        .into_iter()
        .chain(in_out_offer),
    );

    layout::page(
        Page::Saving.heading(),
        [
            layout::setting_row(fl!("settings-file-names"), file_names),
            layout::setting_row(fl!("settings-comments"), comments),
            layout::setting_row(fl!("settings-markers"), markers),
            layout::setting_row(fl!("settings-in-out"), in_out),
        ],
    )
}

fn ai(state: &SettingsState) -> Element<'_, Message> {
    let settings = state.settings();
    let key = key_block(
        ApiKey::Anthropic,
        state.key(ApiKey::Anthropic),
        KeyTexts {
            placeholder: fl!("settings-ai-key-placeholder"),
            get_one: fl!("settings-ai-key-get"),
            remove_question: fl!("settings-ai-key-remove-confirm"),
        },
    );
    layout::page(
        Page::Ai.heading(),
        [
            layout::setting_row_with_info(
                fl!("settings-ai-key-label"),
                key_help(state.key(ApiKey::Anthropic), fl!("settings-ai-used-by")),
                key,
            ),
            layout::setting_row(
                fl!("settings-ai-model-label"),
                layout::controls([
                    form::dropdown(
                        MODELS,
                        Some(Model::from_id(&settings.ai_model)),
                        Message::SetAiModel,
                    )
                    .width(FIELD_WIDTH_L)
                    .into(),
                    text::secondary(fl!("settings-ai-hint")).into(),
                ]),
            ),
            layout::setting_row(
                fl!("settings-ai-language-label"),
                form::dropdown(
                    SummaryLanguage::ALL.map(SummaryLanguageOption),
                    Some(SummaryLanguageOption(settings.summary_language)),
                    |option| Message::SetSummaryLanguage(option.0),
                )
                .width(FIELD_WIDTH_L),
            ),
            layout::setting_row(
                fl!("settings-ai-moments-label"),
                layout::aligned([
                    form::radio_option(
                        fl!("settings-ai-moments-important"),
                        Some(form::description(fl!("settings-ai-moments-important-hint"))),
                        MomentsMode::Important,
                        Some(settings.ai_moments),
                        Message::SetAiMoments,
                    ),
                    form::radio_option(
                        fl!("settings-ai-moments-full"),
                        None,
                        MomentsMode::Full,
                        Some(settings.ai_moments),
                        Message::SetAiMoments,
                    ),
                ]),
            ),
        ],
    )
}

fn subtitles(state: &SettingsState) -> Element<'_, Message> {
    let settings = state.settings();
    let key_section = state.key(ApiKey::Soniox);
    let key = key_block(
        ApiKey::Soniox,
        key_section,
        KeyTexts {
            placeholder: fl!("settings-subtitles-key-placeholder"),
            get_one: fl!("settings-subtitles-key-get"),
            remove_question: fl!("settings-subtitles-key-remove-confirm"),
        },
    );
    let list = state.subtitle_languages();
    let languages = &settings.subtitle_languages;
    let checked = |code: &str| languages.iter().any(|l| l == code);
    // Soniox's own list; until it comes (or without a key) the checked codes stay uncheckable.
    let offered: Vec<(String, String)> = match list {
        LanguageList::Listed(all) => all.clone(),
        _ => languages.iter().map(|c| (c.clone(), c.clone())).collect(),
    };
    let grid = Row::with_children(offered.into_iter().map(|(code, name)| {
        let on = checked(&code);
        form::checkbox(name, on)
            .on_toggle(move |on| Message::SetSubtitleLanguage(code.clone(), on))
            .width(CHOICE_WIDTH)
            .into()
    }))
    .spacing(SPACE_S)
    .wrap()
    .vertical_spacing(SPACE_S);
    let status = match list {
        LanguageList::NotAsked if key_section.state == Some(KeyState::Missing) => {
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
    let cue_length = Some(settings.subtitle_cue_length);
    layout::page(
        Page::Subtitles.heading(),
        [
            layout::setting_row_with_info(
                fl!("settings-subtitles-key-label"),
                key_help(key_section, fl!("settings-subtitles-hint")),
                key,
            ),
            layout::setting_row(
                fl!("settings-subtitles-cue-length-label"),
                layout::aligned([
                    form::radio_option(
                        fl!("settings-subtitles-cue-short"),
                        Some(form::description(fl!("settings-subtitles-cue-short-hint"))),
                        CueLength::Short,
                        cue_length,
                        Message::SetSubtitleCueLength,
                    ),
                    form::radio_option(
                        fl!("settings-subtitles-cue-sentence"),
                        None,
                        CueLength::Sentence,
                        cue_length,
                        Message::SetSubtitleCueLength,
                    ),
                ]),
            ),
            layout::setting_row(
                fl!("settings-subtitles-languages-label"),
                layout::aligned(
                    [grid.into()]
                        .into_iter()
                        .chain(status.map(|s| text::secondary(s).into()))
                        .chain([text::secondary(hint).into()]),
                ),
            ),
        ],
    )
}

fn updates_page(state: &SettingsState, batch_running: bool) -> Element<'_, Message> {
    let mut rows = vec![layout::setting_row(
        fl!("settings-version"),
        updates::view::view(state.updates(), batch_running).map(Message::Updates),
    )];
    // Only a package keeps its settings away from the exe; elsewhere they are next to it.
    if state.updates().installed() {
        rows.push(layout::setting_row(
            fl!("settings-old-title"),
            old_settings_import(state.old_settings_import()),
        ));
    }
    layout::page(Page::Updates.heading(), rows)
}

/// The words of a key row that differ between the services.
struct KeyTexts {
    placeholder: String,
    get_one: String,
    remove_question: String,
}

/// The ⓘ of a key row: where Save key keeps a key (while there is one to save), then what the
/// service is used for.
fn key_help(key: &KeySection, used_for: String) -> String {
    if key.state == Some(KeyState::Saved) && !key.replacing {
        used_for
    } else {
        format!(
            "{} {used_for}",
            fl!("settings-key-save-into", store = key_store())
        )
    }
}

/// Where the keys are kept on this OS.
fn key_store() -> String {
    if cfg!(windows) {
        fl!("settings-key-store-windows")
    } else if cfg!(target_os = "macos") {
        fl!("settings-key-store-macos")
    } else {
        fl!("settings-key-store-other")
    }
}

/// An API key: saved (with Replace… and Remove…), asking before removing, or the field to paste
/// one into (§14.3).
fn key_block(which: ApiKey, key: &KeySection, texts: KeyTexts) -> Element<'_, Message> {
    let message = move |m: KeyMessage| Message::Key(which, m);
    let saved = || {
        layout::inline_status(
            NoticeKind::Success,
            fl!("settings-key-saved-in", store = key_store()),
        )
    };
    let mut items: Vec<Element<'_, Message>> = match key.state {
        Some(KeyState::Unavailable) => vec![layout::notice(
            NoticeKind::Warning,
            fl!("settings-key-unavailable"),
            Some(fl!("settings-key-unavailable-hint")),
            [],
        )],
        Some(KeyState::Saved) if key.confirm_remove => vec![
            saved(),
            layout::notice(
                NoticeKind::Error,
                texts.remove_question,
                Some(fl!("settings-key-remove-confirm-hint")),
                [
                    button::danger(fl!("settings-key-remove"))
                        .on_press(message(KeyMessage::Remove))
                        .into(),
                    button::secondary(fl!("settings-key-keep"))
                        .on_press(message(KeyMessage::CancelRemove))
                        .into(),
                ],
            ),
        ],
        Some(KeyState::Saved) if !key.replacing => vec![
            saved(),
            layout::buttons([
                button::secondary(fl!("settings-key-replace"))
                    .on_press(message(KeyMessage::Replace))
                    .into(),
                button::danger_ghost(fl!("settings-key-remove-ask"))
                    .on_press(message(KeyMessage::AskRemove))
                    .into(),
            ]),
        ],
        _ => {
            let save = (!key.input.trim().is_empty()).then_some(message(KeyMessage::Save));
            let show = if key.shown {
                fl!("settings-key-hide")
            } else {
                fl!("settings-key-show")
            };
            let cancel = key.replacing.then(|| {
                button::secondary(fl!("settings-key-cancel"))
                    .on_press(message(KeyMessage::CancelReplace))
                    .into()
            });
            vec![
                row![
                    form::text_field(&texts.placeholder, &key.input)
                        .secure(!key.shown)
                        .on_input(move |input| message(KeyMessage::Input(input)))
                        .on_submit_maybe(save.clone())
                        .width(Length::Fill),
                    button::secondary(show).on_press(message(KeyMessage::ToggleShow)),
                ]
                .spacing(SPACE_S)
                .align_y(Alignment::Center)
                .into(),
                layout::buttons(
                    std::iter::once(
                        button::primary(fl!("settings-key-save"))
                            .on_press_maybe(save)
                            .into(),
                    )
                    .chain(cancel),
                ),
                text::secondary(texts.get_one).into(),
            ]
        }
    };
    if let Some(error) = &key.error {
        items.push(text::error(error.as_str()).into());
    }
    layout::controls(items).into()
}

/// The way back when the first-start search missed the zip version's folder.
fn old_settings_import(import: &OldSettingsImport) -> Element<'_, Message> {
    let note = match import {
        OldSettingsImport::None => None,
        OldSettingsImport::Scheduled(_) => Some(fl!("settings-old-scheduled")),
        OldSettingsImport::NotFound(folder) => Some(fl!(
            "settings-old-not-found",
            folder = folder.display().to_string()
        )),
        OldSettingsImport::Failed(reason) => {
            Some(fl!("settings-old-failed", reason = reason.clone()))
        }
    };
    layout::controls(
        [
            layout::buttons([button::secondary(fl!("settings-old-import-button"))
                .on_press(Message::ImportOldSettings)
                .into()]),
            text::secondary(fl!("settings-old-hint")).into(),
        ]
        .into_iter()
        .chain(note.map(|note| text::body(note).into())),
    )
    .into()
}

/// The tag for videos with a comment, shown while comments are inside the video: a comment inside
/// the file is not visible in Explorer or in the name, the tag is. The checkbox turns tagging off
/// altogether; the name is kept for when it is turned back on.
fn commented_tag(enabled: bool, tag: &str) -> Element<'_, Message> {
    let help = match frename_core::clean_commented_tag(tag).filter(|_| enabled) {
        Some(tag) => fl!("settings-commented-tag-hint", tag = tag),
        None => fl!("settings-commented-tag-off"),
    };
    column![
        form::checkbox(fl!("settings-commented-tag"), enabled)
            .on_toggle(Message::SetCommentedTagEnabled),
        layout::indented(
            row![
                text::body(fl!("settings-commented-tag-name")),
                form::text_field(frename_core::DEFAULT_COMMENTED_TAG, tag)
                    .on_input_maybe(enabled.then_some(Message::SetCommentedTag))
                    .width(FIELD_WIDTH_S),
                layout::info(help),
            ]
            .spacing(SPACE_S)
            .align_y(Alignment::Center),
        ),
    ]
    .spacing(SPACE_S)
    .into()
}

/// Shown after a storage change: the setting only decides where things are saved from now
/// on, so moving what the files already have is a separate batch action, one click away.
fn move_offer(note: String, label: String, operation: Operation) -> Element<'static, Message> {
    layout::notice(
        NoticeKind::Info,
        note,
        None,
        [button::secondary(label)
            .on_press(Message::OpenBatchAction(operation))
            .into()],
    )
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
