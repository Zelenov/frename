//! Messages for batch mode.

use frename_core::ai::key::ApiKey;
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
    /// Run was pressed while the open clip's marker requests still hold it: wait for them.
    /// Started by the workspace.
    WaitForMarkers,
    /// The wait for the marker requests is over (they let go, or the folder was left). Sent by
    /// the workspace; Cancel and the like end it too.
    StopWaiting,
    /// Stop the running job after the file in progress.
    Cancel,
    /// Dismiss the report of a finished job, with the outcomes shown in the list.
    CloseReport,
    /// Run the finished job's action again on its files that failed or were not reached.
    /// Started by the workspace, which owns the files.
    Retry,
    /// Open the log, which says why each failed file failed. Handled by the workspace.
    OpenLog,
    /// Open the billing page of a paid service to add credit. Handled by the workspace.
    OpenBilling(ApiKey),
}
