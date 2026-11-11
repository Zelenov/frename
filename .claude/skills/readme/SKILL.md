---
name: readme
description: >
  How to write and update frename's README.md, docs/guide.md and version.md entries for users.
  Use whenever a change is visible to the user, when editing README.md or the guide, or when
  reviewing user-facing text.
---

# README and guide for users

Both are for video editors, not developers. Write them with the plain-writing skill
(`.claude/skills/plain-writing/SKILL.md`): simple words, complete sentences, no dashes, no jargon,
sentence case headings, no bold for decoration.

## README.md: the front page

The README reads like the app's page in a store. A visitor must see in seconds what frename is
for (getting footage ready before the edit) and what it does well. Its shape (#164):

1. Name, one or two sentences on what frename is, the download links, the languages.
2. The main image (`docs/images/main.jpg`).
3. One short section per main feature, each with one image and two or three plain sentences:
   describe with AI, markers and in/out, tags in file names, whole folders at once.
4. "What you get in Premiere Pro".
5. "Your files and your keys": short facts that are true today.
6. "More": links to the guide, the development page and `version.md`.

Rules:
- Nothing technical in the README: no keyboard tables, settings, file formats, install details,
  storage paths, build steps or crate names. Those go to `docs/guide.md` (users) or
  `docs/development.md` (developers).
- A new feature normally changes only the guide. It reaches the README only when it is one of
  the main things frename does, and then as a sentence in the matching feature section, or as a
  new feature section with its own image when it is that big. Keep the README under ~100 lines.
- Every README image is made by `docs/images/render.sh` from a `docs/screenshots/readme-*.toml`
  scenario with the footage in `docs/screenshots/footage/`. Never edit an image by hand. When a
  change makes an image wrong, run the script, look at every image, and commit the new ones.
  New footage needs a license that allows it (CC0, CC BY, Pexels, Pixabay) and a line in
  `docs/screenshots/footage/ATTRIBUTION.md`; keep each clip short and small.
- Every image has alt text that says what it shows.

## docs/guide.md: everything in detail

- Organised by what the editor does (install, a session, open a folder, tag, mark moments,
  batch mode, settings), not by code modules.
- Every shortcut the app has is in its keyboard tables; nothing is listed that the app lacks.
- A new feature gets one or two sentences in the section where an editor would look for it, or
  its own short section.
- When adding, check whether an older paragraph can be shortened or removed.

## Both

- Second person ("Press `[` to mark…"), English only (see `AGENTS.md`).
- Never lose information when moving text: what leaves the README goes to the guide or to
  `docs/development.md`.

## version.md

Follow `.claude/skills/create-release-version/SKILL.md`. One line per user-visible change, written
as what the user can now do or what now behaves differently.
