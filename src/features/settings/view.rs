//! UI for the settings window.

use frename_core::{CommentStorage, InOutStorage};
use iced::widget::{
    button, checkbox, column, container, pick_list, radio, row, scrollable, text, text_input,
};
use iced::{Element, Length};

use crate::i18n;
use crate::theme;

use super::{Message, SettingsState};
use crate::features::batch::Operation;

/// Width of the language list.
const LANGUAGE_LIST_WIDTH: f32 = 220.0;

/// An entry of the language list: follow the OS, or one language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LanguageChoice {
    System,
    Fixed(&'static str),
}

impl LanguageChoice {
    fn code(self) -> &'static str {
        match self {
            Self::System => "",
            Self::Fixed(code) => code,
        }
    }
}

impl std::fmt::Display for LanguageChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Language names are in their own language, so a user stuck in one they cannot read
        // still finds theirs; System shows the language it stands for.
        let label = match self {
            Self::System => fl!(
                "settings-language-system",
                language = i18n::language_name(i18n::system_language())
            ),
            Self::Fixed(code) => i18n::language_name(code),
        };
        f.write_str(&label)
    }
}

/// Render the settings window: one titled section per area, one control per setting.
pub fn view(state: &SettingsState) -> Element<'_, Message> {
    let settings = state.settings();

    let choices: Vec<LanguageChoice> = std::iter::once(LanguageChoice::System)
        .chain(i18n::LANGUAGES.into_iter().map(LanguageChoice::Fixed))
        .collect();
    let selected = choices
        .iter()
        .copied()
        .find(|choice| choice.code() == settings.language)
        .unwrap_or(LanguageChoice::System);
    let language = section(
        fl!("settings-language"),
        pick_list(choices, Some(selected), |choice| {
            Message::SetLanguage(choice.code().to_string())
        })
        .text_size(13)
        .width(Length::Fixed(LANGUAGE_LIST_WIDTH))
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
                InOutStorage::FileName => fl!("settings-in-out-move-into-file-names"),
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
    ),]
    .extend(in_out_offer(InOutStorage::InVideo))
    .push(radio(
        fl!("settings-in-out-file-name"),
        InOutStorage::FileName,
        selected_in_out,
        Message::SetInOutStorage,
    ))
    .extend(in_out_offer(InOutStorage::FileName))
    .spacing(8);
    let in_out = section(fl!("settings-in-out"), in_out_options.into());

    // Scrolls when a longer translation or a move offer pushes the last section below the
    // fixed-size window.
    let body = scrollable(
        container(column![language, video, tags, comments, in_out].spacing(20)).padding(20),
    )
    .style(theme::dark_scrollable_style)
    .width(Length::Fill)
    .height(Length::Fill);
    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::main_container_style)
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

fn section(title: String, content: Element<'_, Message>) -> Element<'_, Message> {
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
