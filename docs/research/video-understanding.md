# Automatic clip understanding (issue #17)

Research notes, September 2026. Goal: for each clip, (1) a summary plus time-ranged segments
that work without speech; (2) tag suggestions, especially groups of similar footage; one batch
button for thousands of files.

**Verification.** Anthropic docs and the Google Cloud (Vertex) pricing page were read directly.
ai.google.dev, OpenAI, AWS, Twelve Labs and Hugging Face were blocked, so their facts come from
search excerpts, marked **[snippet]**; recheck before implementing. **[est]** = my estimate.

## 1. Models that take video directly

### Google Gemini (the main candidate)

- **Input.** Video can be sent inline (the limit rose from 20 MB to 100 MB in Jan 2026), through
  the Files API, or since Jan 2026 as a GCS or HTTP/signed URL [snippet]. Files API: 2 GB per
  file (another excerpt says 20 GB for paid users), 20 GB per project, files deleted after 48 h
  [snippet].
- **Sampling.** The default is 1 frame/s with a timestamp every second. You can set
  `videoMetadata.fps` in the range (0, 24], and `startOffset`/`endOffset` to clip the video.
  Prompts and answers can point to moments as `MM:SS` [snippet].
- **Token cost (Gemini 3.x).** A video frame costs 70 tokens at `media_resolution` low or medium,
  280 at high. Audio costs 32 tokens/s. Older models: 258 tokens/frame at default, 66 at low. Google
  gives about 300 tokens/s at default and about 100 tokens/s at low resolution [snippet]. For
  Gemini 3 I use **about 100 tokens/s, or 6 k tokens per minute**. Which resolution Gemini 3 uses
  for video by default is **unverified**; set it explicitly.
- **Maximum length.** 1M-context models: about 1 h at default resolution, 3 h at low
  [snippet]. "Agentic video" (Sep 2026; 3.7/3.6 Flash, 3.5 Flash-Lite) fetches frames on
  demand, up to 88 % fewer tokens on long videos [snippet]; irrelevant for short clips.
- **Structured output.** `responseMimeType: application/json` together with `responseSchema` /
  JSON Schema (a subset of JSON Schema) [snippet]. Timestamps come back as strings; ask for
  `"MM:SS"` or seconds and parse them in code.
- **API surface.** The Interactions API is primary since June 2026; `generateContent` is
  "legacy" but fully supported [snippet]. Plain REST via `reqwest` works.
- **Prices (Vertex pricing page, read directly; standard / batch, per 1M tokens):**

| Model | Input (text/image/video) | Output (includes thinking) |
|---|---|---|
| Gemini 3.5 Flash-Lite | $0.30 / $0.15 | $2.50 / (batch output not captured, presumably half) |
| Gemini 3.1 Flash-Lite | $0.25 / $0.125 (audio $0.50) | $1.50 / $0.75 |
| Gemini 3.5 Flash | $1.50 / $0.75 | $9.00 / $4.50 |
| Gemini 3.7 / 3.8 Flash | $0.75 until 31 Dec 2026, then $1.50 | $3.75, then $7.50 |
| Gemini 3.1 Pro Preview | $2.00 (≤200 k) | $12.00 |

  AI Studio (Developer API) prices assumed identical, **not verified**. Batch: 50 % off, 24 h
  target, inline ≤20 MB or JSONL ≤2 GB [snippet].
- **Cost per hour of footage [est].** 360 k input tokens gives about $0.11 on 3.5 Flash-Lite
  and $0.54 on 3.5 Flash. Output tokens usually cost more than input (see §7).
- **Privacy.** Free tier: Google may use and human-review prompts/outputs; paid tier: not
  (in EEA/CH/UK paid terms apply to both) [snippet]. Require a billing-enabled key.

### Other providers

- **Anthropic Claude:** images only (JPEG/PNG/GIF/WebP; no video;
  only the first frame of an animation is used). Use frame sampling (§2).
- **OpenAI:** no video input in the API; the cookbook extracts frames [snippet]. Images are tokenized in 32 px patches, reportedly with a ×1.2 multiplier on GPT-5.6;
  `detail: low` costs 85 tokens [snippet, unverified]. The Batch API is 50 % off, up to 50 000
  requests or 200 MB per batch [snippet].
- **Amazon Nova 2 Lite (Bedrock):** $0.30 / $2.50 per 1M tokens; 1 fps up to 16 min (960
  frames), 1 GB video via S3, about 30 min per request [snippet]. Usable, but AWS credentials
  are heavy for a desktop app.
- **Twelve Labs Pegasus 1.2:** index-then-query video API. Indexing $0.042/min, analysis
  $0.021/min, output $0.0075/1 k tokens, infra $0.0015/min [snippet], about $4 per footage hour.
  Strong at library search, but another vendor holding the video.

## 2. Frame sampling plus an image model (Claude or GPT)

**Frame extraction in GStreamer (frename already links gstreamer 0.23; 0.25.4 is current):**
`uridecodebin ! videoconvert ! videoscale ! videorate ! video/x-raw,width=448,framerate=1/2 ! appsink`.
This gives uniform frames at 1 frame per 2 s. For cuts, the `scenechange` element in
gst-plugins-bad `videofiltersbad` detects shot changes on raw video and sends a
`GstForceKeyUnit` event. Watch for it with a pad probe and take one extra frame at each
boundary (check the minimal Windows bundle includes it; fallback: histogram differences in
Rust). Handheld travel footage has more camera moves than hard cuts, so embedding distance (§3)
finds boundaries better.

**Claude image cost** (Anthropic vision docs): cost = ⌈w/28⌉ × ⌈h/28⌉ tokens.
- 448×252 costs 16×9 = 144 tokens, 512×288 costs 19×11 = 209 tokens, 1920×1080 costs 2 691 on
  Claude 4.7+ models (high-resolution tier).
- A request can hold up to 600 images (100 on 200 k-context models). Above 20 images, keep
  each side ≤ 2000 px. Limits: 32 MB per request, 10 MB per image. The Files API can be used to
  avoid resending image bytes.
- Prices per 1M tokens, standard / batch: Haiku 4.5 $1/$5 and $0.50/$2.50; Sonnet 5 $2/$10
  and $1/$5; Opus 5.5 $4/$20 and $2/$10. Message Batches: 50 % off, 100 000 requests or 256 MB
  per batch, most finish within 1 h, 24 h expiry, results kept 29 days. The batch discount
  stacks with prompt caching.

**Per minute of footage [est]:** 30 uniform frames plus about 5 scene frames at 209 tokens,
plus text, is about 7.5 k tokens; per hour about 450 k: **$0.45 on Haiku 4.5, $0.90 on
Sonnet 5** (half in batch), plus output. Same budget as Gemini, without motion or audio. Contact
sheets save nothing (cost is linear in pixels).

**Segment boundaries:** put a text label `t=00:12` before each image and ask for segments as
`[start,end]` that must line up with the labelled timestamps. Time resolution is therefore the
sampling interval, about 2 s, which is fine for markers. Then snap boundaries to the nearest
scene or embedding change found locally.

## 3. Local and offline options

**CLIP / SigLIP 2 embeddings through ONNX Runtime.**
- `google/siglip2-base-patch16-224`: ViT-B/16, about 86 M vision parameters. ONNX exports with
  separate vision and text encoders exist (`onnx-community/siglip2-base-patch16-224-ONNX`)
  [snippet]. SigLIP 2 also has multilingual text towers, which helps for Russian tag names.
- Rust: `ort` 2.0.0-rc.13 (crates.io, Jul 2026; 2.0 is still a release candidate). Execution
  providers: DirectML (Windows, DX12 GPUs), CUDA, CoreML. Prebuilt binaries mainly cover
  CPU/CUDA; DirectML may need `load-dynamic` with a shipped `onnxruntime.dll` [snippet].
- Speed [est]: about 20–60 ms per 224² frame on a modern CPU, much faster on GPU. 1000 clips ×
  30 frames = 30 k frames, about 10–30 min on CPU. Model size about 350–400 MB fp32, about half
  at fp16. Download it on first use rather than putting it in the installer.
- Uses:
  - **Zero-shot tagging:** score the mean clip embedding, or per-segment embeddings, against
    text embeddings of the folder's tag library ("a photo of {tag}"). Fully offline, free, instant
    on re-runs.
  - **Clustering:** similar shots of a location or subject group well. SigLIP does not
    understand actions or time ("person climbs, then waves").
  - **Segmentation:** a jump in cosine distance between consecutive frame embeddings marks a
    boundary.

**Local VLMs.**
- Qwen3-VL (2B/4B/8B/32B, Apache-2.0) has official GGUFs that run in llama.cpp (`llama-mtmd-cli`
  / `llama-server`) and Ollama [snippet]. 4B Q4_K_M is about 2.5 GB (fits a 6 GB GPU); 8B at Q4
  needs about 6 GB VRAM [snippet]. Qwen3-VL advertises timestamp-aligned video grounding, but
  through llama.cpp you effectively send a handful of frames as images; true video input there is
  **unverified**.
- Alternatives: MiniCPM-V 4.5 (8.7 B, video-oriented), InternVL3.5-8B, Moondream 3 preview
  (9 B MoE, 2 B active; good for detection and structured output, weaker at narrative), Gemma 4
  26B-A4B (about 18 GB) [snippet].
- Speed [est, unmeasured]: 8 frames at 448 px plus a 300-token answer is about 5–20 s per clip on
  an RTX 3060-class GPU, and minutes per clip on CPU only. 1000 clips on a GPU take a few hours.
- Quality [est]: fine for "what is in the scene", weaker than Gemini Flash at segment
  timelines and recognising places.
- Shipping: `llama-cpp-2` crate (0.1.157; CUDA/Vulkan builds complicate Windows CI), a bundled
  prebuilt `llama-server.exe` sidecar over HTTP (simplest; Vulkan builds run on any GPU), or a
  user-installed Ollama (`ollama-rs` 0.3.6). Models (2.5–6 GB) are optional downloads.

## 4. Transcript (SRT) as a signal

When a sonisub/Soniox SRT exists, pass it as timestamped text (a few hundred tokens per
minute). It improves summaries, disambiguates places and roles, and gives segment boundaries at
speech starts. Gemini also hears ambient audio itself (32 tokens/s); with frames, the SRT is the
only sound signal. An empty SRT is itself a useful tag (`no-dialogue`).

## 5. Tagging design

- **Closed vocabulary first.** Put the folder's tag library (from `.frename`) into the prompt as
  an `enum` in the JSON schema. The model returns `{tag, confidence, segment?}` only from that
  list. This keeps file names consistent.
- **Open proposals separately.** A second field `new_tag_suggestions` holds at most 5 short
  lowercase tags, which are never applied automatically. The user promotes them into the
  library.
- **Confidence.** LLM self-reported confidence is poorly calibrated; use it only for ordering.
  SigLIP sigmoid scores get per-tag thresholds learned from accept/reject history (start about
  0.1–0.2 [est]).
- **Review UI.** "Ghost" chips in the tag panel: accept/reject, or "accept all ≥ threshold".
  Nothing is renamed until accepted; decisions are logged in SQLite.
- **Similarity propagation.**
  - Compute per-clip embeddings: the mean of frame embeddings, or several per segment.
  - Cluster with agglomerative clustering on cosine distance (`kodama` 0.3; simple,
    deterministic, you set a distance cut-off) or HDBSCAN (`hdbscan` 0.12 or
    `linfa-clustering` 0.8; no k needed, marks outliers).
  - Show clusters as groups taggable at once; when a clip is tagged, offer the tag to its
    nearest neighbours.
  - Add capture time (and GPS): clips shot minutes apart and visually similar are one block.

## 6. Batch engineering for thousands of files

- **Cache key:** a BLAKE3 hash of file size, the first and last 1 MB, and mtime. A full hash of
  multi-GB files is slow. Store results in the existing SQLite (`rusqlite`), keyed by
  `(hash, model, prompt_version)`. Write the final summary or segments to XMP or the sidecar so
  it travels with the file.
- **Resumability:** keep one job row per file (`pending → uploading → submitted(batch_id) →
  done/failed`) and persist batch IDs. On restart, poll the open batches. Gemini Files expire
  after 48 h, and Anthropic results after 29 days.
- **Upload size:** do not upload raw 4K. Transcode a proxy with GStreamer first: 480p, 2 fps,
  low-bitrate H.264, mono 16 kHz AAC. That is about 2–5 MB/min and small enough to send inline
  (≤100 MB) in real-time mode [est]. Gemini batch needs a JSONL file that references uploaded
  Files (inline batches are capped at 20 MB).
- **Rate limits:** semaphore (4–8 parallel) plus exponential backoff on 429. Batch APIs have
  separate quotas.
- **Cost preview:** "N clips, M min, ≈X tokens, ≈$Y" = duration × tokens/s × price + output
  allowance, excluding cached files; report real spend from `usage`.
- **Privacy:** show what leaves the machine; local-only (SigLIP) by default; keys in the OS
  credential store (`keyring`).

## 7. Comparison and recommendation

Estimates for **1000 clips × 1 min (16.7 h)**, assuming about 600 output tokens per clip and
no long "thinking" (thinking tokens are billed as output; set a low thinking budget).

| Option | Input | Est. cost (std / batch) | Understands motion/audio | Offline | Effort |
|---|---|---|---|---|---|
| SigLIP 2 + clustering (local) | frames | $0 | no / no | yes | medium |
| Gemini 3.5 Flash-Lite, native video | proxy video | ~$3.3 / ~$1.7 | yes / yes | no | low |
| Gemini 3.5 Flash, native video | proxy video | ~$14.4 / ~$7.2 | yes / yes | no | low |
| Claude Haiku 4.5, 35 frames/min | JPEG frames | ~$10.5 / ~$5.3 | partly / SRT only | no | medium |
| Claude Sonnet 5, 35 frames/min | JPEG frames | ~$21 / ~$10.5 | partly / SRT only | no | medium |
| Twelve Labs Pegasus | video | ~$69 [snippet prices] | yes / yes | no | medium |
| Qwen3-VL-4B/8B local | frames | $0 (GPU hours) | partly / no | yes | high |

How the estimates are calculated:
- Gemini: 6 k tokens/min in and 0.6 k out. Flash-Lite is 6 M × $0.30 + 0.6 M × $2.50.
- Claude: 7.5 k tokens/min in. Haiku is 7.5 M × $1 + 0.6 M × $5.
- All of them are **[est]**; run a 20-clip pilot to measure real tokens.

**Stage 1 (simple): Gemini, one call per clip.** Transcode a proxy, then send one request per
clip with the proxy video, the SRT if present, and the tag library as an enum. The JSON schema
is `{summary, segments:[{start:"MM:SS", end, description, tags[]}], tags[], new_tag_suggestions[]}`.
Use Gemini 3.5 Flash-Lite, and 3.5 Flash for a "high quality" option. Results go to
SQLite, sidecar/XMP comments and, later, Premiere markers (one marker per segment). Show tags
as suggestions for the user to accept. Needs: **one Gemini API key with billing enabled**
(AI Studio). Crates: `reqwest` (JSON plus resumable upload), `serde_json`, `blake3`,
`gstreamer` (proxy), `keyring`. About $2–4 per 1000 one-minute clips.

**Stage 2 (better): local SigLIP 2 + clustering + batch.** `ort` + SigLIP 2 (downloaded on first
use) on 0.5–1 fps frames: offline zero-shot tags from the library, similarity groups, tag
propagation, boundary snapping. Gemini Batch mode (−50 %) for folders over about 50 clips, with
job persistence.

**Stage 3 (full): pluggable providers and offline VLM.** Put providers behind one trait
(`analyze(clip) -> ClipAnalysis`):
- Gemini native video (default);
- Claude frames through Message Batches, with an Anthropic key, for users who prefer
  Anthropic;
- a local Qwen3-VL via a bundled `llama-server` sidecar or Ollama for fully private work.

Add per-folder thresholds learned from accept/reject, embedding search across folders ("drone
over water"), and a budget cap.

Main risks: timestamp accuracy (measure it on real footage, since LLM `MM:SS` can drift by
1–3 s), model churn (Gemini 3.7/3.8 prices double on 1 Jan 2027), and the size and GPU-backend
complexity of shipping ONNX Runtime and llama.cpp on Windows.

## Sources

- https://platform.claude.com/docs/en/build-with-claude/vision
- https://platform.claude.com/docs/en/about-claude/pricing
- https://platform.claude.com/docs/en/build-with-claude/batch-processing
- https://cloud.google.com/gemini-enterprise-agent-platform/generative-ai/pricing
- https://ai.google.dev/gemini-api/docs/video-understanding [snippet]
- https://ai.google.dev/gemini-api/docs/media-resolution [snippet]
- https://ai.google.dev/gemini-api/docs/generate-content/gemini-3 [snippet]
- https://ai.google.dev/gemini-api/docs/pricing [blocked; not verified]
- https://ai.google.dev/gemini-api/docs/batch-api [snippet]
- https://ai.google.dev/gemini-api/docs/structured-output [snippet]
- https://ai.google.dev/gemini-api/docs/changelog [snippet]
- https://ai.google.dev/gemini-api/terms [snippet]
- https://blog.google/innovation-and-ai/technology/developers-tools/gemini-api-new-file-limits/ [snippet]
- https://blog.google/innovation-and-ai/technology/developers-tools/interactions-api-general-availability/ [snippet]
- https://developers.openai.com/api/docs/guides/images-vision [snippet]
- https://developers.openai.com/api/docs/guides/batch [snippet]
- https://github.com/openai/openai-node/issues/1778
- https://docs.aws.amazon.com/nova/latest/userguide/modalities-video.html [snippet]
- https://aws.amazon.com/blogs/aws/introducing-amazon-nova-2-lite-a-fast-cost-effective-reasoning-model/ [snippet]
- https://www.twelvelabs.io/pricing [snippet]
- https://gstreamer.freedesktop.org/documentation/videofiltersbad/scenechange.html
- https://huggingface.co/google/siglip2-base-patch16-224
- https://huggingface.co/onnx-community/siglip2-base-patch16-224-ONNX
- https://huggingface.co/Qwen/Qwen3-VL-4B-Instruct-GGUF
- https://huggingface.co/bartowski/Qwen_Qwen3-VL-4B-Instruct-GGUF [snippet]
- https://tinyweights.dev/posts/best-local-vision-language-models-2026/ [snippet]
- https://crates.io/crates/ort (2.0.0-rc.13), https://crates.io/crates/llama-cpp-2, https://crates.io/crates/hdbscan, https://crates.io/crates/kodama, https://crates.io/crates/ollama-rs
