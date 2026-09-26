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

The Rust toolchain is pinned in `rust-toolchain.toml` so a new stable release cannot turn CI red
overnight. Updating it is its own PR (new lints get fixed there).

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

## Labels

| Label | Set by | Meaning |
|---|---|---|
| `P1` `P2` `P3` | owner/agent | Priority (lower number first). |
| `regression` | owner | A release broke something; picked before everything else. |
| `feature`, `process` | owner/agent | Kind of work. |
| `needs-design` | owner/agent | A design doc in `docs/design/` must be merged before code. |
| `approved` | owner | Makes an `idea` or a non-owner issue implementable. |
| `idea` | agent | Agent's own proposal; not implemented until `approved`. |
| `in-progress` | agent | An agent session is working on it (see heartbeat lock). |
| `awaiting-owner` | agent | Design questions for the owner; owner removes it after answering. |
| `needs-owner` | agent | On an issue: agent is stuck; owner answers and removes it to let the agent retry. |
| `hold` | owner | Do not work on / merge this. |
| `blocked`, `rejected` | owner | Not now / never. |
| `agent` | agent | PR opened by the agent pipeline. |
| `release-failed` | agent | A release run failed twice; blocks version bumps until fixed. |

## Owner setup (one-time, GitHub settings)

Settings → Rules → Rulesets → new branch ruleset for `main`, enforcement Active, **no bypass list
(administrators included)**:
- require a pull request before merging (0 approvals: the agent uses the owner's account);
- require status checks `ci-linux` and `ci-windows`, and branches up to date before merging;
- block force pushes; restrict deletions.

Settings → General → Pull Requests: allow squash merging only.

This makes the gates enforceable, not just written down: the agent merges with the owner's account,
so without the ruleset nothing stops a merge on red CI.

## Looking at the UI

The app runs on Linux under Xvfb:

```sh
sudo apt-get install -y xvfb imagemagick mesa-vulkan-drivers
Xvfb :99 -screen 0 1600x900x24 &
(cd tests/folder && DISPLAY=:99 ../../target/debug/frename) &
sleep 15 && DISPLAY=:99 import -window root screenshot.png
```

Verified: it renders under Xvfb (software rendering). Today it starts on the empty "open a folder"
screen and has no command-line option to open a folder, so screens past it cannot be reached yet;
issue #14 (demo mode) adds that. Until then the product reviewer reviews the `view` code plus a
written description of the UI, and screenshots are taken wherever they can be.

Look at the screenshot of every screen a change touches before asking for review; give the product
reviewer the screenshot path.

## Never

- Commit secrets. API keys (Soniox, Anthropic, OpenAI) live in the user's settings at runtime,
  in environment variables in agent sessions, and in GitHub Actions secrets in CI.
- Skip, disable or weaken a test to get CI green.
- Force-push `main` or rewrite its history.
- Merge a PR whose CI is not green on its latest commit.
