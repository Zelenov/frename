//! Turning the open video 90° (`Ctrl+Alt+←/→`, `↺`/`↻`): the rotation flag in the file changes
//! at once (see [`FileTagger::rotate_video`]), the player reopens the video to show it, and the
//! turn is one undo step.

use frename_core::{FileTagger, RotateVideoCommand, Rotation, UndoError};
use iced::Task;

use super::FolderWorkspace;
use crate::features::batch::rotate::why_not_rotated;
use crate::features::folder_workspace::Message;

impl FolderWorkspace {
    /// Turn the open file by `quarter_turns` clockwise (negative: counter-clockwise).
    pub(super) fn rotate_video(&mut self, quarter_turns: i32) -> Task<Message> {
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

    /// After an undo or redo that turned a video: reopen the shown video when the file's
    /// rotation is no longer the one it was opened with.
    pub(super) fn follow_rotation(&mut self) -> Task<Message> {
        let (Some(shown), Some(file)) = (
            self.media_viewer.video_rotation(),
            self.file_workspace.file(),
        ) else {
            return Task::none();
        };
        match FileTagger::video_rotation(file.file_path()) {
            Ok(now) if now != shown => self.media_viewer.reload_video().map(Message::MediaViewer),
            _ => Task::none(),
        }
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

/// The note after a turn: how the clip is turned now.
fn rotated(rotation: Rotation) -> String {
    match rotation.degrees() {
        90 => fl!("rotate-now-right"),
        180 => fl!("rotate-now-half"),
        270 => fl!("rotate-now-left"),
        _ => fl!("rotate-now-upright"),
    }
}
