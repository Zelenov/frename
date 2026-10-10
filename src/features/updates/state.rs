//! State for the updates feature.
//!
//! `apply` is the state machine: it changes the state and says what to run next (`Effect`), with
//! no network or clock of its own, so tests drive it with plain messages. `update` runs the
//! effects as tasks.

use std::time::{SystemTime, UNIX_EPOCH};

use frename_core::{AppDatabase, AppStateStore, UpdateCheckState};
use iced::futures::SinkExt;
use iced::Task;

use super::{source, Message, Release};
use crate::package::{self, short_version};

/// Seconds between background checks: GitHub allows 60 unauthenticated requests an hour.
const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;

/// What the Updates section is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Nothing is running, and the last check had nothing to say (or was a quiet background one).
    Idle,
    /// A check runs; `then_update` downloads straight away when it finds the release.
    Checking {
        silent: bool,
        then_update: bool,
    },
    UpToDate,
    Downloading(i16),
    /// Downloaded; frename is closing for the update.
    Restarting,
    CheckFailed(String),
    UpdateFailed(String),
}

/// How this frename was delivered, as far as updating goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Build {
    /// The Microsoft Store build: the Store updates it.
    Store,
    /// An installed or portable Windows package: it updates itself.
    Package,
    /// The Mac download or the Linux AppImage: new versions are downloaded by hand.
    Download,
    /// Anything else (`cargo run`, a bare exe, a Linux binary outside an AppImage).
    Bare,
}

/// What the Updates section shows for a build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// The version and "Updates come from the Microsoft Store".
    Store,
    /// The version and "download new versions from the releases page" with the link.
    ReleasesPage,
    /// The version, the status, **Check for updates** and the start-up check box.
    Updater,
}

impl Build {
    /// The Store wins over a package found on disk; a package over the download kinds.
    fn detect(from_store: bool, installed: bool, download: bool) -> Self {
        if from_store {
            Self::Store
        } else if installed {
            Self::Package
        } else if download {
            Self::Download
        } else {
            Self::Bare
        }
    }

    pub fn section(self) -> Section {
        match self {
            Self::Store => Section::Store,
            Self::Download => Section::ReleasesPage,
            Self::Package | Self::Bare => Section::Updater,
        }
    }
}

/// Whether this process is the Mac download or the Linux AppImage.
fn is_download_build() -> bool {
    cfg!(target_os = "macos") || frename_core::is_appimage()
}

/// What `apply` asks `update` to run.
#[derive(Debug)]
enum Effect {
    None,
    Check { silent: bool },
    Download(Release),
    Restart(Release),
    OpenReleases,
}

/// The Updates section: running version, check state and the newest release found.
pub struct UpdatesState {
    /// Updates work only in an installed or portable package.
    installed: bool,
    /// The Microsoft Store build: the Store updates it, frename never checks.
    from_store: bool,
    /// The Mac download or the Linux AppImage: no updater, new versions come from the releases
    /// page.
    download_build: bool,
    /// The running version, `0.67.0` (or `dev`).
    current_version: String,
    saved: UpdateCheckState,
    status: Status,
    /// The newer release the last check found, needed to download it.
    found: Option<Release>,
}

impl Default for UpdatesState {
    fn default() -> Self {
        let package = package::current();
        let mut state = Self::new(
            package.is_some(),
            package::STORE_BUILD,
            package.map_or_else(
                || option_env!("APP_VERSION").unwrap_or("dev").to_string(),
                |p| p.version.clone(),
            ),
            AppDatabase::new().get_update_check().unwrap_or_default(),
        );
        state.download_build = is_download_build();
        state
    }
}

impl UpdatesState {
    /// `from_store` wins over `installed`: the Store build never updates itself, whatever it
    /// finds on disk.
    fn new(
        installed: bool,
        from_store: bool,
        current_version: String,
        saved: UpdateCheckState,
    ) -> Self {
        Self {
            installed: installed && !from_store,
            from_store,
            download_build: false,
            current_version,
            saved,
            status: Status::Idle,
            found: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let saved = self.saved.clone();
        let effect = self.apply(message, now());
        if self.saved != saved {
            AppDatabase::new().set_update_check(self.saved.clone());
        }
        match effect {
            Effect::None => Task::none(),
            Effect::Check { silent } => Task::perform(
                async {
                    tokio::task::spawn_blocking(source::check)
                        .await
                        .unwrap_or_else(|e| Err(e.to_string()))
                },
                move |result| Message::Checked { silent, result },
            ),
            Effect::Download(release) => download(release),
            Effect::Restart(release) => Task::done(Message::ApplyAndRestart(release)),
            Effect::OpenReleases => {
                open_releases_page();
                Task::none()
            }
        }
    }

    fn apply(&mut self, message: Message, now: u64) -> Effect {
        match message {
            Message::OpenReleases => Effect::OpenReleases,
            Message::CheckNow => self.start_check(false, false),
            Message::Tick => {
                if self.background_check_due(now) {
                    self.start_check(true, false)
                } else {
                    Effect::None
                }
            }
            Message::Checked { silent, result } => self.checked(silent, result, now),
            Message::UpdateAndRestart => match self.found.clone() {
                Some(release) if self.can_update() => {
                    self.status = Status::Downloading(0);
                    Effect::Download(release)
                }
                // Known from an earlier run's check: look it up again, then download.
                None if self.can_update() => self.start_check(false, true),
                _ => Effect::None,
            },
            Message::Progress(percent) => {
                if matches!(self.status, Status::Downloading(_)) {
                    self.status = Status::Downloading(percent.clamp(0, 100));
                }
                Effect::None
            }
            Message::Downloaded(Ok(())) => match self.found.clone() {
                Some(release) => {
                    self.status = Status::Restarting;
                    Effect::Restart(release)
                }
                None => Effect::None,
            },
            Message::Downloaded(Err(reason)) => {
                self.status = Status::UpdateFailed(reason);
                Effect::None
            }
            Message::SetCheckOnStart(on) => {
                self.saved.check_on_start = on;
                Effect::None
            }
            // Handled by the app.
            Message::ApplyAndRestart(_) => Effect::None,
        }
    }

    fn start_check(&mut self, silent: bool, then_update: bool) -> Effect {
        if !self.installed || self.is_busy() {
            return Effect::None;
        }
        self.status = Status::Checking {
            silent,
            then_update,
        };
        Effect::Check { silent }
    }

    fn checked(
        &mut self,
        silent: bool,
        result: Result<Option<Release>, String>,
        now: u64,
    ) -> Effect {
        let then_update = matches!(
            self.status,
            Status::Checking {
                then_update: true,
                ..
            }
        );
        match result {
            Err(reason) => {
                log::warn!("update check failed: {reason}");
                self.status = if silent {
                    Status::Idle
                } else {
                    Status::CheckFailed(reason)
                };
                Effect::None
            }
            Ok(found) => {
                self.saved.last_check = now;
                self.saved.newest_version = found
                    .as_ref()
                    .map(|r| r.version().to_string())
                    .unwrap_or_default();
                self.found = found;
                match (&self.found, then_update) {
                    (Some(release), true) => {
                        self.status = Status::Downloading(0);
                        Effect::Download(release.clone())
                    }
                    (Some(_), false) => {
                        self.status = Status::Idle;
                        Effect::None
                    }
                    (None, _) => {
                        self.status = if silent {
                            Status::Idle
                        } else {
                            Status::UpToDate
                        };
                        Effect::None
                    }
                }
            }
        }
    }

    fn background_check_due(&self, now: u64) -> bool {
        self.installed
            && self.saved.check_on_start
            && now >= self.saved.last_check.saturating_add(CHECK_INTERVAL_SECS)
    }

    fn is_busy(&self) -> bool {
        matches!(
            self.status,
            Status::Checking { .. } | Status::Downloading(_) | Status::Restarting
        )
    }

    fn can_update(&self) -> bool {
        self.installed && !self.is_busy() && self.available_version().is_some()
    }

    /// Whether this frename can update itself (installed or portable package).
    pub fn installed(&self) -> bool {
        self.installed
    }

    /// Whether this is the Microsoft Store build, which the Store updates.
    pub fn is_store_build(&self) -> bool {
        self.from_store
    }

    /// What the Updates section shows for this build.
    pub fn section(&self) -> Section {
        Build::detect(self.from_store, self.installed, self.download_build).section()
    }

    /// The running version as the UI shows it, `0.67`.
    pub fn current_version(&self) -> String {
        short_version(&self.current_version)
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn check_on_start(&self) -> bool {
        self.saved.check_on_start
    }

    /// The newer version available, as the UI shows it (`0.68`), when the last check found one
    /// newer than the running version. Survives restarts.
    pub fn available_version(&self) -> Option<String> {
        let newest = semver::Version::parse(&self.saved.newest_version).ok()?;
        let current = semver::Version::parse(&self.current_version).ok()?;
        (self.installed && newest > current).then(|| short_version(&self.saved.newest_version))
    }
}

/// Open the releases page in the browser.
fn open_releases_page() {
    #[cfg(windows)]
    let result = std::process::Command::new("explorer")
        .arg(source::RELEASES_PAGE)
        .spawn();
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open")
        .arg(source::RELEASES_PAGE)
        .spawn();
    #[cfg(not(any(windows, target_os = "macos")))]
    let result = std::process::Command::new("xdg-open")
        .arg(source::RELEASES_PAGE)
        .spawn();
    if let Err(e) = result {
        log::warn!("could not open {}: {e}", source::RELEASES_PAGE);
    }
}

/// Download `release`, forwarding Velopack's progress as messages. Velopack reports on a
/// blocking channel, so a blocking task relays it to the UI's stream.
fn download(release: Release) -> Task<Message> {
    let stream = iced::stream::channel(16, async move |mut output| {
        let mut forward = output.clone();
        let result = tokio::task::spawn_blocking(move || {
            let (progress, received) = std::sync::mpsc::channel();
            let downloader = std::thread::spawn({
                let release = release.clone();
                move || source::download(&release, progress)
            });
            // Ends when the download drops its sender.
            for percent in received {
                let _ = forward.try_send(Message::Progress(percent));
            }
            downloader
                .join()
                .unwrap_or_else(|_| Err("the download stopped unexpectedly".to_string()))
        })
        .await
        .unwrap_or_else(|e| Err(e.to_string()));
        // Awaited, unlike the progress: a full buffer must not lose the outcome.
        let _ = output.send(Message::Downloaded(result)).await;
    });
    Task::run(stream, |message| message)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    const DAY: u64 = CHECK_INTERVAL_SECS;

    fn release(version: &str) -> Release {
        let mut info = velopack::UpdateInfo::default();
        info.TargetFullRelease.Version = version.to_string();
        Release(Arc::new(info))
    }

    /// An installed 0.67 that last checked at `last_check`. `apply` saves nothing; `update` does.
    fn installed(last_check: u64) -> UpdatesState {
        UpdatesState::new(
            true,
            false,
            "0.67.0".to_string(),
            UpdateCheckState {
                check_on_start: true,
                last_check,
                newest_version: String::new(),
            },
        )
    }

    #[test]
    fn each_build_shows_its_own_updates_section() {
        use Build::*;
        assert_eq!(Build::detect(true, true, true), Store);
        assert_eq!(Build::detect(false, true, true), Package);
        assert_eq!(Build::detect(false, false, true), Download);
        assert_eq!(Build::detect(false, false, false), Bare);
        assert_eq!(Store.section(), Section::Store);
        assert_eq!(Package.section(), Section::Updater);
        assert_eq!(Download.section(), Section::ReleasesPage);
        assert_eq!(Bare.section(), Section::Updater);
    }

    #[test]
    fn a_download_build_shows_the_releases_page_and_opens_it_without_checking() {
        let mut state = UpdatesState::new(
            false,
            false,
            "0.67.0".to_string(),
            UpdateCheckState::default(),
        );
        assert_eq!(state.section(), Section::Updater);
        state.download_build = true;
        assert_eq!(state.section(), Section::ReleasesPage);
        assert!(matches!(
            state.apply(Message::OpenReleases, 10),
            Effect::OpenReleases
        ));
        assert!(matches!(state.apply(Message::CheckNow, 10), Effect::None));
        assert_eq!(state.current_version(), "0.67");
    }

    #[test]
    fn the_releases_link_is_the_repository_releases_page() {
        assert_eq!(
            source::RELEASES_PAGE,
            "https://github.com/Zelenov/frename/releases/latest"
        );
    }

    #[test]
    fn a_manual_check_that_finds_nothing_says_up_to_date() {
        let mut state = installed(0);
        assert!(matches!(
            state.apply(Message::CheckNow, 10),
            Effect::Check { silent: false }
        ));
        state.apply(
            Message::Checked {
                silent: false,
                result: Ok(None),
            },
            10,
        );
        assert_eq!(state.status(), &Status::UpToDate);
        assert_eq!(state.available_version(), None);
    }

    #[test]
    fn a_found_update_is_offered_and_downloads_then_restarts() {
        let mut state = installed(0);
        state.apply(Message::CheckNow, 10);
        state.apply(
            Message::Checked {
                silent: false,
                result: Ok(Some(release("0.68.0"))),
            },
            10,
        );
        assert_eq!(state.available_version(), Some("0.68".to_string()));

        assert!(matches!(
            state.apply(Message::UpdateAndRestart, 11),
            Effect::Download(_)
        ));
        state.apply(Message::Progress(42), 11);
        assert_eq!(state.status(), &Status::Downloading(42));
        assert!(
            matches!(state.apply(Message::CheckNow, 11), Effect::None),
            "no check while downloading"
        );

        assert!(matches!(
            state.apply(Message::Downloaded(Ok(())), 12),
            Effect::Restart(r) if r.version() == "0.68.0"
        ));
        assert_eq!(state.status(), &Status::Restarting);
    }

    #[test]
    fn a_failed_download_can_be_retried() {
        let mut state = installed(0);
        state.apply(
            Message::Checked {
                silent: false,
                result: Ok(Some(release("0.68.0"))),
            },
            10,
        );
        state.apply(Message::UpdateAndRestart, 11);
        state.apply(Message::Downloaded(Err("no connection".to_string())), 12);
        assert_eq!(
            state.status(),
            &Status::UpdateFailed("no connection".to_string())
        );
        assert!(matches!(
            state.apply(Message::UpdateAndRestart, 13),
            Effect::Download(_)
        ));
    }

    #[test]
    fn a_manual_check_error_is_shown_and_a_background_one_is_not() {
        let mut state = installed(0);
        state.apply(Message::CheckNow, 10);
        state.apply(
            Message::Checked {
                silent: false,
                result: Err("no connection".to_string()),
            },
            10,
        );
        assert_eq!(
            state.status(),
            &Status::CheckFailed("no connection".to_string())
        );

        let mut state = installed(0);
        state.apply(Message::Tick, DAY);
        state.apply(
            Message::Checked {
                silent: true,
                result: Err("no connection".to_string()),
            },
            DAY,
        );
        assert_eq!(state.status(), &Status::Idle);
    }

    #[test]
    fn the_background_check_runs_at_most_once_a_day_and_only_when_on() {
        let mut state = installed(DAY);
        assert!(matches!(
            state.apply(Message::Tick, DAY + DAY - 1),
            Effect::None
        ));
        assert!(matches!(
            state.apply(Message::Tick, DAY + DAY),
            Effect::Check { silent: true }
        ));

        let mut off = installed(0);
        off.saved.check_on_start = false;
        assert!(matches!(off.apply(Message::Tick, 10 * DAY), Effect::None));
    }

    #[test]
    fn an_update_known_from_an_earlier_run_is_looked_up_again_before_downloading() {
        let mut state = installed(0);
        state.saved.newest_version = "0.68.0".to_string();
        assert_eq!(state.available_version(), Some("0.68".to_string()));

        assert!(matches!(
            state.apply(Message::UpdateAndRestart, 10),
            Effect::Check { silent: false }
        ));
        assert!(matches!(
            state.apply(
                Message::Checked {
                    silent: false,
                    result: Ok(Some(release("0.68.0"))),
                },
                10,
            ),
            Effect::Download(_)
        ));
    }

    #[test]
    fn a_version_found_that_is_not_newer_is_not_offered() {
        let mut state = installed(0);
        state.saved.newest_version = "0.67.0".to_string();
        assert_eq!(state.available_version(), None);
    }

    #[test]
    fn an_unpackaged_build_never_checks() {
        let mut state =
            UpdatesState::new(false, false, "dev".to_string(), UpdateCheckState::default());
        assert!(matches!(state.apply(Message::CheckNow, 10), Effect::None));
        assert!(matches!(state.apply(Message::Tick, 10 * DAY), Effect::None));
        assert!(matches!(
            state.apply(Message::UpdateAndRestart, 10),
            Effect::None
        ));
        assert_eq!(state.status(), &Status::Idle);
    }

    /// The Store build takes the installed version's database on its first start, with its saved
    /// check (on at start-up, a newer version found): that must not make the Store build check
    /// GitHub or offer that version, since the Store updates it.
    #[test]
    fn the_store_build_never_checks_even_with_a_saved_newer_version() {
        // Even where something reports it as installed.
        let mut state = UpdatesState::new(
            true,
            true,
            "0.67.0".to_string(),
            UpdateCheckState {
                check_on_start: true,
                last_check: 0,
                newest_version: "0.68.0".to_string(),
            },
        );
        assert!(state.is_store_build());
        assert!(matches!(state.apply(Message::Tick, 10 * DAY), Effect::None));
        assert!(matches!(state.apply(Message::CheckNow, 10), Effect::None));
        assert!(matches!(
            state.apply(Message::UpdateAndRestart, 10),
            Effect::None
        ));
        assert_eq!(state.available_version(), None);
    }
}
