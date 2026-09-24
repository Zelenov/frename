//! UI for the settings window.

use frename_core::{CommentStorage, InOutStorage};
use iced::widget::{button, checkbox, column, container, radio, text};
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
    let comments = section(
        "Comments",
        column![
            radio(
                "Inside the video file (XMP, Premiere Pro's Description column)",
                CommentStorage::Xmp,
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
        .spacing(8)
        .into(),
    );

    let selected_in_out = Some(settings.in_out_storage);
    let in_out = section(
        "In/out points",
        column![
            radio(
                "Adobe: a marker inside the video file (XMP, a subclip in Premiere Pro)",
                InOutStorage::Xmp,
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
        FolderConversion::Converting => muted("Converting…".into()).into(),
        FolderConversion::Planned(plan) if plan.is_empty() => {
            muted("All files already use these settings.".into()).into()
        }
        FolderConversion::Planned(plan) => {
            let mut lines = column![].spacing(4);
            if plan.comments > 0 {
                let place = match plan.storage.comment {
                    CommentStorage::Xmp => "in .comment.txt files",
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
            if plan.in_outs > 0 {
                let place = match plan.storage.in_out {
                    InOutStorage::Xmp => "in file names",
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
        FolderConversion::Done(report) => {
            let mut lines = column![muted(format!(
                "Converted {}.",
                count(report.converted, "file", "files")
            ))]
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
