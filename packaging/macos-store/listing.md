# Mac App Store listing (English, U.S.)

Paste each field into App Store Connect → Apps → frename. Limits are Apple's
([App information](https://developer.apple.com/help/app-store-connect/reference/app-information/app-information),
[Platform version information](https://developer.apple.com/help/app-store-connect/reference/app-information/platform-version-information)).
`docs/mac-app-store-setup.md` says when to do this. Wording follows the Microsoft Store listing
(`packaging/store/listing.md` on #51's branch) so both stores describe the same app.

## App Information page

**Name** (up to 30 characters)

frename

**Subtitle** (up to 30 characters)

Tag and rename video clips

**Primary category:** Photo & Video. **Secondary category:** Productivity.

**Content rights:** "No, it does not contain, show, or access third-party content."

**Age rating:** answer "None" to every question: 4+.

## Pricing and Availability page

Free (or the price the owner chooses), all countries.

## App Privacy page

**Privacy policy URL:** https://github.com/Zelenov/frename/blob/main/packaging/store/privacy-policy.md
(`packaging/store/privacy-policy.md`, shared with the Microsoft Store, #51; it covers the Mac
builds).

**Data collection:** "No, we do not collect data from this app." frename has no account, analytics
or server. The two optional AI actions send frames or audio of the clips the user selects
directly to Anthropic or Soniox under the user's own API key; neither the developer nor a
partner of the developer receives it, which is outside Apple's definition of collection
("transmitting data off the device in a way that allows you and/or your third-party partners to
access it", [Apple](https://developer.apple.com/app-store/app-privacy-details/)).

## Version page (macOS App, Prepare for Submission)

The version record reads exactly the build's version, e.g. `0.77` (not the default `1.0`).

**Promotional text** (up to 170 characters; can change without a new build)

Tag your footage before you edit it. Watch each clip, tag what you see, and frename renames it
with its tags when you move on.

**Description** (up to 4,000 characters)

frename is for the hour before the edit begins.

You come back from a shoot with hundreds of files named MVI_0001.MP4 to MVI_2000.MP4. Before you
can edit, you need to know what is in each one. frename lets you watch each clip, tag it in
seconds and move on. By the time you open your editing app, the footage is already organized.

HOW IT WORKS
Open a folder of videos. The first clip plays. Click tags (or use the keyboard) for what you see:
pick, skip, wide, drone, golden-hour, people... When you press Page Down for the next clip, the
one you leave is renamed with its tags, for example pick.drone.landscape.MVI_0404.mp4. Tags live
in the file name, so Finder, Premiere Pro and your sync tools all see them. Nothing is locked
inside frename.

WHAT YOU CAN DO
- Tag clips with one click or one key; copy the tags of one clip and paste them onto the next.
- Mark the usable part of a clip with in and out points ([ and ]).
- Press F2 (fn+F2 on a MacBook) to drop a marker at an interesting moment, name it, give it a color, or hold F2 to mark
  a range. Markers are saved inside the video and Premiere Pro shows them on the clip.
- Write a comment per clip; it is saved inside the video, where Premiere Pro shows it, or as a
  text file next to it.
- Save the current frame as a JPEG with F12.
- Undo and redo tagging, markers, in/out points and renames.
- Run batch actions over a whole folder: re-order tags in every name, tag commented clips, and
  more.
- Describe with AI (optional): Claude writes a one-line summary of each clip and what happens when,
  as markers and comment text. Uses your own Anthropic API key; the panel shows the price first.
- Generate subtitles (optional): Soniox turns the speech in each clip into an .srt file next to
  it. Uses your own Soniox API key.
- Every folder keeps its own tag list in a small .frename file that travels with the footage.
- English and Russian interface.

PRIVACY
No account, no ads, no analytics. Your videos stay on your Mac. Only the two optional AI actions
send data (frames or audio of the clips you select), directly to the service you choose, with
your own key. Keys are kept in your Keychain.

Video playback is built in: MP4, MOV, MKV, WebM, AVI and more, with H.264, HEVC, VP9, AV1 and
other common codecs.

frename is open source (MIT license): https://github.com/Zelenov/frename

**Keywords** (up to 100 bytes, comma-separated, no spaces after commas; exactly 100 bytes)

video,tag,rename,footage,clip,organizer,logger,Premiere,markers,editing,review,batch,subtitles,notes

**Support URL:** https://github.com/Zelenov/frename/issues

**Marketing URL:** https://github.com/Zelenov/frename

**Copyright:** 2026 Eugene Zelenov

**What's New in This Version** (up to 4,000 characters; not shown for the first version)

The newest block of `version.md`, without its `# X.Y` heading.

**Screenshots** (16:10, 1 to 10, PNG or JPEG without transparency): upload in this order from
`packaging/macos-store/screenshots/`, all 2560×1600:

| File | What it shows |
|---|---|
| `1-main-window.png` | The window with its parts named: player, files, tags, comment. |
| `2-batch-actions.png` | Batch actions over the checked clips. |
| `3-monochrome-tags.png` | Subtitles, in and out points, a comment; tags in one neutral color. |

They are the README screenshots (`docs/frename-screenshot*.jpg`, 2020×1128) scaled into 2560×1600
on the app's background color. Refresh them when the README ones change:

```sh
for pair in frename-screenshot:1-main-window frename-screenshot-batch:2-batch-actions \
  frename-screenshot-mono:3-monochrome-tags; do
  convert "docs/${pair%%:*}.jpg" -resize 2560x1600 -background 'srgb(40,41,35)' -gravity center \
    -extent 2560x1600 -alpha off -strip "PNG24:packaging/macos-store/screenshots/${pair##*:}.png"
done
```

They show the app as drawn on Windows and Linux; frename draws its own window contents, so the Mac
app looks the same inside its title bar. Replacing them with captures from a Mac (TestFlight) is
better once one is at hand: App Review may ask for screenshots "of the app in use" on Mac.

**App Review Information**

- Sign-in required: no.
- Notes: "frename renames and tags video files in a folder the user opens with the 📂 button
  (bottom of the file list) or by dropping a folder on the window. To try it, open any folder
  with a few .mp4 or .mov files, click a tag on the right, then press Page Down (fn + ↓ on a
  MacBook keyboard) or click ▶ under the file list: the clip you left is renamed with its tag.
  The optional Describe with AI and Generate subtitles actions need the reviewer's own Anthropic
  or Soniox API key and are not needed to review the app."
- Contact: the owner's name, phone and e-mail.
