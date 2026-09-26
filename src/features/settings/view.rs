//! UI for the settings window.

use frename_core::ai::key::KeyState;
use frename_core::ai::SummaryLanguage;
use frename_core::{CommentStorage, InOutStorage};
use iced::widget::{
    button, checkbox, column, container, pick_list, radio, row, scrollable, text, text_input,
};
use iced::{Element, Length};

use crate::theme;

use super::state::KeySection;
use super::{Message, SettingsState};

/// The settings' scrollable content, which "Describe with AI" opens scrolled to its end, where
/// the AI section is.
pub const SETTINGS_SCROLLABLE_ID: &str = "settings-content";
use crate::features::batch::Operation;

/// Render the settings window: one titled section per area, one control per setting.
pub fn view(state: &SettingsState) -> Element<'_, Message> {
    let settings = state.settings();

    let video = section(
        "Video",
        checkbox(settings.autoplay_video)
            .label("Play videos automatically when opened")
            .on_toggle(Message::SetAutoplayVideo)
            .into(),
    );
    let mut tag_options = column![
        checkbox(settings.monochrome_tags)
            .label("Monochrome tags")
            .on_toggle(Message::SetMonochromeTags),
        checkbox(settings.space_after_tags)
            .label("Space after each tag in file names (Food. Goat. clip.mp4)")
            .on_toggle(Message::SetSpaceAfterTags),
    ]
    .spacing(8);
    if state.tag_spacing_changed() {
        let label = if settings.space_after_tags {
            "Add the space to existing file names…"
        } else {
            "Remove the space from existing file names…"
        };
        tag_options = tag_options.push(move_offer(
            "Files keep their names until renamed or saved.",
            label,
            Operation::RespaceTags,
        ));
    }
    let tags = section("Tags", tag_options.into());

    // Each option's own settings sit right under it: the tag under "inside the video", and the
    // offer to move existing files under whichever option was just chosen.
    let selected_storage = Some(settings.comment_storage);
    let comment_offer = state.comment_storage_changed().then(|| {
        let label = match settings.comment_storage {
            CommentStorage::InVideo => "Move existing comments from text files into the videos…",
            CommentStorage::TextFile => "Move existing comments from the videos into text files…",
        };
        move_offer(
            "Files keep their comments where they are until moved.",
            label,
            Operation::MoveComments(settings.comment_storage),
        )
    });
    let mut comment_options = column![radio(
        "Inside the video file (XMP, Premiere Pro's Description column)",
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
            "In a .comment.txt file next to the video",
            CommentStorage::TextFile,
            selected_storage,
            Message::SetCommentStorage,
        ))
        .extend(text_file_offer);
    let comments = section("Comments", comment_options.into());

    let selected_in_out = Some(settings.in_out_storage);
    let in_out_offer = |storage: InOutStorage| {
        (state.in_out_storage_changed() && settings.in_out_storage == storage).then(|| {
            let label = match storage {
                InOutStorage::InVideo => {
                    "Move existing in/out points from file names into the videos…"
                }
                InOutStorage::FileName => {
                    "Move existing in/out points from the videos into file names…"
                }
            };
            move_offer(
                "Files keep their in/out points where they are until moved.",
                label,
                Operation::MoveInOut(storage),
            )
        })
    };
    let in_out_options = column![radio(
        "Adobe: a marker inside the video file (XMP, a subclip in Premiere Pro)",
        InOutStorage::InVideo,
        selected_in_out,
        Message::SetInOutStorage,
    ),]
    .extend(in_out_offer(InOutStorage::InVideo))
    .push(radio(
        "In the file name (in_HH_MM_SS / out_HH_MM_SS)",
        InOutStorage::FileName,
        selected_in_out,
        Message::SetInOutStorage,
    ))
    .extend(in_out_offer(InOutStorage::FileName))
    .spacing(8);
    let in_out = section("In/out points", in_out_options.into());

    let ai = section("AI", ai_options(state.key(), settings.summary_language));

    // Five sections do not fit the window: the content scrolls, the window keeps its size.
    let content =
        scrollable(container(column![video, tags, comments, in_out, ai].spacing(20)).padding(20))
            .id(iced::widget::Id::new(SETTINGS_SCROLLABLE_ID))
            .style(theme::dark_scrollable_style)
            .height(Length::Fill);
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::main_container_style)
        .into()
}

/// The AI section: the Anthropic API key and the language of descriptions.
fn ai_options(key: &KeySection, language: SummaryLanguage) -> Element<'_, Message> {
    let muted = |line: &'static str| text(line).size(12).color(theme::TEXT_MUTED);
    let store = if cfg!(windows) {
        "Windows Credential Manager"
    } else if cfg!(target_os = "macos") {
        "the macOS Keychain"
    } else {
        "the system keyring"
    };
    let key_row: Element<'_, Message> = match key.state {
        Some(KeyState::Unavailable) => text("Cannot store the key on this system")
            .size(13)
            .color(theme::ERROR)
            .into(),
        Some(KeyState::Saved) if !key.replacing => row![
            text("Key saved").size(13),
            small_button("Replace", Message::ReplaceKey),
            small_button("Remove", Message::RemoveKey),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .into(),
        _ => {
            let can_save = !key.input.trim().is_empty();
            row![
                text_input("sk-ant-…", &key.input)
                    .secure(!key.shown)
                    .on_input(Message::KeyInput)
                    .on_submit_maybe(can_save.then_some(Message::SaveKey))
                    .size(13)
                    .padding([3, 6])
                    .width(Length::Fixed(220.0)),
                small_button(
                    if key.shown { "Hide" } else { "Show" },
                    Message::ToggleShowKey
                ),
                button(text("Save").size(12))
                    .on_press_maybe(can_save.then_some(Message::SaveKey))
                    .padding([3, 10]),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .into()
        }
    };
    let where_kept = match key.state {
        Some(KeyState::Saved) if !key.replacing => format!("Saved in {store} on this computer."),
        _ => format!("Save keeps it in {store} on this computer."),
    };
    let mut options = column![
        row![text("Anthropic API key").size(13), key_row]
            .spacing(12)
            .align_y(iced::Alignment::Center),
        text(where_kept).size(12).color(theme::TEXT_MUTED),
        muted("Get a key at console.anthropic.com → API keys."),
    ]
    .spacing(6);
    if let Some(error) = &key.error {
        options = options.push(text(error.as_str()).size(12).color(theme::ERROR));
    }
    options
        .push(
            row![
                text("Description language").size(13),
                pick_list(
                    SummaryLanguage::ALL,
                    Some(language),
                    Message::SetSummaryLanguage
                )
                .text_size(13)
                .padding([3, 8]),
            ]
            .spacing(12)
            .align_y(iced::Alignment::Center),
        )
        .push(muted(
            "Used by Describe with AI in batch mode. The model is Claude Haiku 4.5.",
        ))
        .into()
}

fn small_button(label: &str, message: Message) -> Element<'_, Message> {
    button(text(label).size(12))
        .on_press(message)
        .padding([3, 10])
        .style(theme::icon_button_style(true))
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
        Some(tag) => format!(
            "Checked when you write a comment on a video, e.g. {tag}.IMG_0424.MOV, and unchecked when you clear it (AI descriptions do not count). Otherwise it is yours to change."
        ),
        None => "Videos with a comment get no tag.".to_string(),
    };
    column![
        row![
            checkbox(enabled)
                .label("Tag videos with a comment")
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

fn section<'a>(title: &'a str, content: Element<'a, Message>) -> Element<'a, Message> {
    column![text(title).size(14).color(theme::TEXT_MUTED), content]
        .spacing(8)
        .into()
}

/// Shown after a storage change: the setting only decides where things are saved from now
/// on, so moving what the files already have is a separate batch action, one click away.
fn move_offer(
    note: &'static str,
    label: &'static str,
    operation: Operation,
) -> Element<'static, Message> {
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
