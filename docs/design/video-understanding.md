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
   an *Anthropic API key* field (masked, with *Show*) and under it
   `Saved in Windows Credential Manager on this computer.` (the macOS / Linux store's name on
   those systems) and a **Save** button; *Summary language*: a dropdown whose first entry reads
   **Same as the subtitles (English if none)**, then English, Russian and the other languages of
   the app's users. A line of text says where to get a key. The model is Claude Haiku 4.5; a
   model or provider choice comes with stage 3.
2. **Run it.** Batch mode → check clips → action **Describe with AI**. The action panel shows,
   before *Run*:
   - `12 videos, 38 min · about $0.35 with Claude Haiku 4.5` — or `Estimating…` while durations
     and comments are read (see Cost); when some checked clips are skipped, a second line says
     why: `3 already described and 1 over 30 min are skipped.`;
   - `Frames and subtitles of these videos are sent to Anthropic.`;
   - a checkbox **Redo videos that already have an AI description** (off by default).
   Without a key, *Run* is disabled and the panel shows `Set an Anthropic API key in Settings`
   with an **Open Settings** button (the existing `ActionMessage::OpenSettings`), which opens
   Settings scrolled to the AI section.
3. **Progress and cancel** work like every batch action: per-file green/red result. *Cancel*
   changes the panel to `Stopping after the current video…` and takes effect within a second
   between frame samples, between requests and during retry waits; a request already sent
   finishes (at most 60 s).
   Failed files are listed with their reason (see Batch changes): `No API key`,
   `Anthropic rejected the key`, `Network error`, `Rate limited, try later`,
   `Video could not be read`, `Too long (over 30 min)`.
   A rejected key (401/403) stops the job: `Stopped: Anthropic rejected the key. Check it in Settings.`
4. **Read the result.** In the comment area, the editable box holds only the editor's text; the
   AI block is shown under it, read-only, in muted text, with a small **Remove AI description**
   button. Typing can therefore never damage the block or mix with it. Premiere and
   `.comment.txt` get the whole comment (editor text, blank line, AI block); a clip with no text
   of the editor's shows the AI summary on the comment's first line (see The AI block).
5. **Re-run** (with *Redo*): only the AI block is replaced.
6. **Spend.** When the job ends — finished, cancelled or stopped — the job's summary line adds
   `AI: $0.31 (Claude Haiku 4.5)`, computed from the `usage` the API returned.

## Settings layout

The Settings window is a fixed 560×560 with four sections already (`SETTINGS_WINDOW_SIZE` in
`src/app/state.rs`); a fifth does not fit. Its content becomes a scrollable column (the window
keeps its size). The AI section:

```
AI
  Anthropic API key  [••••••••••••••••••••]  [Show]  [Save]
                     Saved in Windows Credential Manager on this computer.
                     Get a key at console.anthropic.com → API keys.
  Summary language   [Same as the subtitles (English if none) ▾]
```

The key is written to the credential store only on **Save**, through a `Task` (never on each
keystroke, never from `view`). Settings state keeps `has_key: bool` (read once at start-up and
after Save, also through a `Task`) and passes it to the batch panel; `view` never touches the
store.

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
- `frename-core` owns format, parse and replace, and `editor_comment(&str) -> &str` (the comment
  without its AI block), all pure functions, unit-tested. #13 (summary from the SRT only) is meant
  to reuse this block and the provider below; see Open questions.
- The comment area splits the loaded comment with the same parser: the editable box edits the
  editor's part, and every save joins it back with the unchanged block.

The comment is saved through the normal save path, so it follows the Settings comment storage
(XMP or `.comment.txt`). **"Commented" means the editor's text only**, everywhere: a comment that
holds nothing but an AI block does not count as commented, so describing 2000 clips does not add
the tag to (and rename) every clip, and the tag keeps meaning "I commented this". Every place that
asks "does this file have a comment?" uses `editor_comment`:
- the rule that checks the tag when a comment goes from empty to non-empty (file workspace);
- `FileTagger::sync_commented_tag` (the **Tag commented videos** batch action; its hint says
  "each checked video with a comment of yours");
- the **Commented** filter and its count in the file list (`Directory`).
The comment's first line under the file name in the list still shows whatever comes first (the
AI summary when there is no editor text). The README says so.

## Frames

- In the app (`src/`, where GStreamer lives). The clip is opened paused
  (`uridecodebin ! videoconvert ! videoflip video-direction=auto ! videoscale ! appsink`, no audio
  branch; `videoflip` applies the orientation tag, so a phone portrait clip stored as 1920×1080
  with a rotation tag comes out upright) and, for each sample time, the pipeline **seeks** there
  (`FLUSH | KEY_UNIT | SNAP_NEAREST`) and pulls one frame; the frame's real timestamp is what the
  model is told. A sample whose real timestamp equals the previous one (keyframes further apart
  than the interval, e.g. screen recordings) is dropped, so no duplicate frame is billed. That decodes about one GOP per sample instead of the
  whole clip (a 20-min 4K clip would otherwise decode ~36 000 frames to keep 60).
- Size: the long side scaled to 512 px, aspect ratio kept (a portrait phone clip gives 288×512,
  not a letterboxed 162×288 picture in a 512×288 frame); JPEG via the `image` crate (already a
  dependency).
- One sample every 2 s (research §2), at most **60 per clip**; a longer clip is sampled evenly
  (interval = duration / 60). Clips over **30 min** are skipped with `Too long (over 30 min)`.
  A clip shorter than 2 s gets one frame at its middle.
- Each frame is preceded by a text block `t=0:12` so the model can place segments; segment times
  are therefore accurate to the sampling interval (research §2).
- Tokens per frame (research §2): about w·h/750 on Haiku 4.5 (512×288 → 197); the estimate says
  "about". 60 frames ≈ 12 k tokens. Haiku 4.5 has a 200K context and takes up to 100 images per
  request, so 60 fits.
- A test checks extraction time on the test clips (well under a second per frame), and a test
  clip with a rotation tag checks that frames come out upright (portrait size).

## Request

A provider trait in `frename-core::ai`, shaped around the request, not around clips, so #13's
subtitles-only mode and a later OpenAI provider fit it:
`complete(request: AiRequest) -> Result<AiResponse, AiError>` where the request is the model, the
content blocks (text and JPEG images), the JSON schema and `max_tokens`, and the response is the
parsed JSON plus `usage`. The clip prompt is built above it.

The Anthropic implementation: raw HTTP (`POST https://api.anthropic.com/v1/messages`, headers
`x-api-key`, `anthropic-version: 2023-06-01`) with `reqwest` (blocking, `rustls-tls-native-roots`,
so the OS certificate store is trusted, as corporate TLS-inspecting proxies need) on the batch
thread; there is no official Rust SDK.

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
  that reason.
- **Retries:** on 429, 500, 502, 503, 529 and network errors, up to 3 retries, waiting
  `retry-after` seconds when the header is present, else 2, 8, 30 s; waits are slept in 250 ms
  steps that check the cancel token. 400 fails the file at once; 401/403 stop the job.
- **Timeout:** 60 s per request.

## Batch changes

The shared batch job cannot carry this today: `Operation` is `Copy` with `run(self, &Path) ->
ItemResult`, `ItemResult` has only a status and an update, a failed file shows only its name
("see the log for why"), and cancel is a flag on the UI side that a running file cannot see.
Stage 1 changes the shared code, for every action:
- `ItemResult` gains `reason: Option<String>` (shown after the file's name in the failed list;
  the other actions keep `None` for now, so their panel looks as before), `usage: Option<AiUsage>`
  (summed by the job for flow 6), and `stop_job: Option<String>` (the job stops, marks the rest
  as not reached, and shows the reason as its summary line).
- `run` receives a cancel token (`Arc<AtomicBool>`, set by *Cancel*) that long operations check.
- `Operation::DescribeAi` carries its options (language, redo) by value; `Operation` stops
  being `Copy` (it stays `Clone`). The key is read from the credential store inside `run`, never
  carried in messages or logged.

## Cost

- **Estimate before running:** per clip, samples × tokens per frame + SRT characters / 3.5 + 600
  (instructions) input tokens, plus 600 output tokens, times the model's price, labelled "about".
  Prices are a table in the code with the date they were checked (Haiku 4.5 $1 / $5 per million
  input / output tokens).
- **Durations** are not in the file snapshot. When the checked set changes, an async `Task` reads
  the missing durations with GStreamer's discoverer (header only; 4 at a time, on a blocking
  thread) and caches them by `FileId`; meanwhile the panel says `Estimating…`. It also stays on
  `Estimating…` while any checked clip's comment is still loading (`comment_loading`), since
  whether it already has an AI block is not known yet. Clips skipped because they already have an
  AI block (Redo off) or are over 30 min are left out of the count and the estimate and named in
  the "skipped" line.
- **Actual:** the sum of `usage.input_tokens` and `usage.output_tokens` over the job, times the
  same table (flow 6).
- Example: 1000 one-minute clips ≈ 1000 × (30 × 197 + 600 + ~300) ≈ 6.8 M input + 0.6 M output ≈
  **$10 with Haiku 4.5**.

## Key storage

The key is stored with the operating system's credential store through the `keyring` crate
(Windows Credential Manager; Secret Service on Linux; Keychain on macOS), under service
`frename`, user `anthropic-api-key`. It is never written to `frename.db`, logs, or the repository.
`keyring` 3 has no default backend and silently falls back to an in-memory mock without one, so
the features are explicit: `windows-native`, `apple-native`, `sync-secret-service`,
`crypto-rust`, `vendored` (dbus for the AppImage), with the version pinned to `3` (keyring 4 has
different features). If the store is unavailable (e.g. a Linux
desktop without Secret Service), the field says `Cannot store the key on this system` and the
action stays disabled. Tests: a save / read / delete round trip on the Windows CI job; on Linux
CI (no Secret Service running) the "unavailable" state is returned, not a mock that pretends to
save.

## Where the code lives

- `frename-core::ai`: the AI block (format, parse, replace), the provider trait and the Anthropic
  implementation (`reqwest`), prompt and JSON schema building, response parsing and validation,
  cost estimate and price table. No iced, no GStreamer.
- `src/features/batch/`: the shared changes above, and `actions/describe_ai.rs` (options, view,
  run); frame sampling and durations in `src/` (GStreamer).
- `src/features/settings/`: the AI section, the scrollable content, and opening at a section.
- `src/features/file_workspace/`: the comment area split into the editable box and the read-only
  AI block.
- New dependencies: `reqwest` (blocking, rustls-tls, json), `keyring` (features above), `base64`;
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
- **Premiere:** the AI block is part of the comment; with comments in the video it lands in the
  Description column like the rest.

## Test plan

- `frename-core` unit tests (no network): AI block format/parse/replace (user text byte-identical,
  block replaced, unterminated block left alone, `--` variant, Cyrillic and emoji); response
  parsing (valid, invalid segments dropped, empty summary fails, `max_tokens` / `refusal`
  stop reasons); prompt and schema snapshot; cost estimate; retry policy against a mock HTTP
  server (`429` with `retry-after`, `529`, `401` stops the job).
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
  the clip; prints the real token usage.
- UI: the Settings gear sits in the folder controls, which the empty start screen does not show,
  so the AI section cannot be screenshotted under Xvfb until #14's demo mode; its `view` code is
  reviewed with a written description, as CLAUDE.md "Looking at the UI" says.

## Delivery

One PR: core `ai` module, key storage, Settings section (scrollable), the shared batch changes,
the action, README and `version.md`.
Body `Refs #17` (stages 2–3 remain).

## Open questions (with recommended answers)

1. **Provider for stage 1: Claude (frames + SRT) or Gemini (native video + audio)?** Gemini is
   cheaper (research: ~$3 vs ~$10 per 1000 one-minute clips) and hears the audio; Claude is what
   the owner chose for #13, whose issue says it shares the provider layer with this one, and a key
   exists already. *Recommended:* **Claude** for stage 1; Gemini as a second provider in stage 3.
2. **Where the description goes: comment AI block (as the issue says) vs a separate field.**
   *Recommended:* the comment AI block — it reaches Premiere today and #13 wants the same block.
3. **Summary language default** (a UI decision #13 left open for you). With "Same as the
   subtitles (English if none)" one folder can end up mixed: English on silent B-roll, Russian on
   interviews with Russian subtitles. *Recommended:* keep that default but make it a visible
   dropdown, so one change to "Russian" makes every description Russian.
4. **Key storage: OS credential store vs `frename.db`.** *Recommended:* credential store — the
   database sits next to the exe for zip users (possibly in a synced folder).
5. **#13 overlap.** *Recommended:* #13 becomes a second mode of this action ("from subtitles only",
   no frames, much cheaper), designed and shipped after this one, reusing block and provider.
6. **Model choice in stage 1.** *Recommended:* Claude Haiku 4.5 only (cheapest, no thinking to
   control); a better-model option comes with the provider choice in stage 3.
