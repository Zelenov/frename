use crate::db::{AppStateStore, StoredTagStore};

use super::traits::Undoable;
use super::{UndoContext, UndoError};

/// Where a step goes when it is only recorded later: taken with [`History::mark`], given back to
/// [`History::push_at_mark`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mark(u64);

/// Undo/redo history stack. Owned by the app workspace; one instance per app run.
///
/// SD = Directory store type, ST = TagList store type.
pub struct History<SD, ST> {
    undo_stack: Vec<Box<dyn Undoable<SD, ST>>>,
    redo_stack: Vec<Box<dyn Undoable<SD, ST>>>,
    max_depth: usize,
    /// How many steps [`Self::push`] has added so far: what [`Self::mark`] reads. It survives
    /// [`Self::clear`], so a mark taken before a clear still counts the steps pushed after it.
    pushed: u64,
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
            pushed: 0,
        }
    }

    /// Push a new command. Clears the redo stack. Drops the oldest entry if max_depth exceeded.
    pub fn push(&mut self, cmd: Box<dyn Undoable<SD, ST>>) {
        self.redo_stack.clear();
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(cmd);
        self.pushed += 1;
    }

    /// The place a step can later be put back at with [`Self::push_at_mark`]: a step whose
    /// result is only known later (a move to another clip, recorded once the clip left is saved)
    /// takes its mark when it happens.
    pub fn mark(&self) -> Mark {
        Mark(self.pushed)
    }

    /// Empty both stacks, as a new history would be, but keep counting pushes: a step waiting
    /// at a mark taken before still goes under the steps pushed since.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Add a step at `mark`, under every step [`Self::push`] has added since, so it is undone
    /// after them and redone before them. Clears the redo stack and keeps `max_depth`, as
    /// `push` does. A mark from before steps were undone is a caller error: those steps are no
    /// longer on the stack, and the step goes to the bottom of what is left at most.
    pub fn push_at_mark(&mut self, mark: Mark, cmd: Box<dyn Undoable<SD, ST>>) {
        self.redo_stack.clear();
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        let newer = usize::try_from(self.pushed.saturating_sub(mark.0)).unwrap_or(usize::MAX);
        let at = self.undo_stack.len().saturating_sub(newer);
        self.undo_stack.insert(at, cmd);
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

    /// Whether the step [`Self::undo`] would take turns a video (see [`Undoable::turns_a_video`]).
    pub fn undo_turns_a_video(&self) -> bool {
        self.undo_stack
            .last()
            .is_some_and(|cmd| cmd.turns_a_video())
    }

    /// Whether the step [`Self::redo`] would take turns a video.
    pub fn redo_turns_a_video(&self) -> bool {
        self.redo_stack
            .last()
            .is_some_and(|cmd| cmd.turns_a_video())
    }

    /// Whether the step [`Self::undo`] would take only changes the open video (see
    /// [`Undoable::edits_open_video`]).
    pub fn undo_edits_open_video(&self) -> bool {
        self.undo_stack
            .last()
            .is_some_and(|cmd| cmd.edits_open_video())
    }

    /// Whether the step [`Self::redo`] would take only changes the open video.
    pub fn redo_edits_open_video(&self) -> bool {
        self.redo_stack
            .last()
            .is_some_and(|cmd| cmd.edits_open_video())
    }

    /// Whether the step [`Self::undo`] would take moves to another file (see
    /// [`Undoable::switches_file`]).
    pub fn undo_switches_file(&self) -> bool {
        self.undo_stack
            .last()
            .is_some_and(|cmd| cmd.switches_file())
    }

    /// Whether the step [`Self::redo`] would take moves to another file.
    pub fn redo_switches_file(&self) -> bool {
        self.redo_stack
            .last()
            .is_some_and(|cmd| cmd.switches_file())
    }
}
