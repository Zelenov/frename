# Drag files into Premiere Pro

Design for issue #47. Drag a clip from frename's file list and drop it into Premiere Pro (project
panel, timeline, source monitor), Explorer, the desktop or any other program that takes files
dragged from Explorer.

## Problem

frename is where footage is reviewed and tagged before the edit. Getting a clip into Premiere
today means finding it again in Explorer or in Premiere's media browser. The clip is already on
screen in frename; dragging it from there is the shortest way.

iced and winit can receive dropped files but cannot start a drag that leaves the window, so this
needs platform code.

## User flows

1. **One clip.** Press on a file's row in the list and move the mouse a few pixels with the button
   held. The row is selected as by a click (and the open file's edits are saved, as a click on its
   row already does), then the pointer turns into the system drag pointer with the file's icon.
   Drop it on Premiere's project panel, timeline or source monitor: Premiere imports the file under
   its final name, with the comment, in/out and markers frename wrote into it.
2. **Several clips.** In batch mode, check the clips, then drag one of the checked rows: every
   checked file in the list goes, in list order. Dragging an unchecked row drags only that file.
3. **Copy to a folder.** Drop on Explorer or the desktop: the files are copied. frename never
   offers a move, so nothing disappears from the folder, even on the same drive.
4. **Changed your mind.** Press Esc during the drag, or drop back on frename: nothing happens.

Nothing else changes: a click still selects, a double-click still renames in place, tag chips
still drag inside the rename panel and onto the trash zone.

## UI sketch

No new controls. The only visible parts are the operating system's drag pointer and drag image
(the file's icon or thumbnail with a count badge for several files, drawn by Windows), and the
"+ Copy" / "Link" badge the drop target shows.

```
 ┌ files ─────────────────┐
 │ Goat.Food.clip01.mp4   │  press here, move 6 px ──►  [🎞 clip01.mp4 +Copy]  ──►  Premiere
 │   comment line         │
 │ clip02.mp4             │
 └────────────────────────┘
```

If the open file cannot be saved before the drag (read-only, locked by another program), the drag
does not start and the video controls bar shows `Not dragged: the file could not be saved`, the
place other save problems are shown.

## Data format

- **Windows**: the shell's own data object for the files, the same object Explorer hands out when
  its files are dragged. It carries `CF_HDROP` (the paths, which Premiere, Resolve, Explorer and most
  programs read), `Shell IDList Array`, and the drag image. Allowed effects: `DROPEFFECT_COPY |
  DROPEFFECT_LINK`, never `DROPEFFECT_MOVE`.
- **Linux**: `text/uri-list` would be the format, but see "Linux" below: not supported yet.

## Platform approach

### Windows

1. Paths → item ID lists: `SHParseDisplayName` for each path.
2. ID lists → `IShellItemArray` (`SHCreateShellItemArrayFromIDLists`) → the shell data object
   (`IShellItemArray::BindToHandler(BHID_DataObject)`). All files come from the open folder, a flat
   array as `BHID_DataObject` asks. [1]
3. `SHDoDragDrop(hwnd, data, NULL, COPY | LINK)`. With no drop source the shell supplies one
   (Vista and later) and a drag image. [2] It runs the modal OLE drag loop and returns when the
   file is dropped or the drag is cancelled.

Crate: `windows` 0.62 (already in `Cargo.lock` through other dependencies, so nothing new is
downloaded), features `Win32_UI_Shell`, `Win32_UI_Shell_Common`, `Win32_System_Com`,
`Win32_System_Ole`, `Win32_UI_Input_KeyboardAndMouse`, `Win32_UI_WindowsAndMessaging`. About 80
lines in `src/features/drag_out/windows.rs`, behind `#[cfg(windows)]`.

Why not the `drag` crate (CrabNebula): on Windows it builds its own data object with `CF_HDROP`
only; the shell's object is exactly what Explorer gives Premiere. Its Linux side needs a GTK window,
which a winit window is not, so it would not help there either, and it pins its own `windows`
version.

**The window handle.** `iced::window::run(main_window, |window| …)` runs a closure with the window
(`HasWindowHandle`) on the event loop thread; `RawWindowHandle::Win32` gives the `HWND`
(`iced_runtime` 0.14 `window::run`, handled in `iced_winit` 0.14.1 `run_action`,
`window::Action::Run`).

**UI thread and OLE.** `DoDragDrop` must run on a thread where `OleInitialize` was called. winit
calls `OleInitialize` on the event loop thread when it registers its drop target, which it does by
default (`drag_and_drop: true`, winit 0.30.13 `platform_impl/windows/window.rs`), and frename
already relies on that drop target for opening dropped folders. The closure above runs on that
thread. While the modal drag loop runs, winit's window procedure is re-entered; winit buffers the
events it cannot deliver then (`EventLoopRunner::send_event` pushes to `event_buffer`) and delivers
them after. So frename's window does not redraw during the drag (the video frame stands still until
the drop), which is how Explorer-style drags from other winit apps behave too.

**Mouse capture.** winit captures the mouse on a button press and counts presses. The drag loop
takes the capture; winit resets its count on `WM_CAPTURECHANGED`, so no capture is left behind.
The button release is eaten by the drag loop, so frename resets its own drag state when
`SHDoDragDrop` returns rather than waiting for a release.

**Button already up.** If the button is released before the drag begins (the save took a moment),
starting the drag loop would drop at once wherever the pointer is. Before starting, the code checks
`GetAsyncKeyState` for the left button (the right one when buttons are swapped, since it reads
physical buttons) and does nothing when it is up.

### Linux

Not supported yet: the press and move on a row do nothing, as today. What it would take:
- X11: an XDND source owns the `XdndSelection`, sends `XdndEnter`/`XdndPosition`/`XdndDrop` to the
  window under the pointer, and answers `XdndStatus`, `XdndFinished` and `SelectionRequest` events,
  all on the X connection and event loop winit owns and does not expose to the app.
- Wayland: `wl_data_device.start_drag` needs a `wl_data_source` on winit's connection and the serial
  of the button press, neither of which winit hands out.
- GTK (the `drag` crate) needs a GTK window.
A separate X11 connection with its own XDND source is possible but large and X11-only; it becomes
an `idea` issue if the owner wants it. macOS is not a target.

## Triggering the drag

State `DragOutState` in the workspace:
- A press on a row (the existing `SelectFile(index)` from the row's `mouse_area`) arms it with that
  file's ID. The first cursor position seen after the press is the origin.
- A subscription, only while armed, listens to cursor moves and left-button releases.
- A move of at least `DRAG_THRESHOLD` = 6 logical px from the origin (Windows' default `SM_CXDRAG`
  is 4 physical px) asks to start the drag; a release disarms.
- The drag starts at once, not when the pointer leaves the window: it is what Explorer does, the
  user sees the drag pointer from the start, and a drop back on frename is ignored.

**Which files.** Pure function: in batch mode with the pressed file checked, every checked file
that is listed, in list order; otherwise the pressed file. Files whose path no longer exists are
left out; none left → no drag.

**Save before drag.** Pressing the open file's row already saves its pending edits (tags in the
name, comment, in/out, markers), unloading the video first when one plays, as a re-click does
today. Pressing another row saves the previous file and opens a clean one. So at drag time the
workspace decides (pure function):
- a save is still in flight (the video is unloading) → wait; the next cursor move asks again;
- the open file is among the dragged ones and its name on disk differs from the name frename wants
  for it, or its markers are in the not-saved list → save it now (as a re-click) and wait, once;
  after that one attempt → refuse with the notice above;
- otherwise → start, with the paths read from the directory after the save (the new names).

**A drop on frename itself.** winit's drop target accepts files, so dropping the dragged files back
on frename's window would open them. The app tells the drop feature which paths it is dragging;
drops of those paths are ignored while the drag runs and for one second after it ends (the drop
events are buffered by winit and may arrive after the drag's result).

## Edge cases

- Batch job running: rows cannot be pressed, so no drag.
- In-place rename editor open on a row: that row is a text field, pressing it edits text.
- File renamed by the save: the drag carries the new path.
- A dragged file that another program locks: Explorer copy or Premiere import reports it, as for
  any file dragged from Explorer.
- The video is still open for playback during the drop: GStreamer opens files for shared reading,
  so Premiere and Explorer can read it.
- Releasing the button before the save finishes: no drag.
- Several clips from batch mode that are not the open file: they have no unsaved edits (only the
  open file is edited), so they go as they are on disk.
- Files larger than 4 GB, long paths, non-ASCII names: the shell data object handles them as it
  does in Explorer.

## Out of scope

- Dragging from the video preview or from the file name panel: the preview's press pauses and
  scrubs, and the name panel's chips already drag inside frename; the list row is where a file is.
  (Decision below.)
- Linux and macOS drags.
- Dragging a marker range as a subclip, or frames as stills.

## Test plan

Unit tests (all platforms):
- threshold: a move below 6 px does not start, at 6 px starts once, a release disarms, a new press
  re-arms;
- which files: batch mode with the pressed file checked → the checked listed files in list order;
  unchecked → only it; outside batch mode → only it;
- save decision: in flight → wait; unsaved and not tried → save first; unsaved after trying →
  refuse; clean → start;
- own drop: dropped paths of the running drag, and within a second after it, are ignored; other
  paths and later drops open as before.

Windows-only test: build the shell data object for two temp files and read `CF_HDROP` back with
`DragQueryFileW`: the same two paths come out. This checks the data Premiere reads without a UI.

By hand (owner, Windows): drag a tagged clip with in/out and markers into Premiere's project panel,
timeline and source monitor: its name has the tags, in/out and markers are there. Drag onto
Explorer: a copy appears and the original stays. Batch mode, several checked: all arrive. Edit tags
then drag at once: Premiere gets the new name. Make the file read-only and change its tags, then
drag: the notice shows and no drag starts.

## Decisions made without the owner

1. **Own Win32 code or the `drag` crate?** Own code with the `windows` crate and the shell's data
   object: identical to an Explorer drag, no GTK, no new crate downloaded.
2. **When does the OS drag start: after a small move, or when the pointer leaves the window?**
   After a 6 px move with the button held, like Explorer. Leaving the window would leave the user
   without any drag feedback inside frename, and relies on winit reporting cursor moves outside the
   window during capture.
3. **Multi-select?** There is no multi-select outside batch mode; batch mode's check boxes are the
   selection: dragging a checked row drags all checked listed files.
4. **Drop back on frename?** Ignored, so dragging and changing one's mind changes nothing.
5. **Allowed effects?** Copy and link only; Explorer copies, Premiere imports.
6. **Save failure?** No drag, the notice `Not dragged: the file could not be saved`.
7. **Drag from the preview or the name panel?** No (out of scope above); can be added later.
8. **Linux?** Not supported yet, documented in the README and the PR.
9. **Frozen video during the drag?** Accepted: the modal drag loop owns the thread, as in other
   winit apps; playback continues after the drop.

## Sources

1. IShellItemArray::BindToHandler, `BHID_DataObject`:
   https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitemarray-bindtohandler
2. SHDoDragDrop (default drop source when `pdsrc` is NULL, drag image):
   https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shdodragdrop
3. DoDragDrop needs OleInitialize on the calling thread:
   https://learn.microsoft.com/en-us/windows/win32/api/ole2/nf-ole2-dodragdrop
4. GetAsyncKeyState reads physical mouse buttons (swap check):
   https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate
5. iced 0.14 `window::run`: `iced_runtime-0.14.0/src/window.rs`; winit 0.30.13 OLE init and capture:
   `src/platform_impl/windows/window.rs`, `src/platform_impl/windows/event_loop.rs`.
