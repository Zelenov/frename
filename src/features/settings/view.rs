//! UI for the settings window.

use frename_core::{CommentStorage, InOutStorage};
use iced::widget::{button, checkbox, column, container, radio, row, scrollable, text, text_input};
use iced::{Element, Length};

use crate::theme;

use super::state::OldSettingsImport;
use super::{Message, SettingsState};
use crate::features::batch::Operation;
use crate::features::updates;

/// Render the settings window: one titled section per area, one control per setting.
/// `batch_running` holds back **Update and restart** while a batch job writes files.
pub fn view(state: &SettingsState, batch_running: bool) -> Element<'_, Message> {
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

    let updates = section(
        "Updates",
        updates::view::view(state.updates(), batch_running).map(Message::Updates),
    );
    let mut sections = column![video, tags, comments, in_out, updates].spacing(20);
    // Only a package keeps its settings away from the exe; elsewhere they are next to it.
    if state.updates().installed() {
        sections = sections.push(section(
            "Settings from an older frename",
            old_settings_import(state.old_settings_import()),
        ));
    }

    // The window is not resizable: whatever does not fit scrolls.
    container(scrollable(container(sections).padding(20)).style(theme::dark_scrollable_style))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::main_container_style)
        .into()
}

/// The way back when the first-start search missed the zip version's folder.
fn old_settings_import(import: &OldSettingsImport) -> Element<'_, Message> {
    let note = match import {
        OldSettingsImport::None => String::new(),
        OldSettingsImport::Scheduled(_) => {
            "Settings will be imported when frename restarts".to_string()
        }
        OldSettingsImport::NotFound(folder) => {
            format!("No frename.exe with a frename.db in {}", folder.display())
        }
        OldSettingsImport::Failed(reason) => format!("Could not import: {reason}"),
    };
    row![
        button(text("Import from an old frename folder…").size(13))
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
