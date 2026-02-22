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

Always use constants and functions from `src/theme.rs`:

```rust
// Colors
theme::BG_MAIN          // dark background
theme::BG_PANEL         // panel background
theme::BG_ELEVATED      // elevated card background
theme::TRACK            // unchecked/inactive fill
theme::ACCENT           // selected, progress, focus
theme::TEXT             // primary text
theme::TEXT_MUTED       // secondary / placeholder text
theme::SPLITTER         // splitter line (inactive)
theme::SPLITTER_ACTIVE  // splitter line (hovered/dragged)

// Style functions (pass to .style())
theme::panel_container_style        // standard panel container
theme::elevated_container_style     // raised card (file name panel)
theme::dark_scrollable_style        // dark-themed scrollable
theme::tag_row_background_style(theme, is_selected)  // tag row highlight
```

Tag chip colors: always use `tag_colors::TagColors::color(tag.color_index())`.
Palette has 16 entries; index wraps modulo 16.

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

## Layout orchestration

All views flow through `folder_workspace::view`, which arranges:
- Left panel: video player + video controls
- Middle: folder file list
- Right: search bar + tag grid + file name panel (file_workspace::view)

Splitters (`src/widgets/splitter.rs`) sit between panels; their positions are tracked as `left_width: f32` and `folder_width: f32` in `FolderWorkspace`.
