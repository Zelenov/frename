//! UI for the settings window.

use frename_core::{CommentStorage, CueLength, InOutStorage, SUBTITLE_LANGUAGES};
use iced::widget::{
    button, checkbox, column, container, radio, row, scrollable, text, text_input, Space,
};
use iced::{Element, Length};

use crate::theme;

use super::{Message, SettingsState};
use crate::features::batch::Operation;
use crate::soniox_key::{self, KeySource, TypedKey};

/// The window's scrollable content; the Subtitles section is last, so "Open Settings" from the
/// subtitle action scrolls it to the end.
pub const SETTINGS_SCROLLABLE_ID: &str = "settings_scrollable";

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

    let subtitles = section("Subtitles", subtitles(state));

    // Five sections do not fit the fixed window: its content scrolls.
    let content = column![video, tags, comments, in_out, subtitles]
        .spacing(20)
        .padding(20);
    container(
        scrollable(content)
            .id(iced::widget::Id::new(SETTINGS_SCROLLABLE_ID))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::dark_scrollable_style),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::main_container_style)
    .into()
}

/// Generating subtitles with Soniox: the API key, the languages spoken in the footage, and
/// how long a cue may get.
fn subtitles(state: &SettingsState) -> Element<'_, Message> {
    let key = state.soniox_key();
    let muted = |line: String| text(line).size(12).color(theme::TEXT_MUTED);

    let mut key_block = column![text("Soniox API key").size(13)].spacing(6);
    match key.info() {
        None => key_block = key_block.push(muted("Reading…".to_string())),
        Some(info) => {
            if key.editing() {
                let mut input = text_input("Paste the key", key.input())
                    .size(13)
                    .padding([3, 6])
                    .secure(!key.show())
                    .width(Length::Fixed(260.0));
                if !key.busy() {
                    input = input
                        .on_input(|typed| Message::SonioxKeyInput(TypedKey(typed)))
                        .on_submit(Message::SaveSonioxKey);
                }
                let can_save = !key.busy() && !key.input().trim().is_empty();
                key_block = key_block.push(
                    row![
                        input,
                        button(text(if key.show() { "Hide" } else { "Show" }).size(12))
                            .on_press(Message::ToggleShowSonioxKey)
                            .padding([3, 10])
                            .style(theme::icon_button_style(true)),
                        button(text("Save").size(13))
                            .on_press_maybe(can_save.then_some(Message::SaveSonioxKey))
                            .padding([3, 12]),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                );
            } else {
                key_block = key_block.push(
                    row![
                        text("Key saved").size(13),
                        button(text("Replace").size(12))
                            .on_press(Message::ReplaceSonioxKey)
                            .padding([3, 10])
                            .style(theme::icon_button_style(true)),
                        button(text("Remove").size(12))
                            .on_press_maybe((!key.busy()).then_some(Message::RemoveSonioxKey))
                            .padding([3, 10])
                            .style(theme::icon_button_style(!key.busy())),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                );
            }
            if let Some(error) = key.error() {
                key_block = key_block.push(text(error).size(12).color(theme::ERROR));
            }
            let place = if info.store_available {
                format!("Saved in {} on this computer.", soniox_key::store_name())
            } else {
                "Cannot store the key on this system.".to_string()
            };
            key_block = key_block.push(muted(place));
            if matches!(info.key, Some((_, KeySource::Environment))) && key.editing() {
                key_block = key_block.push(muted(
                    "Using the key from the SONIOX_API_KEY environment variable.".to_string(),
                ));
            }
        }
    }
    key_block = key_block
        .push(muted("Get a key at console.soniox.com.".to_string()))
        .push(muted(
            "The audio is sent to Soniox to transcribe it.".to_string(),
        ));

    let settings = state.settings();
    let checked = |code: &str| settings.subtitle_languages.iter().any(|l| l == code);
    let language_rows = SUBTITLE_LANGUAGES.chunks(3).map(|chunk| {
        row(chunk.iter().map(|(code, name)| {
            let code = code.to_string();
            checkbox(checked(&code))
                .label(*name)
                .text_size(13)
                .on_toggle(move |on| Message::SetSubtitleLanguage(code.clone(), on))
                .width(Length::Fixed(120.0))
                .into()
        }))
        .spacing(8)
        .into()
    });
    let hint = if settings.subtitle_languages.is_empty() {
        "None checked: Detect automatically."
    } else {
        "The languages spoken in the footage, as hints."
    };
    let languages = column![text("Languages").size(13)]
        .extend(language_rows)
        .push(muted(hint.to_string()))
        .spacing(6);

    let selected = Some(settings.subtitle_cue_length);
    let cue_length = column![
        text("Cue length").size(13),
        radio(
            "Short: one line of up to 100 characters, at most 8 s",
            CueLength::Short,
            selected,
            Message::SetSubtitleCueLength,
        )
        .text_size(13),
        radio(
            "One sentence per cue",
            CueLength::Sentence,
            selected,
            Message::SetSubtitleCueLength,
        )
        .text_size(13),
        muted("Applies to new subtitles; changing it later means transcribing again.".to_string()),
    ]
    .spacing(6);

    column![key_block, languages, cue_length, Space::new().height(8)]
        .spacing(14)
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
            "Checked when a video gets a comment, e.g. {tag}.IMG_0424.MOV, and unchecked when the comment is cleared.              Otherwise it is yours to change."
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
