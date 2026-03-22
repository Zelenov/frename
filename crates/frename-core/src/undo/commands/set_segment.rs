use crate::db::{AppStateStore, StoredTagStore};
use super::super::{UndoContext, UndoError};
use super::super::traits::Undoable;

/// Records setting or clearing the segment start marker on the current file.
pub struct SetSegmentStartCommand {
    pub old_secs: Option<f32>,
    pub new_secs: Option<f32>,
}

impl<SD, ST> Undoable<SD, ST> for SetSegmentStartCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list.set_segment_start_secs(self.old_secs);
        Ok(())
    }
    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list.set_segment_start_secs(self.new_secs);
        Ok(())
    }
}

/// Records setting or clearing the segment end marker on the current file.
pub struct SetSegmentEndCommand {
    pub old_secs: Option<f32>,
    pub new_secs: Option<f32>,
}

impl<SD, ST> Undoable<SD, ST> for SetSegmentEndCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list.set_segment_end_secs(self.old_secs);
        Ok(())
    }
    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list.set_segment_end_secs(self.new_secs);
        Ok(())
    }
}
