# Generate subtitles (sonisub inside frename)

Design for issue #12. [Zelenov/sonisub](https://github.com/Zelenov/sonisub) turns a video's speech
into a `.srt` with Soniox; this brings it into frename as a batch action.

## Problem

Subtitles make a clip searchable and readable at a glance (frename shows them under the video and
in a list since 0.63), and the AI description of #17 uses them. Today the editor has to run the
sonisub command line on a folder separately, with a key in an environment variable.

## User flows

1. **Set the key once.** Settings → new section **Subtitles**: a *Soniox API key* field (masked,
   *Show*, **Save**; once saved `Key saved` with **Replace** / **Remove**, and a line naming where
   it is kept on this system, e.g. `Saved in Windows Credential Manager on this computer.`, and
   `Get a key at console.soniox.com.` and `The audio is sent to Soniox to transcribe it.`),
   *Languages* (the languages spoken in the footage, as hints: checkboxes English and Russian on by
   default, plus Ukrainian, German, Spanish, French; none checked means `Detect automatically`, no
   hints), and *Cue length*: **Short** (default: one line
   of up to 100 characters and at most 8 s per cue — sonisub's defaults, `Layout::default()`) or
   **One sentence per cue** (no length limit; `Layout::unlimited()`). See Settings layout.
2. **Run it.** Batch mode → check clips → action **Generate subtitles**. Before *Run*, the panel
   shows the plan, computed from the files and their headers only (sonisub's `batch::plan`):
   - `8 videos to transcribe, 41 min of audio · about $0.07` (the price per hour learned from the
     account's last 30 days of Soniox usage; when there is none, or the lookup fails or takes over
     10 s, sonisub's $0.10/h fallback, shown as `about $0.07 (typical price)`; `Estimating…` while
     it is fetched and headers are read, never longer than the timeout; `+ 2 of unknown length`
     when a header cannot be read but ffmpeg is there to try);
   - one line per kind of file not sent, singular/plural correct: `3 already have subtitles`,
     `2 rebuilt free from a saved transcript`, `1 had no speech last time`, `2 have no audio`,
     `1 shares its subtitle name with another video` (e.g. `clip.MP4` and `clip.MOV` both map to
     `clip.srt`; the second is skipped, also with Replace on), `3 cannot be read (install ffmpeg to
     read .mkv, .m2ts, .avi …)` for files the built-in decoder cannot read when no `ffmpeg` is on
     `PATH`;
   - a *Names and terms* field (optional, e.g. `Nairobi, Maasai`; sonisub's context, improves
     recognition) and a checkbox **Replace existing subtitles** (off) with the hint
     `Transcribes again; costs as shown.` — toggling it recomputes the plan and the price (it also
     ignores saved transcripts, sonisub's `force`);
   - `The audio of these videos is sent to Soniox and deleted there afterwards.`
   The run button reads `Transcribe 8 videos` — deliberately not `Run on N files`, because this
   run costs money and the number must be the videos actually sent — and is disabled while
   estimating or without a key (then `Set a Soniox API key in Settings` with **Open Settings**).
   With nothing to send but free rebuilds it reads `Build 2 subtitles (free)`; with nothing to do
   at all it is disabled and the panel says `Nothing to transcribe`.
   **While it runs the app is busy, as with every batch job** (see During the run); the plan says
   how long that may take: `About 25 min` (from the audio length and recent speed; a first run says
   `a few minutes per hour of audio`).
3. **Progress and cancel** like every batch action; *Cancel* stops the current file within a few
   seconds (sonisub checks the job's cancel token during extraction, upload retries and polling)
   and sonisub deletes what it created on Soniox; that file counts as not reached, not as
   failed (an error from `process` while the job's own cancel token is set is read as cancelled).
   **Fatal Soniox errors stop the job:** when an error is fatal in sonisub's sense
   (`ApiError::is_fatal`: balance or budget exhausted, 402/403), the job stops with
   `Soniox stopped: <reason>` (with **Open Settings** when the reason is the key), instead of extracting and uploading every remaining file only to
   fail it the same way.
4. **Result.** Each transcribed video gets `clip.srt` next to it (the path frename already reads:
   same folder and stem). The counts line reads `✓ 8 subtitled – 3 unchanged ✗ 1 failed` (the
   action names its done state), and every file that was not subtitled is listed with its reason
   (`no speech`, `no audio`, `already had subtitles`, `audio format not supported`,
   `failed: <reason>`) in the existing scrollable result list, headed `Not subtitled:` for this
   action (see Batch changes). The list's subtitles marker, the "with subtitles" filter and its
   count are updated from `subtitle_path(..).is_file()` as each item finishes; the reopened file
   shows its new subtitles when the job ends. The
   job's summary line adds `Soniox: 41 min · about $0.07` (seconds sent, `Outcome::uploaded_s`,
   times the price used for the estimate), also written to the log.

## Settings layout

The Settings window is a fixed 560×560 with four sections already; a fifth does not fit
(`SETTINGS_WINDOW_SIZE` in `src/app/state.rs`: "a setting below the bottom edge is simply not
seen"). Its content becomes a scrollable column; the window keeps its size. **Open Settings** from
the action opens it scrolled to the Subtitles section. (#17's design needs the same; whichever ships
first adds it.)

## During the run

The job runs like every batch job today: the open video is closed, other files cannot be opened
or edited, and the folder cannot be changed until the job ends or is cancelled (`run_batch` and the
guards in `folder_workspace/state.rs`). For a job that can take many minutes this is a real
limitation, accepted for this version because every way around it races with the editor's own
saves (an XMP write into the clip being read, a rename moving a `.srt` sonisub is about to write).
So the plan shows the expected wait before *Run*, and the job panel says
`Closing frename stops the run; finished subtitles are kept.` A background mode that lets the
editor keep working is filed as an `idea` for later.

## Batch changes

The shared batch job cannot carry this action today: `Operation` is `Copy + Eq` with
`run(self, path)`, `ItemResult` has only a status and an update (a failed file shows only its
name), `view` never sees the checked files, and cancel is a flag on the UI side that a running file
cannot see. This PR changes the shared code (the same changes #17's design lists):
- `Operation::GenerateSubtitles(Arc<SubtitleJob>)`, where `SubtitleJob` holds the `job::Options`,
  the key and a per-job cancel token; `Operation` drops `Copy` and `Eq` (stays `Clone`).
- `ItemResult` gains `reason: Option<String>` (shown after the file's name) and
  `stop_job: Option<String>` (stops the job, the rest not reached, reason as the summary line);
  other actions leave both `None`.
- *Cancel* also sets the running job's cancel token.
- Each action gives the run button its own label and enabled state, names its done state in the
  counts line and its result list's heading; the others keep `Run on N files`, `changed` and
  `Failed (see the log for why):`.
- The plan: when the checked set, the Replace checkbox or the key changes, the action starts a
  `Task` that works out shared subtitle names (passed to the job as a skip set keyed by `FileId`,
  so `run` returns
  them as skipped even with Replace on), runs `batch::plan` and the price lookup (10 s timeout) on
  a blocking thread; each request carries a generation number, and an answer for an older one is
  dropped. The panel shows `Estimating…` until the current one arrives.
- Every Soniox client call (`check_auth`, `usage::fetch`, the per-file `process`) runs on a
  blocking thread, and the client is created and dropped there: `reqwest::blocking::Client` panics
  if created or dropped on an async runtime thread.

## How frename uses sonisub

- sonisub is a **library dependency**, pinned to a commit, without its command-line parts:
  `sonisub = { git = "https://github.com/Zelenov/sonisub", rev = "<sha>", default-features = false }`. Its `lib.rs` already
  exposes what is needed: `job::process(input, output, &Options, Some(&Client))`,
  `soniox::Client::new(base, key)` / `check_auth`, `batch::plan` + `Totals` for the plan,
  `usage::fetch` + `Summary::stt_usd_per_hour` for the price, `job::Outcome` for the result.
- Per file, on the batch's blocking thread: `job::process(video, subtitle_path(video), &opts,
  Some(&client))`. `Outcome::Written` → done; `Skipped` → unchanged; `NoAudio` / `NoSpeech` →
  unchanged with that reason; `Err` → failed with the error's text.
- **Progress bars:** sonisub draws `indicatif` bars; frename passes a `MultiProgress` with a hidden
  draw target, so nothing is drawn (a release build has no console).
- **Options** from Settings: `languages`, `layout` (`Layout::default()` or `Layout::unlimited()`),
  `context` and `force` from the panel; `keep_json` off; `audio:`
  `Backend::Auto` (the built-in decoder, then `ffmpeg` when it is on `PATH`, which frename's own
  file list includes: .mkv, .m2ts, .avi, .wmv … need it);
  `reference` = `frename-<millis>`, one per run, so a run's usage can be told apart.

### Changes in sonisub (a PR there first)

- **Cancel:** sonisub's cancel flag is process-wide and can only be set (`cancel::interrupt`); a
  second job after a cancelled one would stop at once. Add a per-job cancel token
  (`Arc<AtomicBool>`) that reaches every place `interrupted()` is checked: `job::Options::cancel`,
  `Client::with_cancel(token)` (used by `Client::send`'s retries and the upload progress reader)
  and a `cancel` argument to `audio::extract` (native decoding and the ffmpeg path). Either the
  token or the global flag stops the work; the CLI keeps the global flag. These are public
  signature changes, listed in the sonisub PR and its version notes.
- **No console windows:** on Windows every `Command::new` in `audio.rs` (`ffmpeg` in `with_ffmpeg`,
  `ffprobe` in `probe_duration`) sets `CREATE_NO_WINDOW` (`CommandExt::creation_flags`), so a GUI app
  using the `Auto` backend does not flash a console per file.
- **A `cli` feature** (default on) for `clap` and `ctrlc`: `audio::Backend` derives
  `clap::ValueEnum` only under it, and the binary gets `[[bin]] required-features = ["cli"]`, so
  frename, with `default-features = false`, builds neither, and the crate builds without the
  feature.
- **Cleanup warnings:** `RemoteGuard` reports a failed delete on Soniox with `eprintln!`, which a
  GUI release build throws away; it logs through the `log` crate instead, and the CLI installs a
  small stderr logger for `warn` and above so its users still see the "run `sonisub purge`" hint.
- The rest of the API above is public already. sonisub's own tests and release are unaffected.
- **Build cost in frename:** sonisub brings `reqwest` 0.13 with rustls, whose crypto is
  `aws-lc-rs` / `aws-lc-sys` (a C build; frename has no HTTP stack yet), `symphonia` and `flacenc`.
  The PR records the release binary size and the CI build time before/after and needs both CI jobs
  green (Windows and Linux).

## Things sonisub leaves next to the video

- `clip.srt` (wanted).
- For a clip with audio but no speech, an empty `clip.soniox.json` marker, so a re-run does not pay
  again. frename's file list shows only videos, so it is not listed. frename renames `.srt` files
  with their video (0.63); this PR makes it rename the `.soniox.json` marker too, so the marker
  keeps matching.
- Temporary audio goes to the system temp folder and is deleted, also on errors and cancel.

## Key storage

The Soniox key is kept with the operating system's credential store through `keyring = "3"`,
service `frename`, user `soniox-api-key`, saved on **Save** through a `Task`, read lazily through a
`Task` the first time the section or the action is shown; never in `frename.db`, logs or the
repository. keyring 3 has no default backend and silently falls back to an in-memory mock without
one, so the features are named per system: `windows-native` (Windows Credential Manager),
`apple-native` (macOS Keychain), `sync-secret-service` + `crypto-rust` + `vendored` (Linux Secret
Service; `vendored` builds dbus, so no `libdbus-dev` is needed on CI or in the AppImage). The
"Saved in …" line names the system's store. Where there is no store (a Linux desktop without Secret
Service, headless CI), the field says `Cannot store the key on this system`; frename then reads
the `SONIOX_API_KEY` environment variable itself (the sonisub library does not) and uses it if
set; the action is disabled otherwise. (#17's design uses the same mechanism for the Anthropic key; whichever ships first adds
it, the other reuses it.)

## Edge cases

- **The open video:** closed for the job and reopened after it, as for every batch job.
- **App closed during the run:** the job stops like any batch job; finished `.srt` files are kept,
  and sonisub deletes the current file's upload on Soniox.
- **Files without audio:** skipped, named in the skipped lines, unchanged. (The file list holds only
  videos, so nothing else can be checked; the job still ignores any path that is not a video.)
- **Two videos with the same stem** (`clip.MP4`, `clip.MOV`): both map to `clip.srt`; the plan skips
  the second with `shares its subtitle name with another video`.
- **Long files:** no limit; the plan shows their audio length and cost.
- **Unreadable header:** sonisub plans them as "length unknown"; the estimate says
  `+ 2 of unknown length`.
- **Key rejected (401):** frename calls sonisub's `check_auth` before the first file; the job does
  not start and the panel says `Soniox rejected the key`.
- **A `.soniox.json` next to the video** (from an earlier sonisub run): sonisub uses it and builds
  the `.srt` without calling the API (free); the plan counts it as cached.
- **Installer (#10):** nothing extra ships; sonisub is compiled into frename (its audio decoding is
  pure Rust for MP4/MOV/M4A with AAC, MP3, WAV, FLAC, OGG; other containers need `ffmpeg` on `PATH`,
  which the plan says). Bundling ffmpeg is out of scope.

## Out of scope

- sonisub's other outputs (Premiere subtitle formats), speaker labels and names, custom temp dirs.
- Generating subtitles for photos or audio-only files.
- Choosing the Soniox model.

## Test plan

- sonisub: the per-job cancel token stops `process`, an upload and an extraction (and the global flag
  still does); the crate builds with `--no-default-features`.
- frename, no network: the action on a video whose `clip.soniox.json` fixture (copied from sonisub's
  `tests/fixtures`) lies next to it writes the expected `.srt` without a client call; the plan counts
  skip / cached / cached silent / no audio / shared names / unreadable without ffmpeg; the price
  lookup falls back on error and timeout; outcome → result mapping with
  reasons; a fatal error stops the job; a cancelled file is not reached, not failed; a second job
  after a cancelled one runs (and a cancelled item is not reached, not failed); the subtitles marker
  and count update per item; no languages checked sends no hints; renaming a video
  renames its `.soniox.json` marker; keyring round trip on Windows CI, and "unavailable" (not a mock)
  on Linux CI.
- Live test only when `SONIOX_API_KEY` is set (skipped otherwise): one short clip from
  `tests/self-test-clips.txt` gets a non-empty `.srt`.
- UI: the Settings gear is not reachable under Xvfb until #14; the `view` code is reviewed with a
  written description.

## Delivery

1. A sonisub PR: per-job cancel token, `CREATE_NO_WINDOW` for ffmpeg, `cli` feature, cleanup
   warnings through `log`, tests.
2. The frename PR: dependency pinned to that commit, Settings section, the action, key storage,
   the marker rename, README and `version.md`. Body `Closes #12`.

## Open questions (with recommended answers)

1. **Library or bundled command line?** *Recommended:* library — one binary, one progress and cancel
   model, no process management; the issue prefers it.
2. **Default languages.** *Recommended:* English + Russian (sonisub's default), editable.
3. **Line length choices.** *Recommended:* the two presets above instead of raw numbers — the issue
   asks for "only the options that matter".
4. **Key storage shared with #17.** *Recommended:* the OS credential store for both keys.
