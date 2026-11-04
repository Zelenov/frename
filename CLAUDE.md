# frename — agent guide

Read `AGENTS.md` first: coding standards, Iced Elm architecture, Rust skills in `.cursor/skills/`.
This file adds what an agent needs to build, test and ship without a human in the loop.

## Layout

- `src/` — the app (iced 0.14, GStreamer video via `iced_video_player`), feature folders under `src/features/`.
- `crates/frename-core/` — pure logic, no iced: tags, files, XMP metadata, undo, SQLite.
- Describing clips with Claude (frames, prompt, models, cost, client) is the `clipscribe` crate,
  its own repo `Zelenov/clipscribe`, pinned by commit in `Cargo.toml` and `frename-core`.
- `.claude/skills/` — project skills (app-guide, core-dev, ui-dev, ui-core, undo-dev, bug-hunt, …).
- `docs/design/` — design documents for features that needed one.
- `version.md` — release notes; its first line `# X.Y` is the version. A change to it on `main`
  publishes a release (`.github/workflows/release.yml`). To pause a release PR, convert it to a
  draft (`gh pr ready <n> --undo`); `gh pr ready <n>` takes it back.

## Our own crates

`clipscribe` (describing clips with Claude) is a separate repo under `Zelenov`, pinned by commit
(`rev`) in `Cargo.toml` and `crates/frename-core/Cargo.toml`; `sonisub` (subtitles with Soniox)
comes in the same way with its batch action (#56).

- Never copy their code into frename, not even a small helper: call the crate. If frename needs
  something a crate lacks, change the crate and move the pin.
- Per the owner: the pin follows every release of either crate, not just the ones with a breaking
  change — this is not gated on an issue or an `approved` label. When a session has access to both
  this repo and the crate's, move the pin (`Cargo.toml` + `Cargo.lock`) and make whatever code
  change the crate's `version.md` `## Changed` requires (nothing, for an additive release) in the
  same session as the crate's release, after the crate's CI is green on that commit; run frename's
  own test suite before pushing. A session without access to the crate's repo files an issue here
  instead (labelled `feature`, body `🤖 agent:`, the version, what changed, what frename must do)
  for the next session that has both to pick up.
- Their releases (tags, crates.io, binaries) are the owner's.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked
```

The Rust toolchain is pinned in `rust-toolchain.toml` so a new stable release cannot turn CI red
overnight. Updating it is its own PR (new lints get fixed there).

On the owner's Windows machine three `self_test` tests (`a_folder_of_good_clips_passes`,
`the_ci_clips_decode_a_frame`, `the_windows_media_fixtures_decode_a_frame`) fail with the local
GStreamer and pass in CI. They fail the same way without your change; CI decides.

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
the issue is the request. Do exactly what the issue asks (the agent's design notes only fill in details), nothing beyond it.
Ideas of your own become new issues labelled `idea`, never extra code in the current PR.
Bugs you find become issues labelled `bug` and are fixed without waiting for approval, each in its
own PR (`nightly` → "Bugs the agent finds"); every nightly session also hunts for them
(`.claude/skills/bug-hunt/SKILL.md`).

## Labels

| Label | Set by | Meaning |
|---|---|---|
| `P1` `P2` `P3` | owner/agent | Priority (lower number first). |
| `regression` | owner | A release broke something; picked before everything else. |
| `feature`, `process` | owner/agent | Kind of work. |
| `bug` | owner/agent | A defect, including leaks and flaky tests (`bug-hunt` → "What counts as a bug"). One the agent filed (body `🤖 agent:`) is work without `approved`; `hold`/`rejected` stop it. |
| `needs-design` | owner/agent | The agent writes its own design notes in `docs/design/` on the feature branch before coding. Not a gate; the owner does not approve designs. |
| `approved` | owner | Makes an `idea` or a non-owner issue implementable. An owner comment starting with "Approve" counts as this label (the agent adds it). |
| `idea` | agent | Agent's own proposal for something new (feature, behaviour change, refactor); not implemented until `approved`. Never used for a defect. |
| `in-progress` | agent | An agent session is working on it (see heartbeat lock). |
| `awaiting-owner` | agent | Legacy, no longer set: the pipeline never waits for design answers. |
| `needs-owner` | agent | On an issue: agent cannot proceed at all (guarded file, failed release); owner answers and removes it to let the agent retry. |
| `owner-review` | agent | Code review or CI did not converge: the feature is built on its PR but not merged or released. Owner merges it, or removes the label from the PR to hand it back. |
| `hold` | owner | Do not work on / merge this. |
| `blocked`, `rejected` | owner | Not now / never. An issue the owner closed counts as `rejected` (the agent adds it). |
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

Verified: it renders under Xvfb (software rendering). Demo mode opens a staged folder in a known
state and saves a screenshot of the main window: `frename --demo <scenario.toml> --out <png>`
(scenarios and `render.sh` in `docs/screenshots/`). Other windows (Settings, dialogs) are captured
with `import` under Xvfb as above.

Look at the screenshot of every screen a change touches before asking for review; give the product
reviewer the screenshot path. Every agent PR with a visible change shows its screenshots in the PR
body (`nightly` skill → "Screenshots in the PR").

## Never

- Commit secrets. API keys (Soniox, Anthropic, OpenAI) live in the user's settings at runtime,
  in environment variables in agent sessions, and in GitHub Actions secrets in CI.
- Skip, disable or weaken a test to get CI green.
- Force-push `main` or rewrite its history.
- Merge a PR whose CI is not green on its latest commit.
