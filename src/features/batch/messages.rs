//! Messages for batch mode.

use frename_core::FileId;

use super::{ActionMessage, Operation};

/// Batch mode messages: from the batch panel, and from the folder list via the workspace.
#[derive(Debug, Clone)]
pub enum Message {
    /// Show the batch panel and the check boxes in the folder list (true) or the open file (false).
    SetActive(bool),
    /// Pick an action in the action list.
    SelectAction(super::Action),
    /// A change in the selected action's options.
    Action(ActionMessage),
    /// Enter batch mode with every listed file checked and the action set up to do `Operation`
    /// (from the settings window, after a storage change).
    Prepare {
        operation: Operation,
        files: Vec<FileId>,
    },
    /// Check or uncheck one file.
    Toggle(FileId),
    /// Check these files (the listed ones), keeping the other checks.
    CheckAll(Vec<FileId>),
    /// Uncheck every file.
    CheckNone,
    /// Flip the check of each of these files (the listed ones).
    Invert(Vec<FileId>),
    /// Run the selected action on the checked files. Started by the workspace, which owns them.
    Run,
    /// Stop the running job after the file in progress.
    Cancel,
    /// Dismiss the report of a finished job, with the outcomes shown in the list.
    CloseReport,
    /// Open the log, which says why each failed file failed. Handled by the workspace.
    OpenLog,
}
