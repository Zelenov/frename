# Keyboard approach in editing and cataloguing apps (issue #168)

Research notes, October 2026. Question: how do the apps an editor already knows drive video,
markers, ranges and tags/keywords from the keyboard, so frename can pick a scheme (the proposal is
`docs/design/keyboard.md`).

**Verification, read this first.** The official help sites (helpx.adobe.com, support.apple.com,
mpv.io, Blackmagic, Avid, Kyno, Camera Bits) could not be opened from the research session: page
fetches failed (DNS and a 403 from the egress proxy). Every fact below therefore comes from the
**search results' excerpts** of the cited page, not from reading the page. Where a result is
Adobe's, Apple's or the project's own page the excerpt is first-hand for that page; where it is a
third-party cheat sheet it says so. Tags used below:

- **[official]** the excerpt came from the vendor's or project's own page.
- **[3rd]** a third-party cheat sheet or tutorial; sources often disagree, see the notes.
- **[unverified]** no source found, or sources conflict; do not build on it without checking the
  installed app (each app lists its bindings in its own shortcut dialog).

## 1. Summary

| Idea | Who does it | Source |
|---|---|---|
| `J` back, `K` stop, `L` forward; repeats speed up | Premiere, Resolve, Avid, Kyno | sections 2-6 |
| `I` / `O` set in / out | Premiere, Resolve, FCP, Kyno, Avid (with `E`/`R` as an alternative) | sections 2-6 |
| `M` adds a marker at the playhead | Premiere, Resolve, FCP, Kyno | sections 2-5 |
| A range marker is made with a second step (a dialog, a drag, or a convert command), not with one key | Premiere, Resolve | 2, 3 |
| Marker navigation needs a modifier (`Shift+M`, `Shift+Up/Down`, `Control-'` / `Control-;`) | Premiere, Resolve, FCP | 2, 3, 4 |
| Tagging with a plain key: ratings `1`-`5`, flags `P` `X` `U`, labels `6`-`9` | Lightroom | 7 |
| Tagging with `Control-1`..`9` keyword slots | Final Cut Pro | 4 |
| Tagging with a modifier because plain digits are taken | Bridge (`Ctrl+1`..`5`) | 7 |
| Typing is a mode you enter on purpose (`Command-K`, `Cmd+Shift+T`, a field you click) | FCP, Kyno, Lightroom, Photo Mechanic | 4, 5, 7, 8 |
| Shortcut presets of other apps and a user remap | Premiere (Avid, FCP 7 layouts), Resolve (import), Bridge, mpv | 2, 3, 6, 7, 9 |

None of these apps filters a list by typing a letter anywhere in the window. Their letter keys
always mean transport or marking. Text goes into a field that was entered on purpose. That is the
opposite of frename today (`docs/design/keyboard.md`, section 1).

## 2. Adobe Premiere Pro (the target of frename's output)

- `J` `K` `L`: "J moves the cursor backward, whereas L moves the cursor forward. K can be used to
  pause" [3rd]: <https://www.premiumbeat.com/blog/beginners-guide-to-premiere-pro-shortcuts/>.
- `I` / `O` mark In / Out [3rd]: same page. `Shift+I` is Go to In, and the Markers menu lists Mark
  In, Mark Out, Go to In/Out, Clear In/Out, Clear In and Out, Add Marker, Go to Next Marker with
  `Shift+I` and `M` / `Shift+M` on Windows and Mac [official excerpt]:
  <https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/default-keyboard-shortcuts.html>.
- Previous marker `Ctrl+Shift+M` (Win) / `Shift+Cmd+M` (Mac) [3rd]:
  <https://www.storyblocks.com/resources/blog/keyboard-shortcuts-adobe-premiere-pro>. Clear
  selected marker `Ctrl+Alt+M` / `Opt+M` [3rd]: <https://www.evercast.us/blog/premiere-pro-shortcuts>.
  Clear all markers `Ctrl+Alt+Shift+M` / `Opt+Cmd+M` [3rd]: Storyblocks page above.
- Range (duration) marker: press `M`, press `M` again to open the marker dialog and type a
  Duration; or `Alt`+drag (Option on Mac) a marker to the right. "No dedicated hotkey to set a
  marker's duration" [3rd]: <https://www.provideocoalition.com/tool-tip-tuesday-for-adobe-premiere-pro-markers/>,
  <https://community.adobe.com/t5/premiere-pro-discussions/marker-duration-hotkey/td-p/9161553>.
- A source says `M` on an existing marker opens its dialog, another says `Shift+M`:
  **[unverified]** which (<https://blog.fmctraining.com/blog/adobe-premiere-pro-shortcut-cheatsheet>
  and the Brendan Cescon page <https://www.brendancescon.com/blog/2025/9/24/go-to-next-marker-premiere-pro-keyboard-shortcut-series>
  were returned for this; neither excerpt settled it).
- Frame stepping with `Left`/`Right` and 5 frames with `Shift`+arrow: **[unverified]** (not
  returned by any search).
- Remapping: Keyboard Shortcuts dialog (`Ctrl+Alt+K` / `Cmd+Option+K`) with a Keyboard Layout
  Preset menu offering Premiere Pro, Avid Media Composer 5 and Final Cut Pro 7; custom layouts are
  saved as `.kys` files [3rd]: <https://www.premiumbeat.com/blog/premiere-pro-keyboard-shortcuts/>,
  <https://www.filmeditingpro.com/how-to-install-keyboard-shortcuts-premiere-resolve-avid-final-cut/>.
  A 2017 forum post says the FCP 7 preset lacks many shortcuts [3rd]:
  <https://creativecow.net/forums/thread/looking-for-complete-fcp7-keyboard-layout-for-prem/>.
- Adobe's page for the older CS6 default set exists beside the current one; a source warns to
  check the version in use [3rd]: <https://helpx.adobe.com/au/premiere-pro/using/default-keyboard-shortcuts.chromeless.html>.

**What it means for frename.** frename writes the markers, ranges and in/out points Premiere then
reads, so an editor's hands already know `J K L I O M`. The range is a second gesture in
Premiere; frename's "hold the marker key" is faster than anything here and is worth keeping.

## 3. DaVinci Resolve

- `J` back, `K` stop (one source says pause), `L` forward; repeated presses speed up; `Left`/`Right`
  step one frame; holding `K` while tapping `J` or `L` moves one frame at a time [3rd]:
  <https://www.editorskeys.com/blogs/news/davinci-resolve-keyboard-shortcuts-pdf-cheat-sheet>,
  <https://www.benclaremont.com/blog/davinci-resolve-keyboard-shortcuts>.
- `I` / `O` set In / Out; `M` add marker; `Shift+M` modify marker; `Option+M` (`Alt+M`) delete
  marker [3rd]; `Shift+Up` / `Shift+Down` previous / next marker [3rd]:
  <https://academyclass.com/blog/davinci-resolve-keyboard-shortcuts-cheat-sheet/>,
  <https://micheleong.com/rail-yard/keyboard-shortcuts>. One list shows `Ctrl/Cmd+M` as "add and
  edit marker" - **[unverified]**, sources disagree.
- Range markers: Resolve 17 added converting an In-Out range to a duration marker; the menu entry
  has **no default shortcut**, users assign one [3rd]:
  <https://forum.blackmagicdesign.com/viewtopic.php?f=21&t=126418>.
- Remapping: Keyboard Customization window (`Cmd+Option+K` on Mac); presets for Premiere, FCP 7
  and Avid are importable - one source only, **[unverified]** against Blackmagic's manual:
  <https://pixflow.net/blog/davinci-resolve-keyboard-shortcuts/>.
- The only Blackmagic document found is for Resolve 11, too old to use:
  <https://documents.blackmagicdesign.com/SupportNotes/DaVinci_Resolve_11_Mac_Keyboard_Shortcuts.pdf>.

## 4. Final Cut Pro

- `I` / `O` set range start / end, `X` sets the range to the whole clip, `Option-I` / `Option-O`
  clear the range start / end, `Option-X` clears the range [official excerpt for the Option keys;
  `I` `O` `X` from 3rd]: <https://support.apple.com/guide/final-cut-pro/keyboard-shortcuts-ver90ba5929/mac>,
  <https://motionarray.com/learn/final-cut-pro/final-cut-pro-keyboard-shortcuts/>.
- `M` adds a marker, `Option-M` adds a marker and edits its text [official excerpt]: Apple page above.
- Next / previous marker `Control-'` / `Control-;` [3rd]: <https://blog.frame.io/2018/09/17/fcpx-final-cut-pro-shortcuts/>.
  One guide gives `Shift-M` for next marker, conflicting: <https://vagon.io/blog/mastering-final-cut-pro-keyboard-shortcuts-a-guide-to-efficient-video-editing>.
- Keywords: `Control-1`..`Control-9` apply keyword slots 1-9 to the selection [official excerpt];
  `Control-0` removes all keywords [3rd; Motion Array says `Control-O`, conflicting]; you name the
  slots in the Command Editor (`Option-Command-K`) [3rd]:
  <https://www.linkedin.com/learning/final-cut-pro-x-guru-keyboard-shortcuts/adding-keyword-shortcuts>.
  The keyword editor is `Command-K` in older guides **[unverified]** for the current version.
- `F` favorites the browser selection, `Delete` rejects; `Control-F` shows favorites [official
  excerpt for `F`]: Apple page above.
- Range markers: **[unverified]** (nothing returned).

**What it means.** FCP has the closest model to frename's tags: keyword slots on number keys, with
the user choosing what each slot means.

## 5. Avid Media Composer

- `J` `K` `L` shuttle; repeated `L` / `J` raise forward / reverse speed [3rd]:
  <https://www.evercast.us/blog/avid-shortcuts>.
- Mark In / Out are `E` / `R` in classic Avid; one 2026 guide lists `I` / `O` with `E` / `R` as the
  alternative [3rd, sources differ]: <https://www.editorskeys.com/blogs/news/avid-media-composer-keyboard-shortcuts-pdf-cheat-sheet>,
  <https://www.killerkeys.com/avid-media-composer-keyboard-shortcuts>.
- Locators (Avid's markers): `F3` adds a red locator; other F-keys other colors [3rd, one source
  says `F5`-`F12` add to the active clip]: Avid's own list <https://resources.avid.com/supportfiles/attach/keyboard%20shortcuts.pdf>
  and <https://community.avid.com/cfs-filesystemfile.ashx/__key/CommunityServer.Components.PostAttachments/00.00.35.64.80/CheatSheetv1.pdf>
  were returned but their text was not read. Go to next locator: **[unverified]**.
- The pattern that matters: Avid editors set up marking on F-keys, as frename does with `F2`.

## 6. mpv (and players)

- mpv's default bindings live in `etc/input.conf`; the user file `~/.config/mpv/input.conf` maps
  `key command` lines; `--no-input-default-bindings` turns the defaults off; `--input-test` shows
  what a key is bound to [official]: <https://mpv.io/manual/master/>,
  <https://raw.githubusercontent.com/mpv-player/mpv/master/etc/input.conf>.
- `,` frame back, `.` frame forward, `[` `]` speed -/+0.1, `{` `}` half / double speed, `Backspace`
  reset speed, `l` set A-B loop points, `L` toggle file loop [3rd, a gist of the defaults]:
  <https://gist.github.com/hank-morelock/e3eb001115b289ed8ca641064740a541>,
  <https://defkey.com/mpv-media-player-shortcuts>.
- `PGUP` / `PGDWN` previous / next chapter [official excerpt]: mpv manual above. mpv has no `J K L`
  by default **[unverified]**: not found in any excerpt.
- Arrow-key seek amounts and `Space` / `m` / `f`: **[unverified]** (not returned).
- mpv's `[` `]` collide with frename's in/out keys; its `.` `,` are free in frename today only
  because letters filter tags.

## 7. Adobe Bridge and Lightroom Classic (rating, labels, keywords)

- Bridge: `Ctrl+1`..`Ctrl+5` rate 1-5 stars, `Ctrl+0` clears [3rd]; `Ctrl+Alt+1`..`5` filter to
  items at or above a rating and `Ctrl+Alt+6`..`9` filter by label; `Ctrl+Alt+A` clears filters
  [official excerpt]: <https://helpx.adobe.com/in/bridge/using/adobe-bridge-keyboard-shortcuts.html>,
  <https://shortcutworld.com/Adobe-Bridge/win/Adobe-Bridge_CC_Shortcuts>.
- Bridge keywords are applied in the Keywords panel by click; `Shift`+click also applies the parent
  keywords, `Alt`+click removes [official excerpt]: Adobe page above. No key for keywords found.
- Bridge removed the preference that let plain number keys rate; users reassign them under
  Edit > Keyboard Shortcuts [3rd]: <https://www.damiensymonds.net/bridge-ratings.html>,
  <https://asktimgrey.com/2025/07/02/changing-bridge-keyboard-shortcuts/>.
- Lightroom Classic: `1`-`5` rating, `0` none, `[` `]` raise/lower by a star; `P` pick, `X` reject,
  `U` unflag (with `Shift` also advances to the next photo); `6`-`9` red/yellow/green/blue label
  (purple has no key); `\` shows the Filter bar, `Ctrl+L` toggles filters [official excerpt]:
  <https://helpx.adobe.com/lightroom-classic/help/keyboard-shortcuts.html>,
  <https://jkost.com/blog/2024/06/applying-flags-stars-and-color-labels-in-lightroom-classic.html>.
  `Ctrl+K` for keywords is in two cheat sheets only **[unverified]**:
  <https://shortcuts.kstanchev.com/apps/lightroom/>.

**What it means.** Both apps keep plain letters and digits for applying values and use a filter
bar you open with a key (`\`) or a modifier. Typing text happens in fields, never on the bare
window.

## 8. Photo Mechanic and Kyno

- Photo Mechanic: single key `t` tags (selects) an image; star ratings and color classes are on
  digit keys, and a preference decides whether a bare `0`-`8` sets a color class or a star rating;
  Windows and Mac lists differ [official + 3rd]:
  <https://camerabits.freshdesk.com/support/solutions/articles/48000317837-keyboard-shortcuts-windows>,
  <https://defkey.com/photo-mechanic-windows-shortcuts>, <https://shuttermuse.com/photo-mechanic-keyboard-shortcuts/>.
- Photo Mechanic **Code Replacements**: a tab-separated text file of short codes and their text;
  typed between delimiters (default backslash) in a caption field, `\kj\` becomes the full name
  [official]: <https://sidelinecaptions.com/what-are-code-replacements-photo-mechanic> (third-party
  explainer of an official feature), <http://wiki.camerabits.com/en/index.php/Speeding_Up_Captioning>.
  This is a typing-based way to apply many tags quickly without opening a list.
- Photo Mechanic's Ctrl+T / keyword quick-apply bindings: **[unverified]**; Windows and Mac lists
  conflict.
- Kyno (the closest to frename: it logs footage before Premiere): `I` / `O` set in / out, `M` add or
  edit a marker, `S` add or edit a subclip, `Alt+S` / `Alt+M` remove, number keys rate (`1` = one
  star), `Cmd+Shift+T` edits tags [official]: <https://support.lesspain.software/support/solutions/articles/12000010141-keyboard-shortcuts>.
  The page is an old snapshot and its J/K/L text is garbled in the excerpt: transport keys
  **[unverified]**. Kyno users asked for "a permanent tag bar with big tag buttons"
  <https://support.lesspain.software/support/discussions/topics/12000004850>, so tags there are
  applied by buttons and a tag dialog, not by typing in the main window. Its shortcuts stopped
  working after a macOS update for some users:
  <https://support.lesspain.software/support/discussions/topics/12000030058>.

## 9. Patterns, and what they cost frename

| Pattern | Examples | Fit for frename |
|---|---|---|
| Letters are commands; text goes into a field entered on purpose (a key such as `/`, `Cmd+K`, or a click) | Premiere, Resolve, FCP, Kyno, Lightroom | Best fit for review. Breaks "just type to filter" for everyone used to it. |
| Type anywhere to filter | frename today | The reason tagging is fast today; leaves no key free for `J K L I O M`. |
| Digits apply slots the user filled | FCP keywords, Lightroom stars/labels, Photo Mechanic | Fits starred tags (frename already has a starred row). |
| Modifier for the value-applying key | Bridge `Ctrl+1`, FCP `Control-1` | Needed when digits type text (Classic preset). |
| Hold or double-press for a second meaning | Premiere `M M` opens the dialog | frename already uses `F2 F2` and hold `F2`. |
| Presets of other apps plus a user keymap file | Premiere `.kys`, Resolve import, mpv `input.conf` | Presets for the two schemes frename needs; a text keymap is cheap. |
| One text-expansion step to apply many tags | Photo Mechanic code replacements | Not needed if digits and search work; kept as an idea. |

## 10. Gaps (what could not be verified)

- Every Adobe, Apple, Blackmagic, Avid, Kyno, Camera Bits and mpv page was reached only through
  search excerpts. Re-check the rows tagged **[3rd]** and **[unverified]** against the installed
  apps before copying a key into a preset.
- Premiere: frame step and `Shift`+arrow, which key opens an existing marker's dialog, the key for
  previous/next edit.
- Resolve: the marker dialog key (`Shift+M` vs `Ctrl+M`), the shipped Premiere/Avid/FCP presets.
- FCP: range markers, the keyword editor key in the current version, `Control-0`.
- Avid: Mark In/Out defaults (`E`/`R` vs `I`/`O`), next locator.
- mpv: arrow-key seek amounts, `Space`, `m`, `f`, J/K/L.
- Photo Mechanic: tag/keyword quick-apply bindings. Kyno: transport keys.
