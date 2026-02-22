# Starred Tags Design

This document describes the steps to implement a **star / unstar system** for tags in frename. It is design-only; no implementation is started until explicitly requested.

---

## Goals and constraints

- **Star indicator** displayed on each tag row in the tag panel (list view) and on each tag cell in the tag grid. Placement: next to the checkbox. Not displayed in the file-name panel (chips panel).
- **Starring** a tag pins it into the starred section of the display order (section 2, see below). The starred flag is set to `true`.
- **Unstarring** a tag moves it back to the unstarred stored section (section 3). The starred flag is set to `false`.
- **Only stored tags can be starred.** Attempting to star an unstored (snapshot-only) tag is rejected with an error. Unstored tags have no entry in the database, so there is nowhere to persist the starred flag.
- **`starred` is persisted** to the database as a column on the tags table. The reordering of both ordered collections that results from starring/unstarring is also persisted via `update_tag_orders`.
- **Both ordered collections are always kept in sync.** Every insert or move applied to `display_tag_ids` must be mirrored by the same logical operation on `selected_tag_ids`, using the same anchor `TagId`s (not copied order-key values — see "Mirror rule" below).
- **Star is not in the file-name panel**: the chips panel renders only the checked-tag order in `selected_tag_ids`. Starring does not change which tags are checked; it only changes display order.
- **Core-only logic**: star/unstar methods live in `TagList` inside `frename-core`. The UI dispatches a message; all reordering and persistence happen in core.

---

## Three-section display order

The display order (tag panel, tag grid, and the underlying `selected_tag_ids` order) is divided into **three sections** at all times:

```
┌─────────────────────────────────────────┐
│  Section 1 — Unstored (snapshot-only)   │  tags where stored == false
├─────────────────────────────────────────┤
│  Section 2 — Starred stored             │  tags where stored == true && starred == true
├─────────────────────────────────────────┤
│  Section 3 — Unstarred stored           │  tags where stored == true && starred == false
└─────────────────────────────────────────┘
```

**Rules:**
- Unstored tags are always in section 1, above all stored tags. They appear there because they come from the current file's snapshot and have not yet been saved to the database.
- Starred stored tags are always in section 2, between unstored tags and unstarred stored tags.
- Unstarred stored tags are always in section 3, at the bottom.
- Within each section the relative order of existing tags is preserved; star/unstar only moves the affected tag to the correct section boundary.

This ordering replaces the previous two-section model (starred / unstarred). The three sections now apply to **both** `display_tag_ids` and `selected_tag_ids`.

---

## New operation required on `OrderedCollection`: `insert_after`

The existing `insert_before(key, value, before_id: Option<&K>)` inserts a key _before_ a specified anchor (or at the end when `None`). Star/unstar needs the symmetrical operation: insert a key _after_ a specified anchor.

### Semantics

| Call | Result |
|------|--------|
| `insert_after(key, value, None)` | Inserts key at the **very beginning** (position 0). |
| `insert_after(key, value, Some(anchor))` | Inserts key immediately **after** anchor. If anchor is the last element, key becomes the new last element. |

This is the exact mirror of `insert_before`:

| `insert_before(key, value, None)` | Inserts key at the **very end**. |
| `insert_before(key, value, Some(anchor))` | Inserts key immediately **before** anchor. |

### Implementation approach (no code yet)

`insert_after(key, value, after_id)` is implemented by finding the element that immediately follows `after_id` in the current order, then delegating to the existing `insert_before` internal logic:

- **`insert_after(key, value, None)`** → find the current first element; delegate to `insert_before_impl(key, value, first)`. If the collection is empty, just insert normally.
- **`insert_after(key, value, Some(anchor))`** → find the element immediately after `anchor` in the ordered set (`next_after_anchor`); delegate to `insert_before_impl(key, value, next_after_anchor)`. If `anchor` is the last element, `next_after_anchor` is `None`, so delegate to `insert_before_impl(key, value, None)` (insert at end).

If the key already exists in the collection it must be removed first (same as `insert_before`). The return value is `bool` indicating whether rebalancing occurred (same as `insert_before`).

### Why `insert_after` and not reuse `insert_before`

The star/unstar algorithm needs to express "place this tag immediately after the last unstored tag" or "immediately after the last starred tag." These are natural `insert_after` expressions. Expressing them as `insert_before(next_sibling)` requires the caller to also find `next_sibling` and handle the edge case when the anchor is the last element. Encapsulating that in `insert_after` keeps the star/unstar logic clean and parallel for both collections.

---

## Mirror rule for both ordered collections

`TagList` holds two `OrderedCollection<TagId, ()>` instances:

- **`display_tag_ids`** — drives the tag panel list and tag grid display order.
- **`selected_tag_ids`** — drives the file-name panel chips order (only checked tags are visible, but the underlying order applies to all tags).

Both collections must maintain the same three-section structure. When a star/unstar operation moves a tag in one collection, the **identical logical operation** (same method, same anchor `TagId`) must be applied to the other collection independently.

**The order keys (i64 values) are private to each collection and must never be copied across collections.** Each `OrderedCollection` manages its own key space. The anchor is always identified by `TagId` (not by position or order key), and both collections perform the insert using that `TagId` as the anchor.

**Example**: to move tag `T` to immediately after the last unstored tag (anchor = `U`), the star algorithm calls:
1. `display_tag_ids.insert_after(T, (), Some(&U))`
2. `selected_tag_ids.insert_after(T, (), Some(&U))`

Both calls use the same anchor ID `U`. Each collection computes its own new order key independently.

---

## Formal invariant (three sections)

Let `display_order` be the sequence of `(TagId, Tag)` pairs produced by iterating `display_tag_ids` in order. The invariant is:

> There exist indices `0 ≤ s1 ≤ s2 ≤ len` such that:
> - Positions `[0, s1)` are all unstored tags (`!tag.stored`).
> - Positions `[s1, s2)` are all starred stored tags (`tag.stored && tag.starred`).
> - Positions `[s2, len)` are all unstarred stored tags (`tag.stored && !tag.starred`).

The same invariant applies to `selected_tag_ids`.

---

## Section boundaries: how to find the anchor for a move

Every star/unstar operation needs to find one or two section-boundary anchors before performing the move. The anchors are found by a linear scan of the current ordered sequence. The key helper concepts:

### `last_unstored_id() -> Option<TagId>`

The last tag in the ordered sequence whose `stored == false`. This is the "end of section 1" anchor.
- If `None`, section 1 is empty; the start of section 2 is position 0.

### `last_starred_id(excluding: TagId) -> Option<TagId>`

The last tag in the ordered sequence whose `stored == true && starred == true`, excluding the tag currently being operated on. This is the "end of section 2" anchor.
- If `None`, section 2 is empty (or will be after unstarring the last one); the start of section 3 immediately follows section 1.

---

## `star_tag` algorithm (no code)

**Pre-conditions:**
1. Tag must exist in `tags_by_id`. If not: error `NotFound`.
2. Tag must be stored (`tag.stored == true`). If not: error `CannotStarUnstoredTag`.
3. If already starred: return `Ok` immediately (idempotent, no movement, no persistence call).

**Steps:**
1. Set `tag.starred = true` on the tag in `tags_by_id`.
2. Find `anchor = last_unstored_id()`.
3. Call `display_tag_ids.insert_after(id, (), anchor.as_ref())` — this places the tag immediately after the last unstored tag (= at the start of section 2). If `anchor` is `None` (no unstored tags), `insert_after(id, (), None)` places it at position 0.
4. Call `selected_tag_ids.insert_after(id, (), anchor.as_ref())` — the same logical operation, using the same `TagId` anchor, independently on the second collection.
5. Track rebalancing flags on both collections if their respective `insert_after` returns `true`.
6. Persist: call `save_tag(id)` to write `starred = true` to the database, then call `persist_display_order()` to write the new `sort_order` values for both collections.

**Result:** tag `id` is now the first element of section 2 (immediately after section 1).

---

## `unstar_tag` algorithm (no code)

**Pre-conditions:**
1. Tag must exist in `tags_by_id`. If not: error `NotFound`.
2. Tag must be stored (`tag.stored == true`). If not: should never happen (unstored tags cannot be starred), but return error `CannotStarUnstoredTag` for safety.
3. If already unstarred: return `Ok` immediately (idempotent).

**Steps:**
1. Set `tag.starred = false` on the tag in `tags_by_id`.
2. Find `last_starred = last_starred_id(excluding: id)`.
3. If `last_starred` is `Some(anchor)`:
   - Call `display_tag_ids.insert_after(id, (), Some(&anchor))` — places tag immediately after the last remaining starred tag (= start of section 3).
   - Call `selected_tag_ids.insert_after(id, (), Some(&anchor))` — same anchor, same operation on second collection.
4. If `last_starred` is `None` (no other starred tags remain):
   - Section 2 is now empty. The start of section 3 is immediately after section 1.
   - Find `anchor = last_unstored_id()`.
   - Call `display_tag_ids.insert_after(id, (), anchor.as_ref())` — places tag after the last unstored tag (= start of section 3 which coincides with start of section 2 now that both are empty).
   - Call `selected_tag_ids.insert_after(id, (), anchor.as_ref())` — same.
5. Track rebalancing flags on both collections.
6. Persist: call `save_tag(id)` to write `starred = false`, then `persist_display_order()`.

**Result:** tag `id` is now the first element of section 3 (immediately after the last starred tag, or immediately after the last unstored tag if no starred tags remain, or at position 0 if both sections 1 and 2 are empty).

---

## `persist_display_order` helper

A private helper on `TagList` that writes the current `sort_order` for all stored tags to the database via `store.update_tag_orders(...)`. It collects `(TagId.0, order_key)` pairs for every tag in `display_tag_ids` where `tag.is_stored()`, then calls `store.update_tag_orders`.

This is called unconditionally after every star/unstar operation to ensure the new order is always flushed, whether or not rebalancing occurred. The order key of the moved tag changes even without a full rebalance.

---

## New data

### `Tag` struct — new field

Add `starred: bool` to `Tag`. New accessors: `is_starred() -> bool` and `set_starred(bool)`. Default: `false`.

Unstored tags always have `starred = false` (enforced at construction and by the constraint in `star_tag`).

### `StoredTag` DTO — new field

`StoredTag` gains `starred: bool`. Populated from the `starred` column when reading and written when upserting.

### Database migration

```sql
ALTER TABLE tags ADD COLUMN starred INTEGER NOT NULL DEFAULT 0;
```

All existing rows get `starred = 0` after migration. Schema version bumped.

### `StoredTagStore` — `save_tag` updated

`save_tag` upsert SQL must include the `starred` column. No new trait method is needed.

---

## Startup / load order

On `TagList::new`, stored tags are inserted into both `display_tag_ids` and `selected_tag_ids` using their `sort_order` values from the database. Since star/unstar always persists the new order, the database already reflects the three-section ordering. No re-sorting is needed at startup.

Unstored tags (from the current file snapshot) are inserted at the **front** of both collections during `TagList::new`. This is the existing behaviour; no change needed.

---

## New error variant

Add to `TagListError` (or equivalent error type):

```
CannotStarUnstoredTag   // tag.stored == false; starring is not allowed
```

---

## UI placement

### Tag panel (list view)

Each tag row gets a **star icon button** next to the checkbox. Starred tags render a filled star (★); unstarred render an outline star (☆). Clicking dispatches `TagPanel::Message::ToggleStar(tag_id)`.

The star button is visible only for stored tags. Unstored (snapshot-only) tags show no star button because they cannot be starred.

### Tag grid (grid view)

Each stored tag cell gets a **small star overlay** (top-right corner of the cell). Same rendering rules and same message dispatched. No star overlay on unstored tag cells.

### File-name panel (chips panel)

**No star rendered here.** The chips panel shows only checked tags in `selected_tag_ids` order, which is maintained by star/unstar but carries no star visual.

---

## Messages

Add to `tag_panel::Message`:

```
ToggleStar(TagId)   // toggle the starred state of the given stored tag
```

Handler in `FolderWorkspace::handle_tag_panel`: check current `is_starred()` state, call `file_workspace.star_tag(id)` or `file_workspace.unstar_tag(id)`. Log and surface any error (e.g. `CannotStarUnstoredTag` should not reach the UI but if it does, show a toast).

---

## Folder and file structure

```
crates/frename-core/src/
  ordered/
    collection.rs        # Add: insert_after(key, value, after_id: Option<&K>) -> bool
  tags/
    tag.rs               # Add: starred field, is_starred(), set_starred()
    tag_list.rs          # Add: star_tag(), unstar_tag(), persist_display_order(),
                         #      last_unstored_id(), last_starred_id(excluding)
  db/
    traits.rs            # StoredTag: add starred: bool
    app_database.rs      # Migration: starred column; save_tag includes starred

src/features/
  tag_panel/
    view.rs              # Star button per row (stored tags only)
    messages.rs          # Add ToggleStar(TagId)
  tag_grid/
    view.rs              # Star overlay per cell (stored tags only)
  folder_workspace/
    state.rs             # Handle ToggleStar in handle_tag_panel
```

---

## Steps to implement

### Phase 1: OrderedCollection

1. **Add `insert_after`** to `OrderedCollection` with the semantics above. Add unit tests in `ordered/collection.rs` or a dedicated test file.

### Phase 2: Core data

2. Add `starred: bool` to `Tag` with accessors.
3. Add `starred` to `StoredTag` DTO.
4. Add DB migration for `starred` column.
5. Update `save_tag` SQL to include `starred`.
6. Update `get_stored_tags` to read `starred`.
7. Update `TagList::new` to copy `stored_tag.starred` onto the constructed `Tag`.

### Phase 3: Core logic

8. Add private helpers `last_unstored_id` and `last_starred_id(excluding)` to `TagList`.
9. Extract `persist_display_order` helper from existing inline `update_tag_orders` call.
10. Implement `star_tag(id)` following the algorithm above: validate, set flag, move in both collections, persist.
11. Implement `unstar_tag(id)` following the algorithm above: validate, set flag, move in both collections, persist.
12. Add `CannotStarUnstoredTag` to the error type.
13. Expose `star_tag` and `unstar_tag` as delegates on `FileWorkspace`.

### Phase 4: UI

14. Tag panel row: star button for stored tags; dispatch `ToggleStar`.
15. Tag grid cell: star overlay for stored tags; dispatch `ToggleStar`.
16. `FolderWorkspace::handle_tag_panel`: handle `ToggleStar`.

### Phase 5: Tests

17. Unit tests in `frename-core` covering all cases in `features/star.feature`.

---

## Order invariant test helper

```rust
fn assert_three_section_invariant(tag_list: &TagList<FakeAppStorage>) {
    // Iterate display_tag_ids in order.
    // Allowed transitions: unstored → starred → unstarred.
    // Any other transition is a violation.
    let mut section = 0u8;   // 0=unstored, 1=starred, 2=unstarred
    for (id, _, _) in tag_list.display_tag_ids().iter() {
        let tag = tag_list.get_tag(*id).unwrap();
        let s = match (tag.is_stored(), tag.is_starred()) {
            (false, _)    => 0,
            (true, true)  => 1,
            (true, false) => 2,
        };
        assert!(s >= section,
            "Section violation: tag {:?} (section {}) appeared after section {}",
            id, s, section);
        section = s;
    }
    // Same check for selected_tag_ids
    // (identical logic, different iterator)
}
```

Call `assert_three_section_invariant` after every `star_tag` / `unstar_tag` call in all unit tests.

---

## Out of scope for this design

- Drag-to-reorder within a section: star/unstar only controls section membership, not order within a section.
- Visual grouping headers ("Starred", "Saved", etc.) in the tag panel: UI enhancement, not required for v1.
- Keyboard shortcut to star/unstar the currently focused tag: addable later.
- Starring an unstored tag by first auto-saving it: out of scope; the user must save the tag first.
