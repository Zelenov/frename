//! Messages for the sync panel.

#[derive(Debug, Clone)]
pub enum Message {
    /// Push file name panel order → DB/grid.
    SyncUp,
    /// Push DB/grid order → file name panel.
    SyncDown,
    /// Toggle lock (only active when sequences are equal).
    ToggleLock,
}
