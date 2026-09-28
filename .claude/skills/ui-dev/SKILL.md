---
name: ui-dev
description: >
  UI development guide for frename. Use when: adding or changing a view, adding a new feature panel,
  creating or modifying a widget, changing layout constants, adding a new message to a feature,
  or working with styling, drag-drop, BoundsReporter, scrolling, or the search bar.
disable-model-invocation: false
---

# UI Development Guide — frename

## WHEN to use this skill
- Adding or changing a view (`view.rs`)
- Adding a new feature panel or modifying how a panel looks
- Creating or modifying a reusable widget
- Changing layout constants or sizing
- Adding a new message to a feature
- Working with drag-drop, BoundsReporter, scroll-into-view, or the search bar

---

## Feature module structure

Every feature lives in `src/features/<name>/` with these files:

| File | Role |
|---|---|
| `mod.rs` | Exports: re-exports `Message`, `State`, `view`, constants used by siblings |
| `state.rs` | Feature state: pure data + UI logic; no iced widget types except `Rectangle`, `Subscription`, `Task` |
| `messages.rs` | `pub enum Message` — all messages for this feature |
| `view.rs` | Pure `fn view(state, ...) -> Element<'_, Message>` — no state mutation |
| extra files | Sub-view files (`chips_panel.rs`, `file_name_line.rs`, `trash_zone.rs`) or sub-state |

## UI text

Every string a user sees (labels, tooltips, hints, error/status text) is a key in
`i18n/en/frename.ftl`, read with `fl!("key-id", arg = value)` (see `src/i18n.rs`), never a Rust
string literal. Keys: `<feature>-<element>[-<detail>]`, the feature being the folder under
`src/features/`. A plural gets a Fluent `{ $n -> [one] … *[other] … }` selector; reuse an existing
key when the exact same phrase already exists rather than adding a near-duplicate. A helper that
used to take `&'static str` (a button label, a tooltip) takes an owned `String` instead, since
`fl!` returns one. Exceptions (key names printed on a physical key, badges like `IN`/`OUT`/`CC`,
widget ids, log messages) are listed with their reason in `NOT_UI_TEXT` in `src/i18n.rs`; a test
there fails the build on any other English word left in a view, a widget, a batch action or
`folder_controls`. Every new language is its own small PR once the English key exists.

## Widgets

Reusable components not tied to any feature go in `src/widgets/`:
- `search_bar.rs` — search input with clear (×) and create (○) buttons
- `tag_chip.rs` — colored tag chip with optional leading/trailing slots
- `splitter.rs` — draggable panel splitter
- `file_name_display.rs` — file name rendered as tag chips (wrap=false)
- `bounds_reporter.rs` — invisible Fill widget that reports its layout bounds on every event

**When it belongs in widgets:** used by 2+ features, or purely presentational with no feature-specific messages.
**When it belongs in a feature:** only ever used inside that feature's view.

---

## Styling

The design system is the rule: `docs/design/design-system.md` (tokens, text styles, components,
windows, patterns, every screen). Its code is `src/ui/`:

```rust
use crate::ui::icon_button::IconButton;
use crate::ui::icons::Icon;
use crate::ui::layout::{self, NoticeKind};
use crate::ui::tokens::*;                 // colors, SPACE_*, sizes, regions, radii, fonts
use crate::ui::tooltip::{Position, Tip};
use crate::ui::{badge, button, empty, form, list, scroll, style, text};

text::body(fl!("…")); text::secondary(fl!("…")); text::mono(name).color(TEXT);
button::primary(fl!("…")).on_press(msg); button::ghost(fl!("…")); button::link(fl!("…"));
IconButton::new(Icon::Rewind)
    .tip(Tip::new(fl!("…")).keys(&["F1"]), Position::Top)
    .on_press(msg);
form::checkbox(label, on).on_toggle(Msg::Set);
layout::setting_row(label, layout::aligned([..]));
layout::notice(NoticeKind::Info, headline, None, [button::secondary(label).on_press(msg).into()]);
list::row_item(content, selected, HOVER);      // the one hover + selected look, with its bar
scroll::vertical_with_id(ID, content);         // keeps the scrollbar gutter
empty::pane(Icon::Clapperboard, fl!("…"), None, Some(button.into()));
```

- A view never writes a color, a size, a padding, a spacing or a radius as a number: `ui::lint`
  fails the tests if it does (`0` alone is allowed). Its allow-list `NOT_YET` is empty and stays
  empty: a new size or color is a token in `src/ui/tokens/` (by kind: color, content, space, size,
  region, typography), with a doc comment.
- Every icon is a Lucide SVG in `assets/icons/` with one line in the `icons!` list of
  `src/ui/icons.rs`; every icon-only button is an `IconButton` with a tooltip.
- Something the system lacks is added to `docs/design/design-system.md` first, then to `src/ui/`.
  A piece two views need is one component, never a copy.
- `ui` holds tokens, styles (`ui::style`) and stateless constructors of standard controls;
  `src/widgets/` keeps frename's own stateful or custom-drawn widgets (splitter, tag chip,
  progress bar), which take their values from `ui::tokens` and `ui::style`.
- Every window uses `ui::theme()`; Inter at 13 px is the default font.

Tag chip colors: `ui::palette::TagPalette::color(tag.color_index())` (16 colors, the index
wraps); marker colors: `ui::palette::marker_color`.

---

## Sizing conventions

- `Length::Fill` — take all available space in the axis
- `Length::Shrink` — natural / content size (default when not set)
- `Length::Fixed(px)` — exact pixel size
- Never hardcode raw pixel heights in view code — define a `const` in the module

Layout constants that are shared between view and state (e.g. for hit-testing) are declared in `mod.rs` as `pub const`.

---

## BoundsReporter

`BoundsReporter` is a `Fill×Fill` invisible widget that fires a message with its `Rectangle` on every layout change. Used for cursor-to-index mapping in drag-drop.

**Pattern — use it as a 0-height anchor inside a `stack!`:**

```rust
// In view.rs
let anchor = container(BoundsReporter::new(Message::PanelBounds))
    .width(Length::Fill)
    .height(Length::Fixed(0.0));   // 0 height: doesn't affect stack height

let chips_cell = container(
    stack![anchor, content_widget]
        .width(Length::Fill),
)
.width(Length::Fill);
```

The `Fixed(0.0)` prevents BoundsReporter's `Fill` height from becoming the stack's height.
State only needs `bounds.x`, `bounds.y`, and `bounds.width` for hit-testing; `bounds.height = 0` is fine.

For a full-height panel (e.g. the scrollable tag grid), `BoundsReporter` goes directly in a `stack!` without a fixed-height wrapper — the scrollable's `Fill` height drives layout.

---

## Scroll-into-view

The tag grid and tag list share one scrollable: `TAG_LIST_SCROLLABLE_ID`.

**After any `set_selected()` call, emit `Message::ScrollTagListToSelection`.**
`folder_workspace` handles the scroll operation using stored `scroll_y` and `viewport_height`.

```rust
// In folder_workspace::handle_tag_panel
self.tag_panel.set_selected(Some(id));
return Task::done(Message::ScrollTagListToSelection);
```

`TagPanelState` tracks `bounds`, `row_height`, `cols`, `row_content_height`, and `scroll_y` — all reported by the view via `Message::PanelBounds` and `Message::TagListScrolled`.

---

## Search bar (`widgets/search_bar`)

```rust
widgets::search_bar::view(
    filter,                             // &str — current value
    |s| Message::TagPanel(tag_panel::Message::SetFilter(s)),   // on_input
    || Message::TagPanel(tag_panel::Message::SetFilter(String::new())),  // on_clear
    on_create,  // Option<impl Fn(String)->Message> — None hides the ○ button
)
```

`on_create` is called at **render time** with the current value to produce the message.
Show it only when `filter.trim()` is non-empty AND no existing tag has that name.

---

## Drag-drop (file_name_panel only)

`file_name_panel::Message` owns the drag lifecycle:

| Message | When |
|---|---|
| `DragStarted { tag_id, initial_index }` | Mouse pressed on a chip |
| `DragHoverCursor { x, y }` | Cursor moved (fired by `FileNamePanelState::subscription()`) |
| `DragEnded` | Mouse released (fired by subscription) |

`FileNamePanelState` computes `drop_target_index` from cursor + bounds.
`folder_workspace::handle_file_name_panel` calls `file_workspace.reorder_tag_to_index(dragged_id, drop_index)` on `DragEnded`.

The trash zone uses `TrashBounds` + cursor position to detect drop-on-trash (uncheck the tag).

---

## Subscriptions

Feature state exposes `subscription() -> Subscription<Message>` only when it needs OS-level events (mouse move/release for drag). `folder_workspace::subscription()` composes them:

```rust
pub fn subscription(&self) -> Subscription<Message> {
    Subscription::batch([
        self.video_player.subscription().map(Message::VideoPlayer),
        self.file_name_panel.subscription().map(Message::FileNamePanel),
    ])
}
```

Only `video_player` and `file_name_panel` currently have subscriptions.

---

## iced 0.14 gotchas

**Widget state follows its position in the tree, not its `Id`.** A scrollable, text editor or
text input keeps its scroll offset, cursor and focus only while it stays at the same place among
its siblings. Adding or removing a sibling *before* it (a header shown only in batch mode) moves
it and resets that state — the folder list jumped to the top this way. Put optional parts inside
a container that is always there:

```rust
// Wrong: `body` moves from child 1 to child 2 when the header appears.
let mut content = column![search];
if let Some(batch) = batch { content = content.push(header(batch)); }
content.push(body)

// Right: the header lives inside the first child; `body` is always child 1.
let mut top = column![search];
if let Some(batch) = batch { top = top.push(header(batch)); }
column![top, body]
```

**`push_maybe` is gone.** Add an optional child with
`.extend(condition.then(|| widget.into()))`.

**Progress from a worker thread** (`spawn_blocking`) does not redraw by itself: share it through an
`Arc` the view reads (see `batch::ItemProgress`), and redraw with a subscription that exists only
while the work runs: `iced::time::every(Duration::from_millis(200)).map(|_| Message::Noop)`.

---

## Layout orchestration

All views flow through `folder_workspace::view`, which arranges:
- Left panel: video player + video controls
- Middle: folder file list
- Right: search bar + tag grid + file name panel (file_workspace::view)

Splitters (`src/widgets/splitter.rs`) sit between panels; their positions are tracked as `left_width: f32` and `folder_width: f32` in `FolderWorkspace`.
