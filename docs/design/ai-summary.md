# AI summary of a video from its subtitles

Design for issue #13. It reuses the AI block and provider layer proposed for #17
([`video-understanding.md`, PR #27](https://github.com/Zelenov/frename/pull/27)) so #17 can build
on this code instead of duplicating it.

## What it does

A batch action, **Summarize subtitles with AI**. For each checked video with a `.srt` next to it,
the subtitle text goes to Claude, and a short summary (one or two sentences, plus time-ranged
segments when the clip has clearly different parts) is written into the video's comment as an
**AI block**. The editor's own text is never changed; a re-run replaces only the block.

Out of scope here: frames or audio (#17), ChatGPT (see Providers), tags or markers from the
summary.

## Owner decisions this follows, and the open ones decided here

| Question | Decision |
|---|---|
| Provider | Claude first (owner). ChatGPT comes behind the same trait later. |
| The user's comment | Never replaced (owner). The AI part is a marked block (below). |
| Summary language | A Settings dropdown, **Same as the subtitles** by default, then English, Russian, Ukrainian, German, Spanish, French. Same default #27 recommends; one change makes every summary one language. |
| Model | A Settings dropdown: Claude Opus 5 (default), Claude Sonnet 5, Claude Haiku 4.5. The estimate shows the price for the chosen one, so the owner can pick the cheaper model knowingly. |
| Separate action or a mode of #17's "Describe with AI" | Separate action now (#17 is not built); it becomes that action's "subtitles only" mode when #17 lands, sharing block and provider. |
| Key storage | The local app database (`frename.db`), next to the other settings, as `CLAUDE.md` describes ("API keys live in the user's settings at runtime"). The OS credential store (#27's proposal) can replace it later without touching the action. |

## The AI block

```
Shaky walk into the market; the guide talks about spices.

AI: A guide leads two tourists through a spice market and they taste saffron.
0:00–0:14 Walking through the market entrance.
0:14–0:41 The guide explains spice prices.
— Claude Opus 5, 2026-09-26 —
```

Exactly #27's format: it starts with a line beginning `AI: ` (at the start of the comment or after
a blank line) and ends with the first following `— <model>, <YYYY-MM-DD> —` line (`--` also
accepted). An `AI: ` line with no end line is the editor's text. The summary is the first line,
so a clip with no text of the editor's shows the summary in the file list. Line breaks in model
output are collapsed, so no model text can form an end line. Text found after a block (hand edits
in `.comment.txt` or Premiere) is kept and moved before the block on the next write.

In the open file the comment box holds only the editor's text; the block is shown under it,
read-only, with a **Remove** button. Typing can never break the block, and an F12 timestamp lands
in the editor's text.

**"Commented" means the editor's text only**: an AI block alone does not check the "Commented"
tag, is not counted by the Commented filter, and is skipped by **Tag commented videos**.
Otherwise summarizing 1000 clips would rename all of them.

## Flow

1. Settings → **AI summaries**: model, summary language, Anthropic API key (masked).
2. Batch mode → check clips → **Summarize subtitles with AI**. The panel shows the model and
   language with an *AI settings…* button, a **Redo videos that already have an AI summary**
   checkbox, and **Estimate cost**.
3. **Estimate cost** reads the checked clips' subtitles and comments on a background thread and
   shows `12 videos · about $0.41 with Claude Opus 5`, what is skipped (`3 without subtitles and
   2 already summarized are skipped.`) and `Their subtitles are sent to Anthropic.`. Run stays
   disabled until an estimate is shown and there is a key; changing the checks or *Redo* clears
   the estimate.
4. The run works like every batch action (progress, Cancel, green/red per file). A failed file
   shows its reason in the report (`Anthropic rejected the API key`, `No answer within 180 s`,
   the API's own message for other rejected requests, …). A rejected key or an empty credit
   balance stops the job (`Stopped: …`), since every later file would fail the same way.

## Request

`frename-core::ai`: `AiProvider::complete(&AiRequest, &AtomicBool) -> Result<AiResponse, AiError>`,
a request being model + prompt + JSON schema. `AnthropicProvider` sends raw HTTP to
`POST /v1/messages` (there is no official Rust SDK) through an `HttpTransport` trait, so the retry
and error logic is tested with a fake transport and no network.

- Structured output: `output_config.format` with a strict schema
  `{summary, segments: [{start_s, end_s, description}]}`; the answer is still validated (empty
  summary fails; backwards, overlapping or out-of-range segments are dropped; at most 8).
- Opus 5 and Sonnet 5 run with `effort: low` (a summary is a simple task; keeps thinking short and
  cheap). Opus 5 requests carry `fallbacks: "default"` (beta `server-side-fallback-2026-07-01`), so
  a request a safety classifier declines is re-run on Anthropic's recommended fallback model.
  Haiku 4.5 takes neither.
- `stop_reason` other than `end_turn` fails the file (`refusal`, `max_tokens`).
- Retries: 500/502/503/504/529 and lost connections, 3 retries after 2, 8, 30 s. 429 waits
  `retry-after` (or 30 s) without using up a retry, up to 10 times. 401/403 and out of credit (402,
  or a 400 about the credit balance) stop the job; other 4xx fail the file. Waits check the cancel
  flag every 250 ms.
- Timeout: 180 s per request, not retried (the request may have been processed and billed).
- HTTP: `ureq` 3 with rustls and the OS certificate store (`platform-verifier`), so corporate
  proxies that inspect TLS work; system proxy settings are honoured.

## Cost

Estimate per video: prompt characters / 3 + 300 input tokens, 700 output tokens (thinking
included), priced from a table in the code (checked 2026-09-26: Opus 5 $5/$25, Sonnet 5 $2/$10,
Haiku 4.5 $1/$5 per million input/output tokens). A one-minute talking clip is about 1–1.5 k input
tokens: roughly $25 per 1000 such clips on Opus 5, $5 on Haiku 4.5. Actual usage and cost per
file are written to the log.

## Batch job changes (shared)

- `ItemResult` gains `reason` (shown after the file name in the failed list; the heading becomes
  `Failed:` when every failed file has one) and `stop_job`.
- `Operation::run` receives the job's cancel flag, set by Cancel, so a file waiting to retry stops
  within a quarter second.

## Tests

- Core, no network: block format/parse/replace (editor text byte-identical, unterminated block
  left alone, `--` end line, text after a block, CRLF, Cyrillic, emoji), `has_editor_comment`,
  prompt and schema, answer validation, estimate, retry/rate-limit/timeout/key/credit handling
  with a fake transport, request body per model, settings round trip in the database.
- App: estimate state (stale answers dropped), estimate over real files, job stop and failure
  reasons, cancel flag seen by the running file, comment box keeps the AI block while typing,
  Remove, the Commented tag ignores an AI-only comment.
- Live: `live_summary_from_subtitles` calls the API only when `FRENAME_LIVE_ANTHROPIC_KEY` is set
  (a CI secret can be wired to it); otherwise it passes without network.

## Providers

ChatGPT (OpenAI) is the owner's second provider. It fits `AiProvider` (chat completions with a
`json_schema` response format) plus a provider choice and a second key in Settings. It is left for
a follow-up: api.openai.com is not reachable from the agent environment, so it could not be
tested end to end here.
