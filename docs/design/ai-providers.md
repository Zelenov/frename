# More AI providers for Describe with AI (issue #166)

State when this was written: Anthropic and OpenAI were already in Settings (a key each, a model
list with prices, billing links in the "out of credit" report). This adds **Google Gemini** and the
"Get a key" / "Add credit" buttons. The provider code is in `clipscribe` 0.17.0
(`docs/design/gemini-provider.md` there), not here.

## The set, and why

| Provider | Video / image input | Per 100 one-minute clips* | Quality for "what happens when" | Key | Data terms |
|---|---|---|---|---|---|
| Anthropic (Haiku 4.5) | images (frames) | about $1.0 | good, the reference | API key, prepaid credit | API data not used for training by default |
| OpenAI (GPT-4.1 mini) | images (frames) | about $0.4 (price not live-checked) | good | API key, prepaid credit | API data not used for training by default |
| **Gemini 3.1 Flash-Lite** | images (frames); native video exists but frames fit the pipeline | about $0.3 | good for summaries, not compared on a large set | API key from AI Studio; billing must be on | free tier: Google may use prompts; paid: not |
| Gemini 3.5 Flash | same | about $1.9 | notices more | same | same |

\* 30 frames of a one-minute clip (one per 2 s), Gemini at 280 tokens a frame (measured), Claude 209,
GPT 255, 300 instruction tokens, 600 output tokens. Gemini prices are from Google's Vertex page
(AI Studio assumed the same; `ai.google.dev` was unreachable from the session), so unconfirmed.

Not added, and why: **xAI Grok** and **Mistral** (no editor asked; same frame approach, can be
added as one `Provider` each in clipscribe if wanted), **OpenRouter** (one key for many models, but
the prices and privacy terms are the router's and per-model, so "price shown" would be a guess),
**local models** (Ollama with a vision model: free and offline, but a different kind of work: a
local server to find, frame batching and quality unknown). The research behind this choice is
`docs/research/video-understanding.md` (#17); nothing new was measured except Gemini's request
shape and frame cost, with a real key.

## What changed here

- `ApiKey::Gemini` (credential store entry `gemini-api-key`), `ApiKey::key_url()` (the "Get a key"
  page), `billing_url()` for Gemini, `ApiKey::ALL`. The URLs could not be opened from the session
  (the sites are blocked); `aistudio.google.com/apikey` is where Google creates a key and shows
  the project's billing, and is used for both buttons; Soniox opens `console.soniox.com`.
- Settings → Describe with AI: a Gemini key row (field, Show, **Get a key**, Save; when saved,
  Replace, Remove, **Add credit**), in the Tab order of the page. Soniox gets the same buttons.
- A provider's errors were already mapped per provider by clipscribe (`KeyRejected("Gemini")`,
  `OutOfCredit("Gemini")`); the batch report's "Add credit" button already opens the billing page
  of the job's model's key (`key_of`), which now includes Gemini.

## Decisions made without the owner

- Gemini only, not Grok/Mistral/OpenRouter/local (above).
- Frames, not Gemini's native video input: every feature (key frames, shots, groups, moments at
  frame times) is built on frames.
- Both Gemini models at `MEDIA_RESOLUTION_LOW` (280 tokens a frame instead of 1100).
- The "Get a key" button opens the page in the default browser (`explorer` / `xdg-open`, as the
  existing billing button does); no embedded browser.
