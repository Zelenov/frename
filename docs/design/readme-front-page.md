# README as a store-style front page (#164)

## Goal

The README is the first thing a visitor sees. It should read like frename's page in a store: what
the app is for, the main features with one image each, and links to the details. Everything
technical moves to `docs/guide.md` (users) and `docs/development.md` (developers).

## Structure

1. Name, two sentences on what frename is, download links, languages.
2. The main image: the whole window on a tagged folder.
3. Feature sections, each with one image and a few plain sentences:
   - Describe each clip with AI (`describe.jpg`);
   - Mark key moments and the part to keep (`markers.jpg`);
   - Tag clips so the footage has a structure (`tags.jpg`, before and after);
   - Work on a whole folder at once (`batch.jpg`).
4. What you get in Premiere Pro (a short list, no image).
5. Your files and your keys (short facts).
6. More: the guide, the development page, `version.md`.

## What moved where

| From the old README | To |
|---|---|
| The story of the trip to Africa | `docs/guide.md`, introduction |
| Install (Windows installer and zip, old zip import, Linux AppImage), languages | `docs/guide.md` → Install |
| "What you can do with it" (the detailed feature list) | `docs/guide.md` → What frename does |
| The annotated window screenshot and the three columns | `docs/guide.md` → What frename does |
| How a session goes | `docs/guide.md` → How a session goes |
| "frename sends your footage nowhere unless you ask", the update check | README (short) and `docs/guide.md` → Your files and your data |
| Keyboard shortcuts (all tables) | `docs/guide.md` → Keyboard shortcuts |
| Your files and your data | `docs/guide.md` → Your files and your data |
| Building from source, the libraries | `docs/development.md` |

## Images

All images are screenshots of the app in demo mode (`frename --demo`), made by
`docs/screenshots/render-readme.sh` (next to the older `render.sh`, which makes the guide's
annotated window) in a 1600x860 window. Two scenarios are committed: `readme-main.toml`, a
twelve-clip folder ("Day 3") with tags, markers, an in/out range and AI descriptions, and
`readme-before.toml`, the same folder with the camera's names. The script makes the other
variants from `readme-main.toml` in a temporary folder (one key added or changed with `sed`), so
the folder is written down once.

| Image | Made from | What it shows |
|---|---|---|
| `main.jpg` | `readme-main.toml` | The whole window: video, tagged file list, tags, comment with the AI description, colored markers and in/out on the timeline |
| `describe.jpg` | `readme-main.toml` with MVI_0205 open and `comment_expanded = true` | A dragon clip next to the expanded comment box with its AI description |
| `markers.jpg` | `readme-main.toml` with `marker_list = true` | The video panel with the marker list open and the colored markers on the timeline |
| `tags.jpg` | `readme-before.toml` + `readme-main.toml` | The file list before tagging (camera names) and after, side by side with labels |
| `batch.jpg` | `readme-main.toml` with `--batch --ai` | Batch mode, every clip checked, Describe with AI with its price |

The script crops parts of the window with ImageMagick and adds the labels of `tags.jpg`. The
crop boxes are fixed numbers in the script; a layout change may need new ones, which shows at
once in the images (CI stays green, so the images must be looked at).

### Footage

The test clips are ugly, so the images use short cuts of two films by the Blender Foundation,
Tears of Steel and Sintel, under CC BY 3.0. They come from MDN's `shared-assets` repository (the
stock footage sites are not reachable from the agent's sandbox; GitHub is). Three clips of 8 to
12 seconds, cropped to 16:9 (the films are wider, a camera is not) at 640x360, without sound,
1.8 MB in all, are committed in `docs/screenshots/footage/`; every row of the folder uses one of
them. `docs/screenshots/ATTRIBUTION.md` has the credit, the source commit, the cut points and the
`ffmpeg` line, and the README credits the films under "More". The clips are listed in
`tests/self-test-clips.txt`, so CI checks that they decode.

### Demo mode changes

The images needed three small changes to demo mode, nothing in the normal app:
- a scenario's marker lines keep their color (`0:02.400 [Red] — …`); demo staging dropped it and
  every marker was green;
- the batch panel's key reads answer with the demo's key states (Anthropic saved), as the
  settings window already did, so a batch screenshot no longer shows "The system keyring could
  not be opened" on a machine without a keyring;
- `comment_expanded = true` in a scenario presses the comment box's expand button.

The pointer starts in the middle of the X screen, and a tooltip shows where it rests, so the
script's Xvfb screen (3400x1800) puts the middle outside the window.

### CI

`ci-linux` already builds the app and has Xvfb. A new step there runs `render-readme.sh` under Xvfb and
uploads `docs/images/` as an artifact, so every pull request shows whether the README images still
render, and their new state can be downloaded and compared.

## Decisions made without the owner

- **Committed clips, not a CI download.** 1.8 MB of clips in the repository keeps demo mode
  working offline and on Windows, and the images do not depend on a third-party URL staying up.
- **Film footage, not wildlife.** No CC0 nature footage was reachable; Tears of Steel is live
  action, so the clips look like a real day of shooting (a "Day 3" folder). Sintel adds a dragon,
  tagged `wildlife`.
- **No Premiere Pro image.** Premiere cannot be captured in CI, and a mock could mislead. The
  "What you get in Premiere Pro" section is a short list instead.
- **No animated hero.** A still image loads faster and stays sharp; an animation can come later.
- **No arrows on the images.** Each feature image is cropped to the one part it is about, and
  the before/after image has labels; the guide keeps the annotated window.
- **Undo and recovery** get their own short README section, since they are about your work, not
  your files or keys.
- **Images are committed, CI does not commit them.** CI renders and uploads them; a person or
  the agent commits new images after looking at them (as the readme skill says). A workflow that
  pushes to the branch would need write permissions in CI.
- **Mac download.** There is no Mac build yet (#15 is on hold), so the README says so instead of
  showing a link.
- **The old images** (`docs/frename-screenshot*.jpg`, the posters) stay: the guide still shows the
  annotated window, and `docs/screenshots/render.sh` still makes it.
- **No version bump.** Only documentation changes, so no release.
