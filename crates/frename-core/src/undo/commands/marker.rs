//! Undo commands for the clip markers of the open file. Markers are matched by GUID, both here
//! and when they are written into the file, so undoing and redoing can never duplicate one.

use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::markers::{Marker, MarkerColor};

/// Records adding a marker. Undo takes the marker as it is then, so redo brings back the name
/// and comment typed after adding it.
pub struct AddMarkerCommand {
    pub marker: Marker,
}

impl<SD, ST> Undoable<SD, ST> for AddMarkerCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        if let Some(removed) = remove(ctx, &self.marker) {
            self.marker = removed;
        }
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list.add_marker(self.marker.clone());
        Ok(())
    }
}

/// Records deleting a marker.
pub struct DeleteMarkerCommand {
    pub marker: Marker,
}

impl<SD, ST> Undoable<SD, ST> for DeleteMarkerCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list.add_marker(self.marker.clone());
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        if let Some(removed) = remove(ctx, &self.marker) {
            self.marker = removed;
        }
        Ok(())
    }
}

fn remove<SD, ST>(ctx: &mut UndoContext<'_, SD, ST>, marker: &Marker) -> Option<Marker>
where
    ST: StoredTagStore + Clone,
{
    ctx.tag_list.remove_marker(marker.guid.as_deref()?)
}

/// Records a marker's color change.
pub struct SetMarkerColorCommand {
    pub guid: String,
    pub old: MarkerColor,
    pub new: MarkerColor,
}

impl<SD, ST> Undoable<SD, ST> for SetMarkerColorCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let old = self.old;
        ctx.tag_list.update_marker(&self.guid, |m| m.color = old);
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let new = self.new;
        ctx.tag_list.update_marker(&self.guid, |m| m.color = new);
        Ok(())
    }
}

/// Records a marker's duration change (its end set to the playhead, or cleared).
pub struct SetMarkerDurationCommand {
    pub guid: String,
    pub old_ms: u64,
    pub new_ms: u64,
}

impl<SD, ST> Undoable<SD, ST> for SetMarkerDurationCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let old = self.old_ms;
        ctx.tag_list
            .update_marker(&self.guid, |m| m.duration_ms = old);
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let new = self.new_ms;
        ctx.tag_list
            .update_marker(&self.guid, |m| m.duration_ms = new);
        Ok(())
    }
}
