# Clipboard (Copy / Paste) Design

This document describes the steps to implement a **copy / paste system** for frename. It is design-only; no implementation is started until explicitly requested.

---

## Goals and constraints

- **Copy** captures the ordered list of checked (selected) tags for the currently open file, exactly as they appear in the file-name panel (chips panel), left to right.
- **Paste** replaces the currently open file's checked-tag selection with what was copied: it clears every currently checked tag and re-checks the copied tags in the copied order. It does **not** create new tags.
- **Two clipboard representations** are maintained at the same time:
  - **Internal payload** — a `Vec<String>` of tag names in their copied order. Used exclusively when pasting *inside* frename. Reading this back never touches the OS clipboard.
  - **External (OS) text** — the file name that frename would generate from those tags (e.g. `"tagA.tagB.tagC.mp4"`). Written to the system clipboard so the user can paste the resulting file name into any other application (Notepad, Explorer search box, etc.).
- **Frename never reads from the OS clipboard** for in-app paste. Paste only works when frename itself was the last copy origin (internal payload is non-empty). Arbitrary text copied from elsewhere has no effect inside the app.
- **Testable / debuggable without touching the OS clipboard**: a `FakeClipboard` implementation stores the payload in memory only and is used in tests and debug builds.
- **Core-only trait**: `ClipboardService` and `ClipboardPayload` live in `frename-core` (no Iced dependency). The concrete `SystemClipboard` implementation lives in the app binary because it depends on an OS-level crate.
- **Expandable**: the same clipboard infrastructure can carry richer payloads later (e.g. a full snapshot including tag order keys, colors, etc.) without changing the call sites.

---

## What "selected tags" means

When a file is open, the chips panel shows **all checked tags in `selected_tag_ids` order**. This is exactly what `tag_list.file_snapshot().tags()` returns: a `Vec<String>` of tag names, left-to-right, matching the generated file name.

| Concept | Core location |
|---------|---------------|
| Tag checked state | `Tag::checked` flag inside `TagList::tags_by_id` |
| Chip order | `TagList::selected_tag_ids` (`OrderedCollection`) |
| Ordered list of checked names | `TagList::file_snapshot().tags()` → `&[String]` |
| Full generated file name | `FileSnapshot::file_name()` → `String` |

---

## Clipboard payload

```rust
/// Owned by ClipboardService. Represents one copy operation.
pub struct ClipboardPayload {
    /// Tag names in the order they appeared in the file-name panel at copy time.
    /// Only checked (selected) tags are included.
    pub tags: Vec<String>,
}
```

`ClipboardPayload` is plain data (no `TagId`s, no database references).  Tag names are compared case-sensitively on paste; a tag is matched if `tag.tag == name`.

---

## The `ClipboardService` trait (core)

```rust
/// Abstracts clipboard write and read.
/// Implementations live either in-process (FakeClipboard) or in the app binary (SystemClipboard).
pub trait ClipboardService: Send {
    /// Stores `payload` as the internal paste target.
    /// Also writes `plain_text` (the generated file name) to the OS clipboard if supported.
    fn copy(&mut self, payload: ClipboardPayload, plain_text: String);

    /// Returns the payload from the most recent in-app copy, or None if no copy has been done.
    /// Never reads from the OS clipboard; returns only what *frename* last copied.
    fn paste(&self) -> Option<&ClipboardPayload>;

    /// Discards any stored payload (e.g. when a folder is closed).
    fn clear(&mut self);
}
```

**Why `plain_text` is a separate argument**: the caller (app) computes it once from `FileSnapshot::file_name()` and passes it in, so `ClipboardService` does not need to know about `FileSnapshot`. The trait stays simple.

---

## Two implementations

### `FakeClipboard` (core — for tests and debug)

- Stores the payload in a private `Option<ClipboardPayload>` field.
- `copy`: saves payload; ignores `plain_text` entirely.
- `paste`: returns `Some(&payload)` if one was stored, else `None`.
- `clear`: sets field to `None`.
- Never calls any OS API. Safe to use in unit tests and when running without a windowing system.

```rust
// crates/frename-core/src/clipboard/fake.rs
pub struct FakeClipboard {
    stored: Option<ClipboardPayload>,
}

impl FakeClipboard {
    pub fn new() -> Self { Self { stored: None } }
    /// Direct read for test assertions.
    pub fn stored(&self) -> Option<&ClipboardPayload> { self.stored.as_ref() }
}

impl ClipboardService for FakeClipboard {
    fn copy(&mut self, payload: ClipboardPayload, _plain_text: String) {
        self.stored = Some(payload);
    }
    fn paste(&self) -> Option<&ClipboardPayload> { self.stored.as_ref() }
    fn clear(&mut self) { self.stored = None; }
}
```

### `SystemClipboard` (app binary — production)

- Holds two things: `stored: Option<ClipboardPayload>` (in-process) and a handle to an OS clipboard writer (e.g. `arboard::Clipboard`).
- `copy`:
  1. Stores `payload` in `self.stored`.
  2. Calls `os_clipboard.set_text(plain_text)` to write the file name to the system clipboard. If the OS call fails, logs a warning but does **not** fail the copy (the internal payload is still valid).
- `paste`: returns `Some(&self.stored)` if present. Never reads the OS clipboard.
- `clear`: sets `self.stored = None`. Does not clear the OS clipboard (the text stays in system clipboard so the user can still paste the file name elsewhere).

```rust
// src/services/system_clipboard.rs  (app binary, not frename-core)
pub struct SystemClipboard {
    stored: Option<ClipboardPayload>,
    os: arboard::Clipboard,   // or whichever OS clipboard crate is chosen
}

impl ClipboardService for SystemClipboard {
    fn copy(&mut self, payload: ClipboardPayload, plain_text: String) {
        self.stored = Some(payload);
        if let Err(e) = self.os.set_text(plain_text) {
            log::warn!("OS clipboard write failed: {e}");
        }
    }
    fn paste(&self) -> Option<&ClipboardPayload> { self.stored.as_ref() }
    fn clear(&mut self) { self.stored = None; }
}
```

---

## Copy operation (step by step)

**Trigger:** `Message::CopyTagsToClipboard` (e.g. Ctrl+C when the file-name panel is focused, or a toolbar button).

**Pre-condition:** A file is currently open (file workspace is active).

**Steps (in `FolderWorkspace::update`):**

1. **Get the file snapshot** from the current tag list:
   ```rust
   let snapshot = self.file_workspace.tag_list().file_snapshot();
   ```
2. **Build payload** from the snapshot's tag list:
   ```rust
   let payload = ClipboardPayload { tags: snapshot.tags().to_vec() };
   ```
3. **Build plain-text file name** for the OS clipboard:
   ```rust
   let plain_text = snapshot.file_name();
   ```
4. **Copy to clipboard service:**
   ```rust
   self.clipboard.copy(payload, plain_text);
   ```
5. **Optional UI feedback**: Emit a transient message (e.g. `Message::ShowToast("Copied".into())`) so the user sees confirmation. Not required for v1.

If no file is open, the message is a no-op.

---

## Paste operation (step by step)

**Trigger:** `Message::PasteTagsFromClipboard` (e.g. Ctrl+V when a file is open, or a toolbar button).

**Pre-condition:** A file is currently open; clipboard holds a payload (frename was the last copy origin).

**Steps (in `FolderWorkspace::update`):**

1. **Retrieve payload:**
   ```rust
   let Some(payload) = self.clipboard.paste() else { return Task::none(); };
   let tags_to_apply: Vec<String> = payload.tags.clone();
   ```
   If no payload, no-op.

2. **Clear current selection** — uncheck every currently checked tag:
   ```rust
   let checked_ids: Vec<TagId> = self.file_workspace.tag_list().checked_ids_in_selected_order();
   for id in checked_ids {
       self.file_workspace.toggle_tag_by_id(id);   // toggles off
   }
   ```

3. **Apply tags from payload** — for each tag name in `tags_to_apply`, in order:
   ```rust
   let mut ordered_ids: Vec<TagId> = Vec::new();
   for name in &tags_to_apply {
       if let Some(id) = self.file_workspace.tag_list().find_tag_id_by_name(name) {
           if !self.file_workspace.tag_list().is_checked(id) {
               self.file_workspace.toggle_tag_by_id(id);   // toggles on
           }
           ordered_ids.push(id);
       }
       // Unknown tag names (not in current TagList) are silently skipped.
   }
   ```

4. **Restore order** — reorder `selected_tag_ids` so the pasted tags appear in the same left-to-right order as in the payload. Use `reorder_tag_to_index` sequentially, placing each tag at the correct checked index:
   ```rust
   for (target_checked_index, id) in ordered_ids.iter().enumerate() {
       self.file_workspace.reorder_tag_to_index(*id, target_checked_index);
   }
   ```

5. **Persist** — emit `Message::FileUpdated` (same path as existing "switch file" or "save" logic) so the change is saved to the file tagger. This is the same event that normal tag toggling eventually triggers.

6. **Refresh UI** — the view re-renders chips panel and tag grid from the mutated tag list. No special message is needed for this beyond the state change.

---

## Unknown tag names on paste

If the payload contains a name that does not exist in the current `TagList`:

- **Skip silently.** The remaining tags (those that do exist) are still applied in order.
- No error message, no new tags created. This keeps paste predictable.
- Rationale: the user always copies from the same app, so the most common cause of unknown names is switching to a different folder with a different tag set. Skipping is safer than creating new permanent tags automatically.

---

## New core method required: `find_tag_id_by_name`

The paste logic needs to look up a `TagId` by name string. This does not exist yet. It should be added to `TagList`:

```rust
impl<S> TagList<S> {
    /// Returns the first TagId whose tag string matches `name` (case-sensitive), or None.
    pub fn find_tag_id_by_name(&self, name: &str) -> Option<TagId> {
        self.tags_by_id
            .iter()
            .find(|(_, tag)| tag.tag == name)
            .map(|(id, _)| *id)
    }
}
```

---

## Messages

Add to the top-level message type (whichever enum `FolderWorkspace` dispatches from — likely `app::Message` or `folder_workspace::Message`):

```rust
/// Copy the current file's checked tag list to the clipboard.
CopyTagsToClipboard,
/// Paste the clipboard's tag list onto the current file, replacing its selection.
PasteTagsFromClipboard,
```

These are both dispatched from keyboard shortcuts (Ctrl+C / Ctrl+V) and can also be triggered by toolbar/menu actions.

---

## Where `ClipboardService` lives in the app

`FolderWorkspace` owns the clipboard service as a boxed trait object:

```rust
// in FolderWorkspace fields
clipboard: Box<dyn ClipboardService>,
```

On construction:
- **Production build**: `clipboard = Box::new(SystemClipboard::new(...))`.
- **Test / debug build**: `clipboard = Box::new(FakeClipboard::new())`.

Alternatively, `FolderWorkspace` can be generic over `C: ClipboardService` to avoid the vtable, but `Box<dyn ClipboardService>` is simpler and is fine for one service.

---

## Folder and file structure

```
crates/frename-core/src/
  clipboard/
    mod.rs             # pub use ClipboardService, ClipboardPayload, FakeClipboard
    traits.rs          # ClipboardService trait + ClipboardPayload struct
    fake.rs            # FakeClipboard (in-memory mock)

src/                   # app binary
  services/
    mod.rs
    system_clipboard.rs  # SystemClipboard (writes to OS clipboard via arboard or similar)
```

**Naming conventions** (matching undo system style):
- Trait: `ClipboardService` in `clipboard/traits.rs`
- Payload: `ClipboardPayload` in `clipboard/traits.rs`
- Fake: `FakeClipboard` in `clipboard/fake.rs`
- Real: `SystemClipboard` in `src/services/system_clipboard.rs`

**Crate root:** In `frename-core/src/lib.rs`, add `pub mod clipboard;` and re-export `ClipboardService`, `ClipboardPayload`, `FakeClipboard`.

---

## Two-format summary

| Format | Where stored | Written on copy | Read on paste |
|--------|-------------|-----------------|---------------|
| Internal payload (`ClipboardPayload`) | App memory (inside `ClipboardService` impl) | Yes — always | Yes — only by frename |
| Plain text (file name) | OS clipboard | Yes — always | Never read back by frename |

The OS clipboard text (`"tagA.tagB.mp4"`) is purely for the user's convenience when they want to paste the file name into another program. Frename ignores the OS clipboard on paste; it only reads its own internal payload.

---

## Keyboard shortcuts

| Key | Message |
|-----|---------|
| Ctrl+C (file-name panel focused, or globally when file is open) | `CopyTagsToClipboard` |
| Ctrl+V (file-name panel focused, or globally when file is open) | `PasteTagsFromClipboard` |

Exact binding scope (panel-specific vs global) is a UI concern decided during implementation.

---

## Steps to implement

### Phase 1: Core clipboard infrastructure

1. **Add `clipboard/` module to `frename-core`.**
   - `traits.rs`: `ClipboardPayload` struct, `ClipboardService` trait.
   - `fake.rs`: `FakeClipboard` with `stored: Option<ClipboardPayload>`.
   - `mod.rs`: re-exports.
   - Add `pub mod clipboard;` to `frename-core/src/lib.rs`.

2. **Add `find_tag_id_by_name` to `TagList`.**
   - Search `tags_by_id` by `tag.tag == name`, return `Option<TagId>`.

### Phase 2: App-side service

3. **Add `SystemClipboard` in `src/services/system_clipboard.rs`.**
   - Add chosen OS clipboard crate to `Cargo.toml` (e.g. `arboard`).
   - Implement `ClipboardService`: `copy` stores payload and calls `os.set_text(plain_text)`; `paste` returns stored ref; `clear` drops stored payload.

4. **Wire `ClipboardService` into `FolderWorkspace`.**
   - Add `clipboard: Box<dyn ClipboardService>` field.
   - Construct with `SystemClipboard` in production, `FakeClipboard` in tests.

### Phase 3: Copy and paste messages

5. **Add `CopyTagsToClipboard` and `PasteTagsFromClipboard` messages.**
   - Handle `CopyTagsToClipboard` in `FolderWorkspace::update`: build `ClipboardPayload` from `file_workspace.tag_list().file_snapshot()`, call `clipboard.copy(payload, plain_text)`.
   - Handle `PasteTagsFromClipboard` in `FolderWorkspace::update`: retrieve payload, uncheck all, re-check in order, reorder `selected_tag_ids`, emit `FileUpdated`.

6. **Add keyboard shortcuts in the UI layer.**
   - Map Ctrl+C → `CopyTagsToClipboard` and Ctrl+V → `PasteTagsFromClipboard` via Iced keyboard subscription or `on_key_press`.

### Phase 4: Testing

7. **Unit tests in `frename-core`** using `FakeClipboard`:
   - Copy with a known tag list; assert `fake.stored().unwrap().tags == expected_vec`.
   - Paste onto a file workspace; assert correct tags are checked, in correct order, in the chips panel.
   - Paste when no payload: assert tag list is unchanged.
   - Paste with a mix of known and unknown tag names: assert only known tags are applied.

8. **UI smoke test**: open a file, copy, open another file, paste, verify chips panel shows the same tags in the same order.

---

## Data captured per operation (summary)

| Operation | Data stored in `ClipboardPayload` | Data written to OS |
|-----------|-----------------------------------|--------------------|
| Copy | `tags: Vec<String>` — ordered checked tag names | `file_name()` → plain text string |
| Paste | (read back `tags`) | (nothing written) |

---

## Out of scope for this design

- Pasting arbitrary text from the OS clipboard (e.g. pasting from Notepad into frename to set tags): frename only reads its own internal payload.
- Creating new tags on paste when the clipboard contains an unknown tag name: unknown names are skipped.
- Copying individual tags (only the full ordered selection is copied, not a single tag).
- Cross-session persistence of the clipboard (the payload is lost on app exit).
- Merging pasted tags with existing selection (paste always replaces; it does not add to the current set).

---

After implementation, record the chosen OS clipboard crate and any unusual behavior (e.g. clipboard access failures on some systems) in `docs/DECISIONS.md`.
