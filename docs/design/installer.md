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
   Velopack apply the update and start the new version.
4. **Silent check at startup.** A checkbox in the same section, **Check for updates when frename
   starts**, on by default. At most once per 24 hours, in the background, never blocking. When a
   newer version exists, the Settings gear in the folder controls gets a small accent dot and the
   tooltip `Update available: 0.68`; nothing is downloaded or applied without the button.
5. **Not installed** (`cargo run`, or a plain `frename.exe` copied out of a package): the Updates
   section says `Updates work in the installed version` and the button is disabled
   (`UpdateManager::new` returns `NotInstalled` then — research doc).

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

While downloading: `Downloading 0.68… 42%`, both buttons disabled. While a batch job runs,
**Update and restart** is disabled with the tooltip `Wait for the batch to finish`.

The settings window is not resizable and every section must fit (`SETTINGS_WINDOW_SIZE` in
`src/app/state.rs`); the implementation grows its height by what the section needs and checks
the screenshot.

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
  `dumpbin /dependents` from `frename.exe` and every allowlisted plugin and copies each DLL it
  finds in GStreamer's `bin\`. No GPL parts (no x264/x265, no `ugly`).
- **VC++ runtime** is copied app-local from the runner's Visual Studio redist folder, instead of
  `vpk --framework vcredist143-x64`: the framework bootstrapper installs the redistributable
  machine-wide, which asks for admin rights and breaks "no questions" on machines without it.
- **Size** is measured by CI and written into the PR; the research estimate is ~65–75 MB installed
  and ~28–35 MB for `Setup.exe`.

## Runtime changes

### Start-up order (`src/main.rs`)

1. `velopack::VelopackApp::build().run()` — must be first; it handles install/update hooks and
   exits for them (research doc; `velopack` `app.rs`).
2. `configure_bundled_gstreamer()` — Windows only, before any thread and before `gst::init`, as in
   the research doc: when `lib\gstreamer-1.0\gstcoreelements.dll` exists next to the exe, remove
   `GST_PLUGIN_PATH`, `GST_PLUGIN_PATH_1_0`, `GST_PLUGIN_SYSTEM_PATH`, `GST_PLUGIN_SCANNER`,
   `GST_REGISTRY`, and set `GST_PLUGIN_SYSTEM_PATH_1_0`, `GST_PLUGIN_SCANNER_1_0` and
   `GST_REGISTRY_1_0` (registry file in the data folder). The exe's folder is first in the Windows
   DLL search order, so a system GStreamer on `PATH` is not loaded. Without the bundled plugins
   (a developer build) nothing changes and the system GStreamer is used, as today.
3. Data folder, logging, `gst::init`, database, window — as today, with the paths below.

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
`AppDatabase::new` and the log use it.

**Migration.** A zip user who unzips the new portable zip over their old frename folder keeps
everything: the old `frename.db` sits in the portable root, which is now the data folder. For the
installed app an old `frename.db` cannot be found automatically (the old zip could be anywhere).
What it held: window size, volume, the last folder, and Settings (comment and in/out storage, the
"Commented" tag, autoplay, monochrome tags). Tags live in each folder's `.frename` file since
0.62, so no tags are lost. See open question 1.

Uninstalling removes `%LocalAppData%\frename\` with the database
(`docs/integrating/uninstalling.mdx`); tags and comments live next to the footage and stay.

### Updates

In a new `src/features/updates/` feature (messages / state / view, Elm pattern), shown inside the
Settings window:
- `UpdateManager::new(GithubSource::new("https://github.com/Zelenov/frename", None, false), …)`
  — `prerelease = false`; draft releases (branch builds) are invisible to it (research doc).
- `check_for_updates`, `download_updates` (its progress arrives on a `Sender<i16>`, mapped to a
  message) run in
  `Task::perform` over `spawn_blocking`; they are blocking calls.
- Apply: the app saves the open file through the existing unload → `apply_file_updated` path,
  then calls `wait_exit_then_apply_updates` and closes the window. Velopack applies the update
  after frename exits and starts the new version.
- The last check time is stored in `app_settings`, so the start-up check runs at most once a day
  (GitHub's unauthenticated API allows 60 requests per hour per IP).

### Versions

Velopack needs 3-part semver. `version.md`'s `# X.Y` becomes package version `X.Y.0`, which the
release workflow already builds as `APP_VERSION` and puts in the zip name. The version shown in
Settings comes from Velopack's locator when packaged, else `APP_VERSION` baked in at build time
(`option_env!`), else `dev`.

## CI

### Building (both `ci.yml` and `release.yml`)

The 52 MB `gstreamer-minimal-msvc-x86_64.zip` is removed from the repository. The Windows jobs get
GStreamer from the pinned official **runtime** and **development** MSIs (the development one has
the `pkg-config` files the `gstreamer` crates build against), extracted with `msiexec /a`,
checksum-checked, and cached with `actions/cache` keyed by version.

### Self-test (`frename.exe --self-test <folder>`)

Runs without opening a window and exits `0` on success, `1` otherwise, writing a report to the
log (CI prints it):
- the GStreamer registry has `playbin`, `qtdemux`, `matroskademux`, `avidemux`, `capssetter`,
  `avdec_h264`, `avdec_h265`, `avdec_aac`, `dav1ddec`, `opusdec`;
- every clip in `<folder>` plays through frename's own pipeline builder with `audio-sink=fakesink`
  until the first video sample or end of stream, 20 s timeout each.

Fixtures: tiny clips (a few KB each) committed under `tests/media/`: H.264/AAC `.mp4`, HEVC `.mov`,
VP9/Opus `.webm`, AV1 `.mkv`, MPEG-4/MP3 `.avi`, plus the existing variable-frame-rate case the
`capssetter` retry exists for. `--self-test` is shared with #11 (Linux) and #15 (macOS).

### Installer smoke test

In the existing `ci-windows` job (a required check), after the release build:
1. bundle GStreamer and pack with `vpk pack` as version `0.0.0`;
2. fail if a system GStreamer is on the runner (`GSTREAMER_1_0_ROOT_MSVC_X86_64` or `gst-*` on
   `PATH`), so the test proves the clean-machine case;
3. `Setup.exe --silent`, then `%LocalAppData%\frename\current\frename.exe --self-test tests\media`;
4. install an **older** official GStreamer runtime system-wide (`msiexec /i … /qn`), put it on
   `PATH`, set `GST_PLUGIN_PATH` to its plugins, and run the self-test again: it must still pass,
   which proves a system GStreamer does not interfere;
5. unzip the portable zip to a temp folder and run its self-test too.

Runners have no GPU or audio device, so this covers the software decode path (research doc).

### Release (`release.yml`)

`build-windows` additionally: bundles, runs the smoke test, then
`vpk download github --repoUrl https://github.com/Zelenov/frename` (the previous release, so
deltas are built — `docs/distributing/github-actions.mdx`) and
`vpk pack -u frename -v X.Y.0 -p dist\frename -e frename.exe --packTitle frename --icon frename-icon.ico`.
The `release` job uploads through the existing `softprops/action-gh-release` step:
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
- **Two frename windows/processes**: the update closes the one that applies it; Velopack kills the
  others running from `current\`.
- **A plugin missing at runtime** (allowlist too small for some format): the file fails to play as
  today; the self-test fixture list is the guard, and a new format means a new fixture.
- **Existing zip users who run the new portable zip over their old folder**: their `frename.db` is
  already in the portable root, which is the data folder.
- **SmartScreen / antivirus**: unsigned; documented in the README. Signing is out of scope.

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
   smoke test in `ci-windows`, the vendored zip removed, README and `GSTREAMER_SETUP.md` (becomes
   "building from source" only), `version.md`. Body `Refs #10`.
2. **Updates** — Settings section, start-up check, gear dot, update and restart. Body `Closes #10`.

PR 1 changes `.github/workflows/*`, which are guarded files; issue #10 asks for these CI changes
explicitly.

## Test plan

- Core unit tests: `set_app_data_dir` is used by `AppDatabase` and the log path; the "not packaged"
  fallback is the exe folder.
- `configure_bundled_gstreamer`: a unit test with a temp folder containing a fake
  `lib\gstreamer-1.0\gstcoreelements.dll` checks the variables set and removed; without it, none
  change.
- `--self-test` in `ci-windows` (installed, portable, and with an older system GStreamer on
  `PATH`); a fixture that fails to decode makes the job red (checked once by hand in the PR by
  pointing it at a corrupt file).
- Updates feature: state machine tests with a fake update source (no network): up to date, update
  found, download progress, error, not installed, batch running.
- By the owner, on Windows: install from `Setup.exe` on a machine without GStreamer; play a clip;
  after the next release, **Check for updates** finds it and **Update and restart** lands on the new
  version with settings kept.

## Open questions (with recommended answers)

1. **Settings of users coming from the zip version.** The installed app cannot find the old
   `frename.db`. *Recommended:* accept it — tags are in `.frename` files; the README install section
   says "Settings start fresh; your tags and comments are in your folders". Alternative: a one-time
   "Import settings from the old frename folder…" button in Settings.
2. **Data folder for the installed app: removed on uninstall** (Velopack root) or kept
   (`%AppData%\frename`, roaming)? *Recommended:* the Velopack root, as the issue says
   `%LocalAppData%`; nothing irreplaceable lives in it.
3. **Start-up check default.** *Recommended:* on, at most once a day, never downloads by itself.
4. **Keep `GSTREAMER_SETUP.md`?** *Recommended:* keep it only for building from source; the release
   no longer ships it.
