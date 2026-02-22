Feature: Tag clipboard — copy and paste selected tags between files

  The clipboard captures the ordered list of checked tags for the currently open file
  and allows pasting that exact selection onto another file.
  Two representations are maintained simultaneously:
    - An internal payload (Vec<String> of tag names in order) used for in-app paste.
    - A plain-text file name written to the OS clipboard for pasting into other apps.
  Frename never reads from the OS clipboard for in-app paste.

  Background:
    Given a tag list exists with the following stored tags in display order:
      | name   | checked |
      | alpha  | true    |
      | beta   | true    |
      | gamma  | false   |
      | delta  | true    |
    And the file-name panel shows chips in this order: "alpha", "beta", "delta"
    And the generated file name is "alpha.beta.delta.mp4"

  # ─── COPY ───────────────────────────────────────────────────────────────────

  Scenario: Copy captures checked tags in their current chip order
    When the user triggers CopyTagsToClipboard
    Then the internal clipboard payload contains tags in order: "alpha", "beta", "delta"

  Scenario: Copy writes the generated file name to the OS clipboard as plain text
    When the user triggers CopyTagsToClipboard
    Then the OS clipboard text is "alpha.beta.delta.mp4"

  Scenario: Copy when no file is open is a no-op
    Given no file is currently open
    When the user triggers CopyTagsToClipboard
    Then the internal clipboard payload is empty
    And the OS clipboard is unchanged

  Scenario: Copy reflects the current chip order, not the tag-panel display order
    Given the file-name panel chip order has been reordered to: "delta", "alpha", "beta"
    When the user triggers CopyTagsToClipboard
    Then the internal clipboard payload contains tags in order: "delta", "alpha", "beta"
    And the OS clipboard text is "delta.alpha.beta.mp4"

  # ─── PASTE ──────────────────────────────────────────────────────────────────

  Scenario: Paste replaces the current file's checked tags with the copied tags
    Given the user has previously copied tags in order: "alpha", "beta", "delta"
    And the current file has tags "gamma" and "delta" checked
    When the user triggers PasteTagsFromClipboard
    Then the checked tags are exactly: "alpha", "beta", "delta"
    And the file-name panel shows chips in order: "alpha", "beta", "delta"

  Scenario: Paste sets chip order to match the copied order exactly
    Given the user has previously copied tags in order: "delta", "alpha", "beta"
    And the current file has no tags checked
    When the user triggers PasteTagsFromClipboard
    Then the file-name panel shows chips in order: "delta", "alpha", "beta"

  Scenario: Paste when no copy has been done is a no-op
    Given the clipboard has no internal payload
    And the current file has tags "gamma" checked
    When the user triggers PasteTagsFromClipboard
    Then the checked tags are still: "gamma"

  Scenario: Paste clears all previously checked tags before applying the payload
    Given the user has previously copied tags in order: "alpha"
    And the current file has tags "beta", "gamma", "delta" all checked
    When the user triggers PasteTagsFromClipboard
    Then the checked tags are exactly: "alpha"
    And "beta", "gamma", and "delta" are unchecked

  Scenario: Paste skips tag names that do not exist in the current tag list
    Given the user has previously copied tags in order: "alpha", "unknown-tag", "delta"
    When the user triggers PasteTagsFromClipboard
    Then the checked tags are exactly: "alpha", "delta"
    And no new tags are created in the tag list

  Scenario: Paste with all unknown tag names results in no checked tags
    Given the user has previously copied tags in order: "no-such-tag", "also-missing"
    When the user triggers PasteTagsFromClipboard
    Then no tags are checked
    And no new tags are created in the tag list

  Scenario: Paste triggers FileUpdated so the selection is persisted
    Given the user has previously copied tags in order: "alpha", "beta"
    When the user triggers PasteTagsFromClipboard
    Then a FileUpdated event is emitted for the current file path

  # ─── FAKE CLIPBOARD (debug / test isolation) ────────────────────────────────

  Scenario: FakeClipboard stores internal payload without touching the OS clipboard
    Given a FakeClipboard is in use
    When the user triggers CopyTagsToClipboard
    Then the FakeClipboard stored payload contains tags in order: "alpha", "beta", "delta"
    And the OS clipboard is unchanged

  Scenario: FakeClipboard paste returns the last copied payload
    Given a FakeClipboard is in use
    And the user has previously copied tags in order: "alpha", "delta"
    When the user triggers PasteTagsFromClipboard
    Then the checked tags are exactly: "alpha", "delta"

  Scenario: FakeClipboard clear discards any stored payload
    Given a FakeClipboard is in use
    And the user has previously copied tags in order: "alpha"
    When the clipboard is cleared
    Then the FakeClipboard stored payload is empty
    And a subsequent PasteTagsFromClipboard is a no-op

  # ─── ROUND-TRIP ─────────────────────────────────────────────────────────────

  Scenario: Copy on one file then paste on another transfers the full selection
    Given file A has tags "alpha" and "beta" checked (in that order)
    And the user opens file A and triggers CopyTagsToClipboard
    And the user switches to file B which has tags "gamma" and "delta" checked
    When the user triggers PasteTagsFromClipboard on file B
    Then file B's checked tags are exactly: "alpha", "beta"
    And file B's file-name panel shows chips: "alpha", "beta"

  Scenario: Pasting does not alter the OS clipboard text
    Given the OS clipboard currently holds the text "some-other-text"
    And the clipboard has no internal payload
    When the user triggers PasteTagsFromClipboard
    Then the OS clipboard still holds "some-other-text"
