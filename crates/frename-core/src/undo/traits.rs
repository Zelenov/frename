use super::{UndoContext, UndoError};
use crate::FileId;

/// Trait implemented by every undoable command. Generic over the two store types used by
/// `UndoContext` (SD for Directory, ST for TagList).
pub trait Undoable<SD, ST>: Send {
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError>;
    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError>;

    /// Whether undoing or redoing it turns a video, so the player must reopen the video to
    /// show it.
    fn turns_a_video(&self) -> bool {
        false
    }

    /// Whether it moves to another file (a navigation), so undoing it is not undoing an edit.
    fn switches_file(&self) -> bool {
        false
    }

    /// Whether redoing it renames a file on disk, which the player may hold open (on Windows,
    /// until its video is unloaded; #602).
    fn redo_renames_file(&self) -> bool {
        false
    }

    /// Whether it only changes the open video (its markers or its turn), which stay editable in
    /// batch mode, unlike tags and the file name.
    fn edits_open_video(&self) -> bool {
        false
    }

    /// The file undoing it leaves with edits that are not in the file on disk: an undone move
    /// renames the clip back to its name from before the move's save, while the clip comes back
    /// with the edits that save applied, to be undone in turn (#611).
    fn undo_leaves_unsaved(&self) -> Option<FileId> {
        None
    }
}

/// Trait for anything that can receive pushed commands.
pub trait CommandSink<SD, ST> {
    fn push(&mut self, cmd: Box<dyn Undoable<SD, ST>>);
}
