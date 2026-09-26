//! Messages of the updates feature.

use std::sync::Arc;

/// A newer release found by a check, kept to download and apply it.
#[derive(Clone)]
pub struct Release(pub Arc<velopack::UpdateInfo>);

impl Release {
    /// The release's version, `0.68.0`.
    pub fn version(&self) -> &str {
        &self.0.TargetFullRelease.Version
    }
}

impl std::fmt::Debug for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Release({})", self.version())
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    /// The **Check for updates** button.
    CheckNow,
    /// Sent at start-up and every hour: checks in the background when a day has passed since
    /// the last check and the start-up check is on.
    Tick,
    /// A check finished: the newer release, or none, or why it failed.
    Checked {
        silent: bool,
        result: Result<Option<Release>, String>,
    },
    /// The **Update and restart** button.
    UpdateAndRestart,
    /// Download progress in percent.
    Progress(i16),
    /// The download finished, or why it failed.
    Downloaded(Result<(), String>),
    /// The **Check for updates when frename starts** check box.
    SetCheckOnStart(bool),
    /// The release is downloaded: the app saves the open file, exits, and Velopack applies the
    /// update and starts the new version. Handled by the app.
    ApplyAndRestart(Release),
}
