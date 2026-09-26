# Generate subtitles (sonisub inside frename)

Design for issue #12. [Zelenov/sonisub](https://github.com/Zelenov/sonisub) turns a video's speech
into a `.srt` with Soniox; this brings it into frename as a batch action.

## Problem

Subtitles make a clip searchable and readable at a glance (frename shows them under the video and
in a list since 0.63), and the AI description of #17 uses them. Today the editor has to run the
sonisub command line on a folder separately, with a key in an environment variable.

## User flows

1. **Set the key once.** Settings → new section **Subtitles**: a *Soniox API key* field (masked,
   *Show*, **Save**; once saved `Key saved` with **Replace** / **Remove**, and the line
   `Saved in Windows Credential Manager on this computer.`), *Languages* (the languages spoken in
   the footage, as hints: checkboxes English and Russian on by default, plus a short list), and
   *Line length*: **Short** (default; at most 2 lines of 50 characters and 8 s per cue, sonisub's
   defaults), **One sentence per cue** (no length limit; sonisub's `-u`).
2. **Run it.** Batch mode → check clips → action **Generate subtitles**. Before *Run*, the panel
   shows the plan, computed from the files and their headers only (sonisub's `batch::plan`):
   - `8 videos to transcribe, 41 min of audio · about $0.07` (the price per hour learned from the
     account's last 30 days of Soniox usage, or sonisub's $0.10/h fallback when there is none;
     `Estimating…` while it is fetched and headers are read);
   - a line for what is skipped: `3 already have subtitles, 2 have no audio, 1 photo are skipped.`;
   - a *Names and terms* field (optional, e.g. `Nairobi, Maasai`; sonisub's context, improves
     recognition) and a checkbox **Replace existing subtitles** (off);
   - `The audio of these videos is sent to Soniox.`
   The run button reads `Transcribe 8 videos` and is disabled while estimating or without a key
   (then `Set a Soniox API key in Settings` with **Open Settings**).
3. **Progress and cancel** like every batch action; *Cancel* stops the current upload or wait
   within a few seconds (sonisub checks its cancel flag during extraction, upload retries and
   polling) and sonisub deletes what it created on Soniox. Failed files show their reason.
4. **Result.** Each transcribed video gets `clip.srt` next to it (the path frename already reads:
   same folder and stem). The open file's subtitles appear at once; the list's subtitles marker
   updates. The job's summary line adds `Soniox: 41 min · $0.07`, also written to the log.

## How frename uses sonisub

- sonisub is a **library dependency**, pinned to a commit:
  `sonisub = { git = "https://github.com/Zelenov/sonisub", rev = "<sha>" }`. Its `lib.rs` already
  exposes what is needed: `job::process(input, output, &Options, Some(&Client))`,
  `soniox::Client::new(base, key)` / `check_auth`, `batch::plan` + `Totals` for the plan,
  `usage::fetch` + `Summary::stt_usd_per_hour` for the price, `job::Outcome` for the result.
- Per file, on the batch's blocking thread: `job::process(video, subtitle_path(video), &opts,
  Some(&client))`. `Outcome::Written` → done; `Skipped` → unchanged; `NoAudio` / `NoSpeech` →
  unchanged with that reason; `Err` → failed with the error's text.
- **Progress bars:** sonisub draws `indicatif` bars; frename passes a `MultiProgress` with a hidden
  draw target, so nothing is drawn (a release build has no console).
- **Options** from Settings: `languages`, `layout` (`Layout::default()` or `Layout::unlimited()`),
  `context` and `force` from the panel; `keep_json` off; `reference` = `frename`, so the run's cost
  can be found in the usage logs.

### Changes in sonisub (a PR there first)

- **Cancel:** sonisub's cancel flag is process-wide and can only be set (`cancel::interrupt`). A
  second job after a cancelled one would stop at once. Add `cancel::reset()`; frename resets at
  the start of each job and calls `interrupt()` on *Cancel*. (One job runs at a time.)
- Nothing else: the API above is public already. sonisub's own tests and release are unaffected.

## Things sonisub leaves next to the video

- `clip.srt` (wanted).
- For a clip with audio but no speech, an empty `clip.soniox.json` marker, so a re-run does not pay
  again. frename's file list shows only videos, so it is not listed. frename renames `.srt` files
  with their video (0.63); this PR makes it rename the `.soniox.json` marker too, so the marker
  keeps matching.
- Temporary audio goes to the system temp folder and is deleted, also on errors and cancel.

## Key storage

The Soniox key is kept with the operating system's credential store (`keyring` 3; Windows
Credential Manager; Secret Service on Linux), service `frename`, user `soniox-api-key`, saved on
**Save** through a `Task`, read lazily through a `Task` the first time the section or the action
is shown; never in `frename.db`, logs or the repository. When there is no store, the field says
`Cannot store the key on this system` and the action is disabled. (#17's design uses the same
mechanism for the Anthropic key; whichever ships first adds it, the other reuses it.)

## Edge cases

- **The open video:** transcribing does not need the file unlocked (sonisub only reads it); the
  batch handles the open file as every action does.
- **A `.srt` appears for the open file:** the player loads subtitles when a video opens; after the
  job, the open file's subtitles are reloaded so they show at once.
- **Files without audio, photos:** skipped, named in the skipped line, unchanged.
- **Long files:** no limit; the plan shows their audio length and cost.
- **Unreadable header:** sonisub plans them as "length unknown"; the estimate says
  `+ 2 of unknown length`.
- **Key rejected (401):** frename calls sonisub's `check_auth` before the first file; the job does
  not start and the panel says `Soniox rejected the key`.
- **A `.soniox.json` next to the video** (from an earlier sonisub run): sonisub uses it and builds
  the `.srt` without calling the API (free); the plan counts it as cached.
- **Installer (#10):** nothing extra ships; sonisub is compiled into frename (its audio decoding is
  pure Rust; `ffmpeg` is used only if it is on `PATH` and the built-in decoder fails).

## Out of scope

- sonisub's other outputs (Premiere subtitle formats), speaker labels and names, custom temp dirs.
- Generating subtitles for photos or audio-only files.
- Choosing the Soniox model.

## Test plan

- sonisub: a unit test for `cancel::reset`.
- frename, no network: the action on a video whose `clip.soniox.json` fixture (copied from sonisub's
  `tests/fixtures`) lies next to it writes the expected `.srt` without a client call; the plan counts
  skip / cached / no audio / photos; outcome → result mapping; cancel resets between jobs; renaming a
  video renames its `.soniox.json` marker.
- Live test only when `SONIOX_API_KEY` is set (skipped otherwise): one short clip from
  `tests/self-test-clips.txt` gets a non-empty `.srt`.
- UI: the Settings gear is not reachable under Xvfb until #14; the `view` code is reviewed with a
  written description.

## Delivery

1. A sonisub PR: `cancel::reset()` + test.
2. The frename PR: dependency pinned to that commit, Settings section, the action, key storage,
   the marker rename, README and `version.md`. Body `Closes #12`.

## Open questions (with recommended answers)

1. **Library or bundled command line?** *Recommended:* library — one binary, one progress and cancel
   model, no process management; the issue prefers it.
2. **Default languages.** *Recommended:* English + Russian (sonisub's default), editable.
3. **Line length choices.** *Recommended:* the two presets above instead of raw numbers — the issue
   asks for "only the options that matter".
4. **Key storage shared with #17.** *Recommended:* the OS credential store for both keys.
