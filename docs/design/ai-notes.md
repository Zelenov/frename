# Notes for the AI (#310)

The editor knows things the AI cannot see: who a person is, what is worth marking, which tags to
suggest. A free-text note is added to every AI request of a folder.

## Behaviour

- **Folder notes** live in the folder's `.frename` file (`ai_notes`, a TOML string, omitted when
  empty), so they travel with the footage like its tags. They are read when the folder opens and
  edited in the Describe with AI panel ("Notes for the AI"). Typing is written to the file once it
  pauses for 700 ms, from the UI thread, in the order typed, so a keystroke never rewrites a file
  on a slow drive.
- **One-off line** ("Extra line for this run"): kept in memory, joined to the notes
  (`For this clip or run only: …`) for the next run, and dropped when a run takes it.
- **Where they apply** — every request frename sends through clipscribe's `Options.instructions`:
  batch Describe with AI and Prepare clips (the `Run` carries the text), the clip buttons (they go
  through the same batch options), tag suggestions (the same request), and describing a marker
  (folder notes only). clipscribe adds the section after the fixed instructions; the answer's schema
  does not change.
- **Cap:** 2,000 characters (clipscribe's `MAX_INSTRUCTIONS_CHARS`). Over it the panel says so in
  red and Run is off; a marker request fails with the library's text.
- **Cost:** the notes' bytes are added to the subtitle bytes of every estimate, for clips and
  markers.
- **Re-runs:** clipscribe's cache key includes the notes, so a changed note never reuses an old
  answer. A described clip is still skipped unless Redo is ticked, as before: old descriptions
  are never overwritten silently.
- **"Notes on":** a mark in the corner of the comment box, before the AI buttons, with the notes as
  its tooltip, while the folder has notes.
- **Privacy:** the README says the notes go to the AI company with the frames. Names come only from
  the notes.

## Decisions made without the owner

- One text for the folder, no per-run saved notes; the one-off line is a single line.
- The mark sits in the clip's comment corner and the Describe panel; the marker ✨ buttons do not
  get their own mark (they share the folder's notes).
- The first/last-take example of the issue's second comment needs the clip's subtitles in the
  request, which Describe with AI already sends; the wording of such a note is the editor's. No
  special code path, and no "generate subtitles first" prompt in this step.
- Machine translations in all ten shipped languages.
