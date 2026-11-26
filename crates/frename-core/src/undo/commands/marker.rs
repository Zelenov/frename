//! Undo commands for the clip markers of the open file. Markers are matched by GUID, both here
//! and when they are written into the file, so undoing and redoing can never duplicate one.
//!
//! A change by the editor makes an AI marker theirs ([`Marker::ai`] goes): every redo goes
//! through [`crate::TagList::edit_marker`]. Each command that changes a marker keeps whether it
//! was the AI's (`was_ai`), so an undo makes it the AI's again.

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
    /// Whether the marker was the AI's before the change.
    pub was_ai: bool,
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
        let (old, ai) = (self.old, self.was_ai);
        ctx.tag_list
            .update_marker(&self.guid, |m| (m.color, m.ai) = (old, ai));
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let new = self.new;
        ctx.tag_list.edit_marker(&self.guid, |m| m.color = new);
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
    /// Whether the marker was the AI's before the change.
    pub was_ai: bool,
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
        set_span(ctx, &self.guid, self.old, self.was_ai);
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let (start, duration) = self.new;
        ctx.tag_list.edit_marker(&self.guid, |m| {
            (m.start_ms, m.duration_ms) = (start, duration)
        });
        Ok(())
    }
}

fn set_span<SD, ST>(
    ctx: &mut UndoContext<'_, SD, ST>,
    guid: &str,
    (start, duration): (u64, u64),
    ai: bool,
) where
    ST: StoredTagStore + Clone,
{
    ctx.tag_list.update_marker(guid, |m| {
        m.start_ms = start;
        m.duration_ms = duration;
        m.ai = ai;
    });
}

/// Records editing a marker's name: one step per row open-to-close, not per key.
pub struct SetMarkerNameCommand {
    pub guid: String,
    pub old: String,
    pub new: String,
    /// Whether the marker was the AI's before the change.
    pub was_ai: bool,
}

impl SetMarkerNameCommand {
    fn set<SD, ST>(&self, ctx: &mut UndoContext<'_, SD, ST>, name: &str, ai: bool)
    where
        ST: StoredTagStore + Clone,
    {
        let name = name.to_string();
        if ai {
            ctx.tag_list
                .update_marker(&self.guid, |m| (m.name, m.ai) = (name, true));
        } else {
            ctx.tag_list.edit_marker(&self.guid, |m| m.name = name);
        }
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
        self.set(ctx, &self.old, self.was_ai);
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.set(ctx, &self.new, false);
        Ok(())
    }
}

/// Records changing a marker's name and comment together: "Describe with AI" on a marker fills
/// both in one step.
pub struct SetMarkerTextCommand {
    pub guid: String,
    /// `(name, comment)` before the change.
    pub old: (String, String),
    /// `(name, comment)` after it.
    pub new: (String, String),
    /// Whether the marker was the AI's before the change.
    pub was_ai: bool,
}

impl SetMarkerTextCommand {
    fn set<SD, ST>(
        &self,
        ctx: &mut UndoContext<'_, SD, ST>,
        (name, comment): &(String, String),
        ai: bool,
    ) where
        ST: StoredTagStore + Clone,
    {
        let text = |m: &mut Marker| (m.name, m.comment) = (name.clone(), comment.clone());
        if ai {
            ctx.tag_list.update_marker(&self.guid, |m| {
                text(m);
                m.ai = true;
            });
        } else {
            ctx.tag_list.edit_marker(&self.guid, text);
        }
    }
}

impl<SD, ST> Undoable<SD, ST> for SetMarkerTextCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn edits_open_video(&self) -> bool {
        true
    }

    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.set(ctx, &self.old, self.was_ai);
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.set(ctx, &self.new, false);
        Ok(())
    }
}
