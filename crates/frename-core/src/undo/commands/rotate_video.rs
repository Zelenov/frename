//! Undo command for turning a video (see [`crate::FileTagger::rotate_video`]).

use std::path::PathBuf;

use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::file::FileId;
use crate::tags::FileTagger;

/// Records turning a video by `quarter_turns` clockwise. The turn is already in the file; undo
/// turns it back, redo turns it again.
pub struct RotateVideoCommand {
    pub file: FileId,
    /// Where the file was when it was turned; used when the folder no longer lists `file`.
    pub path: PathBuf,
    pub quarter_turns: i32,
}

impl RotateVideoCommand {
    fn turn<SD, ST>(
        &self,
        ctx: &UndoContext<'_, SD, ST>,
        quarter_turns: i32,
    ) -> Result<(), UndoError>
    where
        SD: AppStateStore + Clone,
    {
        // The file may have been renamed since (a tag change saved on leaving it).
        let path = ctx
            .directory
            .file_by_id(self.file)
            .map(|f| f.file_path().to_path_buf())
            .unwrap_or_else(|| self.path.clone());
        FileTagger::rotate_video(&path, quarter_turns)
            .map(|_| ())
            .map_err(|e| UndoError::Rotation(path, e))
    }
}

impl<SD, ST> Undoable<SD, ST> for RotateVideoCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.turn(ctx, -self.quarter_turns)
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.turn(ctx, self.quarter_turns)
    }

    fn turns_a_video(&self) -> bool {
        true
    }

    fn edits_open_video(&self) -> bool {
        true
    }
}
