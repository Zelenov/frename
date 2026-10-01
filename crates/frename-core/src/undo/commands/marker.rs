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
    fn edits_open_video(&self) -> bool {
        true
    }

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
    fn edits_open_video(&self) -> bool {
        true
    }

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
    fn edits_open_video(&self) -> bool {
        true
    }

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

/// Records moving a marker's start and end together: a drag of a range's handle, or a range
/// turned back into a point.
pub struct SetMarkerSpanCommand {
    pub guid: String,
    /// `(start_ms, duration_ms)` before the change.
    pub old: (u64, u64),
    /// `(start_ms, duration_ms)` after it.
    pub new: (u64, u64),
}

impl<SD, ST> Undoable<SD, ST> for SetMarkerSpanCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn edits_open_video(&self) -> bool {
        true
    }

    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        set_span(ctx, &self.guid, self.old);
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        set_span(ctx, &self.guid, self.new);
        Ok(())
    }
}

fn set_span<SD, ST>(ctx: &mut UndoContext<'_, SD, ST>, guid: &str, (start, duration): (u64, u64))
where
    ST: StoredTagStore + Clone,
{
    ctx.tag_list.update_marker(guid, |m| {
        m.start_ms = start;
        m.duration_ms = duration;
    });
}

/// Records editing a marker's name: one step per row open-to-close, not per key.
pub struct SetMarkerNameCommand {
    pub guid: String,
    pub old: String,
    pub new: String,
}

impl SetMarkerNameCommand {
    fn set<SD, ST>(&self, ctx: &mut UndoContext<'_, SD, ST>, name: &str)
    where
        ST: StoredTagStore + Clone,
    {
        ctx.tag_list
            .update_marker(&self.guid, |m| m.name = name.to_string());
    }
}

impl<SD, ST> Undoable<SD, ST> for SetMarkerNameCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn edits_open_video(&self) -> bool {
        true
    }

    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.set(ctx, &self.old);
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.set(ctx, &self.new);
        Ok(())
    }
}
