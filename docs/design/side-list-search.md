# Search in the side lists

Issue #170. `Ctrl+F` over the video opens a bar in the side list that narrows it to the rows that
match, and steps the playhead from match to match. The look is in `design-system.md` §13.3.6.

## How it works

- **Matcher.** `frename_core::search` is the file search's matcher, now public: `words`,
  `matches_all`, `search_hits`. `matching_cues` and `matching_markers` use it, so the side lists and
  the file list agree on every query (every word, any case). A word typed as a time (`1:03`,
  `1:02:03`) also matches the marker that starts in that second.
- **State.** `SideSearch` (open, text, words) lives in `FolderWorkspace`, which owns the markers;
  the subtitles come from the player. The bar sends `video::Message::SideSearch`, which the
  workspace intercepts like `Markers`. Everything is computed at view time from the text.
- **Rows keep their index** in the whole list when others are hidden, so row heights and scroll
  offsets still work (`filtered_row_offset`, `filtered_cue_offset`).
- **Stepping** goes by the playhead, not by a remembered row: the next match that starts after it,
  the previous one that starts before it, wrapping round the ends. It seeks exactly and scrolls the
  list to the row. While the text has words, the playhead does not scroll the list.
- **Esc** clears the text, then closes the bar, after an open marker row has been closed.
- **Another clip** empties the text; whether the bar is open is kept for the session.

## Decisions made without the owner

- No `е`/`ё` folding: the file search has none, and one rule everywhere is worth more.
- Closing the bar also empties the text, so a hidden bar never hides rows.
- `↓` / `↑` as keys are not bound: in the field they belong to the tag grid like every arrow key
  (the app routes them there); the two buttons after the field do the same and `Enter` /
  `Shift+Enter` work from the keyboard.
- The bar is the file search's own style, not a tinted variant: over the 72 % black list it reads
  well (see the PR's screenshots).
- The tab count is shown as "matching of all" only while the text has words and the list is not
  empty.
- The search button is in the header also on a wide pane (the issue's "narrow pane: same layout").
