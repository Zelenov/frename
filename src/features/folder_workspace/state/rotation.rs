//! Turning the open video 90° (`Ctrl+Alt+←/→`, `↺`/`↻`): the rotation flag in the file changes
//! at once (see [`FileTagger::rotate_video`]), the player reopens the video to show it, and the
//! turn is one undo step.

use frename_core::{FileTagger, RotateVideoCommand, UndoError};
use iced::Task;

use super::FolderWorkspace;
use crate::features::folder_workspace::Message;
use crate::features::rotation_text::{rotated, why_not_rotated};

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

    /// After an undo or redo step that turned a video: reopen the shown video. Always, not only
    /// when its rotation looks different: a reopen still loading from the turn being undone
    /// would otherwise land and show the turn the file no longer has.
    pub(super) fn follow_rotation(&mut self) -> Task<Message> {
        self.media_viewer.reload_video().map(Message::MediaViewer)
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
