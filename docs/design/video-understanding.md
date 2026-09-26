# AI video understanding — stage 1

Design for issue #17, **stage 1 only** ("one provider, sampled frames + transcript → summary +
segments into the comment AI block; batch action with cost estimate, progress, cancel"). Stages
2 and 3 get their own sections in later PRs, as the issue asks.
Research: [`docs/research/video-understanding.md`](../research/video-understanding.md).

## Problem

An editor with a folder of hundreds of clips wants to know what happens in each one, and when,
without watching them. Today the only text per clip is what the editor typed into the comment.

## What stage 1 does

A batch action, **Describe with AI**, for the checked videos. For each clip it samples frames,
adds the subtitles when a `.srt` is next to the video, sends both to Claude, and writes a
summary and time-ranged segments into the clip's comment, inside a marked **AI block** that a
re-run replaces and that never touches the editor's own text.

Out of stage 1: tags and tag suggestions, markers, clustering, other providers, local models,
batch-API pricing, caching across runs (stages 2–3).

## User flows

1. **Set the key once.** Settings → new section **AI** (last section; see Settings layout):
   an *Anthropic API key* field (masked; *Show*, **Save** and, while replacing, *Cancel* under it;
   `Enter` saves) and the line `Save keeps it in Windows Credential Manager on this computer.`
   (the macOS / Linux store's name on those systems). Once a key is saved the field shows
   `Key saved` with **Replace** and **Remove** buttons instead, and the line reads `Saved in …`;
   Remove asks first (`Remove the saved key? You will need to paste it again.`), since a key is
   shown only once when it is made. Without a password store the section says so and what would
   help (`The system keyring could not be opened` / `It may be locked, or there is none (such as
   GNOME Keyring or KWallet).`; the batch panel says the same with an **Open Settings** link,
   which reads the state again). Save or Remove failing says so in plain words; the store's own
   error goes to the log. The key's state is read
   each time Settings opens, so a keyring unlocked meanwhile is noticed; *Description language*: a dropdown whose first entry reads
   **Same as the subtitles (English if none)**, then English, Russian, Ukrainian, German, Spanish,
   French. A line of text says where to get a key. The model is Claude Haiku 4.5; a
   model or provider choice comes with stage 3.
2. **Run it.** Batch mode → check clips → action **Describe with AI**. The action panel shows,
   before *Run*:
   - `12 videos, 38 min · about $0.20 with Claude Haiku 4.5` — or `Estimating… 340 / 2000`
     while durations and comments are read (see Cost), or `No videos to describe.`; amounts
     under a cent read `under $0.01`;
   - `Takes about 7 min. The folder is locked until it ends. Cancel keeps the videos already
     described; running it again skips them.` — one file at a time, like every batch action, so
     the editor knows before paying; estimated as 15 s per request plus 0.3 s per frame, rounded
     to minutes or hours;
   - when some checked clips are skipped, a line in list form (no plural traps):
     `Skipped: 3 already described, 1 over 30 min, 200 photos, 1 unreadable.`; when some have no
     `.srt`, another: `Without subtitles (only the picture is described): 4 videos.`;
   - `Descriptions in the subtitles' language (English if none)` (or `Descriptions in Russian`)
     with a *Change* link to Settings;
   - (in the action's description at the top) that frames and subtitles are sent to Anthropic;
   - a checkbox **Redo videos that already have an AI description** (off by default).
   The run button reads `Describe 12 videos` (the videos that will actually be sent, not the
   checked count; the job runs over exactly those, so its progress reads `0 / 12`; the action
   hands `start_batch` the list) and is disabled while the panel says `Estimating…` or when no
   video would be sent, so nothing runs without a price shown. Without a key, it is also disabled and the panel shows `Set an Anthropic API key in Settings`
   with an **Open Settings** button (the existing `ActionMessage::OpenSettings`), which opens
   Settings scrolled to the AI section. The options scroll; the run button and the key line
   stay pinned under them, so a small window or a job report never hides them.
3. **Progress and cancel** work like every batch action: per-file green/red result. *Cancel*
   changes the panel to `Stopping…` (as for every action) and takes effect within a second
   between frame samples, between requests and during retry waits: a video stopped there ends
   as not reached; a request already sent finishes and its video is written (at most its
   timeout, see Request).
   A key removed while the job runs stops it: `Stopped: no Anthropic API key. Set one in Settings.`
   Failed files are listed with their reason (see Batch changes):
   `Anthropic rejected the key`, `Network error`, `No answer in time`,
   `Video could not be read`, `The answer was too long` (`max_tokens`), `Claude declined to
   describe it` (`refusal`); the raw stop reason goes to the log. Clips skipped as announced
   (already described, over 30 min, photos) are not in the job; the panel named them before it
   ran. Three files in a row failing on the connection (or timing out) stop the job:
   `Stopped: no connection to Anthropic. Run it again to describe the rest.`
   A rejected key (401/403) stops the job: `Stopped: Anthropic rejected the key. Check it in Settings.`
4. **Read the result.** In the comment area, the editable box holds only the editor's text; the
   AI block is shown under it, read-only, in muted text: collapsed to its summary line by default,
   with a **Show segments** toggle that expands it inside a scrollable area of at most 6 lines, so
   the tag grid above keeps its room. A small **Remove AI description** button asks
   `Remove the AI description? Getting it back needs a new AI run.` first (comment edits are not in the undo
   history). Typing can therefore never damage the block or mix with it. Premiere and
   `.comment.txt` get the whole comment (editor text, blank line, AI block); a clip with no text
   of the editor's shows the AI summary on the comment's first line (see The AI block).
5. **Re-run** (with *Redo*): only the AI block is replaced.
6. **Spend.** When the job ends — finished, cancelled or stopped — the job's summary line adds
   `AI: $0.31 (Claude Haiku 4.5)`, computed from the `usage` the API returned (`at least $0.31`
   when a request timed out, since it may have been billed without reporting it); amounts under a
   cent read `under $0.01`. The same line goes to the log, so the spend of earlier jobs can be looked up after their report is closed.

## Settings layout

The Settings window is a fixed 560×660 with four sections already (`SETTINGS_WINDOW_SIZE` in
`src/app/state.rs`); a fifth does not fit. Its content becomes a scrollable column (the window
keeps its size). The AI section:

```
AI
  Anthropic API key  [••••••••••••••••••••]  [Show]  [Save]
                     Saved in Windows Credential Manager on this computer.
                     Get a key at console.anthropic.com → API keys.
  Description language [Same as the subtitles (English if none) ▾]
```

The key is written to the credential store only on **Save**, through a `Task` (never on each
keystroke, never from `view`). Settings state keeps `has_key: Option<bool>`, read through a
`Task` the first time the AI section or the Describe action is shown (not at start-up: a locked
Linux keyring would prompt every launch) and after Save, and passes it to the batch panel; `view` never touches the
store. While the state is still being read, the run button is disabled as while `Estimating…`,
and no "set a key" message shows.

## The AI block

Plain text, readable in Premiere's Description column and in `.comment.txt`, and parseable.
The summary is the block's **first line**: how Premiere shows a multi-line Description has not
been checked, and if its column shows one line, a clip without the editor's own text then shows
the summary, not a header.

```
Shaky walk into the market; the guide talks about spices.

AI: A guide leads two tourists through a spice market, stopping at a stall to taste saffron.
0:00–0:14 Walking through the market entrance, crowd, handheld.
0:14–0:41 Close-ups of spice sacks; the guide explains prices.
0:41–1:02 Tasting at a stall, laughter, the camera turns to the street.
— Claude Haiku 4.5, 2026-09-26 —
```

- The block starts with a line beginning `AI: ` (at the start of the comment or after a blank
  line) and ends with the first following line of the form `— <model>, <YYYY-MM-DD> —` (em dashes;
  `--` also accepted so hand-typed edits survive). An `AI: ` line with no such end line is the
  editor's text and is never deleted.
- A comment has at most one AI block. Writing: the old block (if any) is removed with the one
  blank line before it; the new block is appended after the editor's text, separated by one
  blank line. The editor's text is kept byte for byte.
- Times are `m:ss` below an hour and `h:mm:ss` from an hour; ranges use `–`.
- `frename-core` owns format, parse and replace, and `editor_comment(&str) -> String` (the comment
  without its AI block: the text before it and any text after it, joined), all pure functions,
  unit-tested. Line breaks may be `\r\n` (a `.comment.txt` saved by a Windows editor): a
  trailing `\r` is ignored when matching the `AI: ` and end lines and `\r\n\r\n` is a blank
  line, while the editor's bytes stay as they are; CRLF cases are in the tests. Text found after a block (hand edits in `.comment.txt` or Premiere) is kept and
  moved before the block on the next save.
- When formatting, whitespace and line breaks inside the summary and each description are
  collapsed to single spaces, so the summary stays on one line and no model text can form an end
  line. #13 (summary from the SRT only) is meant
  to reuse this block and the provider below; see Open questions.
- **In the open file**, `FileWorkspace` keeps the two parts apart: `comment_content` (the editable
  box) holds only the editor's part, and the AI block is a separate field. `set_file`,
  `set_comment`, the F12 timestamp append and `store_comment` all work on the editor's part; the
  one function that produces the snapshot's comment joins editor part + blank line + block. So an
  F12 timestamp on a described clip lands in the editor's text, and a comment never gets two
  blocks.

The comment is saved through the normal save path, so it follows the Settings comment storage
(XMP or `.comment.txt`). **"Commented" means the editor's text only**, everywhere: a comment that
holds nothing but an AI block does not count as commented, so describing 2000 clips does not add
the tag to (and rename) every clip, and the tag keeps meaning "I commented this". Every place that
asks "does this file have a comment?" uses a non-allocating `has_editor_comment(&str) -> bool`
(the count is computed in `view` for thousands of files):
- the rule that checks the tag when a comment goes from empty to non-empty (file workspace);
- `FileTagger::sync_commented_tag` (the **Tag commented videos** batch action; its hint says
  "each checked video with a comment of yours");
- the **Commented** filter and its count in the file list (`Directory`);
- the Settings hint under "Tag videos with a comment", which says AI descriptions do not count.
The comment's first line under the file name in the list still shows whatever comes first (the
AI summary when there is no editor text). The README says so.

## Frames

- In the app (`src/`, where GStreamer lives). The clip is opened paused
  (`uridecodebin ! videoscale ! videoconvert ! appsink`, scaling
  first so 4K frames are not converted at full size; other streams go to a `fakesink`) and, for each sample time, the pipeline **seeks** there
  (`FLUSH | KEY_UNIT | SNAP_NEAREST`) and pulls one frame; the frame's real timestamp is what the
  model is told. When a snapped timestamp is not after the last frame taken, or is more than half
  an interval from the time asked for (keyframes further apart than the interval, e.g. screen
  recordings or a 10 s GOP), that sample is re-sought with `ACCURATE`, so a sparse-keyframe clip
  still gets its frames in order and no duplicate is billed; a pure function decides this and is
  tested on a 10 s GOP. Every wait is bounded: reaching paused (5 s) and each frame after a seek
  (10 s, `try_pull_preroll`); past that the file fails with `Video could not be read`, so a
  half-copied clip cannot hang the job. That decodes about one GOP per sample instead of the
  whole clip (a 20-min 4K clip would otherwise decode ~36 000 frames to keep 60).
- **Orientation** is applied in Rust, not with `videoflip`: that element (gst-plugins-good
  `videofilter`) is not in the vendored Windows GStreamer bundle, so a pipeline using it could
  fail on Windows while CI stays green. The sampler reads the `image-orientation` tag from the
  sticky tag event on the appsink's pad (`rotate-90`, `flip-rotate-270`, …) and turns or mirrors
  the already scaled 512-px frame with `image::imageops`; a phone portrait clip stored as
  1920×1080 with a rotation tag comes out upright. The mapping from tag to transform is a pure
  function, unit-tested on both systems. The pipeline then uses only elements of the vendored
  bundle: `uridecodebin` (playback), `videoscale` and `videoconvert` (both registered by its
  `videoconvertscale` plugin), `capsfilter`, `appsink` and `fakesink`. The size is not computed
  from the input: the capsfilter after `videoscale` allows 1–512 px on each side with square
  pixels, and `videoscale` fixates the largest size in that box that keeps the display aspect
  ratio (pixel aspect ratio applied). A frame that still comes out larger is scaled down in Rust
  before encoding.
- Size: the long side scaled to 512 px, aspect ratio kept (a portrait phone clip gives 288×512,
  not a letterboxed 162×288 picture in a 512×288 frame); JPEG via the `image` crate (already a
  dependency).
- One sample every 2 s (research §2), at most **60 per clip**; a longer clip is sampled evenly
  (interval = duration / 60). Clips over **30 min** are left out before the run and named in the
  skipped line (`1 over 30 min`).
  A clip shorter than 2 s gets one frame at its middle.
- Each frame is preceded by a text block `t=0:12` so the model can place segments; segment times
  are therefore accurate to the sampling interval (research §2).
- Tokens per frame (research, "Claude image cost", from Anthropic's vision docs): ⌈w/28⌉ × ⌈h/28⌉
  (512×288 → 209); the estimate says "about". 60 frames ≈ 12.5 k tokens. Haiku 4.5 has a 200K context and takes up to 100 images per
  request, so 60 fits.
- A test logs extraction time on the test clips and asserts only a generous bound (under 5 s per
  frame, so a loaded CI runner does not flake), and a test
  clip with a rotation tag checks that frames come out upright (portrait size). That clip is made
  once with `ffmpeg -display_rotation 90` from an existing test clip (command in
  `tests/self-test-clips.txt`), committed in `tests/folder`, and added to that list, so the
  Linux self-test decodes it too. The sampler tests on real clips run on Linux only, like the
  self-test: the Windows CI job has a build-only GStreamer without decoders. Windows is covered
  by the orientation unit test and by the pipeline using only elements its bundle has.

## Request

A provider trait in `frename-core::ai`, shaped around the request, not around clips, so #13's
subtitles-only mode and a later OpenAI provider fit it:
`complete(request: AiRequest) -> Result<AiResponse, AiError>` where the request is the model, the
content blocks (text and JPEG images), the JSON schema and `max_tokens`, and the response is the
parsed JSON plus `usage`. The clip prompt is built above it.

The Anthropic implementation: raw HTTP (`POST https://api.anthropic.com/v1/messages`, headers
`x-api-key`, `anthropic-version: 2023-06-01`) with `reqwest = "0.12"` (features `blocking`,
`rustls-tls-native-roots`, `default-features = false`; the OS certificate store is
trusted, as corporate TLS-inspecting proxies need) on the batch thread; there is no official Rust
SDK. `base64 = "0.22"`.

- `model`: `claude-haiku-4-5`; `max_tokens`: 4000. No `thinking` parameter: Haiku 4.5 does not
  think unless asked, so output tokens are only the answer. (A later model choice must handle
  models that think by default.)
- Content: one text block with the instructions (summary language, what a segment is, at most
  `clamp(duration / 30 s, 3, 12)` segments so the comment stays short, stay factual, no speculation
  about identities), the SRT as `[m:ss–m:ss] text` lines when present, then the
  frames with their `t=` labels.
- **Guaranteed JSON** through structured outputs,
  `output_config: {format: {type: "json_schema", schema}}`, schema
  `{summary: string, segments: [{start_s: number, end_s: number, description: string}]}` with
  `required` on every field and `additionalProperties: false` on both objects.
  The parser still validates: segments sorted, within the clip, `end_s > start_s`; invalid
  segments are dropped, an empty summary fails the file.
- Response `stop_reason` other than `end_turn` (e.g. `max_tokens`, `refusal`) fails the file with
  a plain reason (flow 3); its usage still counts.
- **Retries:** on 500, 502, 503, 529 and connection errors (no response), up to 3 retries after 2,
  8, 30 s; a connect timeout counts as a connection error (nothing was sent). A **429** waits
  `retry-after` (or 30 s) and does not use up a retry: a new key's per-minute token limit is
  easily reached by a long job, which should slow down, not fail. After 20 waits in a row the
  file fails (`Anthropic's rate limit was still reached after many waits`), so an account whose
  limit is below one request cannot hold the job forever. Waits
  are slept in 250 ms steps that check the cancel token. 400 fails the file at once; 401/403 stop
  the job.
- **Out of credit:** a 402, or a 400 whose error is about the credit balance, stops the job like
  401: `Stopped: the Anthropic account has no credit left.` A 400 about a usage or spend limit
  stops it too, with the API's message (`Stopped: <message>`), so 2000 remaining clips do not
  each fail with the same message. Any other 400 fails the file with the API's own error message
  as the reason.
- **Timeout:** 60 s for the answer plus 1 s per 50 KB of request (about 60 s more for 60 frames),
  so a slow uplink does not time out while still uploading. A timeout fails the file
  (`No answer in time`) instead of retrying: the server may have processed and billed the
  request already. A test with a server that never answers checks it is sent once.

## Batch changes

The shared batch job cannot carry this today: `Operation` is `Copy` with `run(self, &Path) ->
ItemResult`, `ItemResult` has only a status and an update, a failed file shows only its name
("see the log for why"), and cancel is a flag on the UI side that a running file cannot see.
Stage 1 changes the shared code, for every action:
- Each action can give the run button its own label and the files the job runs over
  (`Describe 12 videos`: only the videos that will be sent); the others keep `Run on N files`
  and all checked files. Stopping reads `Stopping…` for every action.
- The failed-list heading becomes `Failed:` when every failed item carries a reason.
- `ItemResult` gains `reason: Option<String>` (shown after the file's name in the failed list;
  the other actions keep `None` for now, so their panel looks as before), `usage: Option<AiUsage>`
  (summed by the job for flow 6, with `usage_unknown` when a request may have been billed without
  a usage), `stop_job: Option<String>` (the job stops, marks the rest as not reached, and shows
  the reason as its summary line), and `stop_if_repeated: Option<String>` (the same, once three
  files in a row end with it: the network is gone).
- `run` receives a cancel token (`Arc<AtomicBool>`, set by *Cancel*) that long operations check.
- `Operation::DescribeAi` carries its options (language, redo) by value; `Operation` stops
  being `Copy` (it stays `Clone`). The key is read from the credential store inside `run`, never
  carried in messages or logged.

## Cost

- **Estimate before running:** per clip, samples × tokens per frame + SRT characters / 3.5 + 600
  (instructions) input tokens, plus 600 output tokens, times the model's price, labelled "about".
  Prices are a table in the code with the date they were checked (Haiku 4.5 $1 / $5 per million
  input / output tokens).
- **Only videos count:** checked files of other kinds (photos) are left out of the job and named
  in the skipped line.
- **Durations** are not in the file snapshot. While Describe with AI is the action shown (and when
  the checked set changes while it is), an async `Task` reads
  the missing durations with a duration query on a paused `uridecodebin` pipeline, the way the
  frame sampler opens a clip (4 at a time, on a blocking thread, 5 s timeout each), and caches the
  result by `FileId`, including "unreadable" for a clip that fails or times out, which is then
  skipped, named in the skipped line, and never queried again; meanwhile the panel says `Estimating…`. It also stays on
  `Estimating…` while any checked clip's comment is still loading (`comment_loading`), since
  whether it already has an AI block is not known yet. Clips skipped because they already have an
  AI block (Redo off) or are over 30 min are left out of the count and the estimate and named in
  the "skipped" line.
- **Actual:** the sum of `usage.input_tokens` and `usage.output_tokens` over the job, times the
  same table (flow 6).
- Example: 1000 one-minute clips ≈ 1000 × (30 × 209 + 600 + ~300) ≈ 7.2 M input + 0.6 M output ≈
  **$10 with Haiku 4.5**.

## Key storage

The key is stored with the operating system's credential store through the `keyring` crate
(Windows Credential Manager; Secret Service on Linux; Keychain on macOS), under service
`frename`, user `anthropic-api-key`. It is never written to `frename.db`, logs, or the repository.
`keyring` 3 has no default backend and silently falls back to an in-memory mock without one, so
the features are explicit: `windows-native`, `apple-native`, `sync-secret-service`,
`crypto-rust`, `vendored` (dbus for the AppImage), with the version pinned to `3` (keyring 4 has
different features). If the store is unavailable (e.g. a Linux
desktop without Secret Service, or a keyring left locked), the field says `The system keyring
could not be opened` and the action stays disabled, the panel saying the same with an Open
Settings link (opening Settings reads the state again). Tests use an entry of their own (user
`anthropic-api-key-test-<pid>`), so a developer's real key is never touched: a save / read /
delete round trip where a store works (always on the Windows CI job); on Linux CI (no Secret
Service running) the "unavailable" state is returned, not a mock that pretends to save.

## Where the code lives

- `frename-core::ai`: the AI block (format, parse, replace), the provider trait and the Anthropic
  implementation (`reqwest`), prompt and JSON schema building, response parsing and validation,
  cost estimate and price table. No iced, no GStreamer.
- `src/features/batch/`: the shared changes above, and `actions/describe_ai/` (options, view,
  run; `frames.rs`: frame sampling and durations with GStreamer).
- `src/features/settings/`: the AI section, the scrollable content, and opening at a section.
- `src/features/file_workspace/`: the comment area split into the editable box and the read-only
  AI block.
- New dependencies: `reqwest` 0.12 and `base64` 0.22 (as above), `keyring` 3 (features above);
  `serde_json` is already used by `frename-core`.

## Edge cases

- **No video stream / unreadable file:** red result `Video could not be read`.
- **Very short clip (< 2 s):** one frame at the middle.
- **Clip with no SRT:** frames only; the language setting "same as subtitles" means English.
- **SRT in several languages / garbage:** passed as is; the model is told it may be inaccurate.
- **Comment still loading (XMP not read yet):** the action reads the file's metadata itself (as
  the move actions do) before writing, so an unread comment is never overwritten.
- **Open file in the batch:** handled as every batch action handles it (saved first, closed while
  the job runs).
- **Key revoked mid-job (401):** the job stops; remaining files stay pending.
- **Rate limit waits** (429) are logged; the job panel shows only the file in work. Showing the
  wait in the panel is left for stage 3, where long unattended runs are designed.
- **Premiere:** the AI block is part of the comment; with comments in the video it lands in the
  Description column like the rest.

## Test plan

- `frename-core` unit tests (no network): AI block format/parse/replace (user text byte-identical,
  block replaced, unterminated block left alone, `--` variant, Cyrillic and emoji); response
  parsing (valid, invalid segments dropped, empty summary fails, `max_tokens` / `refusal`
  stop reasons); prompt and schema snapshot; cost estimate; retry policy against a mock HTTP
  server (`429` with `retry-after`, `529`, `401` stops the job).
- AI block text: line breaks in model output are collapsed; text after a block is kept and moves
  before it on save; `editor_comment` of before + block + after.
- Open file: F12 on a described clip adds the timestamp to the editor's part only; one block after
  the save.
- The Commented rule everywhere: a comment that is only an AI block does not check the tag, is
  not counted or shown by the Commented filter, and is skipped by **Tag commented videos**;
  editor text plus an AI block counts; removing the editor's text with the AI block kept unchecks
  the tag.
- Comment area: typing edits only the editor's part; saving joins it with the unchanged block;
  **Remove AI description** removes only the block.
- Batch job tests: a failed item's reason is shown; `stop_job` stops the job and leaves the rest
  not reached; usage is summed across items, including after a cancel; the cancel token is seen
  by a running item.
- App tests: frame sampler on `tests/self-test-clips.txt` clips (Linux, like the self-test):
  sample count, sizes for landscape and portrait, the 60-frame cap, the < 2 s case, extraction
  time.
- Live test, only when `FRENAME_ANTHROPIC_API_KEY` is set (skipped otherwise, never in CI
  without the secret): one real clip, Haiku 4.5, checks a non-empty summary and segments inside
  the clip; prints the real token usage. An account without credit skips it (nothing about the
  code to test).
- UI: demo mode (#14) gets an `--ai` flag: with it the open file's AI description is shown with
  its segments, and with `--batch` the Describe with AI action is selected. A scenario whose open
  clip has editor text plus an AI block, and another clip with only a block, gives screenshots
  of the comment area and of the run panel under Xvfb. The Settings window cannot be opened in
  demo mode, so its AI section is reviewed from its `view` code.

## Delivery

One PR: this design, core `ai` module, key storage, Settings section (scrollable), the shared
batch changes, the action, and the README. `version.md` is left alone: the owner asked for this
to merge without a release, so the release notes come with the next version.
Body `Refs #17` (stages 2–3 remain).

## Decisions

The owner accepted the recommended answers (2026-09-26, "implement as you see it"):

1. **Provider for stage 1:** Claude (frames + SRT). Gemini comes as a second provider in stage 3.
2. **Where the description goes:** the comment AI block.
3. **Summary language default:** "Same as the subtitles (English if none)", as a visible dropdown
   in Settings and a line in the run panel.
4. **Key storage:** the OS credential store.
5. **#13 overlap:** #13 becomes a "from subtitles only" mode of this action, shipped after it,
   reusing the block and the provider.
6. **Model:** Claude Haiku 4.5 only in stage 1.
