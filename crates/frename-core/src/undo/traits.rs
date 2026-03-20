use super::{UndoContext, UndoError};

/// Trait implemented by every undoable command. Generic over the two store types used by
/// `UndoContext` (SD for Directory, ST for TagList).
pub trait Undoable<SD, ST>: Send {
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError>;
    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError>;
}

/// Trait for anything that can receive pushed commands.
pub trait CommandSink<SD, ST> {
    fn push(&mut self, cmd: Box<dyn Undoable<SD, ST>>);
}
