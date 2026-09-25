---
name: nightly
description: >
  Unattended feature pipeline for frename. Use when a scheduled (nightly) session starts, or when
  asked to "take the next issue", "work the backlog", or ship a feature end-to-end without a human:
  pick an issue, design if needed, implement, independent review loop, CI, merge, release.
---

# Nightly pipeline

One session ships as many issues as the usage limit allows, one at a time, each through every gate
below. A gate that fails sends the work back; it is never skipped. When the limit is about to run
out, leave the current PR in a state the next session can resume (pushed branch, status comment).

## 0. Bootstrap

1. `git fetch origin main && git checkout main && git pull`.
2. Install Linux build deps (see `CLAUDE.md`) and `xvfb` if missing; `cargo build --locked` once.
3. Read `CLAUDE.md`, `AGENTS.md`, `.claude/skills/app-guide/SKILL.md`.

## 1. Resume before starting anything new

List open PRs labelled `agent`. For each, oldest first:
- CI red or merge conflict → fix it (step 4 onward).
- Open review threads or owner comments → address them.
- Green and approved by the review gate → merge (step 7).
An open PR updated less than 90 minutes ago may belong to a session that is still running: leave it.

## 2. Pick the issue

Open issues, excluding labels `blocked`, `awaiting-owner`, `rejected`, `in-progress` (unless its
last update is older than 12 hours — then it was abandoned: take it over).
Order: `P1` < `P2` < `P3` < unlabelled < `idea`; ties by issue number.
An `idea` issue is taken only once it is at least 24 hours old and the owner has not closed it or
labelled it `rejected` — that day is the owner's veto window.
Owner comments on the issue override the issue body.

Label the issue `in-progress`.

### Empty queue

File up to 3 new `idea` issues: things that make the edit after frename faster (the product's
purpose: review and prepare footage before editing in Premiere Pro). Each idea issue says what, why
it saves the editor time, rough size, and risk. Check closed/rejected issues first so a rejected
idea is not proposed again. Then stop.

## 3. Design gate (issues labelled `needs-design`)

If `docs/design/<slug>.md` for this issue is not merged yet:
1. Research what the feature depends on (formats, APIs, Premiere behaviour) and cite sources.
2. Write `docs/design/<slug>.md`: problem, user flows, UI sketch (ASCII or SVG), keyboard shortcuts,
   data format, edge cases, what is out of scope, test plan, open questions with a recommended answer.
3. Run the review gate on the design (`review-gate`, design mode).
4. Open a docs-only PR, merge it when green, comment on the issue with a short summary
   and the questions, label the issue `awaiting-owner`, remove `in-progress`. Go to step 2 for the
   next issue.

The owner answers on the issue and removes `awaiting-owner`; then this issue is implementable.
Owner decisions are folded into the design doc as part of the implementation PR.

## 4. Implement

- Branch `agent/<issue-number>-<slug>` from fresh `main`.
- Follow `AGENTS.md`, the Iced Elm skill, and the matching project skills.
- Every behaviour change in `frename-core` gets unit tests; bug fixes get a test that failed before.
- UI changes: when the app can run on Linux, run it under Xvfb and look at a screenshot of the
  changed screen before asking for review.
- User-facing change → update `README.md` (per `readme` skill) and add a `version.md` entry
  (per `create-release-version` skill). Bump the minor version (`0.66` → `0.67`) once per PR.
- Commit in small logical steps; messages in English.

## 5. Local gate

All of these pass locally before review:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --release --locked
```

## 6. Review gate

Follow `.claude/skills/review-gate/SKILL.md`: independent reviewer subagents with fresh context,
verdicts, fix loop. Push and open the PR (label `agent`, body: `Closes #N`, summary, test evidence,
review rounds with their verdicts). Then wait for CI on the latest commit.

- CI red → diagnose from the job logs, fix, re-run the local gate, push. A failure is never "flaky"
  until the same job passed on the same commit.
- After 4 review rounds or 3 CI fix rounds without convergence: label the PR `needs-owner`, comment
  on it what is stuck and why, and move to the next issue.

## 7. Merge and release

Merge (squash) only when all hold on the latest commit:
- review gate verdict APPROVE from every reviewer of the final round;
- every CI check green;
- no merge conflict; branch up to date with `main` (merge `main` in and re-run CI if not).

After merge the `version.md` change on `main` triggers the release workflow. Watch it to completion;
a failed release is fixed immediately as the next PR.

Comment on the issue: what shipped, which version, how to try it, anything the owner has
to check by hand (e.g. Premiere Pro behaviour). Remove `in-progress`.

## Language

Everything on GitHub and in the repository is English: issues, PRs, comments, commits, docs,
README, release notes. Other languages appear only as test data (e.g. localization or Cyrillic
file-name tests).

## 8. End of session

Post nothing else. The owner reads issue comments and release notes.
