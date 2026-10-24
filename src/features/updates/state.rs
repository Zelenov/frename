//! State for the updates feature.
//!
//! `apply` is the state machine: it changes the state and says what to run next (`Effect`), with
//! no network or clock of its own, so tests drive it with plain messages. `update` runs the
//! effects as tasks.

use std::time::{SystemTime, UNIX_EPOCH};

use frename_core::{AppDatabase, AppStateStore, UpdateCheckState};
use iced::futures::SinkExt;
use iced::Task;

use semver::Version;

use super::notes::{self, VersionNotes};
use super::{source, Message, Release};
use crate::package::{self, short_version};

/// Versions of release notes the page shows before **Show more**, and each press of it adds.
const NOTES_PAGE: usize = 10;

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

/// What the release notes block of the Updates page is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Heading {
    /// Every version, newest first (**Release notes**).
    History,
    /// What changed between the installed version and this newer one (`0.68`).
    Update(String),
    /// What is new in the version just installed (`0.68`).
    WhatsNew(String),
}

/// The release notes the Updates page shows now.
#[derive(Debug)]
pub struct Shown<'a> {
    pub heading: Heading,
    /// Newest first; at most as many as asked for.
    pub versions: Vec<&'a VersionNotes>,
    /// Whether older versions wait behind **Show more**.
    pub more: bool,
    /// The notes of a newer version are being fetched.
    pub loading: bool,
    /// They could not be fetched: the bundled ones are all there is.
    pub failed: bool,
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
    /// The release notes known: the ones bundled in this build, and the newest release's.
    notes: Vec<VersionNotes>,
    /// The version whose `version.md` is being fetched.
    fetching: Option<String>,
    /// Fetching the notes failed; a new check tries again.
    fetch_failed: bool,
    /// **Release notes** is on: the whole history instead of the notes of the update.
    history_open: bool,
    /// How many versions the notes show.
    notes_limit: usize,
    /// The version this one replaced, when this start is the first one after an update.
    updated_from: Option<Version>,
    /// The notes of the update were on screen.
    update_seen: bool,
}

impl Default for UpdatesState {
    fn default() -> Self {
        let package = package::current();
        let saved = AppDatabase::new().get_update_check().unwrap_or_default();
        let mut state = Self::new(
            package.is_some(),
            package::STORE_BUILD,
            package.map_or_else(
                || option_env!("APP_VERSION").unwrap_or("dev").to_string(),
                |p| p.version.clone(),
            ),
            saved.clone(),
        );
        state.download_build = is_download_build();
        // The first start that knows its version records it, so the next update can tell.
        if state.saved != saved {
            AppDatabase::new().set_update_check(state.saved.clone());
        }
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
        let mut saved = saved;
        let running = Version::parse(&current_version).ok();
        let mut updated_from = None;
        if let Some(running) = &running {
            match Version::parse(&saved.seen_version) {
                Ok(seen) if seen < *running => updated_from = Some(seen),
                Ok(seen) if seen > *running => saved.seen_version = current_version.clone(),
                Ok(_) => {}
                // Never recorded: nothing to say about a first start.
                Err(_) => saved.seen_version = current_version.clone(),
            }
        }
        Self {
            installed: installed && !from_store,
            from_store,
            download_build: false,
            current_version,
            saved,
            status: Status::Idle,
            found: None,
            notes: notes::parse(notes::BUNDLED),
            fetching: None,
            fetch_failed: false,
            history_open: false,
            notes_limit: NOTES_PAGE,
            updated_from,
            update_seen: false,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let saved = self.saved.clone();
        let effect = self.apply(message, now());
        if self.saved != saved {
            AppDatabase::new().set_update_check(self.saved.clone());
        }
        let fetch = self.notes_request().map(fetch_notes);
        let task = match effect {
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
        };
        match fetch {
            Some(fetch) => Task::batch([task, fetch]),
            None => task,
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
            Message::ToggleHistory => {
                self.history_open = !self.history_open;
                self.notes_limit = NOTES_PAGE;
                self.see_whats_new();
                Effect::None
            }
            Message::ShowMore => {
                self.notes_limit += NOTES_PAGE;
                Effect::None
            }
            Message::NotesFetched(result) => {
                self.fetching = None;
                match result {
                    Ok(markdown) => {
                        let fetched = notes::parse(&markdown);
                        // A file without the block of the release is not asked for again
                        // until the next check.
                        let target = Version::parse(&self.saved.newest_version).ok();
                        if target.is_some_and(|t| !fetched.iter().any(|n| n.version == t)) {
                            self.fetch_failed = true;
                        }
                        self.notes = notes::merge(std::mem::take(&mut self.notes), fetched);
                    }
                    Err(reason) => {
                        log::warn!("release notes not fetched: {reason}");
                        self.fetch_failed = true;
                    }
                }
                Effect::None
            }
            Message::SeeWhatsNew => {
                self.see_whats_new();
                Effect::None
            }
            // Handled by the app.
            Message::ApplyAndRestart(_) => Effect::None,
        }
    }

    /// A demo that shows a found update counts as an installed package.
    pub fn stage_installed(&mut self) {
        self.installed = true;
    }

    /// The notes of the update just installed are on screen: it is not new any more.
    fn see_whats_new(&mut self) {
        if self.updated_from.is_some() && !self.update_seen && self.available_version().is_none() {
            self.update_seen = true;
            self.saved.seen_version = self.current_version.clone();
        }
    }

    /// The version whose `version.md` is to be fetched now: a newer release is known and its
    /// notes are not. Marks the fetch as running.
    fn notes_request(&mut self) -> Option<String> {
        if self.fetching.is_some() || self.fetch_failed {
            return None;
        }
        let target = Version::parse(&self.saved.newest_version).ok()?;
        self.available_version()?;
        if self.notes.iter().any(|n| n.version == target) {
            return None;
        }
        self.fetching = Some(self.saved.newest_version.clone());
        self.fetching.clone()
    }

    fn start_check(&mut self, silent: bool, then_update: bool) -> Effect {
        if !self.installed || self.is_busy() {
            return Effect::None;
        }
        self.fetch_failed = false;
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

    /// A check, a download or the restart runs: the buttons wait.
    pub fn is_busy(&self) -> bool {
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

    /// Whether the Updates page shows **Import settings from an older frename**: only a package
    /// or the Store build keeps its settings away from the exe.
    pub fn shows_import(&self) -> bool {
        self.installed || self.is_store_build()
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

    /// The controls of the Version row that take a click now, in page order: the ones Tab
    /// reaches in Settings (#167). `batch_running` holds **Update and restart** back.
    pub fn controls(&self, batch_running: bool) -> Vec<Control> {
        let mut controls = match self.section() {
            Section::Store => Vec::new(),
            Section::ReleasesPage => vec![Control::OpenReleases],
            Section::Updater => [
                (Control::Check, self.check_enabled()),
                (
                    Control::UpdateAndRestart,
                    self.available_version().is_some() && self.update_enabled(batch_running),
                ),
                (Control::CheckOnStart, self.installed),
            ]
            .into_iter()
            .filter_map(|(control, enabled)| enabled.then_some(control))
            .collect(),
        };
        controls.push(Control::ReleaseNotes);
        if self.section() != Section::ReleasesPage {
            controls.push(Control::AllReleases);
        }
        if self.shown().is_some_and(|shown| shown.more) {
            controls.push(Control::ShowMore);
        }
        controls
    }

    /// Whether this start is the first one since an update, and the notes of it were not on
    /// screen yet: the version, `0.68`, for the quiet note on the settings button.
    pub fn whats_new(&self) -> Option<String> {
        (self.updated_from.is_some() && !self.update_seen).then(|| self.current_version())
    }

    /// The release notes the Updates page shows: the history when asked for, else what changed up
    /// to a newer version that is ready, else what is new in the version just installed; `None`
    /// when there is nothing to say.
    pub fn shown(&self) -> Option<Shown<'_>> {
        let current = Version::parse(&self.current_version).ok();
        let target = self
            .available_version()
            .and_then(|_| Version::parse(&self.saved.newest_version).ok());
        let (heading, versions): (Heading, Vec<&VersionNotes>) = if self.history_open {
            (Heading::History, self.notes.iter().collect())
        } else if let (Some(current), Some(target)) = (&current, &target) {
            (
                Heading::Update(short_version(&self.saved.newest_version)),
                notes::between(&self.notes, current, target),
            )
        } else if let (Some(from), Some(current)) = (&self.updated_from, &current) {
            (
                Heading::WhatsNew(self.current_version()),
                notes::between(&self.notes, from, current),
            )
        } else {
            return None;
        };
        let more = versions.len() > self.notes_limit;
        Some(Shown {
            heading,
            versions: versions.into_iter().take(self.notes_limit).collect(),
            more,
            loading: target.is_some() && self.fetching.is_some() && !self.history_open,
            failed: target.is_some() && self.fetch_failed && !self.history_open,
        })
    }

    /// Whether the notes block is the history, for the label of its button.
    pub fn history_open(&self) -> bool {
        self.history_open
    }

    /// Whether **Check for updates** takes a click: in a package, while nothing runs.
    pub fn check_enabled(&self) -> bool {
        self.installed && !self.is_busy()
    }

    /// Whether **Update and restart**, when shown, takes a click: not while a check, a download
    /// or a batch job runs.
    pub fn update_enabled(&self, batch_running: bool) -> bool {
        !self.is_busy() && !batch_running
    }

    /// What Space or Enter on `control` does: what a click does.
    pub fn press(&self, control: Control) -> Message {
        match control {
            Control::Check => Message::CheckNow,
            Control::UpdateAndRestart => Message::UpdateAndRestart,
            Control::CheckOnStart => Message::SetCheckOnStart(!self.check_on_start()),
            Control::OpenReleases | Control::AllReleases => Message::OpenReleases,
            Control::ReleaseNotes => Message::ToggleHistory,
            Control::ShowMore => Message::ShowMore,
        }
    }
}

/// A control of the Version row that keyboard focus can reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Check,
    UpdateAndRestart,
    CheckOnStart,
    /// The link to the releases page of a build that is updated by hand.
    OpenReleases,
    /// The **Release notes** button.
    ReleaseNotes,
    /// The link to all releases on GitHub, under the notes.
    AllReleases,
    /// **Show more** under the notes.
    ShowMore,
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

/// Fetch the `version.md` of the release `version` in the background.
fn fetch_notes(version: String) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || source::fetch_notes(&version))
                .await
                .unwrap_or_else(|e| Err(e.to_string()))
        },
        Message::NotesFetched,
    )
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
                .unwrap_or_else(|_| Err(fl!("updates-reason-download-stopped")))
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
                seen_version: String::new(),
            },
        )
    }

    #[test]
    fn the_store_build_has_no_update_controls_but_shows_the_import() {
        let store = UpdatesState::new(
            false,
            true,
            "0.67.0".to_string(),
            UpdateCheckState {
                check_on_start: true,
                last_check: 0,
                newest_version: String::new(),
                seen_version: String::new(),
            },
        );
        assert_eq!(
            store.controls(false),
            [Control::ReleaseNotes, Control::AllReleases],
            "only the release notes"
        );
        assert!(store.shows_import());
        assert!(installed(0).shows_import());
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
                seen_version: String::new(),
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

    const NOTES: &str = "# 0.69\n## Fixed\n- Sixty-nine.\n\n# 0.68\n## Added\n- Sixty-eight.\n\n# 0.67\n## Added\n- Sixty-seven.\n\n# 0.66\n## Added\n- Sixty-six.\n";

    fn labels(state: &UpdatesState) -> Vec<String> {
        state
            .shown()
            .map(|shown| shown.versions.iter().map(|v| v.label()).collect())
            .unwrap_or_default()
    }

    /// An installed 0.67 whose last start was `seen`, knowing `NOTES` up to 0.67.
    fn started_after(seen: &str) -> UpdatesState {
        let mut state = UpdatesState::new(
            true,
            false,
            "0.67.0".to_string(),
            UpdateCheckState {
                seen_version: seen.to_string(),
                ..UpdateCheckState::default()
            },
        );
        state.notes = notes::parse(NOTES)
            .into_iter()
            .filter(|n| n.label() != "0.69" && n.label() != "0.68")
            .collect();
        state
    }

    #[test]
    fn the_first_start_that_knows_its_version_says_nothing_and_records_it() {
        let state = started_after("");
        assert_eq!(state.whats_new(), None);
        assert!(state.shown().is_none());
        assert_eq!(state.saved.seen_version, "0.67.0");
    }

    #[test]
    fn the_first_start_after_an_update_offers_what_is_new_until_it_is_seen() {
        let mut state = started_after("0.65.0");
        assert_eq!(state.whats_new(), Some("0.67".to_string()));
        assert_eq!(
            labels(&state),
            ["0.67", "0.66"],
            "after the old one, up to the new one"
        );
        assert_eq!(
            state.shown().map(|s| s.heading),
            Some(Heading::WhatsNew("0.67".to_string()))
        );
        assert_eq!(
            state.saved.seen_version, "0.65.0",
            "kept until the notes are seen"
        );

        state.apply(Message::SeeWhatsNew, 10);
        assert_eq!(state.whats_new(), None, "the dot goes");
        assert_eq!(state.saved.seen_version, "0.67.0");
        assert_eq!(
            labels(&state),
            ["0.67", "0.66"],
            "the notes stay on the page"
        );

        // Another start: nothing new.
        let again = started_after("0.67.0");
        assert_eq!(again.whats_new(), None);
        assert!(again.shown().is_none());
    }

    #[test]
    fn a_build_without_a_version_has_no_notes_to_offer() {
        let state = UpdatesState::new(false, false, "dev".to_string(), UpdateCheckState::default());
        assert_eq!(state.whats_new(), None);
        assert!(state.shown().is_none());
    }

    #[test]
    fn a_found_update_fetches_the_notes_once_and_shows_every_version_up_to_it() {
        let mut state = started_after("0.67.0");
        state.apply(
            Message::Checked {
                silent: false,
                result: Ok(Some(release("0.69.0"))),
            },
            10,
        );
        assert_eq!(state.notes_request(), Some("0.69.0".to_string()));
        assert_eq!(state.notes_request(), None, "not twice at once");
        let shown = state.shown().expect("the update's notes");
        assert_eq!(shown.heading, Heading::Update("0.69".to_string()));
        assert!(
            shown.versions.is_empty() && shown.loading,
            "while they load"
        );

        state.apply(Message::NotesFetched(Ok(NOTES.to_string())), 11);
        assert_eq!(labels(&state), ["0.69", "0.68"], "not the installed one");
        assert!(!state.shown().expect("shown").loading);
        assert_eq!(state.notes_request(), None, "they are known now");
    }

    #[test]
    fn notes_that_could_not_be_fetched_are_asked_for_again_only_by_a_new_check() {
        let mut state = started_after("0.67.0");
        state.apply(
            Message::Checked {
                silent: false,
                result: Ok(Some(release("0.69.0"))),
            },
            10,
        );
        state.notes_request();
        state.apply(Message::NotesFetched(Err("no connection".to_string())), 11);
        let shown = state.shown().expect("shown");
        assert!(shown.failed && !shown.loading);
        assert_eq!(state.notes_request(), None);
        state.apply(Message::CheckNow, 12);
        assert_eq!(state.notes_request(), Some("0.69.0".to_string()));
    }

    #[test]
    fn the_history_is_every_known_version_ten_at_a_time_and_toggles_back() {
        let mut state = started_after("0.67.0");
        state.notes = (1..=25)
            .rev()
            .map(|minor| notes::parse(&format!("# 0.{minor}\n- v{minor}\n")).remove(0))
            .collect();
        assert!(state.shown().is_none());

        state.apply(Message::ToggleHistory, 10);
        let shown = state.shown().expect("history");
        assert_eq!(shown.heading, Heading::History);
        assert_eq!((shown.versions.len(), shown.more), (10, true));
        assert_eq!(shown.versions[0].label(), "0.25", "newest first");
        assert!(state.controls(false).contains(&Control::ShowMore));

        state.apply(Message::ShowMore, 11);
        state.apply(Message::ShowMore, 11);
        let shown = state.shown().expect("history");
        assert_eq!((shown.versions.len(), shown.more), (25, false));
        assert!(!state.controls(false).contains(&Control::ShowMore));

        state.apply(Message::ToggleHistory, 12);
        assert!(state.shown().is_none(), "back to nothing to say");
    }

    #[test]
    fn a_fetched_file_without_the_releases_block_is_not_asked_for_again() {
        let mut state = started_after("0.67.0");
        state.apply(
            Message::Checked {
                silent: false,
                result: Ok(Some(release("0.70.0"))),
            },
            10,
        );
        assert!(state.notes_request().is_some());
        state.apply(Message::NotesFetched(Ok(NOTES.to_string())), 11);
        assert_eq!(state.notes_request(), None, "no endless fetching");
        assert!(state.shown().expect("shown").failed);
    }

    #[test]
    fn after_a_downgrade_the_next_update_is_still_offered_its_notes() {
        let state = started_after("0.80.0");
        assert_eq!(
            state.saved.seen_version, "0.67.0",
            "the running version is recorded"
        );
        assert_eq!(state.whats_new(), None);
    }

    #[test]
    fn whats_new_waits_while_a_newer_update_is_ready() {
        let mut state = started_after("0.65.0");
        state.apply(
            Message::Checked {
                silent: false,
                result: Ok(Some(release("0.69.0"))),
            },
            10,
        );
        state.apply(Message::SeeWhatsNew, 11);
        assert_eq!(
            state.whats_new(),
            Some("0.67".to_string()),
            "not shown yet, so not seen"
        );
    }

    #[test]
    fn a_build_updated_by_hand_has_no_second_link_to_the_releases_page() {
        let mut state = UpdatesState::new(
            false,
            false,
            "0.67.0".to_string(),
            UpdateCheckState::default(),
        );
        state.download_build = true;
        assert_eq!(
            state.controls(false),
            [Control::OpenReleases, Control::ReleaseNotes]
        );
    }
}
