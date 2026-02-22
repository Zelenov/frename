# Undo System — Gherkin Test Cases

Test specification for the undo/redo system described in `UNDO_DESIGN.md`.
These are the acceptance criteria for implementation; they translate directly into Rust unit tests.

---

## Feature: NavigateFileCommand — happy paths

```gherkin
Feature: Undo and redo file navigation (NavigateFileCommand)

  Background:
    Given a folder is open with three files at indices 0, 1, 2:
      | index | file name on disk       |
      | 0     | action.vacation.mp4     |
      | 1     | unedited.mp4            |
      | 2     | comedy.summer.mp4       |
    And file at index 0 is selected
    And the undo and redo stacks are empty

  Scenario: Undo "next file" navigation restores file name on disk and returns to previous file
    Given the user has checked the "Holiday" tag on the current file
      And the file name preview shows "holiday.vacation.mp4"
    When the user navigates to the next file
    Then file at index 0 is renamed to "holiday.vacation.mp4" on disk
      And file at index 1 is selected
      And the undo stack contains 1 entry
      And the redo stack is empty
    When the user triggers Undo
    Then file at index 0 is renamed back to "action.vacation.mp4" on disk
      And file at index 0 is selected
      And the file workspace shows file "action.vacation.mp4"
      And the undo stack is empty
      And the redo stack contains 1 entry

  Scenario: Undo "previous file" navigation restores file name and returns to original file
    Given file at index 1 is selected
      And the "Comedy" tag is checked on the current file
      And the file name preview shows "comedy.unedited.mp4"
    When the user navigates to the previous file
    Then file at index 1 is renamed to "comedy.unedited.mp4" on disk
      And file at index 0 is selected
      And the undo stack contains 1 entry
    When the user triggers Undo
    Then file at index 1 is renamed back to "unedited.mp4" on disk
      And file at index 1 is selected
      And the undo stack is empty
      And the redo stack contains 1 entry

  Scenario: Undo navigation when tags were unchanged leaves file name unchanged
    Given no tags are checked on the current file
      And the file name preview shows "vacation.mp4" (name only, no tags)
    When the user navigates to the next file
    Then the file on disk remains "action.vacation.mp4"
      # NOTE: save_and_reparse is still called; path_before == path_after in this case
      And file at index 1 is selected
      And the undo stack contains 1 entry
    When the user triggers Undo
    Then the file on disk is still "action.vacation.mp4"
      And file at index 0 is selected

  Scenario: Redo re-applies the navigation and rename
    Given the user has checked the "Holiday" tag on file at index 0
    When the user navigates to the next file
      And the user triggers Undo
      And the user triggers Redo
    Then file at index 0 is renamed to "holiday.vacation.mp4" on disk
      And file at index 1 is selected
      And the undo stack contains 1 entry
      And the redo stack is empty

  Scenario: Navigate by clicking a file in the list also records a NavigateFileCommand
    Given the user has checked the "Comedy" tag on file at index 0
    When the user clicks file at index 2 in the file list
    Then file at index 0 is renamed to "comedy.vacation.mp4" on disk
      And file at index 2 is selected
      And the undo stack contains 1 entry
    When the user triggers Undo
    Then file at index 0 is renamed back to "action.vacation.mp4" on disk
      And file at index 0 is selected

  Scenario: Multiple navigations produce multiple undo entries; they unwind in reverse order
    Given the "Action" tag is checked on file at index 0
      And no tags are checked on file at index 1
    When the user navigates to the next file     # 0 → 1, saves "action.vacation.mp4" → renames
      And the "Summer" tag is checked on file at index 1
      And the user navigates to the next file     # 1 → 2, saves "summer.unedited.mp4" → renames
    Then the undo stack contains 2 entries
    When the user triggers Undo                   # revert 1 → 2
    Then file at index 1 is renamed back to "unedited.mp4" on disk
      And file at index 1 is selected
    When the user triggers Undo                   # revert 0 → 1
    Then file at index 0 is renamed back to "action.vacation.mp4" on disk
      And file at index 0 is selected
      And the undo stack is empty
      And the redo stack contains 2 entries
```

---

## Feature: NavigateFileCommand — failure paths

```gherkin
Feature: Navigate undo/redo failure handling

  Background:
    Given a folder is open with files:
      | index | file name on disk   |
      | 0     | holiday.old.mp4     |
      | 1     | unedited.mp4        |
    And file at index 0 was previously navigated away from
      And the NavigateFileCommand on the undo stack records:
        | from_index      | 0                   |
        | to_index        | 1                   |
        | path_before     | old.mp4             |
        | path_after      | holiday.old.mp4     |
    And file at index 1 is currently selected

  Scenario: Undo fails gracefully when path_after no longer exists on disk
    Given "holiday.old.mp4" has been deleted from disk by an external process
    When the user triggers Undo
    Then an error is reported to the user (e.g. "File not found: holiday.old.mp4")
      And the undo stack still contains the failed NavigateFileCommand
      And the redo stack is unchanged
      And the file selection is unchanged (still index 1)
      And no file rename is performed on disk

  Scenario: Undo fails gracefully when from_index is out of range
    Given the directory was rescanned and now contains only 1 file
      And the stored from_index (0) is still valid but to_index (1) is out of range
    When the user triggers Undo
    Then an error is reported to the user
      And the undo stack still contains the failed NavigateFileCommand
      And the redo stack is unchanged

  Scenario: Redo fails gracefully when path_before no longer exists on disk
    Given a NavigateFileCommand is on the redo stack with path_before = "old.mp4"
      And "old.mp4" no longer exists on disk
    When the user triggers Redo
    Then an error is reported to the user (e.g. "File not found: old.mp4")
      And the redo stack still contains the failed NavigateFileCommand
      And the undo stack is unchanged
```

---

## Feature: ReorderTagCommand — happy paths

```gherkin
Feature: Undo and redo tag reorder in the file-name panel (ReorderTagCommand)

  Background:
    Given a file is open with three checked tags in order: ["Action", "Comedy", "Summer"]
      # selected_tag_ids order: Action(0), Comedy(1), Summer(2)
    And the file name panel shows chips in that order
    And the undo and redo stacks are empty

  Scenario: Undo a forward tag reorder restores original order
    When the user drags the "Action" chip from index 0 to index 2
    Then the file-name panel shows chips in order: ["Comedy", "Summer", "Action"]
      And the undo stack contains 1 entry (ReorderTagCommand: moved_id=Action, from=0, to=2)
    When the user triggers Undo
    Then the file-name panel shows chips in order: ["Action", "Comedy", "Summer"]
      And the undo stack is empty
      And the redo stack contains 1 entry

  Scenario: Undo a backward tag reorder restores original order
    When the user drags the "Summer" chip from index 2 to index 0
    Then the file-name panel shows chips in order: ["Summer", "Action", "Comedy"]
    When the user triggers Undo
    Then the file-name panel shows chips in order: ["Action", "Comedy", "Summer"]

  Scenario: Redo re-applies the reorder
    When the user drags the "Action" chip from index 0 to index 2
      And the user triggers Undo
      And the user triggers Redo
    Then the file-name panel shows chips in order: ["Comedy", "Summer", "Action"]
      And the undo stack contains 1 entry
      And the redo stack is empty

  Scenario: Multiple reorders can be undone in sequence
    When the user drags "Action" from index 0 to index 2
      # order: Comedy, Summer, Action
      And the user drags "Summer" from index 1 to index 0
      # order: Summer, Comedy, Action
    Then the undo stack contains 2 entries
    When the user triggers Undo
    Then the file-name panel shows: ["Comedy", "Summer", "Action"]
    When the user triggers Undo
    Then the file-name panel shows: ["Action", "Comedy", "Summer"]
      And the undo stack is empty
      And the redo stack contains 2 entries
```

---

## Feature: ReorderTagCommand — failure paths

```gherkin
Feature: Reorder undo failure handling

  Scenario: Undo reorder fails when the tag was deleted from the list
    Given a file is open with tags: ["Action", "Comedy"]
      And the ReorderTagCommand (moved_id=Action, from=0, to=1) is on the undo stack
    When the "Action" tag is deleted from the tag list
      And the user triggers Undo
    Then an error is reported to the user (e.g. "Tag not found")
      And the undo stack still contains the failed ReorderTagCommand
      And the redo stack is unchanged
      And the tag order is unchanged (Comedy shown first, Action gone)
```

---

## Feature: History management

```gherkin
Feature: History stack management (push, can_undo, can_redo, max depth)

  Scenario: History is empty at startup
    Given the application has just started
    Then can_undo returns false
      And can_redo returns false

  Scenario: can_undo and can_redo update correctly through action, undo, redo cycle
    Given the undo and redo stacks are empty
    When an action is performed and pushed to history
    Then can_undo returns true
      And can_redo returns false
    When the user triggers Undo
    Then can_undo returns false
      And can_redo returns true
    When the user triggers Redo
    Then can_undo returns true
      And can_redo returns false

  Scenario: Pushing a new action after undo clears the redo stack
    Given the user has navigated twice (undo stack: 2 entries)
    When the user triggers Undo (undo: 1 entry, redo: 1 entry)
      And the user performs a new action (e.g. reorder a tag)
    Then the undo stack contains 2 entries (the older navigate + the new reorder)
      And the redo stack is empty

  Scenario: History respects max depth; oldest entry is dropped when exceeded
    Given the max undo depth is 50
      And 50 navigation commands are already on the undo stack
    When the user performs one more navigation
    Then the undo stack still contains exactly 50 entries
      And the oldest navigation entry has been dropped
      And the newest navigation entry is at the top

  Scenario: Triggering Undo with an empty stack is a no-op
    Given the undo stack is empty
    When the user triggers Undo
    Then no error is shown
      And no state changes occur
      And the redo stack is still empty

  Scenario: Triggering Redo with an empty stack is a no-op
    Given the redo stack is empty
    When the user triggers Redo
    Then no error is shown
      And no state changes occur
```

---

## Feature: UI integration

```gherkin
Feature: Keyboard bindings and UI state for undo/redo

  Scenario: Ctrl+Z triggers Undo
    Given an undoable action has been performed
    When the user presses Ctrl+Z
    Then the top command is undone
      And the UI reflects the reverted state

  Scenario: Ctrl+Y triggers Redo
    Given an action was performed then undone
    When the user presses Ctrl+Y
    Then the top redo command is re-applied
      And the UI reflects the re-applied state

  Scenario: Ctrl+Shift+Z is an alternative Redo binding
    Given an action was performed then undone
    When the user presses Ctrl+Shift+Z
    Then the top redo command is re-applied

  Scenario: Undo is ignored when no folder is open
    Given no folder is currently open
    When the user presses Ctrl+Z
    Then no error is shown
      And the application remains idle

  Scenario: Undo failure shows an error message to the user
    Given a NavigateFileCommand is on the undo stack
      And the file referenced by path_after has been deleted
    When the user presses Ctrl+Z
    Then an error message is displayed (e.g. "Cannot undo: file no longer exists")
      And the undo stack is unchanged
      And no file is selected or deselected

  Scenario: After successful undo, the file workspace refreshes to match the reverted selection
    Given the user undoes a navigation (returning to file at index 0)
    Then the file workspace reloads the file at index 0
      And the tag list is rebuilt from the file at index 0's (now restored) name
      And the video player loads the file at index 0
```

---

## Feature: NavigateFileCommand — atomic navigate + save

```gherkin
Feature: Navigate and deferred save are one atomic undo unit

  Scenario: One Ctrl+Z reverses both the file rename and the file selection
    Given file at index 0 is open with tags ["Action", "Holiday"] checked
      And the file name preview shows "action.holiday.vacation.mp4"
    When the user navigates to file at index 1
    Then the following happen as a single recorded action:
      | event                          | result                               |
      | deferred save of file at idx 0 | "vacation.mp4" → "action.holiday.vacation.mp4" on disk |
      | file selection changes         | index 0 → index 1                   |
    And the undo stack contains exactly 1 entry (not 2)
    When the user triggers Undo
    Then file at index 0 is renamed back to "vacation.mp4" on disk
      And file at index 0 is selected
      # Both the rename and the selection change are reversed by a single Undo press

  Scenario: No separate "save" undo entry is recorded — save is always bundled with navigate
    Given the user performs 3 navigations
    Then the undo stack contains exactly 3 entries
      # There are no extra "save" entries interspersed between navigation entries
```

---

## Feature: ReorderTagCommand does not affect the undo/redo of navigation

```gherkin
Feature: Reorder and navigate commands coexist independently on the stack

  Scenario: Undo interleaved reorder and navigate commands in LIFO order
    Given file at index 0 is open with tags: ["Action", "Comedy"]
    When the user reorders "Comedy" from index 1 to index 0
      # stack: [ReorderTagCommand]
      And the user navigates to the next file
      # stack: [ReorderTagCommand, NavigateFileCommand]
    Then the undo stack contains 2 entries
    When the user triggers Undo    # reverts NavigateFileCommand
    Then the user is back on file at index 0
      And the file name on disk is restored to its pre-navigate value
      And the undo stack contains 1 entry (the ReorderTagCommand)
    When the user triggers Undo    # reverts ReorderTagCommand
    Then the tag order in the file-name panel is restored to: ["Action", "Comedy"]
      And the undo stack is empty
```
