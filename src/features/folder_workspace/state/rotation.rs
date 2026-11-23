//! Turning the open video 90° (`Ctrl+Alt+←/→`, `↺`/`↻`): the rotation flag in the file changes
//! at once (see [`FileTagger::rotate_video`]), the player reopens the video to show it, and the
//! turn is one undo step.

use frename_core::{FileTagger, RotateVideoCommand, UndoError};
use iced::Task;

use super::FolderWorkspace;
use crate::features::folder_workspace::Message;
use crate::features::rotation_text::{not_rotated, rotated, turned};

impl FolderWorkspace {
    /// Turn the open video by `quarter_turns` clockwise (negative: counter-clockwise). Nothing
    /// happens when no video is shown (the note would go to a player that is not there), or
    /// while the player is closing a file to save it (a reopen then would race that save).
    pub(super) fn rotate_video(&mut self, quarter_turns: i32) -> Task<Message> {
        if !self.media_viewer.is_previewable() || !self.pending_file_updates.is_empty() {
            return Task::none();
        }
        let (Some(id), Some(path)) = (
            self.file_workspace.file().map(|f| f.id()),
            self.open_clip_path(),
        ) else {
            return Task::none();
        };
        match FileTagger::rotate_video(&path, quarter_turns) {
            Ok(rotation) => {
                self.history.push(Box::new(RotateVideoCommand {
                    file: id,
                    path,
                    quarter_turns,
                }));
                Task::batch([
                    self.media_viewer.reload_video().map(Message::MediaViewer),
                    Self::notice(&turned(quarter_turns, rotation)),
                ])
            }
            Err(error) => Self::notice(&not_rotated(&error)),
        }
    }

    /// After an undo or redo step that turned a video: reopen the shown video and say how it is
    /// turned now. Always reopen, not only when its rotation looks different: a reopen still
    /// loading from the turn being undone would otherwise land and show the turn the file no
    /// longer has.
    ///
    /// The turned file is the open one: a turn is only made on the open file, and the steps
    /// that switch files are undone and redone around it, so the file is open again by then.
    pub(super) fn follow_rotation(&mut self) -> Task<Message> {
        if !self.pending_file_updates.is_empty() {
            return Task::none();
        }
        let note = self
            .open_clip_path()
            .and_then(|path| FileTagger::video_rotation(&path).ok())
            .map_or_else(Task::none, |rotation| Self::notice(&rotated(rotation)));
        Task::batch([
            self.media_viewer.reload_video().map(Message::MediaViewer),
            note,
        ])
    }

    /// Say why an undo or redo step did not happen, when the user can do something about it:
    /// a turn of a file that is read-only or open in another app, a rename the system refused
    /// (the file is in use or read-only), or a rename that would replace another file or leave a
    /// sidecar behind under a name that is too long, or a step whose file is gone.
    pub(super) fn undo_failed_notice(error: &UndoError) -> Task<Message> {
        match error {
            UndoError::Rotation(_, error) => Self::notice(&not_rotated(error)),
            UndoError::NameTaken(path) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                Self::notice(&fl!("undo-name-taken", name = name))
            }
            UndoError::NameTooLong(_) => Self::notice(&fl!("undo-name-too-long")),
            // The clip was renamed, moved or deleted outside frename since the step (#612).
            UndoError::FileNotFound(path) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                Self::notice(&fl!("undo-file-gone", name = name))
            }
            // A rename refused by the system: the clip is open in another app (Premiere) or
            // read-only (#602). Other I/O errors (a database, a lost drive) say nothing of a file
            // in use, so they stay in the log.
            UndoError::Io(e) if refused_by_the_system(e) => Self::notice(&fl!("undo-file-in-use")),
            _ => Task::none(),
        }
    }
}

/// A file the system would not change: read-only, or held by another program (Windows' sharing
/// and lock violations, 32 and 33).
fn refused_by_the_system(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ResourceBusy
    ) || (cfg!(windows) && matches!(error.raw_os_error(), Some(32 | 33)))
}
