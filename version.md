# 0.135
## Changed
- Settings → Updates on the Mac download and the Linux AppImage now says where new versions come from ("download them from the releases page") with a link to it, and keeps the running version, instead of a disabled Check for updates button.

# 0.133
## Fixed
- A clip's screenshots (`*.snap.*.jpg`) now follow it when it is renamed, and undo and redo of the rename move them back; a screenshot already under the target name stops the undo instead of being replaced.

# 0.132
## Fixed
- Pressing Enter on a tag that is already saved no longer counts as a step, so Ctrl+Z after it no longer removes the tag from the folder.

# 0.131
## Fixed
- Undo and redo of a rename no longer replace another clip that has taken the old name; the step is refused with a note and stays in the history.
- Undo and redo of a rename move the clip's comment along with it, as they already did for subtitles.

# 0.130
## Fixed
- Dragging a clip out to Premiere is no longer refused with "Not dragged: can't save the file" (nor a false "Not saved" shown) after a tag with a dot, such as v1.2, was saved.
- Dragging works when frename was opened with a relative path (`frename .`, `frename clip.mp4`).
- A very fast drag no longer starts before the press's own save of the open file ran, so Premiere does not get stale metadata.

# 0.129
## Fixed
- Closing the Settings window while the main window is closing no longer stops frename from quitting on the first close, and closing Settings no longer stops a running batch.

# 0.128
## Fixed
- Pressing next or previous quickly while a clip loads no longer makes the switch wait on the newer clip's load.

# 0.127
## Added
- A line under the file name says "All changes saved", or that your changes wait in recovery until you leave the clip.

# 0.126
## Fixed
- The marker list scrolls a far marker fully into view in a long list of long names, also in a narrow panel, instead of leaving it partly out of view.

# 0.125
## Added
- A frame step that takes long (a step back in a heavy 4K clip can take seconds) shows a turning spinner and dims the time beside the controls until the frame shows, so you can tell it is working and need not press again.

# 0.124
## Added
- "Describe with AI" says when it is waiting to send a request again, for example "retrying in 8 s (rate limit)", instead of "waiting for Claude" through the whole wait.

# 0.123
## Fixed
- The time shown beside a paused frame is exact to the millisecond in clips of several hours (it could be a millisecond or two off past about two and a half hours).
- The first frame step back in a clip no longer fails to change the picture after a frame step in the clip before.

# 0.122
## Changed
- In a narrow player, the More menu stays open after you step a frame, so you can click through frames.

# 0.121
## Fixed
- A clip you watched to the end and left again without playing it opens at its in point again, instead of two seconds before it with a "Resumed" note.
- `Home` in the search bar starts a clip over while the "Resumed" note shows, also when a file name is being edited on another row.

# 0.120
## Changed
- Starting a batch job while a marker is being described says the wait is usually a few seconds (it said up to a couple of minutes).

## Fixed
- A save refused because a marker description was still reading the clip says so, instead of only "read-only or in use".
- Describe with AI on a marker no longer skips a description just because the comment happens to end in the same last letters.

# 0.119
## Fixed
- After a clip played to its end, stepping back a frame (or jumping, or clicking the progress bar) and pressing Play goes on from there instead of starting over from 0:00. Play at the very end still starts over.

# 0.118
## Added
- Recent folders: the last ten you opened are under the arrow next to **Open a folder** (or `Ctrl+R`) and on the start screen; click one to reopen it with the clip you had open.

# 0.117
## Added
- Marker list: **Describe N unnamed** names and describes every marker without a name with AI in one click, three at a time; the others wait their turn, **Stop all** stops them, and a name you type meanwhile is kept.

## Changed
- The AI In/Out pill ends in an arrow instead of a check (it applies the suggestion, it is not "done"), and its tooltip says the range is rounded to whole seconds.

# 0.116
## Fixed
- Tag grid: the "+" on a tag that is not in the folder's tags is as easy to read as its name (it was dark grey on the dark chip).

# 0.115
## Added
- A video with a `clip.premiere.json` (the Premiere Pro transcript Generate subtitles writes) and no other subtitles shows its words like a `.srt`: under the picture, in the CC list and in fullscreen, with the subtitles icon and the "has subtitles" filter, renamed with its video, and read by Describe with AI. Lines follow Settings → Subtitles (short line or whole sentence) and start with the speaker's name when the speaker changes.

# 0.114
## Added
- A video with a `.ass` or `.ssa` subtitle file next to it shows its text like a `.srt`: under the picture, in the CC list and in fullscreen, with the subtitles icon and the "has subtitles" filter, renamed with its video, and read by Describe with AI. Styles and effects are not drawn. If a video has several, `.srt` wins, then `.ass`. Generate subtitles skips a video that has only a `.ass`, unless you tick Replace.

# 0.113
## Added
- Describe with AI also suggests which of the folder's tags fit each clip. They show under the clip's name with how sure the AI is; click one, or press `F6` for the first, or Add all (`Shift+F6`). Nothing is added until you do. Turn it off in Settings → Describe with AI.

# 0.112
## Fixed
- Fullscreen: the buttons show their tooltips again, above the picture (they had been switched off there).

# 0.111
## Added
- frename is available as a Microsoft Store package (Windows). It updates through the Store, and on its first start it copies the settings of an installed frename.

## Changed
- Settings → Describe with AI lists only Claude models. The GPT models it offered would have been sent your Anthropic key; a saved GPT choice becomes Claude Haiku 4.5.

# 0.110
## Added
- Settings → Describe with AI → Moments: segments only for what stands out (as before) or covering the whole clip. Only "Only what stands out" suggests an In and Out (the **AI** pill).

# 0.109
## Fixed
- The file search hint is shorter, "Find a file or a comment", so it is no longer cut off in the file list.

# 0.108
## Added
- Describe a marker with AI: ✨ on a marker's row, or `Ctrl+F2` on the marker under the playhead, names an unnamed marker and adds what happens at that moment to its comment, which the marker list now shows under the name. A name you gave stays. One undo takes it back.

# 0.107
## Added
- A clip you open again continues where you stopped watching it, two seconds earlier, also after a restart. A note says so; a click on the note starts it over, and `Home` always goes to the start of a clip. A clip watched to the end starts over, at its in point if it has one.

# 0.106
## Added
- Describe with AI suggests an In and Out for clips with a lead-in or lead-out around the part worth keeping. With the clip open, the suggestion shows as an **AI** pill next to the IN and OUT points; click it to use it. frename never sets them by itself.

# 0.105
## Added
- Step through a clip frame by frame: `Alt+←` / `Alt+→`, or the new buttons next to play, show the previous or next frame and pause there. Hold the key to keep stepping. While paused, the time next to the timeline shows milliseconds (`00:10.250`). It goes by the real frames, so phone clips with a variable frame rate step correctly too.

# 0.104
## Changed
- A file's full name in the file list opens beside the list instead of over the next row, so it no longer covers the file you move to.
- The marker's name over the timeline is now a pill tinted with the marker's color, so it no longer looks like a tooltip when one opens next to it.
- Tooltips show `PgUp` / `PgDn` for the previous and next file, and `Esc` for leaving batch mode.
- The chip you drag over the trash shows in a box like a tooltip's.

## Fixed
- The ✕ on the IN and OUT timecodes and the lock on a file closed by a batch action now have tooltips.

# 0.103
## Removed
- The spending tracker: Settings no longer shows what frename spent or the top-up form, and the batch page no longer shows the estimated credit left. The cost estimate before a run, the cost in the report, and **Add credit** when a service runs out stay.

# 0.102
## Fixed
- The Subtitles / Markers switch above the side list no longer shifts its icons, words and counts when you change the selection.

# 0.101
## Fixed
- A button's tooltip is no longer drawn under the marker's name over the timeline: the tooltip shows whole, on top.
- In fullscreen, the file list and tag cells under the video no longer react to the pointer, so their tooltips do not appear over the picture. Open menus (More, the file filter) now also keep the controls under them from showing tooltips.

# 0.100
## Fixed
- Clicking a subtitle or a marker in its list no longer scrolls the list: it stays where it was, and follows the playhead again once playback moves on.

# 0.99
## Fixed
- Moving markers between the video and the comment no longer leaves two markers on one moment: a comment line for a moment that already has a marker renames it, and more lines for that moment join its text.
- A clip that already has doubled markers is repaired by running either "Markers ⇄ comment" action on it; the result lists the clips where markers were merged. Names that differ are kept as `name — other name`, which reads back as a name and a comment after the next move.

# 0.98
## Fixed
- In fullscreen, buttons no longer show tooltips over the picture: not on the controls bar, not on the ⋯ button, not in the marker list. Outside fullscreen nothing changes.

# 0.97
## Changed
- Describe with AI: frames are chosen where the picture changes the most instead of a fixed interval, and a description's moments now cover only what stands out (a static or uniform clip can get none) instead of always tiling the whole clip, and a moment that only repeats the summary is left out.

# 0.96
## Added
- Your unsaved work on the open clip (tags, comment, in/out points, markers) is saved to a recovery file a second after each change. After a crash, a killed program or a power cut, the next start applies it, opens that clip and says so ("frename closed unexpectedly. Restored your unsaved work on …").
- A clip that changed in the meantime is never overwritten: the message says so and names the folder where a readable copy of your work is kept. Undo history is not restored.

# 0.95
## Fixed
- The marker list and the subtitle list keep their scroll position when you switch fullscreen on or off (the lit marker or cue is brought into view if it would be hidden).

# 0.94
## Added
- Undo (`Ctrl+Z`) and Redo (`Ctrl+Y`) also cover a rename by hand (double-click), the comment (what you type until the box loses the keys is one step), a marker's name (one step per row), and Sync up, Sync down and the lock.

## Fixed
- Two quick moves to other clips, before the video has let go of the first, are two undo steps, not one.
- Dropping a tag chip on 🗑 can be undone.
- `Shift+F2` on a marker another tool wrote without an id says that marker is read-only, not "No marker here".

# 0.93
## Fixed
- In the batch window's action list and in Settings' navigation, the mouse cursor no longer drops to an arrow between two items.

# 0.92
## Fixed
- With markers kept in the comment (Settings → Saving) and comments in text files, markers are written into the comment as you add, rename, recolor, move or delete them, not only when you leave the clip. A comment file that cannot be written marks the file with the red ✕ and is tried again at your next change.
- With markers kept in the comment, a clip whose markers are still inside the video shows them; changing one moves them all into the comment.

# 0.91
## Fixed
- The last marker in the marker list is no longer cut off at the bottom, however long its name is.

# 0.90
## Fixed
- Undo (`Ctrl+Z`) brings back a deleted marker in batch mode, and works while a marker's name is open for editing (the name field closes first).
- A turn of the video is undone in batch mode too.
- `Ctrl+Z`, `Ctrl+Y`, `Ctrl+C` and `Ctrl+V` work on a Russian or any other non-Latin keyboard layout.
- While the comment box has the keys, `Ctrl+Z` no longer undoes marker or tag changes behind your back.
- When Undo steps back to the previous clip (the history is shared by the whole folder), a note says so; Undo again undoes the change in that clip.

# 0.89
## Fixed
- Marker colors are no longer lost when markers are copied into the comment: a line reads `0:41 [red] — Lion` (green has no tag), and reading the comment back gives the same colors. Older lines still read as green.

# 0.88
## Fixed
- The marker list lights a marker only while the playhead is on it; the last marker no longer stays lit until the end of the clip.

# 0.87
## Added
- Spending on the paid services: Settings → Describe with AI and → Subtitles show what frename spent today, this month and since your last top-up. Record a top-up (the amount and the day) and they show about how much is left, marked as an estimate; the batch page shows it before Run and warns when a run costs more, with a button to the billing page. After a "no credit left" stop the estimate reads zero until you record a new top-up.

# 0.86
## Added
- The file search also looks in comments: type a word from a comment and the clips that have it are listed, with the matching line and the word marked under each name. All words you type must be found, in the name or the comment.

## Changed
- The file search matches each word on its own, so `trip goat` no longer needs the two words next to each other in a name.

# 0.85
## Fixed
- Switching from one clip to the next no longer flashes the movie icon between them: the video pane stays empty, then shows the loading spinner.

# 0.84
## Fixed
- A comment you clear no longer comes back when you return to the clip right away.

# 0.83
## Added
- Generate subtitles: choose what is written for each video, **SRT subtitles** (`clip.srt`, as before) and/or a **Premiere Pro transcript** (`clip.premiere.json`, for Import Static Transcript in Premiere's Text panel). The choice is remembered like the other batch options.

# 0.82
## Changed
- The whole main window follows the design system that Settings got in 0.77: one font (Inter, bundled), the same dark surfaces, and icons with tooltips that show their keys instead of emoji buttons.
  - Video: the controls in groups with a time readout, and a timeline with a visible playhead. The subtitle and marker lists get tabs with counts. The rotate buttons are in the new ⋯ menu, which also takes the buttons that don't fit a narrow player, the volume slider included; the wheel over the volume still changes it.
  - The in/out span is a band above the timeline, where the played part no longer hides it, and a line at the top of the marker list.
  - Subtitles in the side list take only the height their text needs.
  - File list: file names in a fixed-width font with small tag chips; the open file is marked by a bar; a clip with subtitles shows the subtitles icon. Tags that do not fit a row become "+2" instead of being cut at the edge, and long names end in "…". While a batch action runs, a line says the list is locked; after a job, each file shows an icon for what happened to it.
  - The file list's filter is a button with a menu: tick several filters, **Show all** clears them.
  - Tags: a checked tag's name is bold; stars are black so they show on every chip color. Tags not yet in the folder's tags are listed first, under a caption of their own, then the folder's tags. The columns no longer jump while you search. The order strip says in words what reordering does.
  - Batch actions: every action has an icon, and the paid ones show who bills them. Each page says what it changes. Run names what it will do ("Move 14 comments"), and when it is off, the reason is shown next to it. A finished job shows its outcome once, the counts as figures, and the files not done as a table.
  - Empty screens say what is missing and offer the button that fixes it, instead of showing a lone symbol.
  - Settings → Subtitles: the cue length comes before the list of languages.
- The main window can no longer be made smaller than 900 × 560.
- Picking another batch action after a job finished now opens it (it used to keep showing the result).
- A double-click to rename a clip no longer loads its video twice.

## Removed
- The batch action "Move in/out points out of file names", which only existed for names written by old versions.

# 0.81
## Added
- Right-click a file in the list, or the open file's name, for a menu: Show in Explorer (the folder opens with the file selected; on Linux the file manager), Copy full path, Copy file name. The keys work without the menu on the open file: `F11`, `Shift+F11`, `Ctrl+F11`. The file's tags are saved first, so what you copy is its final name.
- Select several clips without turning on batch mode by hand: `Ctrl+click` a clip to add it (and the one open before it) to the selection, or drop it if it is already selected; `Shift+click` a clip to select every clip between it and the last one you `Ctrl`-clicked. Either turns on batch mode with exactly that selection checked; `Esc` leaves it, same as clicking ☑ again.
- Batch mode remembers the action you last ran, and its own options (e.g. the subtitle "Replace existing" checkbox, which way in/out points or markers move), and opens with them selected, even after a restart.
- Each folder remembers the clip you were last viewing in it, in its own `.frename` file, so switching between shoots no longer loses your place in the one you switch back to. A renamed clip is still found; if it is gone, the first clip in the list opens instead.

## Changed
- Monochrome tags now use two grays instead of one: a tag you added shows darker, a tag that is not in your list (a stray word from a file name) stays the lighter gray it always was. The two were indistinguishable before.
- Pasting tags onto a clip no longer wipes its comment.
- A rename that would give a clip the same name as another one (or its comment, subtitle or transcript) is refused instead of replacing it, with a note that it was not saved.
- Leaving a clip and moving to the next one before its video has finished loading no longer risks the tag you just set failing to reach the file.

## Fixed
- The volume control no longer disappears when the video panel is narrow: below a width it collapses to its icon, which you scroll to change the volume. The panel can no longer be narrowed past the point where a button would be cut off.

# 0.80
## Changed
- Closing frename, opening another folder, or opening a file outside the current folder now saves the open file's tags, comment and in/out instead of losing them.

# 0.79
## Added
- Settings → In/out points: keep them in the comment, as one line (`In/Out: 00:01:05.250 – 00:02:10.000`) you change with `[` and `]`, or inside the video (the default).
- Batch action "Move in/out points out of file names": moves `in_…` / `out_…` from the names of the checked files to where in/out points are kept now, and says which files kept the points they already had.

## Changed
- In/out points in file names (`in_00_01_05`, `out_00_02_10`) are no longer read or written: they show as part of the name until you run "Move in/out points out of file names". If you kept them in names, Settings now keep them inside the video.
- "Move in/out points" is now "In/out points: comment ⇄ video (XMP)".
- mkv and other formats that cannot hold XMP keep in/out points in `.comment.txt` instead of the file name.

# 0.78
## Added
- Rotate a clip shot sideways: ↺ / ↻ in the video controls or `Ctrl+Alt+←` / `Ctrl+Alt+→`. Only the MP4/MOV rotation flag changes (no re-encoding, comment and markers stay), Premiere Pro shows the clip turned when it imports it (a clip it imported before needs its media cache cleared and a new import), and `Ctrl+Z` turns it back.
- Batch action "Rotate videos": 90° right, 90° left, 180°, or reset to no rotation, for the checked files.

## Changed
- Clips a phone recorded in portrait now play upright instead of on their side.
- Short notes such as "Frame saved" now appear over the picture instead of in the controls bar, and the controls bar is a little tighter.

# 0.77
## Added
- Drag clips from the file list into Premiere Pro, Explorer or other programs (Windows): the clip is saved first, and dropping on a folder copies it. In batch mode a checked file drags all checked files.
## Changed
- Settings is rebuilt: five pages (Interface, Saving, Describe with AI, Subtitles, Updates) instead of one long list, one option per line with a short note, and clear buttons for the API keys.
- Settings closes with its Close button or `Esc`, and `Ctrl+Tab` moves between its pages. It opens on the page that is needed: on Updates when an update is ready, on Describe with AI or Subtitles from those batch actions.
- Settings uses frename's own fonts, so it looks the same on Windows and Linux.

# 0.76
## Added
- Right-click 📂 to pick one clip: its whole folder opens with that clip selected.
- `frename <folder>` or `frename <file>` opens it: "Open with", dropping onto frename.exe or its shortcut.
- The installer adds "Open in frename" to the Explorer menu of folders and videos; uninstalling removes it.

## Changed
- 📂 chooses a folder instead of a file.
- Dropping a folder on the window opens that folder, not the one around it. Dropping several items opens the first folder among them, or else the first file.

# 0.75
## Changed
- Images are no longer opened; frename works with video files only.

# 0.74
## Added
- A UI language, in Settings: follows the system by default, or pick English or Русский. More
  languages ship as they are ready.

# 0.73
## Added
- Ranges: hold `F2` (or 📍) while a clip plays to mark a stretch, or `Alt`+drag on the paused bar. A band above the progress bar shows it; overlapping bands stack.
- Drag the handles at the ends of the current range to change it (`Shift` snaps to other markers and to in/out); drag them together or `Alt`+click the band to make it a single moment again. Click a band to play just that stretch.
- Settings → Markers and ranges: keep them inside the video (XMP) or in the comment, one line each; changing it offers to move the existing ones.
- Describe with AI writes the moments it finds as white "AI" markers in the video (with markers kept in the video); a new run replaces them and leaves your own markers alone.
- Markers ⇄ comment turns the moments of an AI description into white markers, and no longer copies them back as extra lines.
- The color picker shows "AI" (white) apart from the other colors.

## Changed
- The progress bar always has its own row above the buttons.
- The marker label sits over the middle of a range and is cut with "…" to fit the player.
- Marker names wrap in the list and in the name field; a click on a row renames it (no ✎).
- 📍 shows pressed while `F2` is held; holding `F2` no longer opens the marker's name.
- Markers can be edited with the batch panel open when no job runs.
- The comment box fills its space when expanded; ⛶ / ⊡ replace Expand / Collapse.
- The file list keeps the marker count next to 📍 however long the name.

# 0.72
## Added
- Batch action "Generate subtitles": transcribes the checked videos with Soniox and saves a `.srt` next to each, with the audio length and cost shown before you start, and why each video got no subtitles listed afterwards. You need your own Soniox API key.
- Settings → Subtitles: the Soniox API key (kept in the system's password store), the languages spoken in the footage (any language Soniox supports), and short or sentence-long subtitles.

## Changed
- A saved `.soniox.json` transcript is renamed together with its video.

# 0.71
## Added
- Batch action "Describe with AI": Claude watches each checked video (a frame every 2 s, and its subtitles) and writes a one-line summary and time-ranged segments into its comment, after your own text. The panel shows the price and time before it runs; you need your own Anthropic API key.
- Settings → AI: the Anthropic API key (kept in the system's password store), the model (Claude Haiku 4.5, Sonnet 5 or Opus 5, with prices) and the language of descriptions.
- A new run replaces only the AI part of a comment, from its last `AI: ` line to the end; everything above it is kept as it is.
- The batch report offers Add credit when the Anthropic account has no credit left, and Retry for the files that failed or were not reached.
- The comment box scrolls, can be made taller or shorter with the bar above it, and Expand gives it the whole panel.

## Changed
- Batch mode starts with the open file checked.
- The batch progress bar moves while a file is in work (frames read, waiting for Claude) instead of only when it finishes.
- Turning batch mode on or off keeps the file list's scroll position.

# 0.70
## Added
- Markers: `F2` (or 📍) marks a moment of the video, `F2` again names it. Each marker has a name and one of Premiere's colors, and is saved inside the video, where Premiere Pro shows it on the clip after import.
- The marker list (◆, in fullscreen too) shares the side of the picture with the subtitle list, with a tab for each (CC Subtitles, ◆ Markers): click a marker to jump to it or edit it. An empty list has an Add button.
- `Shift+F1` / `Shift+F3` jump to the previous / next marker, `Shift+F2` deletes the marker under the playhead, and dragging the progress bar with `Shift` snaps to markers. Markers show as pins in their colors on the progress bar.
- While the playhead is on a marker, its name becomes the head of its pin above the progress bar, the way a subtitle line shows. Click it to rename the marker.
- The batch report has an Open log button next to the failed files: the log says why each one failed.
- The file list shows 📍 and the number of markers of each file, and the Filter dropdown can show only files with markers.
- Batch action "Markers ⇄ comment" turns comment lines like `03:24 — Take 3` into markers, or copies markers into the comment.

## Changed
- `F12` just saves the current frame as a JPEG next to the video, like VLC: no mark on the progress bar, no time in the comment, and the picture is no longer renamed with the video. Frames saved before stay on disk.
- `F1`, `F3` and `F12` no longer react while `Shift` is held.
- The Filter dropdown moved from the bottom bar into the file search bar.
- In fullscreen the CC button is shown too and opens or closes the subtitle list; before, fullscreen always showed the list.
- A video whose content is not a video (all zeros, as a download that did not finish leaves it) is reported as damaged in the log, not as a format that cannot hold markers.
- Long button labels and file names in the batch panel no longer run past its edge.
- When the video panel is narrow, the progress bar gets a row of its own above the buttons, so it stays long enough to seek.

# 0.69
## Added
- A Windows installer: download `frename-win-Setup.exe` from the release page and run it. It installs without questions, and video plays with nothing else to install.
- Updates: Settings → Updates checks for a new version and installs it with **Update and restart**. frename also checks once a day by itself and puts a dot on ⚙ when an update is ready.
- On first start, the installed frename finds the folder of an older zip version and offers to import your settings and recent folders; Settings → **Import from an old frename folder…** does it for a folder kept elsewhere.

## Changed
- The Windows zip is now a portable frename with video playback built in: unzip and run, no GStreamer to install. It keeps its settings in its own folder and updates itself too.
- The installed frename keeps its settings in `%LocalAppData%\frename`.

# 0.68
## Added
- A setting puts a space after each tag in file names (`Food. Goat. clip.mp4`); names are read the same either way.
- Batch action "Apply tag spacing" renames checked files to the chosen spacing; Settings offer it after the setting changes.

# 0.67
## Added
- A Linux version: one AppImage file from the release page, with video playback built in.

# 0.66
## Added
- Batch mode (☑ in the controls bar): check files in the list (All / Invert) and run an action on all of them, with progress, Cancel and a per-file result — a green or red check box.
- Batch actions: move comments between the video and text files, move in/out points between the file name and the video, tag commented videos, fix tags by priority (tag panel order), and reset cache and reload.
- After switching comment or in/out storage, Settings offer the matching batch action for files already stored the other way.
- A check box turns the "Commented" tag off.

## Changed
- The "Commented" tag is checked when a video gets its first comment and unchecked when the comment is cleared; otherwise it is yours to change, and it is no longer forced back on save. A new one goes last.
- Settings no longer convert the whole folder; batch actions do that.
- Starred tags are filtered by the tag search too.
- The progress bar and subtitle highlight no longer freeze after seeking.

# 0.65
## Added
- While comments are stored inside the video, every video with a comment gets a "Commented" tag in its name, removed again with the comment. The tag's wording is a setting.
- The file list shows the first line of each file's comment under its name.
- Converting a folder shows progress and can be cancelled.

## Changed
- Folders open instantly again with comments stored inside the videos: comments load in the background, with a spinner in the list, and are remembered in the folder's `.frename` file. A file you open always shows its comment right away.
- `.frename` is now TOML, with multi-line comments as plain text. The older JSON files are no longer read.
- The subtitles marker sits on the file name line.

# 0.64
## Added
- Comments can be saved inside the video file (XMP), where Premiere Pro shows them in its Description column and finds them by search. Settings choose this (the default) or a `.comment.txt` next to the video.
- In/out points can be saved inside the video as an Adobe marker, which Premiere Pro turns into a subclip; the file name then carries no in/out. Settings choose this or the file name (the default).
- Settings show which files in the open folder still keep comments or in/out the other way, and convert the whole folder in one click, in either direction.
- Filter the file list to files with subtitles or with a comment. All filters are in one Filter dropdown, with counts.
- Double-click a file in the list to rename it in place: Enter renames, Esc cancels. A name another file already has is refused.

## Changed
- Writing comments or in/out into a video keeps its modified and created dates.
- A video no longer fails to play after it was renamed while open.
- The file list scrollbar no longer covers long names.

# 0.63
## Added
- Subtitles: a `.srt` next to a video is shown under the picture, and over it in fullscreen with a list of every line. Click a line to jump to it; the list follows playback.
- The CC button opens the subtitle list outside fullscreen too.
- The file list marks videos that have subtitles.
- A settings window (gear button): start videos playing when opened, and draw every tag in one neutral color.

## Changed
- The file list shows only videos; subtitles, text files and images are hidden.
- Subtitles are renamed together with their video, undo included.
- Comments are saved as UTF-8 with a BOM, so Windows editors no longer show Cyrillic as garbage.
- The per-file "Paste tags" button is gone; Ctrl+C / Ctrl+V still copy tags.

# 0.62
## Added
- Show only the files that have no tags yet, so a folder can be worked through to the end.
- Each folder now keeps its own tags in a `.frename` text file next to the footage. It travels with the folder and can be edited by hand or by an AI: one tag per line, and the line order is the tag order.
- Search the file list by name, level with the tag search. It matches the name as it is on disk, so the tags already in a name are searchable too, and it narrows the list together with the untagged filter.

## Changed
- Tags belong to the folder instead of being shared by every folder. A folder without a tag file starts from the built-in set.
- iPhone clips recorded at a variable frame rate now play instead of showing an error.
- Tags saved in earlier versions are not carried over: the app database no longer stores tags.

# 0.61
## Changed
- Dragging the panel splitters, moving the window and the volume slider no longer stutter.
- Folders with thousands of files open almost instantly.
- Scrolling and typing stay smooth in large folders.
- A paused video no longer keeps redrawing the window.

# 0.60
## Added
- Initial release. Tag-based video file reviewer for editors: play clips, assign tags, copy tags from previous file, add comments, mark screenshots.
