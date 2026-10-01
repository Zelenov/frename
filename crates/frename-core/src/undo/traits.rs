use super::{UndoContext, UndoError};

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

    /// Whether it only changes the open video (its markers or its turn), which stay editable in
    /// batch mode, unlike tags and the file name.
    fn edits_open_video(&self) -> bool {
        false
    }
}

/// Trait for anything that can receive pushed commands.
pub trait CommandSink<SD, ST> {
    fn push(&mut self, cmd: Box<dyn Undoable<SD, ST>>);
}
