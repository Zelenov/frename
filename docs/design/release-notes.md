# Release notes in Settings → Updates (#252)

The editor sees what changed, before updating, after updating, and at any time.

## Where the text comes from

| Need | Source |
|---|---|
| Installed version and older, offline | `version.md` embedded in the build (`include_str!`) |
| Versions between the installed and a newer release | `version.md` at the newer release's tag, `https://raw.githubusercontent.com/Zelenov/frename/v0.X/version.md` |

Why not the alternatives:

- **Velopack `NotesMarkdown`**: needs `--releaseNotes` in `release.yml` (a guarded file) and only
  carries the target version, so the versions in between would need a second source anyway.
- **GitHub Releases API**: one request per version, and 60 unauthenticated requests an hour are
  shared with the update check. The file at a tag holds every version up to that release, so one
  request to `raw.githubusercontent.com` covers the whole range. The branch is the fallback when
  the tag is not there.

Both sources are the same format, so one parser (`features/updates/notes.rs`) reads them. Fetched
blocks replace the bundled block of the same version.

## Behaviour

- **Update ready** (the last check found a newer version): the Release notes row on the Updates page
  lists the versions after the installed one up to the new one, newest first. The fetch starts
  after a check finds the release, and at start-up and on the hourly tick for a version known
  from an earlier run. While it runs the block says so; if it fails the block says the notes could not be
  loaded, and a new check tries again. A fetched file without the release's block counts as a failure, so
  it is not asked for in a loop.
- **After an update**: the database keeps `seen_version` (migration 26). At start-up, a running
  version newer than `seen_version` puts a dot on the Settings button (tooltip "What's new in
  0.X") and opens Settings on Updates. Showing the Updates page records the running version, so
  the dot goes; the block stays on the page for the session. The first start that records a
  version (empty `seen_version`) says nothing.
- **Any time**: **Release notes** toggles the history of every known version, ten at a time
  (**Show more**). **All releases on GitHub** opens the releases page. Works in the Store build,
  the AppImage and the Mac download too.
- Rendering: headings, bullets, `**bold**` and `` `code` `` as styled text. Nothing else of
  Markdown is drawn. The notes stay English.

## Decisions made without the owner

- The first start after the version that brings this feature shows no "What's new" (nothing
  recorded the previous version); later updates do.
- While an update is ready its notes win over "What's new", and the dot stays until the update is
  applied (the next start then offers what changed since the version seen).
- The update's notes win over "What's new" when both apply; the history button wins over both.
- Bullets that wrap in `version.md` are joined into one bullet.
