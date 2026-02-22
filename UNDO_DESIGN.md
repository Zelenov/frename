# Undo System Design

Design-only document. No implementation begins until explicitly requested.

Test specification (Gherkin scenarios): see `UNDO_TESTS.md`.

---

## Goals

- **Undo and redo** for specified user actions.
- **Transparent interface**: existing call sites keep the same method signatures. No extra return values, no history/sink parameters, no undo types visible at call sites.
- **Core-only**: all undo types live in `frename-core`. No Iced or UI crate dependency.
- **Expandable**: new undoable actions are added as new command types without redesigning the system.

---

## Undoable actions

| User action | Command | Data stored |
|-------------|---------|-------------|
| Navigate to next / previous / indexed file (+ its deferred save) | `NavigateFileCommand` | `from_index`, `to_index`, `path_before`, `path_after`, `snapshot_before`, `snapshot_after` |
| Reorder tags in the file-name panel | `ReorderTagCommand` | `moved_id: TagId`, `from_index: usize`, `to_index: usize` |

### Why navigation and save are one command

Tags are never written to disk while editing; the rename happens when the user navigates away (deferred save). To the user, "go to next file" is one atomic action — it renames File A and opens File B. A single `NavigateFileCommand` captures both, so one Ctrl+Z restores File A's name on disk **and** returns focus to it.

### Scope of undo-navigate (version 1 limitation)

After undoing a navigation, the file on disk is restored to its pre-save name, and that file is re-selected. The tag state is re-read from the restored file name (same as opening the file fresh). The in-memory checked/unchecked state the user had before navigating is **not** restored; that is a future enhancement.

---

## Approach

**Option C (wrapper)** is the chosen pattern: a wrapper type has the same public method signatures as the inner core type; inside, it captures before-state, calls the inner method, then pushes a command to a `CommandSink`. Call sites are unchanged.

**Exception — navigate action:** the `NavigateFileCommand` spans several deferred messages (`FileOpened` → `VideoUnloaded` → `apply_file_updated`). A wrapper cannot collect all the data in one call. Instead, the command is pushed directly at the end of `apply_file_updated`, which is the natural completion point where all data is available.

Options A (explicit at call site) and B (wrapper returns `Option<Command>`) were rejected: both require call sites to handle commands, changing the interface.

---

## Core types

### `Undoable` — trait implemented by every command

```rust
pub trait Undoable: Send {
    fn undo(&mut self, ctx: &mut UndoContext<impl AppStateStore + Clone>) -> Result<(), UndoError>;
    fn redo(&mut self, ctx: &mut UndoContext<impl AppStateStore + Clone>) -> Result<(), UndoError>;
}
```

`undo` reverts the action. `redo` re-applies it. The command struct stores all data needed for both.

### `CommandSink` — trait implemented by anything that can receive commands

```rust
pub trait CommandSink {
    fn push(&mut self, cmd: Box<dyn Undoable>);
}
```

`Rc<RefCell<History>>` implements this. Wrappers hold a `CommandSink` and push internally.

### `UndoContext` — transient struct built per undo/redo call

```rust
pub struct UndoContext<'a, S> {
    pub directory: &'a mut Directory<S>,
    pub tag_list:  &'a mut TagList<S>,
}
```

Never stored globally. Built in the binary just before calling `history.undo(ctx)` or `history.redo(ctx)`, using split borrows on `FolderWorkspace` fields:

```rust
// In FolderWorkspace::update, Message::Undo arm
let Some(dir) = self.directory.as_mut() else { return Task::none() };
let tag_list = self.file_workspace.tag_list_mut(); // new method to add
let mut ctx = UndoContext { directory: dir, tag_list };
match self.history.undo(&mut ctx) { ... }
```

`FileWorkspace::tag_list_mut() -> &mut TagList<S>` does not exist yet and must be added (binary side).

### `History` — singleton, one per app run

```rust
pub struct History {
    undo_stack: Vec<Box<dyn Undoable>>,
    redo_stack: Vec<Box<dyn Undoable>>,
    max_depth:  usize,  // e.g. 50; push drops oldest when exceeded
}

impl History {
    pub fn push(&mut self, cmd: Box<dyn Undoable>);
    pub fn undo<S: AppStateStore + Clone>(&mut self, ctx: &mut UndoContext<S>) -> Result<(), UndoError>;
    pub fn redo<S: AppStateStore + Clone>(&mut self, ctx: &mut UndoContext<S>) -> Result<(), UndoError>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
}
```

`push` clears the redo stack. `undo`/`redo` follow peek-then-commit: run the command first; only move it between stacks on `Ok`. On `Err`, leave both stacks unchanged.

Owned by `FolderWorkspace` as a plain `history: History` field. No `Rc<RefCell>` needed for the two initial commands since both are pushed directly (not via wrappers).

### `UndoError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum UndoError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Index out of range: {0}")]
    IndexOutOfRange(usize),
    #[error("Tag not found: {0:?}")]
    TagNotFound(TagId),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## Commands

### `NavigateFileCommand`

**Stored data:**
```rust
pub struct NavigateFileCommand {
    from_index: usize,
    to_index:   usize,
    path_before:     PathBuf,       // path of from_index file before rename
    path_after:      PathBuf,       // path after save_and_reparse
    snapshot_before: FileSnapshot,  // snapshot that was saved
    snapshot_after:  FileSnapshot,  // canonical snapshot from reparse
}
```

**`undo`:**
1. `std::fs::rename(&self.path_after, &self.path_before)` — restore file name on disk.
2. `ctx.directory.update_file(&self.path_before, &self.snapshot_before)` — sync in-memory entry.
3. `ctx.directory.select_index(self.from_index)` — return to original file.

**`redo`:**
1. `std::fs::rename(&self.path_before, &self.path_after)`.
2. `ctx.directory.update_file(&self.path_after, &self.snapshot_after)`.
3. `ctx.directory.select_index(self.to_index)`.

**Where it is pushed:** `FolderWorkspace::apply_file_updated`, after `save_and_reparse` returns.

**Required binary-side change — track `from_index` through the deferred save chain:**

```rust
// Before (current)
pending_file_updated: Option<(PathBuf, FileSnapshot)>,

// After
pending_file_updated: Option<(PathBuf, FileSnapshot, Option<usize>)>,
//                                                    ^^^^^^^^^^^^ from_index before navigation
```

Capture `from_index = self.directory.as_ref().and_then(|d| d.selected_index())` in `select_next()`, `select_previous()`, and `select_file_at()` **before** advancing the directory. Pass it into `pending_file_updated` when setting it in `apply_file_opened`.

At push time in `apply_file_updated`:
- `from_index` = stored in `pending_file_updated.2`
- `to_index` = `directory.selected_index()` (already advanced by this point)
- `path_before` = path passed to `apply_file_updated`
- `path_after` / `snapshot_after` = returned from `save_and_reparse`
- `snapshot_before` = snapshot passed to `apply_file_updated`

### `ReorderTagCommand`

**Stored data:**
```rust
pub struct ReorderTagCommand {
    moved_id:   TagId,
    from_index: usize,  // index in selected_tag_ids before the move
    to_index:   usize,  // index in selected_tag_ids after the move
}
```

**`undo`:** `ctx.tag_list.reorder_tag_to_index(self.moved_id, self.from_index)`
**`redo`:** `ctx.tag_list.reorder_tag_to_index(self.moved_id, self.to_index)`

**Where it is pushed:** `FolderWorkspace::handle_file_name_panel`, on `DragEnded`, before `file_workspace.reorder_tag_to_index`.

**Capture pattern:**
```rust
// In handle_file_name_panel, DragEnded branch (before the existing reorder call)
if let (Some(did), Some(idx)) = (dragged_id, drop_index) {
    let from_index = self.file_workspace.tag_list().checked_index_of(did);
    self.file_workspace.reorder_tag_to_index(did, idx);
    if let Some(fi) = from_index {
        self.history.push(Box::new(ReorderTagCommand {
            moved_id: did, from_index: fi, to_index: idx,
        }));
    }
}
```

`TagList::checked_index_of(id)` — returns position of `id` in `selected_tag_ids`; must exist or be added.

---

## UI integration

### New messages

Add to `folder_workspace::Message` (or app-level if undo spans workspaces):
```rust
Undo,
Redo,
```

### Keyboard bindings

In the existing keyboard subscription (`src/app/`), map:
- `Ctrl+Z` → `Message::FolderWorkspace(folder_workspace::Message::Undo)`
- `Ctrl+Y` or `Ctrl+Shift+Z` → `Message::FolderWorkspace(folder_workspace::Message::Redo)`

### Handler in `FolderWorkspace::update`

```rust
Message::Undo => {
    if !self.history.can_undo() { return Task::none(); }
    let Some(dir) = self.directory.as_mut() else { return Task::none(); };
    let tl = self.file_workspace.tag_list_mut();
    let mut ctx = UndoContext { directory: dir, tag_list: tl };
    match self.history.undo(&mut ctx) {
        Ok(()) => self.refresh_after_undo_redo(),
        Err(e) => {
            log::warn!("Undo failed: {}", e);
            Task::done(Message::ReportError(e.to_string()))
        }
    }
}
// Message::Redo: symmetric
```

### Post-undo/redo UI refresh

```rust
fn refresh_after_undo_redo(&mut self) -> Task<Message> {
    // Always emit FileOpened for the now-selected file.
    // This handles both: navigate undo (different file) and reorder undo (same file,
    // tag list in core is already updated; file_workspace.set_file re-reads it).
    let file = self.directory.as_ref()
        .and_then(|d| d.selected_file())
        .cloned();
    match file {
        Some(f) => Task::done(Message::FileOpened(f)),
        None    => Task::none(),
    }
}
```

### Exposing can_undo / can_redo to the view

Pass `self.history.can_undo()` and `self.history.can_redo()` into `folder_workspace::view` so the view can disable or dim undo/redo controls. No undo types reach the view layer.

---

## Failure policy

1. **Leave stacks unchanged** on failure: if `command.undo(ctx)` (or `redo`) returns `Err`, do not pop the command. It remains at the top of its stack so the user can try again or handle the underlying cause.
2. **Report to user**: emit `Message::ReportError(e.to_string())` through the app's existing error display path.
3. **Log**: `log::warn!("Undo failed: {}", e)`.
4. **No UI refresh**: since no state changed, skip `refresh_after_undo_redo`.

`History::undo` implementation sketch:
```
peek top command (do not pop yet)
call command.undo(ctx)
  Ok  → pop from undo stack, push to redo stack, return Ok
  Err → leave stacks unchanged, return Err
```

### Typical failure cases

| Command | Failure scenario |
|---------|-----------------|
| `NavigateFileCommand::undo` | `path_after` no longer exists on disk (deleted/moved externally) |
| `NavigateFileCommand::redo` | `path_before` no longer exists |
| `NavigateFileCommand` | `from_index` or `to_index` out of range (directory was reloaded or rescanned) |
| `ReorderTagCommand` | `moved_id` no longer in `selected_tag_ids` (tag was deleted or file changed) |

---

## File layout

```
crates/frename-core/src/
  undo/
    mod.rs               // pub use all public undo types
    traits.rs            // Undoable, CommandSink
    history.rs           // History
    context.rs           // UndoContext
    error.rs             // UndoError
    commands/
      mod.rs             // pub use NavigateFileCommand, ReorderTagCommand
      navigate_file.rs   // NavigateFileCommand
      reorder_tag.rs     // ReorderTagCommand
    wrappers/
      mod.rs             // (empty for now; add UndoableXxx types here when Option C wrappers are needed)
```

In `crates/frename-core/src/lib.rs`: add `pub mod undo;` and re-export `History`, `UndoContext`, `UndoError`, `NavigateFileCommand`, `ReorderTagCommand`.

---

## Binary-side changes summary

| File | Change |
|------|--------|
| `src/features/folder_workspace/state.rs` | Add `history: History` field; extend `pending_file_updated` to carry `from_index`; add `Message::Undo`, `Message::Redo` handling; capture `from_index` in navigate methods; push commands in `apply_file_updated` and `handle_file_name_panel` |
| `src/features/file_workspace/state.rs` | Add `pub fn tag_list_mut(&mut self) -> &mut TagList<S>` |
| `src/app/` (keyboard subscription) | Map `Ctrl+Z` → `Message::Undo`, `Ctrl+Y`/`Ctrl+Shift+Z` → `Message::Redo` |
| `src/features/folder_workspace/messages.rs` | Add `Undo`, `Redo` variants |

---

## Expanding the system

### Adding a new command

1. Create `undo/commands/<name>.rs` with `pub struct <Name>Command { ... }`.
2. Implement `Undoable`: `fn undo(...)` and `fn redo(...)`.
3. In `undo/commands/mod.rs`: add `pub mod <name>; pub use <name>::<Name>Command;`.
4. Push `Box::new(<Name>Command { ... })` at the right handler (binary side).
5. Write a unit test: perform action → push command → undo → assert state; redo → assert state.

### Adding a new core class with undoable operations

1. The core type stays clean — no undo in its API.
2. Add `undo/wrappers/<name>.rs`: `pub struct UndoableXxx<S, H: CommandSink> { inner: Xxx<S>, sink: H }`.
3. Each mutating method: capture before-state → call `self.inner.method(...)` → push command via `self.sink.push(...)` → return same value as `inner`.
4. Read-only methods delegate to `self.inner` directly.
5. App holds `UndoableXxx` instead of `Xxx`; call sites are unchanged.

---

## Singleton / transient summary

| Thing | Lifetime | Notes |
|-------|----------|-------|
| `History` | App run — owned by `FolderWorkspace` | One instance; all commands push here |
| `UndoContext` | Per undo/redo call — built and dropped inline | Holds `&mut Directory` + `&mut TagList` |
| `NavigateFileCommand`, `ReorderTagCommand` | Owned by `History` stacks | Created at action time, used on undo/redo |
| Wrappers (`UndoableXxx`) | Same as the inner type they wrap | Hold a ref or clone of the `CommandSink` |

---

## Out of scope

- Undoing actions not yet in core (delete tag, rename tag, create tag): add commands later.
- Keyboard binding placement specifics (UI concern).
- Undo history persistence across restarts.
- Restoring exact in-memory tag state (checked, unchecked) after undo-navigate (version 1 re-reads from restored file name).

---

After implementation: record chosen trait names, context shape, and crate layout in `docs/DECISIONS.md`.
