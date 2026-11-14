//! Undo of a restore after a crash (#156): the restore writes the journaled edits into the clip
//! at start, before any clip is open, so no edit command of the run that crashed is left to
//! undo. This one step puts the clip back as it was on disk before the restore (as unsaved
//! edits, saved like any other when the clip is left), and redo brings the restored edits back.

use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::FileSnapshot;

/// Records the restore of one clip's journaled edits.
pub struct RestoreCommand {
    /// The clip as it was on disk before the restore, as the open clip's state.
    before: FileSnapshot,
    /// The clip as the restore left it (the open clip's state when it was opened).
    after: FileSnapshot,
}

impl RestoreCommand {
    /// The step for a clip opened right after its restore. `on_disk_before` is the clip before
    /// the restore ([`crate::recovery::Restored::Applied`]); `restored` is the open clip's
    /// state now. The clip's name on disk is the restored one, so undoing renames it back
    /// from there. `None` when the restore changed nothing the editor can see.
    pub fn new(on_disk_before: &FileSnapshot, restored: FileSnapshot) -> Option<Self> {
        let mut before = FileSnapshot::new(
            on_disk_before.tags().to_vec(),
            on_disk_before.name_without_extension(),
            on_disk_before.extension(),
            restored.initial_file_name(),
        );
        before.set_comment(on_disk_before.comment().to_string());
        before.set_segment_start(on_disk_before.segment_start());
        before.set_segment_end(on_disk_before.segment_end());
        before.set_markers(on_disk_before.markers().map(<[_]>::to_vec));
        let same = before.tags() == restored.tags()
            && before.name_without_extension() == restored.name_without_extension()
            && before.extension() == restored.extension()
            && before.comment() == restored.comment()
            && before.segment_start() == restored.segment_start()
            && before.segment_end() == restored.segment_end()
            && before.markers() == restored.markers();
        (!same).then_some(Self {
            before,
            after: restored,
        })
    }

    /// The clip as it was before the restore.
    pub fn before(&self) -> &FileSnapshot {
        &self.before
    }

    fn show<ST: StoredTagStore + Clone>(tag_list: &mut crate::TagList<ST>, state: &FileSnapshot) {
        tag_list.reinitialize_from_snapshot(state.clone());
        tag_list.set_markers(state.markers().map(<[_]>::to_vec));
    }
}

impl<SD, ST> Undoable<SD, ST> for RestoreCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        Self::show(ctx.tag_list, &self.before);
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        Self::show(ctx.tag_list, &self.after);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Marker;

    fn restored() -> FileSnapshot {
        // The restore renamed `clip.mp4` to `pick clip.mp4`: that is the open clip's name now.
        let mut snapshot =
            FileSnapshot::new(vec!["pick".to_string()], "clip", "mp4", "pick clip.mp4");
        snapshot.set_comment("A note".to_string());
        snapshot.set_segment_start(Some(1.0));
        snapshot.set_markers(Some(vec![Marker::new(5_000)]));
        snapshot
    }

    #[test]
    fn undo_goes_back_to_the_state_before_from_the_name_the_restore_gave() {
        let mut on_disk = FileSnapshot::parse("clip.mp4");
        on_disk.set_markers(Some(Vec::new()));
        let command = RestoreCommand::new(&on_disk, restored()).expect("a change");
        let before = command.before();
        assert!(before.tags().is_empty());
        assert_eq!(before.comment(), "");
        assert_eq!(before.segment_start(), None);
        assert_eq!(before.markers(), Some(&[][..]));
        assert_eq!(
            before.initial_file_name(),
            "pick clip.mp4",
            "the save renames from where the clip is now"
        );
    }

    #[test]
    fn a_restore_that_changed_nothing_visible_is_no_step() {
        let now = restored();
        // Read from disk before the restore: its own name then, the same edits.
        let mut on_disk = FileSnapshot::new(now.tags().to_vec(), "clip", "mp4", "pick clip.mp4");
        on_disk.set_comment(now.comment().to_string());
        on_disk.set_segment_start(now.segment_start());
        on_disk.set_markers(now.markers().map(<[_]>::to_vec));
        assert!(RestoreCommand::new(&on_disk, now).is_none());
    }
}
