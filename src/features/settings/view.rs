//! UI for the settings window.

use frename_core::{CommentStorage, InOutStorage};
use iced::widget::{button, checkbox, column, container, progress_bar, radio, row, text, text_input};
use iced::{Element, Length};

use crate::theme;

use super::{FolderConversion, Message, SettingsState};

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
    let tags = section(
        "Tags",
        checkbox(settings.monochrome_tags)
            .label("Monochrome tags")
            .on_toggle(Message::SetMonochromeTags)
            .into(),
    );

    let selected_storage = Some(settings.comment_storage);
    let mut comment_options = column![
        radio(
            "Inside the video file (XMP, Premiere Pro's Description column)",
            CommentStorage::InVideo,
            selected_storage,
            Message::SetCommentStorage,
        ),
        radio(
            "In a .comment.txt file next to the video",
            CommentStorage::TextFile,
            selected_storage,
            Message::SetCommentStorage,
        ),
    ]
    .spacing(8);
    if settings.comment_storage == CommentStorage::InVideo {
        comment_options = comment_options.push(commented_tag(&settings.commented_tag));
    }
    let comments = section("Comments", comment_options.into());

    let selected_in_out = Some(settings.in_out_storage);
    let in_out = section(
        "In/out points",
        column![
            radio(
                "Adobe: a marker inside the video file (XMP, a subclip in Premiere Pro)",
                InOutStorage::InVideo,
                selected_in_out,
                Message::SetInOutStorage,
            ),
            radio(
                "In the file name (in_HH_MM_SS / out_HH_MM_SS)",
                InOutStorage::FileName,
                selected_in_out,
                Message::SetInOutStorage,
            ),
        ]
        .spacing(8)
        .into(),
    );

    let folder = section("Current folder", folder_conversion(state));

    container(column![video, tags, comments, in_out, folder].spacing(20))
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::main_container_style)
        .into()
}

/// The tag for videos with a comment, shown while comments are inside the video: a comment inside the
/// file is not visible in Explorer or in the name, the tag is.
fn commented_tag(tag: &str) -> Element<'_, Message> {
    let input = text_input(frename_core::DEFAULT_COMMENTED_TAG, tag)
        .on_input(Message::SetCommentedTag)
        .size(13)
        .padding([3, 6])
        .width(Length::Fixed(160.0));
    let hint = match frename_core::clean_commented_tag(tag) {
        Some(tag) => format!(
            "Added to the name of every video with a comment, e.g. {tag}.IMG_0424.MOV, and removed when the comment is cleared."
        ),
        None => "Empty: videos with a comment get no tag.".to_string(),
    };
    column![
        row![text("Tag for videos with a comment").size(13), input]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        text(hint).size(12).color(theme::TEXT_MUTED),
    ]
    .spacing(4)
    .padding(iced::Padding { top: 0.0, right: 0.0, bottom: 0.0, left: 26.0 })
    .into()
}

fn section<'a>(title: &'a str, content: Element<'a, Message>) -> Element<'a, Message> {
    column![text(title).size(14).color(theme::TEXT_MUTED), content]
        .spacing(8)
        .into()
}

/// "5 files" / "1 file".
fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Where the open folder stands against the storage settings, and the button to convert it.
fn folder_conversion(state: &SettingsState) -> Element<'_, Message> {
    let muted = |line: String| text(line).size(13).color(theme::TEXT_MUTED);
    match state.conversion() {
        FolderConversion::NoFolder => muted("Open a folder to convert its files.".into()).into(),
        FolderConversion::Checking => muted("Checking the files…".into()).into(),
        FolderConversion::Converting { done, total, stopping } => {
            let status = if *stopping {
                format!("Stopping after the current files… {done} / {total}")
            } else {
                format!("Converting… {done} / {total}")
            };
            let mut cancel = button(text("Cancel").size(13));
            if !*stopping {
                cancel = cancel.on_press(Message::CancelConversion);
            }
            column![
                text(status).size(13),
                progress_bar(0.0..=(*total).max(1) as f32, *done as f32).girth(6),
                cancel,
            ]
            .spacing(8)
            .into()
        }
        FolderConversion::Planned(plan) if plan.is_empty() => {
            muted("All files already use these settings.".into()).into()
        }
        FolderConversion::Planned(plan) => {
            let mut lines = column![].spacing(4);
            if plan.comments > 0 {
                let place = match plan.storage.comment {
                    CommentStorage::InVideo => "in .comment.txt files",
                    CommentStorage::TextFile => "inside video files (XMP)",
                };
                lines = lines.push(
                    text(format!(
                        "{} {place}",
                        count(plan.comments, "comment is", "comments are")
                    ))
                    .size(13),
                );
            }
            if plan.commented_tags > 0 {
                let tag = plan.commented_tag.as_deref().unwrap_or_default();
                lines = lines.push(
                    text(format!(
                        "{} the \u{201c}{tag}\u{201d} tag added or removed; {} renamed",
                        count(plan.commented_tags, "file needs", "files need"),
                        count(plan.commented_tags, "file will be", "files will be"),
                    ))
                    .size(13),
                );
            }
            if plan.in_outs > 0 {
                let place = match plan.storage.in_out {
                    InOutStorage::InVideo => "in file names",
                    InOutStorage::FileName => "in XMP markers",
                };
                lines = lines.push(
                    text(format!(
                        "{} {place}; {} renamed",
                        count(plan.in_outs, "in/out point is", "in/out points are"),
                        count(plan.in_outs, "file will be", "files will be"),
                    ))
                    .size(13),
                );
            }
            column![
                lines,
                button(
                    text(format!(
                        "Convert {}",
                        count(plan.files.len(), "file", "files")
                    ))
                    .size(13)
                )
                .on_press(Message::ConvertFolder),
            ]
            .spacing(8)
            .into()
        }
        FolderConversion::Done { report, cancelled } => {
            let summary = if *cancelled {
                format!("Cancelled. Converted {} before stopping.", count(report.converted, "file", "files"))
            } else {
                format!("Converted {}.", count(report.converted, "file", "files"))
            };
            let mut lines = column![muted(summary)]
            .spacing(4);
            if !report.not_converted.is_empty() {
                lines = lines.push(
                    text(format!(
                        "{} could not be converted (the format cannot hold XMP, or the file could not be written; see the log).",
                        count(report.not_converted.len(), "file", "files"),
                    ))
                    .size(13),
                );
            }
            lines.into()
        }
    }
}
