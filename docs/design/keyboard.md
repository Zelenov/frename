# Keyboard approach (issue #168)

Proposal only: this issue changes no key behaviour. Research and sources:
`docs/research/keyboard.md`. Related: #62 (shortcut list / cheat sheet), #162 (frame stepping needs
free keys), #167 (Settings from the keyboard, built).

## Summary

1. Today any unmodified character typed in the main window goes to the tag filter. That is why
   tagging is fast, and why `J K L I O M`, `.` `,` and every other letter are unavailable.
2. Every editing app researched keeps letters for transport and marking and puts text in a field
   you enter on purpose. frename needs both.
3. Proposal: **key presets** chosen in Settings. **Classic** is today's behaviour, unchanged.
   **Editor** gives letters to transport, markers and ranges (`J K L`, `I O`, `M`), opens the tag
   search with `/`, applies starred tags with `1`-`9`, and leaves a text field with `Esc`.
4. **Tab** walks a focus ring through every control of the main window (the engine of Settings,
   #167), so every function has a key, and a **keymap** (actions, not hard-coded matches) allows
   remapping and the cheat sheet (#62).
5. Existing users stay on Classic; new installs start on Editor; Editor shows a hint the first
   times a letter that used to filter is pressed.

## 1. Current bindings

From the code at `main` (63a2bb5). "Always" = works while a text field has the cursor; "not typing"
= only when no text widget took the key (`status == Ignored`).

### Main window

| Key | Context | Action | Source |
|---|---|---|---|
| Any character, `Backspace`, `Delete` | not typing, after the keys below did not match | Focus the tag search and emulate the key | `src/app/state.rs` `main_window_event` (end, `FocusSearchBarAndKey`) |
| `Shift+Space` | always | Toggle the selected tag | `state.rs` `main_window_event` |
| `Space` | not typing | Play / pause | same |
| `←` `→` `↑` `↓` | not typing | Move the tag grid selection (wraps); also walks the recent-folders list when open | same; `folder_workspace/state.rs` `SelectDown` |
| `PgUp` / `PgDn` | not typing | Previous / next file (renames the one left) | same |
| `Delete` | not typing | Delete the selected tag from the folder | same |
| `Enter` | not typing | Save the selected unsaved tag / typed name as a tag | same (`SaveSelectedTag`) |
| `Esc` | always | In order: close More, file menu, "describe all" question, marker row, inline rename, fullscreen, batch mode, else clear tag and file searches | `state.rs`; `folder_workspace/state.rs` `EscapePressed` |
| `Ctrl+C` | not typing | Copy the file's tags | `state.rs` |
| `Ctrl+V` | always, only while tags are copied | Paste tags (replaces the file's tags) | `state.rs` `ctrl_v_paste_tags_handler` |
| `Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z` | not typing | Undo, redo, redo (Latin letter by physical key, so Russian layout works) | `state.rs` `latin_key` |
| `Ctrl+R` | always | Recent folders list | `state.rs` |
| `F5` | always | Fullscreen | `state.rs` |
| `F6`, `Shift+F6` | always, no repeat | Add the first / all tags the AI suggested | `state.rs` |
| `F11`, `Shift+F11`, `Ctrl+F11` | always, no repeat | Show file in file manager, copy path, copy name | `state.rs`; `features/file_menu/action.rs` |
| `[` / `]` | always | Set in / out point (matches the logical character, so not on a layout where `[` is another letter - to be checked) | `state.rs` |
| `Home` | not typing (typing: only in a search field while the "Resumed at" note shows) | Go to the clip's start | `state.rs` (#161) |
| `Alt+←` / `Alt+→` | not in the comment box | Step one frame (held: keeps stepping) | `state.rs` (#162 relates) |
| `Ctrl+Alt+←` / `Ctrl+Alt+→` | not in the comment box or a marker name | Rotate 90°; no repeat | `state.rs` |
| `Tab`, `Esc` | any | Only ask where the comment box focus is | `folder_workspace/state.rs` `focus_may_move` |
| `Ctrl+click`, `Shift+click` on a file | mouse | Select several files, turns on batch mode (modifiers tracked via `ModifiersChanged`) | `state.rs` |

### Video (a second subscription, always on)

| Key | Action | Source |
|---|---|---|
| `F1` / `F3` | Back / forward 10 s | `src/features/video_controls/state.rs` `shortcut` |
| `Shift+F1` / `Shift+F3` | Previous / next marker | same |
| `F2` | Add a marker; again within 1.5 s, or on a marker, opens its row; **held** = range (press to release); repeat ignored | same (`MarkerKeyPressed` / `released`) |
| `Shift+F2` | Delete the marker under the playhead | same |
| `Ctrl+F2` | Describe the marker under the playhead with AI | same |
| `F12` | Save the frame as a JPEG | same |

### Other contexts

| Key | Context | Action | Source |
|---|---|---|---|
| `Enter` | marker name editor | Close the row | `features/markers/view.rs` |
| `Esc`, `F2`; `[` `]`, `Shift+Space`, undo/redo, copy/paste, `Shift+F2` ignored | marker row open | Close row / add next marker without opening / guarded | `docs/design/markers.md`, `folder_workspace/state.rs` |
| `Esc` | file name inline rename, batch mode, fullscreen, context menu | Cancel / leave (see `Esc` above) | `folder_workspace/state.rs` |
| `Esc`, `Ctrl+Tab`, `Ctrl+Shift+Tab`, `Tab`, `Shift+Tab`, `Space`, `Enter` | Settings window | Close, page, focus ring, press | `state.rs` `settings_window_key`; `features/settings/focus.rs` |
| Arrows, `Home`, `PgUp/PgDn`, `Delete`, `Ctrl+C` | a text field with the cursor | The field takes them; `Esc` gives keys back | `docs/guide.md` "When a text box has the cursor" |

### Functions with no key today

By reading the views (`on_press` and tooltips): Open folder (button; open one file is a
right-click), Settings, Batch mode toggle (only `Esc` leaves it), the toolbar's More menu and
"scroll to the open file", the file filter menu, batch Check all / Invert / Run / Cancel and the
action list, selecting several files (mouse only), the file-name chips (reorder = drag, untag =
middle-click or drag to the bin), the sync lock and ↑ / ↓ copy-order buttons, starring a tag (☆),
each AI tag suggestion except the first (`F6`), the "AI" In/Out pill, the comment box (focus by
click, expand ⛶), the marker list and the subtitle list (CC) and their rows, marker color, marker
resize (handles), "Describe N unnamed", volume (wheel only), the "Resumed at" note, rename by
hand (double-click), the file context menu (right-click), recent folders: remove and clear. There
is no playback speed, mute, go-to-in/out, clear in/out, or move-a-marker function at all.

## 2. Options for the main window

| Option | How it works | For | Against |
|---|---|---|---|
| A. Keep, add modifiers | Letters still filter; new functions on `Ctrl`/`Alt` | No migration | Premiere keys (`J K L I O M`) impossible; two-hand chords for the most frequent actions |
| B. Hold a modifier to type | Letters are commands; hold `Alt`/`Space` to type into the filter | Stateless | Typing a tag name needs a chord for every letter or a held key; fights IME and Russian layout |
| C. Typing is a mode (entered by key, left by `Esc`) | `/` opens the tag search; `Esc` ends it; letters are commands otherwise | What Premiere, Resolve, FCP, Lightroom, Kyno do (research section 9); one extra key per search | Habit change for current users |
| D. Letters filter only while paused | Playing = commands, paused = typing | None | Same key does two things by an invisible state; rejected |
| E. Presets over C | Classic (A) and Editor (C), user switches in Settings; remap on top | Nobody is forced; Premiere-like by choice | Two schemes to document and test |

Chosen: **E**. Classic is today's table, verbatim, plus only keys that conflict with nothing.

## 3. Proposal

### Concepts

- **Action**: a named thing frename does (`play_pause`, `marker_add`, `tag_search_focus`, ...).
  A **keymap** maps keys to actions; a **preset** is a keymap; the user's remaps are a diff
  stored over the active preset.
- **Zone**: where the keys go when no text field has the cursor: **Player** (default), **Tags**
  (grid), **Files** (list), **Comment** (text box), **Markers** (list). Arrows and `Enter`/`Space`
  mean what the zone says. `Esc` always returns to Player. Zones exist in Editor only.
- The text-field rule stays as in `docs/guide.md`: a field with the cursor takes the keys it
  needs; F-keys, `Ctrl`+letter and `Esc` still reach the app.

### Preset "Classic" (default for existing users)

Today's keys, unchanged. Additions that conflict with nothing, so Classic users get the new
functions too: `Ctrl+/` cheat sheet (#62), `Ctrl+O` open folder, `Ctrl+,` Settings, `Ctrl+B`
batch mode, `Ctrl+Alt+1`..`9` toggle starred tag N, and the `Tab` ring.

### Preset "Editor" (default for new installs)

Follows Premiere Pro where its key is free in frename, Resolve for marker navigation, FCP for
keyword slots. Keys not marked "new" are as today.

| Group | Key | Action |
|---|---|---|
| Transport | `Space` | Play / pause |
| | `J` `K` `L` | Reverse / pause / forward; repeated `J` `L` speed up 1x, 2x, 4x, 8x; `K` held with `J`/`L` steps one frame (new; needs playback rate) |
| | `←` `→` | One frame back / forward, held repeats (today `Alt+←/→`, kept as alias) |
| | `Shift+←` `Shift+→` | 1 s back / forward (new) |
| | `F1` `F3` | 10 s back / forward (as today) |
| | `Home` / `End` | Start / end of clip (`End` new) |
| | `F` or `F5` | Fullscreen |
| Points | `I` / `O` | In / out (aliases `[` `]`) |
| | `Shift+I` / `Shift+O` | Go to in / out (new) |
| | `Alt+I` / `Alt+O` | Clear in / out (new; `Option-I/O` in FCP) |
| Markers | `M` | Add marker; `M` again within 1.5 s or on a marker opens its name row (as `F2 F2`) |
| | hold `M` | Range from press to release (as hold `F2`) |
| | `Shift+M` | Range marker from In to Out (new; Resolve has this as an unbound menu item) |
| | `Alt+M` | Delete marker under the playhead (Resolve, Kyno) |
| | `Shift+↑` `Shift+↓` | Previous / next marker (Resolve) |
| | `Alt+Shift+←` `Alt+Shift+→` | Move the marker under the playhead by a frame (new) |
| | `F2` etc. | Aliases of the Classic marker keys stay bound |
| | `Ctrl+M` | Marker list |
| Files | `↑` `↓`, `PgUp` `PgDn` | Previous / next file (arrows new in Player zone) |
| Tags | `/` | Focus the tag search (new); `Esc` leaves it |
| | `1`..`9`, `0` | Toggle starred tag 1..9, 10th (new; FCP `Control-1`..`9`) |
| | `Enter` in search | Add the typed name as a tag (as today) |
| | `Ctrl+Enter` in search | Toggle the first match, clear the search, return to Player (new) |
| | `Shift+Space`, `Ctrl+C`, `Ctrl+V`, `Ctrl+Z`, `Ctrl+Y`, `F6` | As today |
| | `Alt+T` | Zone Tags: arrows move the grid, `Space`/`Shift+Space` toggle, `Delete` deletes (new) |
| Search | `Ctrl+F` | Focus the file search (new) |
| Comment | `C` | Focus the comment box (new); `Esc` leaves |
| App | `Ctrl+/` | Cheat sheet (#62) |
| | `Ctrl+O`, `Ctrl+Shift+O` | Open folder, open one file (new) |
| | `Ctrl+,` | Settings (new) |
| | `Ctrl+B` | Batch mode (new) |
| | `Ctrl+Shift+C` | Subtitle list (new) |

(Rows marked "new" are the work; this table is the contract for steps 4-9 below.)

### Why Editor stays safe for the tagging flow

1. `/` then letters then `Ctrl+Enter` tags and returns to the video: two chords for a tag that is
   not starred. Starred tags (the common ones) are one digit.
2. The tag search keeps everything it has today: filter, `Enter` adds, arrows move.
3. If a letter that would have filtered is pressed with no field focused, a toast says
   "Press / to search tags" the first 3 times (stored in settings), then goes quiet.

### Remapping and the keymap file

- `keymap.toml` next to the settings: `[editor]` table of `action = ["Key", ...]` overrides. A
  Settings page "Keys" lists actions with their keys, a **Change** button that listens for the
  next press, conflict warnings naming the other action, Reset (one / all), Import / Export of
  the file.
- Matching: letters and digits by the **physical key** (`Code`), then by the layout's Latin
  letter (`key.to_latin`, as `latin_key` does now). This keeps `J K L` and `Ctrl+Z` working on a
  Russian layout, the way undo does today. Punctuation (`/`, `[`) is bound by physical code too,
  because on a Cyrillic layout the character differs.
- One lookup table replaces the `match` in `main_window_event`, `video_controls::shortcut` and
  `FileAction::from_key`; the "text field has the cursor" rule is a per-action flag
  (`works_while_typing`) instead of scattered `status` checks.

### Full keyboard control

- **Tab ring.** `Tab` / `Shift+Tab` move a ring through every control of the main window; `Space`
  or `Enter` presses it. Order: toolbar (previous, next, locate, open, recent, Settings, Batch,
  More), file search, file filter, file list, tag search, tag grid, file-name chips, sync lock and
  buttons, AI suggestions, comment box, video controls (play ... More), marker list. Reuses the
  `Control` enum and `Press` of `src/features/settings/focus.rs`. A click clears the ring (as
  Settings does).
- `Esc` leaves a field, then the ring, then the zone, then (as today) fullscreen / batch / search.
- **Batch**: in the Files zone `Space` checks the file, `Ctrl+A` checks all, `Ctrl+I` inverts,
  `Shift+↑/↓` extends the selection (replaces Ctrl/Shift+click), `Enter` on the action list steps it.
- **Chips**: with the ring on a chip, `Alt+←/→` reorders it (Editor frees this key because frames
  move to plain arrows; Classic keeps frame step), `Delete` untags it.
- **Menus** (file menu, More, recent): arrows, `Enter`, `Esc`; the context menu opens with the
  Menu key or `Shift+F10`.
- **Markers list**: zone Markers (`Ctrl+M`): `↑↓` rows, `Enter` opens the name, `Delete` deletes,
  `C` cycles the color, `Space` jumps there, `A` describes with AI (unnamed rows).
- Missing functions to be added with keys: go to in / out, clear in / out, end of clip, mute /
  volume (`Ctrl+↑/↓`), playback speed, move a marker, change a range's ends by keyboard
  (`Alt+Shift`+arrows act on the nearer end, `Shift` swaps).

### Conflicts

| New key | Today | Resolution |
|---|---|---|
| Letters, digits, `.` `,` `/` as commands | Type-to-filter | Editor only; Classic keeps filtering |
| `←` `→` frames | Move tag grid | Editor: Player zone steps frames; `Alt+T` enters the tag grid |
| `↑` `↓` files | Move tag grid / recent list | Same zone rule; recent list keeps arrows while open |
| `Space` | Play (not typing) / `Shift+Space` toggles tag | Unchanged |
| `M`, `Alt+M`, `Shift+M` | Type into the filter | Editor only |
| `Alt+←/→` reorders chips | Frame step in Classic | Editor only; Classic frame step stays |
| `Ctrl+B`, `Ctrl+O`, `Ctrl+,`, `Ctrl+/` | Unused | Classic too |
| `Ctrl+M` marker list, `Ctrl+Shift+C` subtitles | Unused (`Ctrl+C` copy tags is a different key) | Both |
| `F2` in file lists is "rename" in Explorer | Marker | Unchanged; keep (Premiere users have `M`) |
| `Ctrl+Alt+arrows` rotation | Graphics driver hotkeys on some PCs | Unchanged; remappable |
| `Tab` | Comment-focus check only | New ring; in the comment box `Tab` leaves it |
| `F6` (Windows pane cycle) | AI tags | Unchanged |

### Migration

1. Ship the keymap and presets with Classic as the active preset for everyone: no key changes.
2. A one-time note after the update: "New: Editor keys (J K L, I O, M). Settings > Keys." Dismiss
   remembers.
3. New installs (no settings database) start on Editor, with the hint described above.
4. Switching presets is instant and shown in the tooltip of every button (keys shown follow the
   active preset).
5. Classic is never removed.

## 4. Decisions made without the owner

- **Presets, not one new scheme.** Existing users keep their muscle memory; new ones get the
  editor layout. Reason: the owner's own flow is tag typing and the editors' flow is Premiere.
- **Two presets only (Classic, Editor).** Avid, FCP and Resolve layouts add maintenance for a
  small audience; the keymap file covers them.
- **Editor, not "Premiere".** It copies Premiere only where the key is free and sensible, so the
  name should not promise more than it keeps.
- **Typing is a mode entered by `/` and left by `Esc`**, not a held modifier. `/` is the filter
  key in many tools and is free in frename.
- **Hold `M` makes a range** (extends today's hold `F2`), because it is faster than Premiere's
  dialog or drag. `Shift+M` makes a range from In to Out because Resolve users ask for it and
  has no default key there.
- **Starred tags on digits.** The starred row is what a user already curates; the first ten are
  enough. Unstarred tags go through `/`.
- **Plain arrows are frames in Player zone**, with `Alt+T` to enter the tag grid; chosen over
  keeping arrows on the grid because frame stepping is the key editors use most while marking.
- **Keys bound by physical key**, so J/K/L work on a Russian layout.
- **Classic is the default for existing users and is never removed.**
- **No change to `F`-keys.** They stay bound in both presets as aliases; the docs (README, guide)
  teach the Editor keys first only for new installs.
- **The cheat sheet (#62) is generated from the keymap**, so it cannot drift; README tables are
  generated or checked against it.
- **`Tab` ring over zones for full keyboard control**; zones are for speed, the ring for reach.

## 5. Options not taken

- **Space as the "typing" modifier** (hold Space, type): collides with Space = play and with
  repeated-key handling of OS layouts.
- **Caps Lock or a visible toggle for typing mode**: invisible on some keyboards; `/` and `Esc`
  already give the same two states with a focused field as the indicator.
- **Letters filter when paused** (option D): the same key would do two things by hidden state.
- **A Premiere preset that matches Premiere completely** (`Shift+M` next marker, `Ctrl+Shift+M`
  previous, `Ctrl+Alt+M` clear): those keys are less comfortable than Resolve's; per research,
  several Premiere details could not be verified.
- **Keys for AI actions beyond what exists** (describe all, suggestions): a suggestion pill
  needs the Tab ring, not a new global key.
- **Photo Mechanic style code replacements** (type `\kj\`): digits plus search cover it; an idea
  for later.
- **Ratings and flags (`P` `X` `U`, stars)**: frename has none; a new feature, not part of this.
- **Chord sequences (vim-style `g g`)**: hard to discover and to remap.

## 6. Implementation steps

Each step is one issue (label `idea`), in this order; a step changes no keys unless it says so.

| # | Title | Scope | Size | Risk |
|---|---|---|---|---|
| 1 | Keymap: one table of actions and keys | Introduce `Action` and `KeyMap`; move `main_window_event`, `video_controls::shortcut`, `FileAction::from_key` and the Settings keys onto it with Classic as the only preset; physical-key matching; `works_while_typing` flag. A test proves Classic equals the table in section 1. No behaviour change. | L | Medium: touches every key path; guarded by the equality test |
| 2 | Shortcut cheat sheet from the keymap (#62) | `Ctrl+/` overlay listing actions and keys by group; README / guide tables checked against the keymap by a test. | M | Low |
| 3 | Key presets and Settings > Keys page | Preset choice (Classic, Editor), stored in settings; list of actions with keys; no Editor keys active yet except the ones that conflict with nothing (`Ctrl+O`, `Ctrl+,`, `Ctrl+B`, `Ctrl+Shift+C`, `Ctrl+M`). | M | Low |
| 4 | Editor preset: typing mode | In Editor letters are not sent to the tag search; `/` and `Ctrl+F` focus the searches; `Esc` leaves; hint toast; first-run default for new installs; update note for existing ones. | M | High: the central behaviour change; test every `Esc` path |
| 5 | Playback rate and J/K/L | Rate control in the video player (reverse and 2x/4x/8x via GStreamer seek with rate), `K`+`J`/`L` frame step, plain-arrow frame step in Editor, `Shift+←/→` 1 s, `End` (relates #162). | L | High: GStreamer reverse playback is codec-dependent; needs a fallback of repeated frame steps |
| 6 | Editor preset: points, markers and ranges | `I` `O`, go to / clear in and out, `M` (tap, double, hold), `Shift+M` In-to-Out range, `Alt+M`, `Shift+↑/↓`, move marker by a frame; new undo commands where needed. | M | Medium: new core commands, many existing marker rules (1.5 s window, guard while a row is open) |
| 7 | Starred tags on digits and apply-and-return | `1`..`9`,`0` toggle starred tag N; `Ctrl+Enter` in search toggles the first match, clears and returns to Player; Classic gets `Ctrl+Alt+1`..`9`. | S | Low |
| 8 | Tab focus ring over the main window | `Control` enum and order for the main window on the engine of `settings/focus.rs`; ring drawn on every control; `Space`/`Enter` press; click clears. | L | High: many widgets (iced has no focus tree), the comment box's own Tab |
| 9 | Zones and keys for the missing functions | Files zone (batch check, `Ctrl+A`, `Ctrl+I`, `Shift+↑/↓`), Tags zone (`Alt+T`), Markers zone (rows, color, jump, describe), chip reorder and untag, menus, volume, context menu key. | L | Medium |
| 10 | Remapping: `keymap.toml`, rebinding UI, import / export | Overrides stored as a diff over the preset; Change button, conflict warning, reset; physical keys; documentation. | M | Medium: layout edge cases |
| 11 | Docs: README and guide for presets | README keyboard tables per preset (from step 2), guide section "Choosing a key scheme", screenshots of the Keys page. | S | Low |
