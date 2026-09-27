# frename privacy policy

Last updated: 2026-09-27

frename is a desktop app for reviewing, tagging and renaming video clips. It is made by one
developer, Eugene Zelenov ([github.com/Zelenov](https://github.com/Zelenov)). This policy covers
every build of frename: the Microsoft Store app, the installer and the portable zip from GitHub,
and the Linux AppImage.

## The short version

frename has no account, no analytics, no ads and no telemetry. The developer receives no data from
you. Your videos, tags, comments and settings stay on your computer. Only two optional features
send data off your computer, only to the service they use, and only with an API key of your own.

## What stays on your computer

- **Your videos and what you write about them.** Tags are written into file names. Comments,
  markers and in/out points are written into the video files or into files next to them
  (`.comment.txt`, `.srt`, `.frename`), in the folders you open.
- **Settings and history**: the folders you opened recently, the last clip in each, window size,
  volume and your choices in Settings. They are kept in a local database (`frename.db`) in
  `%LocalAppData%\frename-store` (Microsoft Store), `%LocalAppData%\frename` (installed version),
  the portable folder, or `~/.local/share/frename` (Linux).
- **A log file** (`frename_debug.log`) next to that database, recreated at every start. It holds
  technical messages such as file paths and errors. It is never sent anywhere.
- **API keys** you enter in Settings, kept in your operating system's password store (Windows
  Credential Manager, macOS Keychain, or a keyring on Linux), never in files, logs or the database.

## What leaves your computer, and when

No video data is sent until you run one of these actions on the videos you select, and each shows
what it will send and about what it costs before you start:

| Action | What is sent | To whom | Needs |
|---|---|---|---|
| **Describe with AI** | frames taken from each selected video (one every 2 seconds, at most 60 per clip) and its subtitles (`.srt`), if it has them | [Anthropic](https://www.anthropic.com) (the Claude API) | your own Anthropic API key |
| **Generate subtitles** | the audio track of each selected video, under the video's file name (which holds its tags) | [Soniox](https://soniox.com) (speech-to-text API) | your own Soniox API key |

One more request, without any video data: while a Soniox key is saved, opening Settings asks
Soniox for its list of supported languages, sending only your key.

The request goes directly from your computer to that service over an encrypted connection (HTTPS),
under your API key and your account with that service. The developer of frename never sees it.
What the service does with the data is governed by its own terms and privacy policy:

- Anthropic: <https://www.anthropic.com/legal/privacy> and its commercial terms for the API;
- Soniox: <https://soniox.com/privacy-policy>.

The results (descriptions, markers, subtitles) are written back to your files on your computer.

## Updates

- **Microsoft Store app:** the Microsoft Store installs and updates frename. frename itself makes
  no update requests.
- **Installer and portable zip from GitHub:** frename asks GitHub
  (`api.github.com`, `github.com`) whether a newer release exists: when you click **Check for
  updates**, and at most once a day at start-up unless you turn that off in Settings. The request
  carries nothing but what any web request carries (your IP address, the app's user agent). An
  update is downloaded only when you click **Update and restart**. See
  [GitHub's privacy statement](https://docs.github.com/site-policy/privacy-policies/github-general-privacy-statement).

## Children

frename is a general-purpose tool, not directed at children, and collects no personal information
from anyone.

## Removing your data

Uninstalling the Microsoft Store app or the installed version removes its database and log (the
Store app keeps its own copy, in `%LocalAppData%\frename-store`; on its first start it copies the
settings of an installed version, if there is one, and leaves that version's data as it was). The
portable version keeps its data in its own folder, which you delete. API keys can be removed in
Settings (or in Windows Credential Manager, the entries with `frename` in their name). Tags, comments, markers and subtitles are
part of your files and stay with them.

## Changes

A change to this policy is published here, in frename's public repository, with the date above
updated. The history of this file is the record of every change.

## Contact

Questions: open an issue at <https://github.com/Zelenov/frename/issues>.
