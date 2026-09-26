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

1. **Set the key once.** Settings → new section **AI**: an *Anthropic API key* field (masked, with
   *Show*), a *Model* choice (**Fast and cheap** = Claude Haiku 4.5, default; **Better** = Claude
   Sonnet 5), and *Summary language* (default **Same as the subtitles, English without them**;
   or a fixed language from a short list). A link text says where to get a key.
2. **Run it.** Batch mode → check clips → action **Describe with AI**. The action panel shows,
   before *Run*:
   - `12 videos, 38 min · about $0.35 with Claude Haiku 4.5` (estimate, see Cost);
   - `Frames and subtitles of these videos are sent to Anthropic.`;
   - a checkbox **Redo videos that already have an AI description** (off by default).
   *Run* is disabled with `Set an Anthropic API key in Settings` when there is no key.
3. **Progress and cancel** work like every batch action: per-file green/red result, *Cancel*
   stops after the current file. A file's result tooltip gives the reason on failure
   (`No API key`, `Anthropic rejected the key`, `Network error`, `Rate limited, try later`,
   `Video could not be read`).
4. **Read the result.** The comment box (and Premiere's Description column, when comments are
   stored in the video) shows the editor's text first, then the AI block.
5. **Re-run** (with *Redo*): only the AI block is replaced. Editing text inside the AI block is
   allowed but is overwritten by the next re-run; the README says so.
6. **Total spend.** When the job ends, the batch summary line adds `AI: $0.31 (Claude Haiku 4.5)`,
   computed from the `usage` the API returned.

## The AI block

Plain text, readable in Premiere's Description column and in `.comment.txt`, and parseable:

```
Shaky walk into the market; the guide talks about spices.

— AI (Claude Haiku 4.5, 2026-09-26) —
A guide leads two tourists through a spice market, stopping at a stall to taste saffron.
0:00–0:14 Walking through the market entrance, crowd, handheld.
0:14–0:41 Close-ups of spice sacks; the guide explains prices.
0:41–1:02 Tasting at a stall, laughter, the camera turns to the street.
— end AI —
```

- Starts with a line matching `— AI (…) —` and ends with the line `— end AI —` (em dashes; the
  parser also accepts `--` so hand-typed edits survive).
- A comment has at most one AI block. Writing: the old block (if any) is removed with the one
  blank line before it; the new block is appended after the editor's text, separated by one
  blank line. The editor's text is kept byte for byte.
- A start line without an end line is treated as editor text (never deleted).
- Times are `m:ss` below an hour and `h:mm:ss` from an hour; ranges use `–`.
- `frename-core` owns format, parse and replace (pure functions, unit-tested). #13 (summary from
  the SRT only) is meant to reuse this block and the provider below; see Open questions.

The comment is saved through the normal save path, so it follows the Settings comment storage
(XMP or `.comment.txt`) and the "Commented" tag rule (a clip that gets its first comment gets the
tag).

## Frames

- A GStreamer pipeline in the app (`src/`, where GStreamer lives):
  `uridecodebin ! videoconvert ! videoscale ! videorate ! video/x-raw,format=RGB,width=512,height=288,framerate=1/2 ! appsink`
  with `pixel-aspect-ratio` kept by letterboxing, encoded to JPEG with the `image` crate
  (already a dependency). Audio is not decoded.
- One frame every 2 s (research §2), capped at **60 frames per clip**; a longer clip is sampled
  evenly (interval = duration / 60). Clips over **30 min** are skipped with the result
  `Too long for stage 1 (over 30 min)`.
- Each frame is preceded by a text block `t=0:12` so the model can place segments; segment times
  are therefore accurate to the sampling interval (research §2).
- 512×288 costs ⌈512/28⌉×⌈288/28⌉ = 19×11 = **209 input tokens** per frame (research §2, Anthropic
  vision docs). 60 frames ≈ 12.5 k tokens. Haiku 4.5 has a 200K context and takes up to 100
  images per request, so 60 fits both models.

## Request

Raw HTTP (`POST https://api.anthropic.com/v1/messages`, headers `x-api-key`,
`anthropic-version: 2023-06-01`) with `reqwest` (blocking, rustls) from the batch thread; there is
no official Rust SDK.

- `model`: `claude-haiku-4-5` or `claude-sonnet-5`; `max_tokens`: 4000.
- Content: one text block with the instructions (summary language, what a segment is, stay factual,
  no speculation about identities), the SRT as `[m:ss–m:ss] text` lines when present, then the
  frames with their `t=` labels.
- **Guaranteed JSON** through structured outputs,
  `output_config: {format: {type: "json_schema", schema}}` with
  `{summary: string, segments: [{start_s: number, end_s: number, description: string}]}`.
  The parser still validates: segments sorted, within the clip, `end_s > start_s`; invalid
  segments are dropped, an empty summary fails the file.
- Response `stop_reason` other than `end_turn` (e.g. `max_tokens`, `refusal`) fails the file with
  that reason.
- **Retries:** on 429, 500, 502, 503, 529 and network errors, up to 3 retries, waiting
  `retry-after` seconds when the header is present, else 2, 8, 30 s. 400/401/403 fail at once;
  401/403 also stop the whole job (every further file would fail the same way).
- **Timeout:** 120 s per request.
- **Cancel:** checked between files and between retries; an in-flight request is not aborted.

## Cost

- **Estimate before running:** per clip, frames × 209 + SRT characters / 3.5 + 600 (instructions)
  input tokens, plus 600 output tokens; times the model's price. Prices are a table in the code
  with the date they were checked (Haiku 4.5 $1 / $5, Sonnet 5 $2 / $10 per million input / output
  tokens). Clip duration comes from the snapshot's clip length when known, else a GStreamer
  discoverer query (fast, header only).
- **Actual:** the sum of `usage.input_tokens` and `usage.output_tokens` over the job, times the
  same table, shown at the end (flow 6).
- Example: 1000 one-minute clips ≈ 1000 × (30 × 209 + 600 + ~300) ≈ 7.2 M input + 0.6 M output ≈
  **$10 with Haiku 4.5, $20 with Sonnet 5**.

## Key storage

The key is stored with the operating system's credential store through the `keyring` crate
(Windows Credential Manager; Secret Service on Linux; Keychain on macOS), under service
`frename`, user `anthropic-api-key`. It is never written to `frename.db`, logs, or the repository.
If the credential store is unavailable (e.g. a Linux desktop without Secret Service), the field
says `Cannot store the key on this system` and the action stays disabled.

## Where the code lives

- `frename-core::ai`: the AI block (format, parse, replace), prompt and JSON schema building,
  response parsing and validation, cost estimate and price table, and a `ClipDescriber` trait with
  the Anthropic implementation (`reqwest`). No iced, no GStreamer.
- `src/features/batch/actions/describe_ai.rs`: the action (options, view, run) following the
  existing action pattern; frame sampling in `src/` (GStreamer).
- `src/features/settings/`: the AI section.
- New dependencies: `reqwest` (blocking, rustls-tls, json), `keyring`, `base64`; `serde_json` is
  already used by `frename-core`.

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
- App tests: frame sampler on `tests/self-test-clips.txt` clips (Linux, like the self-test):
  frame count and size for a 10 s clip, the 60-frame cap, the < 2 s case.
- Live test, only when `FRENAME_ANTHROPIC_API_KEY` is set (skipped otherwise, never in CI
  without the secret): one real clip, Haiku 4.5, checks a non-empty summary and segments inside
  the clip; prints the real token usage.
- UI: the Settings gear sits in the folder controls, which the empty start screen does not show,
  so the AI section cannot be screenshotted under Xvfb until #14's demo mode; its `view` code is
  reviewed with a written description, as CLAUDE.md "Looking at the UI" says.

## Delivery

One PR: core `ai` module + key storage + Settings section + batch action + README + `version.md`.
Body `Refs #17` (stages 2–3 remain).

## Open questions (with recommended answers)

1. **Provider for stage 1: Claude (frames + SRT) or Gemini (native video + audio)?** Gemini is
   cheaper (research: ~$3 vs ~$10 per 1000 one-minute clips) and hears the audio; Claude is what
   the owner chose for #13, whose issue says it shares the provider layer with this one, and a key
   exists already. *Recommended:* **Claude** for stage 1; Gemini as a second provider in stage 3.
2. **Where the description goes: comment AI block (as the issue says) vs a separate field.**
   *Recommended:* the comment AI block — it reaches Premiere today and #13 wants the same block.
3. **Summary language default.** *Recommended:* same as the subtitles, English without them; a
   fixed language can be chosen in Settings.
4. **Key storage: OS credential store vs `frename.db`.** *Recommended:* credential store — the
   database sits next to the exe for zip users (possibly in a synced folder).
5. **#13 overlap.** *Recommended:* #13 becomes a second mode of this action ("from subtitles only",
   no frames, much cheaper), designed and shipped after this one, reusing block and provider.
