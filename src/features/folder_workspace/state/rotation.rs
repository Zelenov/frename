//! Turning the open video 90° (`Ctrl+Alt+←/→`, `↺`/`↻`): the rotation flag in the file changes
//! at once (see [`FileTagger::rotate_video`]), the player reopens the video to show it, and the
//! turn is one undo step.

use frename_core::{FileTagger, RotateVideoCommand, RotationError};
use iced::Task;

use super::FolderWorkspace;
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
                    Self::notice(&fl!("rotate-done", degrees = i64::from(rotation.degrees()))),
                ])
            }
            Err(error) => Self::notice(&not_rotated(&error)),
        }
    }

    /// After an undo or redo: reopen the video when the file's rotation is no longer the one it
    /// was opened with (the step turned it).
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

    /// Say why an undo or redo of a turn did not happen.
    pub(super) fn rotation_undo_failed(error: &RotationError) -> Task<Message> {
        Self::notice(&not_rotated(error))
    }
}

/// The note for a turn that did not happen.
fn not_rotated(error: &RotationError) -> String {
    match error {
        RotationError::CannotRotate => fl!("rotate-failed-format"),
        RotationError::Damaged => fl!("rotate-failed-damaged"),
        RotationError::NoVideoTrack => fl!("rotate-failed-no-video"),
        RotationError::UnusualMatrix => fl!("rotate-failed-matrix"),
        RotationError::Io(_) => fl!("rotate-failed-in-use"),
    }
}
