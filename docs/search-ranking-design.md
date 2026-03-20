# Search Ranking Design — Tag Grid

## Overview

Replace the current flat "contains" filter in `TagList` with a three-tier ranked sort. Visibility is unchanged (same tags shown); only the order within each existing display section changes.

---

## Current behaviour (what exists today)

**File:** `crates/frename-core/src/tags/tag_list.rs`

`tag_matches_filter` (line 57):
```rust
fn tag_matches_filter(&self, t: &Tag) -> bool {
    let q = self.filter_query.trim().to_lowercase();
    q.is_empty() || t.tag().to_lowercase().contains(&q)
}
```

`rebuild_filtered_display_tag_ids` (line 62) collects matching tags then stable-sorts by a `u8` section key:
```
0 = unstored (snapshot-only, shown with ○ button)
1 = starred stored
2 = unstarred stored
```
Within each section the original display order is preserved (stable sort).

---

## New behaviour

Within each section the stable sort key gains a **rank** sub-key based on how the query matches the tag name. The section boundaries are not changed.

### Rank definition (case-insensitive, applied to trimmed query)

| Rank | Condition | Example query `Hel` |
|------|-----------|---------------------|
| `0` | `tag_lower == query_lower` (full match) | `hel` == `hel` ✓ |
| `1` | `tag_lower.starts_with(query_lower)` | `hello`.starts_with(`hel`) ✓ |
| `2` | `tag_lower.contains(query_lower)` | `ahello`.contains(`hel`) ✓ |
| `None` | no match → tag hidden | — |

Empty query → rank `None` is never reached (all tags pass); display order unchanged (existing behaviour, no sort applied).

### New sort key: `(section: u8, rank: u8)`

Stable sort by `(section, rank)`. Tags with identical `(section, rank)` keep their existing relative order.

---

## Implementation plan

### 1. New helper function — `match_rank`

Add to `tag_list.rs` (module-level, not a method):

```rust
/// Returns the match rank for a non-empty, pre-lowercased query against a tag name.
/// None  = no match (tag should be hidden)
/// Some(0) = full match
/// Some(1) = prefix match
/// Some(2) = contains match
fn match_rank(query_lower: &str, tag_name: &str) -> Option<u8> {
    let name_lower = tag_name.to_lowercase();
    if name_lower == query_lower      { return Some(0); }
    if name_lower.starts_with(query_lower) { return Some(1); }
    if name_lower.contains(query_lower)    { return Some(2); }
    None
}
```

### 2. Update `tag_matches_filter`

Change from:
```rust
fn tag_matches_filter(&self, t: &Tag) -> bool {
    let q = self.filter_query.trim().to_lowercase();
    q.is_empty() || t.tag().to_lowercase().contains(&q)
}
```
To:
```rust
fn tag_matches_filter(&self, t: &Tag) -> bool {
    let q = self.filter_query.trim().to_lowercase();
    q.is_empty() || match_rank(&q, t.tag()).is_some()
}
```
Visibility is identical to today — only the sort order inside `rebuild_filtered_display_tag_ids` changes.

### 3. Update `rebuild_filtered_display_tag_ids`

Change the `sort_by_key` closure from returning `u8` (section only) to `(u8, u8)` (section, rank):

Before:
```rust
filtered.sort_by_key(|id| match self.tags_by_id.get(id) {
    Some(t) if !t.is_stored() => 0u8,
    Some(t) if t.is_starred() => 1u8,
    _ => 2u8,
});
```

After:
```rust
let q = self.filter_query.trim().to_lowercase();
filtered.sort_by_key(|id| {
    let tag = self.tags_by_id.get(id);
    let section: u8 = match tag {
        Some(t) if !t.is_stored() => 0,
        Some(t) if t.is_starred() => 1,
        _ => 2,
    };
    let rank: u8 = if q.is_empty() {
        0  // no query → treat everything as rank 0 so original order is kept
    } else {
        tag.and_then(|t| match_rank(&q, t.tag())).unwrap_or(2)
    };
    (section, rank)
});
```

`sort_by_key` is already a stable sort in Rust's standard library, so same-key items keep their existing order.

---

## What does NOT change

| Thing | Why unchanged |
|-------|---------------|
| `TagList::set_filter` public API | same signature, same call sites |
| `TagList::filter_query` | same field, same accessor |
| `filtered_display_tag_ids` field and accessor | same type `Vec<TagId>` |
| All UI code (`tag_grid/view.rs`, `tag_panel/`) | consumes `filtered_display_tag_ids()` with no knowledge of rank |
| All messages and state | no new messages, no new state fields |
| Section ordering (unstored → starred → unstarred) | rank is a sub-key inside each section |
| Empty query behaviour | rank `0` for all → stable sort leaves order unchanged |

---

## Worked examples (case-insensitive)

List (original display order): `Hello`, `aHello`, `Hel` — all stored, unstarred (section 2).

| Query | Hello rank | aHello rank | Hel rank | Result order |
|-------|-----------|------------|---------|--------------|
| `l`   | 2 (contains) | 2 (contains) | 2 (contains) | Hello, aHello, Hel |
| `H`   | 1 (prefix)   | 2 (contains) | 1 (prefix)   | Hello, Hel, aHello |
| `He`  | 1 (prefix)   | 2 (contains) | 1 (prefix)   | Hello, Hel, aHello |
| `Hel` | 1 (prefix)   | 2 (contains) | 0 (full)     | Hel, Hello, aHello |

---

## Tests to add

New unit tests in the `#[cfg(test)]` block of `tag_list.rs`:

1. **full match sorts first** — `Hel` query against `[Hello, aHello, Hel]` → `[Hel, Hello, aHello]`
2. **prefix before contains** — `H` query → `[Hello, Hel, aHello]`
3. **section boundary preserved** — starred tag with rank 2 still sorts after unstarred tag with rank 0
4. **empty query unchanged** — no reordering when query is empty
5. **case-insensitive rank** — lowercase `hel` gives the same result as `Hel`

Existing test `test_filtered_display_tag_ids_case_insensitive_contains` remains valid — it only checks visibility counts, not order.

---

## Files touched

| File | Change |
|------|--------|
| `crates/frename-core/src/tags/tag_list.rs` | Add `match_rank`, update `tag_matches_filter`, update `rebuild_filtered_display_tag_ids`, add tests |

No other files need changes.
