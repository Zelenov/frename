//! Turning the open video 90° (`Ctrl+Alt+←/→`, `↺`/`↻`): the rotation flag in the file changes
//! at once (see [`FileTagger::rotate_video`]), the player reopens the video to show it, and the
//! turn is one undo step.

use frename_core::{FileTagger, RotateVideoCommand, UndoError};
use iced::Task;

use super::FolderWorkspace;
use crate::features::file_workspace::view::COMMENT_EDITOR_ID;
use crate::features::folder_workspace::Message;
use crate::features::rotation_text::{rotated, why_not_rotated};

impl FolderWorkspace {
    /// Turn the open video by `quarter_turns` clockwise (negative: counter-clockwise). Nothing
    /// happens when no video is shown: the note would go to a player that is not there.
    pub(super) fn rotate_video(&mut self, quarter_turns: i32) -> Task<Message> {
        if !self.media_viewer.is_previewable() {
            return Task::none();
        }
        let Some(file) = self.file_workspace.file() else {
            return Task::none();
        };
        let (id, path) = (file.id(), file.file_path().to_path_buf());
        match FileTagger::rotate_video(&path, quarter_turns) {
            Ok(rotation) => {
                self.history.push(Box::new(RotateVideoCommand {
                    file: id,
                    path,
                    quarter_turns,
                }));
                Task::batch([
                    self.media_viewer.reload_video().map(Message::MediaViewer),
                    Self::notice(&rotated(rotation)),
                ])
            }
            Err(error) => Self::notice(&fl!("rotate-failed", reason = why_not_rotated(&error))),
        }
    }

    /// `Ctrl+Alt+←/→` pressed while a text field had the keys: turn the video, unless the field
    /// is the comment box, where the keys belong to the text (the search fields hold nothing
    /// the keys would do, so there they still turn the video).
    pub(super) fn rotate_video_unless_writing(&self, quarter_turns: i32) -> Task<Message> {
        iced::widget::operation::is_focused(iced::widget::Id::new(COMMENT_EDITOR_ID)).map(
            move |writing| {
                if writing {
                    Message::Noop
                } else {
                    Message::RotateVideo(quarter_turns)
                }
            },
        )
    }

    /// After an undo or redo step that turned a video: reopen the shown video and say how it is
    /// turned now. Always reopen, not only when its rotation looks different: a reopen still
    /// loading from the turn being undone would otherwise land and show the turn the file no
    /// longer has.
    pub(super) fn follow_rotation(&mut self) -> Task<Message> {
        let note = self
            .file_workspace
            .file()
            .and_then(|file| FileTagger::video_rotation(file.file_path()).ok())
            .map_or_else(Task::none, |rotation| Self::notice(&rotated(rotation)));
        Task::batch([
            self.media_viewer.reload_video().map(Message::MediaViewer),
            note,
        ])
    }

    /// Say why an undo or redo step did not happen, when the user can do something about it:
    /// a turn of a file that is read-only or open in another app.
    pub(super) fn undo_failed_notice(error: &UndoError) -> Task<Message> {
        match error {
            UndoError::Rotation(_, error) => {
                Self::notice(&fl!("rotate-failed", reason = why_not_rotated(error)))
            }
            _ => Task::none(),
        }
    }
}
