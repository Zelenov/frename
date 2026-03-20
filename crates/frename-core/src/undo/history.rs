use crate::db::{AppStateStore, StoredTagStore};

use super::{UndoContext, UndoError};
use super::traits::Undoable;

/// Undo/redo history stack. Owned by the app workspace; one instance per app run.
///
/// SD = Directory store type, ST = TagList store type.
pub struct History<SD, ST> {
    undo_stack: Vec<Box<dyn Undoable<SD, ST>>>,
    redo_stack: Vec<Box<dyn Undoable<SD, ST>>>,
    max_depth: usize,
}

impl<SD, ST> History<SD, ST>
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    /// Push a new command. Clears the redo stack. Drops the oldest entry if max_depth exceeded.
    pub fn push(&mut self, cmd: Box<dyn Undoable<SD, ST>>) {
        self.redo_stack.clear();
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(cmd);
    }

    /// Undo the top command (peek-then-commit): runs undo first; moves to redo stack only on Ok.
    pub fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let cmd = match self.undo_stack.last_mut() {
            Some(c) => c,
            None => return Ok(()),
        };
        cmd.undo(ctx)?;
        let cmd = self.undo_stack.pop().expect("just peeked");
        self.redo_stack.push(cmd);
        Ok(())
    }

    /// Redo the top command (peek-then-commit): runs redo first; moves to undo stack only on Ok.
    pub fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let cmd = match self.redo_stack.last_mut() {
            Some(c) => c,
            None => return Ok(()),
        };
        cmd.redo(ctx)?;
        let cmd = self.redo_stack.pop().expect("just peeked");
        self.undo_stack.push(cmd);
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
