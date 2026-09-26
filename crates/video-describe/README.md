# video-describe

Describe what happens in a video clip, and when, with Claude.

**In:** a video file, its subtitles if it has any, and options (Anthropic API key, model,
language).
**Out:** a one-sentence summary, time-ranged key moments, and the tokens the request was billed for.

```rust
use std::sync::atomic::AtomicBool;
use video_describe::{describe, Options, SummaryLanguage, MODELS};

let options = Options {
    api_key: std::env::var("ANTHROPIC_API_KEY")?,
    model: MODELS[0], // Claude Haiku 4.5; MODELS also has Sonnet 5 and Opus 5
    language: SummaryLanguage::English,
};
let cancel = AtomicBool::new(false);
let described = describe("clip.mp4".as_ref(), &[], &options, &cancel, |stage| {
    println!("{stage:?}"); // Frame { done, total }, then Asking
})?;
println!("{}", described.description.summary);
for moment in &described.description.segments {
    println!("{:.0}–{:.0} s: {}", moment.start_s, moment.end_s, moment.description);
}
println!("cost: ${:.4}", options.model.cost_usd(described.usage));
```

## How it works

- Frames are read with GStreamer: one every 2 s, at most 60 per clip (a longer clip is sampled
  evenly), scaled to 512 px on the long side, turned upright if the clip has a rotation tag, and
  encoded as JPEG in memory. Nothing is written to disk.
- One request goes to the Anthropic Messages API: the instructions, the subtitles as
  `[m:ss–m:ss] text` lines, then each frame labelled `t=m:ss`. The answer is structured JSON
  (`summary`, `segments[{start_s, end_s, description}]`); segments outside the clip are dropped.
- Clips over 30 minutes are refused (`Error::TooLong`). `cancel` is checked between frames and
  while waiting for the answer.
- `estimate_usage` and `Model::cost_usd` price a clip before sending it;
  `frames::clip_duration_s` reads a clip's length for that.

Everything blocks; call it from a worker thread.

## Features

- `frames` (default): GStreamer frame reading and `describe`. Needs the GStreamer runtime and,
  to build, its development files.
- Without it: the models, languages, request and answer types, the estimate and the Anthropic
  client, with no GStreamer dependency.

## Tests

`cargo test -p video-describe`. The frame tests read clips from the frename repository and run on
Linux. `live_description_of_a_test_clip` makes a real request when `FRENAME_ANTHROPIC_API_KEY` is
set.
