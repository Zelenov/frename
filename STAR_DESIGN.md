# Starred Tags Design

This document describes the steps to implement a **star / unstar system** for tags in frename. It is design-only; no implementation is started until explicitly requested.

---

## Goals and constraints

- **Star indicator** displayed on each tag row in the tag panel (list view) and on each tag cell in the tag grid. Placement: next to the checkbox. Not displayed in the file-name panel (chips panel).
- **Starring** a tag moves it to the **very top** (position 0) of the display order, above all other tags including other already-starred tags. The starred flag is set to `true`.
- **Unstarring** a tag moves it to the position **immediately after the last remaining starred tag** in display order (= immediately before the first unstarred tag). If no starred tags remain, the newly unstarred tag moves to position 0.
- **`starred` is persisted** to the database as a column on the tags table. The reordering of `display_tag_ids` that results from starring/unstarring is also persisted via `update_tag_orders`.
- **Order invariant**: at all times the display order satisfies *all starred tags before all unstarred tags*. Star/unstar are the only operations that change a tag's section membership; they always restore the invariant as part of the same mutation.
- **Star is not in the file-name panel**: the chips panel renders only the checked-tag order in `selected_tag_ids`, which is unchanged by star/unstar.
- **Core-only logic**: star/unstar methods live in `TagList` inside `frename-core`. The UI dispatches a message; all reordering and persistence happen in core.

---

## New data

### `Tag` struct — new field

```rust
pub struct Tag {
    id: TagId,
    tag: String,
    checked: bool,
    color_index: u8,
    stored: bool,
    order: OrderKey,
    starred: bool,   // ← NEW: whether this tag is pinned to the top of the list
}
```

**Accessor** (to add to `Tag`):

```rust
pub fn is_starred(&self) -> bool { self.starred }
pub fn set_starred(&mut self, v: bool) { self.starred = v; }
```

### `StoredTag` DTO — new field

`StoredTag` (the DTO used by `StoredTagStore::save_tag`) gains:

```rust
pub struct StoredTag {
    // ... existing fields ...
    pub starred: bool,   // ← NEW
}
```

### Database migration

Add a new migration step to the tags table:

```sql
ALTER TABLE tags ADD COLUMN starred INTEGER NOT NULL DEFAULT 0;
```

All existing rows will have `starred = 0` (false) after migration. The migration must be added to `AppDatabase`'s inline migration list so it runs automatically on startup when the schema version is bumped.

---

## Order invariant (formal)

Let `display_order` be the sequence produced by iterating `display_tag_ids` in order. The invariant is:

> For every pair of indices `i < j` in `display_order`, it must NOT be the case that `tag[i].starred == false` and `tag[j].starred == true`.

In plain language: no unstarred tag appears before any starred tag in the display list. After every star/unstar operation this invariant must hold.

---

## New `TagList` methods

### `star_tag(id: TagId) -> Result<(), TagListError>`

```rust
pub fn star_tag(&mut self, id: TagId) -> Result<(), TagListError> {
    // 1. Set flag
    let tag = self.tags_by_id.get_mut(&id).ok_or(TagListError::NotFound)?;
    tag.set_starred(true);

    // 2. Move to front of display_tag_ids
    let first_id = self.display_tag_ids.iter().next()
        .map(|(fid, _, _)| *fid)
        .filter(|fid| *fid != id);          // avoid no-op if already first
    let rebalanced = self.display_tag_ids.insert_before(id, (), first_id.as_ref());
    if rebalanced { self.display_tags_rebalanced = true; }

    // 3. Persist starred flag + new order
    self.save_tag(id)?;     // save_tag already calls update_tag_orders when rebalanced
    if !self.display_tags_rebalanced {
        // insert_before may not rebalance but the order key did change → force persist
        self.persist_display_order()?;
    }
    Ok(())
}
```

### `unstar_tag(id: TagId) -> Result<(), TagListError>`

```rust
pub fn unstar_tag(&mut self, id: TagId) -> Result<(), TagListError> {
    // 1. Clear flag
    let tag = self.tags_by_id.get_mut(&id).ok_or(TagListError::NotFound)?;
    tag.set_starred(false);

    // 2. Find the item that comes right after the last remaining starred tag.
    //    "Remaining starred" = starred AND not the tag being unstarred.
    let anchor_next: Option<TagId> = {
        let mut last_starred_pos: Option<usize> = None;
        let display: Vec<TagId> = self.display_tag_ids
            .iter()
            .map(|(tid, _, _)| *tid)
            .collect();
        for (pos, tid) in display.iter().enumerate() {
            if *tid == id { continue; }
            if self.tags_by_id.get(tid).map_or(false, |t| t.is_starred()) {
                last_starred_pos = Some(pos);
            }
        }
        // anchor_next = the item right after the last starred item
        last_starred_pos.and_then(|p| display.get(p + 1).copied())
    };

    // 3. Move tag to immediately after last starred (insert before anchor_next)
    //    If anchor_next is None and no starred remain → move to front.
    //    If anchor_next is None because last starred is at end → insert at end (after it).
    let rebalanced = match anchor_next {
        Some(anchor) => self.display_tag_ids.insert_before(id, (), Some(&anchor)),
        None => {
            // Either no starred tags remain → move to front
            // Or last starred is already the last item → move to end (insert_before None = end)
            let has_remaining_starred = self.tags_by_id.values()
                .any(|t| t.id() != id && t.is_starred());
            if has_remaining_starred {
                // Last starred is last in list; insert after it = insert at end
                self.display_tag_ids.insert_before(id, (), None)
            } else {
                // No starred remain: move to top
                let first = self.display_tag_ids.iter().next()
                    .map(|(fid, _, _)| *fid)
                    .filter(|fid| *fid != id);
                self.display_tag_ids.insert_before(id, (), first.as_ref())
            }
        }
    };
    if rebalanced { self.display_tags_rebalanced = true; }

    // 4. Persist
    self.save_tag(id)?;
    if !self.display_tags_rebalanced {
        self.persist_display_order()?;
    }
    Ok(())
}
```

### `persist_display_order` (new private helper)

```rust
fn persist_display_order(&mut self) -> Result<(), TagListError> {
    let tag_orders: Vec<(Uuid, i64)> = self.display_tag_ids
        .iter()
        .filter_map(|(tid, _, ord)| {
            self.tags_by_id.get(tid)
                .filter(|t| t.is_stored())
                .map(|_| (tid.0, ord))
        })
        .collect();
    self.store.update_tag_orders(&tag_orders)?;
    self.display_tags_rebalanced = false;
    Ok(())
}
```

This mirrors the existing inline `update_tag_orders` call inside `save_tag`. Extract it here so both `star_tag` and `unstar_tag` can force a persist without triggering a full `save_tag`.

---

## Persistence flow

| Event | What is persisted |
|-------|-------------------|
| `star_tag(id)` | `save_tag(id)` → writes `starred = 1` (and tag name, color). Then `persist_display_order()` → writes new `sort_order` for all stored tags. |
| `unstar_tag(id)` | Same: `save_tag(id)` → `starred = 0`. Then `persist_display_order()`. |
| App startup | `get_stored_tags()` reads `starred` column; `TagList::new` restores the flag on each `Tag` and places it in `display_tag_ids` in DB sort order. Order invariant must already hold in the DB; no re-sort on load. |

---

## UI placement

### Tag panel (list view)

Each tag row already has a checkbox. Add a **star icon button** immediately to the left of (or right of) the checkbox. Clicking it dispatches `TagPanel::Message::ToggleStar(tag_id)`.

- Starred state renders a filled star (★); unstarred renders an outline star (☆) or a dim dot.
- The star button is always visible (not just on hover) so the user can discover it.

### Tag grid (grid view)

Each tag cell is a coloured chip. Add a **small star overlay** in the top-right corner of the cell. Clicking it dispatches the same `ToggleStar(tag_id)` message.

- If the cell is too small, the star can appear only on hover/focus.

### File-name panel (chips panel)

**No star rendered here.** The chips panel shows the checked tag order only. Starring is a display-order concern for the tag list/grid, not the file-name composition.

---

## Messages

Add to `tag_panel::Message`:

```rust
/// Toggle the starred state of the given tag.
ToggleStar(TagId),
```

Handled in `FolderWorkspace::handle_tag_panel`:

```rust
tag_panel::Message::ToggleStar(id) => {
    if let Some(fw) = self.file_workspace.as_mut() {
        let tag = fw.tag_list().get_tag(id);
        if tag.map_or(false, |t| t.is_starred()) {
            fw.unstar_tag(id);
        } else {
            fw.star_tag(id);
        }
    }
    Task::none()
}
```

`FileWorkspace` exposes `star_tag` and `unstar_tag` as thin delegates to `TagList`.

---

## Folder and file structure

```
crates/frename-core/src/
  tags/
    tag.rs               # Add: starred field, is_starred(), set_starred()
    tag_list.rs          # Add: star_tag(), unstar_tag(), persist_display_order()
  db/
    traits.rs            # StoredTag: add starred field; StoredTagStore unchanged (save_tag already upserts)
    app_database.rs      # Migration: ALTER TABLE tags ADD COLUMN starred INTEGER NOT NULL DEFAULT 0
                         # save_tag: include starred in upsert

src/features/
  tag_panel/
    view.rs              # Add star icon button per row
    messages.rs          # Add ToggleStar(TagId)
  tag_grid/
    view.rs              # Add star overlay per cell
  folder_workspace/
    state.rs             # Handle ToggleStar in handle_tag_panel
```

---

## Steps to implement

### Phase 1: Core data

1. **Add `starred: bool` to `Tag`** and add `is_starred()` / `set_starred()` accessors.
2. **Add `starred` to `StoredTag`** DTO.
3. **Add `starred` to DB migration** in `AppDatabase`. Bump schema version. On startup, the migration applies and all existing tags get `starred = 0`.
4. **Update `save_tag` in `AppDatabase`** to include `starred` in the upsert SQL.
5. **Update `get_stored_tags`** to read the `starred` column and populate `StoredTag::starred`.
6. **Update `TagList::new`** to copy `stored_tag.starred` onto the `Tag` when building `tags_by_id`.

### Phase 2: Core logic

7. **Extract `persist_display_order`** helper from the inline `update_tag_orders` call in `save_tag`.
8. **Implement `star_tag(id)`** on `TagList`: set flag, move to front of `display_tag_ids`, persist.
9. **Implement `unstar_tag(id)`** on `TagList`: clear flag, move to after last starred, persist.
10. **Expose on `FileWorkspace`**: add `star_tag` and `unstar_tag` delegates that call through to `TagList`.

### Phase 3: UI

11. **Tag panel row**: add star icon button; dispatch `ToggleStar(id)` on click. Render filled/outline based on `tag.is_starred()`.
12. **Tag grid cell**: add small star overlay; dispatch `ToggleStar(id)` on click.
13. **`handle_tag_panel` in `FolderWorkspace`**: handle `ToggleStar(id)`.

### Phase 4: Tests

14. **Unit tests in `frename-core`** (see Gherkin spec in `features/star.feature`):
    - Starring moves tag to position 0.
    - Starring multiple tags stacks them at top in reverse-star order (most recently starred is first).
    - Unstarring moves tag to after the last remaining starred tag.
    - Unstarring the last starred tag moves it to position 0.
    - Invariant: after any sequence of star/unstar operations, no unstarred tag precedes a starred tag.
    - `save_tag` is called with `starred = true` / `false` and `update_tag_orders` is called with updated positions.

---

## Order invariant test

The invariant can be asserted as a helper:

```rust
fn assert_starred_before_unstarred(tag_list: &TagList<FakeAppStorage>) {
    let mut seen_unstarred = false;
    for (id, _, _) in tag_list.display_tag_ids().iter() {
        let starred = tag_list.get_tag(*id).unwrap().is_starred();
        if !starred { seen_unstarred = true; }
        assert!(
            !(seen_unstarred && starred),
            "Starred tag {:?} appeared after an unstarred tag in display order",
            id
        );
    }
}
```

Call this after every `star_tag` / `unstar_tag` call in unit tests.

---

## Out of scope for this design

- Drag-to-reorder within the starred section or the unstarred section: star/unstar only controls section membership, not ordering within a section. Fine-grained ordering within sections is a future feature.
- Starring tags affecting `selected_tag_ids` (file-name panel order): no interaction.
- Visual grouping headers ("Starred" / "All") in tag panel or tag grid: UI enhancement, not required for v1.
- Keyboard shortcut to star/unstar the currently focused tag: can be added later without design changes.
