# Installer, auto-update and bundled GStreamer

Design for issue #10. Research: [`docs/research/windows-installer.md`](../research/windows-installer.md).
Velopack facts below are from the Velopack docs repository
([velopack/velopack.docs](https://github.com/velopack/velopack.docs), files named inline) and the
`velopack` crate 1.2.158 source.

## Problem

Today a user downloads `frename-windows-x64-vX.Y.0.zip`, which holds only `frename.exe` and
`GSTREAMER_SETUP.md`, installs GStreamer from its website, and adds it to `PATH` by hand. A wrong
GStreamer version or a missing plugin shows up as "video does not play". Updating means downloading
the zip again. The goal: one download, at most one click, video plays on a clean Windows, and the
app can update itself.

## User flows

1. **Install.** The user downloads `frename-win-Setup.exe` from the latest release and runs it.
   No questions: it installs per user to `%LocalAppData%\frename\` (no admin rights), adds Start
   menu and desktop shortcuts, and starts frename (Velopack one-click installer:
   `docs/packaging/installer.mdx`, `docs/packaging/operating-systems/windows.mdx`). Windows shows
   SmartScreen "unknown publisher" once, because the build is not signed; the README says to click
   **More info → Run anyway**.
2. **Portable.** `frename-windows-x64-vX.Y.0.zip` stays on every release (the nightly pipeline
   checks for exactly this asset). It becomes Velopack's portable package: unzip anywhere, run
   `frename.exe`, video plays; it can update itself too (`docs/packaging/overview.mdx`: "Portable
   Release: … run and update your app without installing").
3. **Check for updates.** Settings → **Updates** shows the running version and a
   **Check for updates** button. States: `Checking…` → `frename is up to date` /
   `Version 0.68 is available` with an **Update and restart** button /
   `Could not check for updates: <short reason>`. **Update and restart** downloads (progress in
   percent), saves the open file the same way as moving to another file, then exits and lets
   Velopack apply the update and start the new version, which reopens the same folder and file
   (the last session in `folder_history`, as every start does today).
4. **Silent check at startup.** A checkbox in the same section, **Check for updates when frename
   starts**, on by default. At most once per 24 hours, in the background, never blocking. The newest
   version found is stored in `app_settings` with the check time, so the notice survives restarts:
   while that version is newer than the running one, the Settings gear in the folder controls has a
   small accent dot and the tooltip `Update available: 0.68`. Nothing is downloaded or applied
   without the button.
5. **Not installed** (`cargo run`, or a plain `frename.exe` copied out of a package): the Updates
   section says `Updates work in the installed version` and the button is disabled
   (`UpdateManager::new` returns `NotInstalled` then — research doc).
6. **Settings from the zip version.** On the first interactive start of an installed or portable
   package with no `frename.db` in the data folder, frename **looks** for an old one, the issue's "database found next to the exe":
   a `frename.db` next to a `frename.exe` in the user's Downloads, Desktop and Documents folders
   and their subfolders up to two levels deep (the places a downloaded zip gets unpacked), newest
   first. Only when one is found, a native dialog (`rfd`, already a dependency) asks, before the
   main window opens:
   `Import your settings and recent folders from C:\Users\…\Downloads\frename?` — **Import** /
   **Start fresh**. Nothing found → no dialog, the app just starts: a clean install asks nothing.
   Either answer creates the new database, so the question never comes back. `--self-test` and any
   other non-interactive start never look and never ask. Settings also get a small
   **Import from an old frename folder…** button (folder picker, then the same import, then
   `Restart frename to use the imported settings`) — the only way back when the search misses a
   zip kept elsewhere (e.g. `D:\Tools\frename`), which would otherwise silently put a user who
   chose text-file comments back on the default, writing into their videos.

## UI sketch

Settings window, new last section:

```
Updates
  frename 0.67
  [ Check for updates ]   frename is up to date
  [x] Check for updates when frename starts
```

With an update:

```
Updates
  frename 0.67
  [ Check for updates ]   Version 0.68 is available   [ Update and restart ]
```

While downloading: `Downloading 0.68… 42%`, both buttons disabled. Errors: `no connection`
for network failures; `GitHub did not respond, try later` for server errors and rate limits.
A failed download shows `Could not update: no connection` (or the reason) and enables both
buttons again. Closing frename during a download cancels it; nothing is applied, and the next
**Update and restart** downloads again (Velopack keeps finished packages only). Versions are shown as `version.md` writes them: a trailing `.0` of Velopack's
3-part version is dropped everywhere in the UI (`0.67`, not `0.67.0`). While a batch job runs,
**Update and restart** is disabled with the tooltip `Wait for the batch to finish`.

The settings window is not resizable and every section must fit (`SETTINGS_WINDOW_SIZE` in
`src/app/state.rs`, 560×560 today). It becomes 560×680 (fits a 1366×768 laptop with the taskbar).
The PR checks the worst case in a screenshot: both "move existing…" offers shown, the update row
with **Update and restart**, and the import button. If that does not fit in 680 px, the settings
content becomes a scrollable column instead of growing further.

## Keyboard shortcuts

None new.

## Package layout

Velopack's Windows layout (`docs/packaging/operating-systems/windows.mdx`):

```
%LocalAppData%\frename\
├── current\                    replaced whole on every update
│   ├── frename.exe
│   ├── gstreamer-1.0-0.dll, glib-2.0-0.dll, … (GStreamer, GLib, FFmpeg DLLs)
│   ├── gst-plugin-scanner.exe
│   ├── vcruntime140.dll, vcruntime140_1.dll, msvcp140.dll
│   ├── lib\gstreamer-1.0\*.dll  (allowlisted plugins)
│   └── licenses\                 (LGPL texts, GStreamer/FFmpeg source links)
├── Update.exe
├── frename.exe                 launcher stub
└── (frename data, see below)
```

The portable zip has the same layout with a `.portable` marker file in its root
(`velopack` crate `locator.rs`: `IsPortable: root_dir.join(".portable").exists()`).

- **GStreamer** comes from the official MSVC x86_64 runtime MSI, extracted in CI without
  installing (`msiexec /a … TARGETDIR=…`), pinned to one version (1.28.x, the research doc's
  measurements) and checked against a SHA-256 pinned in the workflow. Plugins are an allowlist
  file, `packaging/windows/gstreamer-plugins.txt`, with the research doc's set (core/base, audio
  out, isomp4, matroska, avi, parsers, `debugutilsbad` for `capssetter`, libav, d3d11, dav1d,
  opus, vorbis, mpg123, jpeg). The DLLs they need are not hand-listed: a CI script walks
  `dumpbin /dependents` recursively from `frename.exe`, `gst-plugin-scanner.exe` and every
  allowlisted plugin, and copies each DLL it finds in GStreamer's `bin\` or in the Visual Studio
  CRT redist folder (so CRT DLLs imported only by GStreamer, FFmpeg or a C++ plugin, such as
  `msvcp140_1.dll`, come along too). `dumpbin` is not on `PATH` on the runner; the script finds it
  with `vswhere`. No GPL parts (no x264/x265, no `ugly`).
- **VC++ runtime** is copied app-local (by the walk above) from the runner's Visual Studio redist
  folder, instead of
  `vpk --framework vcredist143-x64`. `bootstrapping.mdx` does not say how the redistributable is
  installed; it is a per-machine install (Microsoft's `vc_redist.x64.exe` writes to
  `System32`), so on a machine without it the one-click install would stop at a UAC prompt.
  App-local CRT DLLs avoid that and are a supported Microsoft deployment.
- **Size** is measured by CI and written into the PR; the research estimate is ~65–75 MB installed
  and ~28–35 MB for `Setup.exe`.

## Runtime changes

### Start-up order (`src/main.rs`)

1. `velopack::VelopackApp::build().set_auto_apply_on_startup(false).run()` — must be first; it
   handles install/update hooks and exits for them (research doc; `velopack` `app.rs`). Auto-apply
   is turned off: by default (`auto_apply: true` in `app.rs`) a start that finds a downloaded
   package applies it and restarts, which would update without the button and could kill another
   running frename with unsaved edits. Updates are applied only by **Update and restart**.
2. Work out the data folder (below) with Velopack's locator — no dialogs or threads yet.
3. `configure_bundled_gstreamer(exe_dir, data_dir)` — Windows only, before any thread (so before
   the import dialog, whose shell COM code starts threads) and before `gst::init`, as in
   the research doc: when `lib\gstreamer-1.0\gstcoreelements.dll` exists next to the exe, remove
   `GST_PLUGIN_PATH`, `GST_PLUGIN_PATH_1_0`, `GST_PLUGIN_SYSTEM_PATH`, `GST_PLUGIN_SCANNER`,
   `GST_REGISTRY`, and set `GST_PLUGIN_SYSTEM_PATH_1_0`, `GST_PLUGIN_SCANNER_1_0` and
   `GST_REGISTRY_1_0` (registry file in the data folder). The exe's folder is first in the Windows
   DLL search order, so a system GStreamer on `PATH` is not loaded. Without the bundled plugins
   (a developer build) nothing changes and the system GStreamer is used, as today.
4. Only for an installed or portable Windows package (the locator says so), not for `--self-test`
   or an unpackaged build: if the data folder has no `frename.db`, the old-database search and
   dialog (flow 6).
   It is split into a pure function that returns the variables to remove and set (tested on every
   OS) and a Windows-only caller that applies them.
5. Logging, `gst::init`, database, window — as today, with the paths below.

### Data folder

`frename.db`, `frename_debug.log` and the GStreamer registry move out of the exe's folder, which
Velopack replaces on update (`docs/integrating/preserved-files.mdx`: settings that persist through
updates go "one level up (`..\`) outside of the `current` dir").

| How frename runs | Data folder |
|---|---|
| Installed | `%LocalAppData%\frename\` (Velopack root; kept by updates, removed on uninstall) |
| Portable | the portable root folder (where the user unzipped it) |
| Not packaged (`cargo run`, CI tests, Linux) | next to the exe, as today |

`main` finds it with Velopack's locator and hands it to `frename-core` through a setter
(`set_app_data_dir`), like the storage settings: `frename-core` does not depend on `velopack`.
`AppDatabase::new` and the log use it. The choice of folder is a pure function (tested); the
process-wide setter is called once in `main` and not by tests (tests use `AppDatabase::with_path`
or the pure function, so parallel test threads do not race).

**Migration.** `frename.db` holds the folder history (recent folders and the last file in each),
window size, volume, and Settings (comment and in/out storage, the "Commented" tag, autoplay,
monochrome tags). The installed app cannot know where the old zip was, so the first start searches
for it (flow 6). The import is done in `frename-core` at start-up only, before any connection to
the new database exists, with SQLite's `VACUUM INTO` from the old database (opened read-only):
the old app keeps its database in WAL mode (`open_tuned` in `app_database.rs`), so copying the
`frename.db` file alone could miss recent changes still in `frename.db-wal`, and `VACUUM INTO`
reads them. The old files are left untouched. A zip user who unzips the new portable zip over their old frename folder needs nothing:
the old `frename.db` is already in the portable root, the data folder; unzipped anywhere else, the
same first-start dialog appears. The README tells zip users to extract into their existing
frename folder.

Uninstalling removes `%LocalAppData%\frename\` with the database
(`docs/integrating/uninstalling.mdx`); tags and comments live next to the footage and stay.

### Updates

In a new `src/features/updates/` feature (messages / state / view, Elm pattern), shown inside the
Settings window:
- `UpdateManager::new(GithubSource::new("https://github.com/Zelenov/frename", None, false), …)`
  — `prerelease = false`; draft releases (branch builds) are invisible to it (research doc).
- `check_for_updates` runs in `Task::perform` over `spawn_blocking` (a blocking call).
  `download_updates` reports progress on a `std::sync::mpsc::Sender<i16>`; it runs in
  `spawn_blocking` inside a `Task::run` stream that forwards each value as a message, so the
  percentage reaches the UI (`Task::perform` yields only one message).
- Apply: the app saves the open file through the existing unload → `apply_file_updated` path,
  then calls `wait_exit_then_apply_updates` and closes the window. Velopack applies the update
  after frename exits and starts the new version.
- The last check time is stored in `app_settings`, so the start-up check runs at most once a day
  (GitHub's unauthenticated API allows 60 requests per hour per IP).

### Versions

Velopack needs 3-part semver. `version.md`'s `# X.Y` becomes package version `X.Y.0`, which the
release workflow already builds as `APP_VERSION` and puts in the zip name. `release.yml` today
appends `.0` even to a 3-part heading (`0.68.1` → `0.68.1.0`, which `vpk` rejects); PR 1 appends
it only to a 2-part version, so `X.Y` keeps its asset name `frename-windows-x64-vX.Y.0.zip`. The version shown in
Settings comes from Velopack's locator when packaged, else `APP_VERSION` baked in at build time
(`option_env!`), else `dev`.

## CI

### Building (both `ci.yml` and `release.yml`)

The 52 MB `gstreamer-minimal-msvc-x86_64.zip` is removed from the repository. The Windows jobs get
GStreamer from the pinned official **runtime** and **development** packages (the development one
has the `pkg-config` files the `gstreamer` crates build against), checksum-checked and cached with
`actions/cache` keyed by version. The research doc (and GSTREAMER_SETUP.md today) names MSIs,
extracted with `msiexec /a`. The exact file names and URLs could not be checked from the design
session (the download site is blocked there); PR 1's first step checks them in CI and pins URL
and SHA-256 in the workflow. If the pinned version ships only as an `.exe` installer, the build job
installs it silently into a temp folder instead (the build runner does not need to stay clean; the
smoke-test runner below never gets it).

### Self-test (`frename.exe --self-test <folder>`)

Runs without opening a window and exits `0` on success, `1` otherwise, writing a report to the
log (CI prints it):
- the GStreamer registry has `playbin`, `qtdemux`, `matroskademux`, `avidemux`, `capssetter`,
  `avdec_h264`, `avdec_h265`, `avdec_aac`, `dav1ddec`, `opusdec`;
- every clip in `<folder>` plays through frename's own pipeline builder with `audio-sink=fakesink`
  until the first video sample or end of stream, 20 s timeout each.

Fixtures: tiny clips (a few KB each) under `tests/media/`: H.264/AAC `.mp4`, HEVC `.mov`,
VP9/Opus `.webm`, AV1 `.mkv`, MPEG-4/MP3 `.avi` — trimmed from the clips already in `tests/folder`
where they have that codec, generated with `ffmpeg` otherwise (the command lines go in
`tests/media/README.md`) — plus a new variable-frame-rate clip (`ffmpeg` with `-fps_mode vfr` from
a source with dropped frames) for the
`capssetter` retry exists for. `--self-test` is shared with #11 (Linux) and #15 (macOS).

### Installer smoke test

The `ci-windows` job, after the release build, bundles GStreamer, packs with `vpk pack` as version
`0.0.0` and uploads `Setup.exe` and the portable zip as artifacts. It cannot test them itself: it
has the build GStreamer on `PATH`, which would hide a DLL missing from the bundle. A new job,
`ci-windows-install` (`needs: ci-windows`, a fresh `windows-2022` runner), downloads the artifacts
and:
1. fails if a system GStreamer is on the runner (`GSTREAMER_1_0_ROOT_MSVC_X86_64`, or
   `gst-launch-1.0` on `PATH`), so the test proves the clean-machine case;
2. runs `Setup.exe --silent` with `Start-Process -Wait` (it is a GUI exe too), walks
   `dumpbin /dependents` over every `.exe` and `.dll` under `current\` and fails on any import that
   is neither in `current\` nor a Windows system DLL (KnownDLLs, `api-ms-win-*`, `ucrtbase`) — the
   runner has the VC++ runtime in `System32`, which would otherwise hide a missing app-local CRT
   DLL, and a plugin that fails to load is dropped silently, then runs the
   installed app's self-test as `current\frename.exe` directly, not through the root stub, whose
   exit-code behaviour is not documented;
3. installs an **older** official GStreamer runtime system-wide (`msiexec /i … /qn`), put it on
   `PATH`, set `GST_PLUGIN_PATH` to its plugins, and run the self-test again: it must still pass,
   which proves a system GStreamer does not interfere;
4. unzips the portable zip to a temp folder and runs its self-test too.

Release builds are GUI-subsystem exes (`windows_subsystem = "windows"`), so PowerShell does not wait
for them: each self-test runs as `Start-Process -Wait -PassThru` and checks `.ExitCode`, then
prints `frename_debug.log` from the data folder (`%LocalAppData%\frename\` installed, the unzip
folder portable).

`ci-windows-install` must become a required check: the owner adds it to the `main` ruleset
(CLAUDE.md "Owner setup"; the agent cannot). Until then the nightly merge rule "every CI check is
green on the head commit" still blocks agent merges on it.

Runners have no GPU or audio device, so this covers the software decode path (research doc).

### Release (`release.yml`)

`build-windows` additionally bundles GStreamer and runs
`vpk download github --repoUrl https://github.com/Zelenov/frename` (the previous release, so
deltas are built — `docs/distributing/github-actions.mdx`). The first Velopack release follows
v0.66, which has no Velopack files; what `vpk download` does then is not documented, so the step
tolerates a failure there (`continue-on-error` with a log line) — it only costs the delta. Then
`vpk pack -u frename -v X.Y.0 -p dist\frename -e frename.exe --packTitle frename --icon frename-icon.ico`.
A new `release-install-test` job on a fresh runner runs the same steps as `ci-windows-install` on
these artifacts; the `release` job needs it, so a broken installer is never published. The
`release` job uploads through the existing `softprops/action-gh-release` step:
- `frename-windows-x64-vX.Y.0.zip` — Velopack's `frename-win-Portable.zip`, renamed (the pipeline's
  "published" check looks for this name);
- `frename-win-Setup.exe`;
- `frename-X.Y.0-full.nupkg`, `frename-X.Y.0-delta.nupkg` (when a previous release exists),
  `releases.win.json`, `assets.win.json` — what `UpdateManager` reads
  (`docs/packaging/overview.mdx`, `docs/packaging/channels.mdx`).
The legacy `RELEASES` file is not uploaded (only for Squirrel migrations — `channels.mdx`).
`vpk` is installed as a pinned `dotnet tool` version (1.2.158, matching the crate).

## Edge cases

- **The first installed version.** `0.66` and older have no updater: users install `Setup.exe`
  once by hand; from then on updates come by themselves. The README says so.
- **Update while a video is open**: the file is saved and the video unloaded before exit, so
  nothing holds files in `current\`. Velopack kills leftover processes in `current\` and asks the
  user if something else locks it (`windows.mdx`, "Updating").
- **Offline or GitHub down**: the check shows `Could not check for updates: no connection`; the
  start-up check fails silently (logged).
- **Portable copy on a read-only drive**: the update fails; the message says
  `Could not update: the folder is read-only`.
- **Two frename processes**: before **Update and restart**, frename checks for another `frename.exe`
  process from the same install and, if one runs, says `Close the other frename window first` and
  does not update — Velopack would kill it with its unsaved edits.
- **A plugin missing at runtime** (allowlist too small for some format): the file fails to play as
  today; the self-test fixture list is the guard, and a new format means a new fixture.
- **GStreamer installed by hand for older versions**: no longer needed; the README says it can be
  uninstalled, and one left installed does not interfere (smoke test step 3).
- **Existing zip users who run the new portable zip over their old folder**: their `frename.db` is
  already in the portable root, which is the data folder.
- **SmartScreen / antivirus**: unsigned; documented in the README, for `Setup.exe` and for the
  portable zip (right-click the zip → Properties → **Unblock** before extracting, so its
  `frename.exe` and `Update.exe` are not blocked). Signing is out of scope.

## Out of scope

- Code signing.
- MSI / per-machine install (`vpk --msi`).
- Linux and macOS packages (#11, #15); only `--self-test` and the data-folder rule are shared.
- Release channels / beta updates.
- Automatic download of updates without the button.

## Delivery

Two PRs, each through the review gate:
1. **Self-contained app** — data folder, `configure_bundled_gstreamer`, `--self-test` and fixtures,
   GStreamer from the official MSIs in CI, bundling, `vpk pack`, Setup + portable zip on releases,
   smoke test in the new `ci-windows-install` job, the vendored zip removed, README (Requirements
   becomes "Download `frename-win-Setup.exe`; Windows may warn once: More info → Run anyway", the
   `frename.db` "next to the executable" line is corrected, zip users are told to delete the old
   frename folder — or unzip the portable zip over it — after the first start of the new version,
   and that the GStreamer they installed can be uninstalled) and `GSTREAMER_SETUP.md` (becomes
   "building from source" only), `version.md`. Body `Refs #10`.
2. **Updates** — Settings section, start-up check, gear dot, update and restart. Body `Closes #10`.

PR 1 changes `.github/workflows/*`, which are guarded files; issue #10 asks for these CI changes
explicitly.

## Test plan

- Core unit tests: `set_app_data_dir` is used by `AppDatabase`; the "not packaged" fallback is the
  exe folder. App-crate tests: the log path and the data-folder choice (installed / portable /
  not packaged) from a locator stub.
- `configure_bundled_gstreamer`'s pure part, on every OS: with a temp folder containing a fake
  `lib/gstreamer-1.0/gstcoreelements.dll` it returns the variables to set and remove, with the
  registry in the data folder; without it, nothing.
- Import: `VACUUM INTO` from an old database with un-checkpointed WAL content keeps that content;
  the old files are unchanged; the search finds `frename.db` next to `frename.exe` up to two
  levels deep in given folders and ignores a `frename.db` without an exe next to it;
  `--self-test` with an empty data folder neither searches nor asks.
- `--self-test` in `ci-windows-install` (installed, portable, and with an older system GStreamer on
  `PATH`); a fixture that fails to decode makes the job red (checked once by hand in the PR by
  pointing it at a corrupt file).
- Updates feature: state machine tests with a fake update source (no network): up to date, update
  found, download cancelled by closing, download progress, error, not installed, batch running.
- By the owner, on Windows: install from `Setup.exe` on a machine without GStreamer; play a clip;
  after the next release, **Check for updates** finds it and **Update and restart** lands on the new
  version with settings kept.

## Open questions (with recommended answers)

1. **Settings of users coming from the zip version.** *Recommended:* the automatic search with a
   dialog only when an old database is found, plus the Settings import button for a zip kept
   elsewhere (flow 6). Rejected: starting fresh, which puts comment storage back on the default.
2. **Data folder for the installed app: removed on uninstall** (Velopack root) or kept
   (`%AppData%\frename`, roaming)? *Recommended:* the Velopack root, as the issue says
   `%LocalAppData%`. It holds folder history and settings only: the tag library is not in it
   (tags live in each folder's `.frename` file via `FolderTagStore`; the old `stored_tags` table is
   unused since 0.62), so uninstalling loses nothing that cannot be set again.
3. **Portable data stays in the portable folder**, not `%LocalAppData%`. *Recommended:* yes — a
   portable build must not write outside its folder, and that is where the old zip kept its data.
4. **Start-up check default.** *Recommended:* on, at most once a day, never downloads by itself.
5. **Keep `GSTREAMER_SETUP.md`?** *Recommended:* keep it only for building from source; the release
   no longer ships it.
