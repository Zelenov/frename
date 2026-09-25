# frename — agent guide

Read `AGENTS.md` first: coding standards, Iced Elm architecture, Rust skills in `.cursor/skills/`.
This file adds what an agent needs to build, test and ship without a human in the loop.

## Layout

- `src/` — the app (iced 0.14, GStreamer video via `iced_video_player`), feature folders under `src/features/`.
- `crates/frename-core/` — pure logic, no iced: tags, files, XMP metadata, undo, SQLite.
- `.claude/skills/` — project skills (app-guide, core-dev, ui-dev, ui-core, undo-dev, image-preview, …).
- `docs/design/` — design documents for features that needed one.
- `version.md` — release notes; its first line `# X.Y` is the version. A change to it on `main`
  publishes a release (`.github/workflows/release.yml`).

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked
```

Linux needs GStreamer development packages:

```sh
sudo apt-get install -y libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
  libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
  gstreamer1.0-libav gstreamer1.0-tools
```

## Autonomous work

Features are GitHub issues in `Zelenov/frename`. The unattended pipeline — picking an issue,
implementing it, independent review, CI, merge and release — is `.claude/skills/nightly/SKILL.md`.
Reviewers follow `.claude/skills/review-gate/SKILL.md`. User-facing text follows
`.claude/skills/readme/SKILL.md`.

In autonomous mode the "implement only what is explicitly requested" rule of `AGENTS.md` means:
the issue is the request. Do exactly what the issue and its approved design ask, nothing beyond it.
Ideas of your own become new issues labelled `idea`, never extra code in the current PR.

## Never

- Commit secrets. API keys (Soniox, Anthropic, OpenAI) live in the user's settings at runtime,
  in environment variables in agent sessions, and in GitHub Actions secrets in CI.
- Skip, disable or weaken a test to get CI green.
- Force-push `main` or rewrite its history.
- Merge a PR whose CI is not green on its latest commit.
