Feature: Starred tags — pin stored tags into the starred section of the display order

  The display order is always divided into three sections:
    Section 1 — Unstored (snapshot-only) tags
    Section 2 — Starred stored tags
    Section 3 — Unstarred stored tags

  Starring a stored tag moves it to the START of section 2 (immediately after the last unstored tag).
  Unstarring moves it to the START of section 3 (immediately after the last starred tag).
  Only stored tags can be starred; attempting to star an unstored tag is rejected.
  Both display_tag_ids and selected_tag_ids are kept in sync using the same anchor TagId.
  The starred flag and both collection orders are persisted to the database.

  # ═══════════════════════════════════════════════════════════════════════════
  # BACKGROUND LEGEND
  #   stored=yes   → tag exists in the database (stored == true)
  #   stored=no    → snapshot-only tag (stored == false); CANNOT be starred
  #   starred=yes  → tag.starred == true (only possible when stored=yes)
  #   starred=no   → tag.starred == false
  # ═══════════════════════════════════════════════════════════════════════════

  # ─── INSERT_AFTER on OrderedCollection ─────────────────────────────────────

  Scenario: insert_after with None places key at position 0
    Given an OrderedCollection with elements in order: A, B, C
    When insert_after(X, None) is called
    Then the order is: X, A, B, C

  Scenario: insert_after with Some(anchor) places key immediately after the anchor
    Given an OrderedCollection with elements in order: A, B, C
    When insert_after(X, Some(B)) is called
    Then the order is: A, B, X, C

  Scenario: insert_after with Some(last element) appends key at the end
    Given an OrderedCollection with elements in order: A, B, C
    When insert_after(X, Some(C)) is called
    Then the order is: A, B, C, X

  Scenario: insert_after moves a key that already exists in the collection
    Given an OrderedCollection with elements in order: A, B, C, D
    When insert_after(A, Some(C)) is called
    Then the order is: B, C, A, D
    And A is no longer at its original position

  Scenario: insert_after on a single-element collection with None places key before that element
    Given an OrderedCollection with elements in order: A
    When insert_after(X, None) is called
    Then the order is: X, A

  Scenario: insert_after on an empty collection inserts the key as the only element
    Given an empty OrderedCollection
    When insert_after(X, None) is called
    Then the order is: X

  # ─── STARRING: ALL STORED, NO UNSTORED ─────────────────────────────────────

  Scenario: Star a stored tag — no unstored tags, no existing starred tags
    Given the tag list has only stored unstarred tags in display order:
      | name  | stored | starred |
      | alpha | yes    | no      |
      | beta  | yes    | no      |
      | gamma | yes    | no      |
    When the user stars "gamma"
    Then the display order is: "gamma", "alpha", "beta"
    And "gamma" is at position 0 in display_tag_ids
    And "gamma" is at position 0 in selected_tag_ids
    And "gamma" has starred = true
    And the three-section invariant holds

  Scenario: Star a second stored tag — it goes before the first starred tag
    Given the tag list has:
      | name  | stored | starred |
      | beta  | yes    | yes     |
      | alpha | yes    | no      |
      | gamma | yes    | no      |
    When the user stars "gamma"
    Then the display order is: "gamma", "beta", "alpha"
    And "gamma" is at position 0 in display_tag_ids
    And "gamma" is at position 0 in selected_tag_ids
    And the three-section invariant holds

  Scenario: Star a third stored tag — it becomes the new first starred tag
    Given the display order is: "beta", "delta", "alpha", "gamma"
    And "beta" and "delta" are stored and starred; "alpha" and "gamma" are stored and unstarred
    When the user stars "alpha"
    Then the display order is: "alpha", "beta", "delta", "gamma"
    And the three-section invariant holds

  # ─── STARRING: WITH UNSTORED TAGS ──────────────────────────────────────────

  Scenario: Star a stored tag — unstored tags exist and remain in section 1
    Given the tag list has:
      | name    | stored | starred |
      | snap1   | no     | no      |
      | snap2   | no     | no      |
      | alpha   | yes    | no      |
      | beta    | yes    | no      |
    When the user stars "beta"
    Then the display order is: "snap1", "snap2", "beta", "alpha"
    And "snap1" and "snap2" remain in section 1
    And "beta" is at the start of section 2 (immediately after "snap2")
    And the three-section invariant holds

  Scenario: Star a second stored tag — unstored tags still precede both starred tags
    Given the display order is: "snap1", "beta", "alpha", "gamma"
    And "snap1" is unstored; "beta" is stored and starred; "alpha" and "gamma" are stored and unstarred
    When the user stars "gamma"
    Then the display order is: "snap1", "gamma", "beta", "alpha"
    And "snap1" remains in section 1
    And "gamma" and "beta" are both in section 2
    And the three-section invariant holds

  Scenario: Star a stored tag — only unstored tags exist alongside it
    Given the tag list has:
      | name  | stored | starred |
      | snap1 | no     | no      |
      | alpha | yes    | no      |
    When the user stars "alpha"
    Then the display order is: "snap1", "alpha"
    And "alpha" is immediately after "snap1"
    And the three-section invariant holds

  # ─── UNSTARRING: ALL STORED ─────────────────────────────────────────────────

  Scenario: Unstar a stored tag — other starred tags remain in section 2
    Given the display order is: "beta", "gamma", "alpha", "delta"
    And "beta" and "gamma" are stored and starred; "alpha" and "delta" are stored and unstarred
    When the user unstars "beta"
    Then the display order is: "gamma", "beta", "alpha", "delta"
    And "beta" is at the start of section 3 (immediately after "gamma")
    And "gamma" remains in section 2
    And "beta" has starred = false
    And the three-section invariant holds

  Scenario: Unstar the last remaining starred tag — it goes to position 0 (section 3 starts at top)
    Given the display order is: "gamma", "alpha", "beta"
    And only "gamma" is stored and starred; "alpha" and "beta" are stored and unstarred
    When the user unstars "gamma"
    Then the display order is: "gamma", "alpha", "beta"
    And "gamma" is at position 0
    And no tags are starred
    And the three-section invariant holds

  Scenario: Unstar the last starred tag when it is the last element in the list
    Given the display order is: "alpha", "beta", "gamma"
    And "alpha" and "beta" are stored and unstarred; "gamma" is stored and starred
    When the user unstars "gamma"
    Then the display order is: "gamma", "alpha", "beta"
    And "gamma" is at position 0
    And the three-section invariant holds

  # ─── UNSTARRING: WITH UNSTORED TAGS ─────────────────────────────────────────

  Scenario: Unstar a stored tag — unstored tags stay in section 1, tag goes to start of section 3
    Given the display order is: "snap1", "beta", "gamma", "alpha"
    And "snap1" is unstored; "beta" and "gamma" are stored and starred; "alpha" is stored and unstarred
    When the user unstars "beta"
    Then the display order is: "snap1", "gamma", "beta", "alpha"
    And "snap1" remains in section 1
    And "gamma" remains in section 2
    And "beta" is now in section 3 immediately after "gamma"
    And the three-section invariant holds

  Scenario: Unstar the last starred tag — tag goes right after last unstored tag, not to position 0
    Given the display order is: "snap1", "snap2", "gamma", "alpha", "beta"
    And "snap1" and "snap2" are unstored; "gamma" is the only starred stored tag; "alpha" and "beta" are unstarred stored
    When the user unstars "gamma"
    Then the display order is: "snap1", "snap2", "gamma", "alpha", "beta"
    And "gamma" is immediately after "snap2"
    And no tags are starred
    And the three-section invariant holds

  # ─── CANNOT STAR AN UNSTORED TAG ────────────────────────────────────────────

  Scenario: Attempting to star an unstored tag is rejected
    Given the tag list has:
      | name  | stored | starred |
      | snap1 | no     | no      |
      | alpha | yes    | no      |
    When the user attempts to star "snap1"
    Then the operation returns error CannotStarUnstoredTag
    And the display order is unchanged
    And no database write occurs

  Scenario: Star button is not shown for unstored tags in the tag panel
    Given "snap1" is an unstored tag
    When the tag panel is rendered
    Then no star button appears on the "snap1" row

  Scenario: Star overlay is not shown for unstored tags in the tag grid
    Given "snap1" is an unstored tag
    When the tag grid is rendered
    Then no star overlay appears on the "snap1" cell

  # ─── IDEMPOTENCY ─────────────────────────────────────────────────────────────

  Scenario: Starring an already-starred tag is a no-op
    Given the display order is: "gamma", "alpha", "beta"
    And "gamma" is already starred
    When the user stars "gamma" again
    Then the display order is unchanged: "gamma", "alpha", "beta"
    And no database write occurs

  Scenario: Unstarring an already-unstarred tag is a no-op
    Given the display order is: "gamma", "alpha", "beta"
    And "alpha" is not starred
    When the user unstars "alpha"
    Then the display order is unchanged: "gamma", "alpha", "beta"
    And no database write occurs

  # ─── MIRROR: BOTH COLLECTIONS STAY IN SYNC ──────────────────────────────────

  Scenario: Starring mirrors the move in selected_tag_ids using the same anchor TagId
    Given the tag list has:
      | name  | stored | starred |
      | snap1 | no     | no      |
      | alpha | yes    | yes     |
      | beta  | yes    | no      |
    When the user stars "beta"
    Then display_tag_ids order is:  "snap1", "beta", "alpha"
    And selected_tag_ids order is:  "snap1", "beta", "alpha"
    And the selected_tag_ids order key for "beta" was independently computed, not copied from display_tag_ids

  Scenario: Unstarring mirrors the move in selected_tag_ids using the same anchor TagId
    Given the display order is: "snap1", "beta", "alpha", "gamma"
    And "snap1" is unstored; "beta" and "alpha" are stored and starred; "gamma" is stored and unstarred
    When the user unstars "beta"
    Then display_tag_ids order is:  "snap1", "alpha", "beta", "gamma"
    And selected_tag_ids order is:  "snap1", "alpha", "beta", "gamma"

  # ─── THREE-SECTION INVARIANT: SEQUENCE OF OPERATIONS ────────────────────────

  Scenario: Invariant holds after a sequence of mixed star and unstar operations
    Given the tag list has:
      | name  | stored | starred |
      | snap1 | no     | no      |
      | alpha | yes    | no      |
      | beta  | yes    | no      |
      | gamma | yes    | no      |
      | delta | yes    | no      |
    When the following operations are performed in order:
      | op    | tag   |
      | star  | delta |
      | star  | alpha |
      | star  | gamma |
      | unstar| alpha |
      | star  | beta  |
      | unstar| gamma |
      | unstar| delta |
    Then the three-section invariant holds for both display_tag_ids and selected_tag_ids

  Scenario: Invariant holds when all tags are starred then all unstarred
    Given the tag list has stored unstarred tags: "alpha", "beta", "gamma"
    When "alpha", "beta", "gamma" are starred in that order
    And "gamma", "beta", "alpha" are unstarred in that order
    Then the three-section invariant holds
    And no tags are starred

  Scenario: Invariant holds when unstored tags are mixed with star/unstar operations
    Given the tag list has:
      | name  | stored | starred |
      | snap1 | no     | no      |
      | snap2 | no     | no      |
      | alpha | yes    | no      |
      | beta  | yes    | no      |
    When "alpha" is starred
    And "beta" is starred
    And "alpha" is unstarred
    Then the display order is: "snap1", "snap2", "beta", "alpha"
    And "snap1" and "snap2" are in section 1
    And "beta" is in section 2
    And "alpha" is in section 3
    And the three-section invariant holds

  # ─── PERSISTENCE ─────────────────────────────────────────────────────────────

  Scenario: Starring persists starred=true via save_tag
    Given "alpha" is a stored unstarred tag
    When the user stars "alpha"
    Then save_tag is called for "alpha" with starred = true

  Scenario: Starring persists the new display order via update_tag_orders
    Given the tag list has stored unstarred tags: "alpha", "beta", "gamma"
    When the user stars "gamma"
    Then update_tag_orders is called with "gamma" at a sort_order smaller than "alpha" and "beta"

  Scenario: Unstarring persists starred=false via save_tag
    Given "alpha" is a stored starred tag
    When the user unstars "alpha"
    Then save_tag is called for "alpha" with starred = false

  Scenario: Unstarring persists the new display order via update_tag_orders
    Given the display order is: "beta", "alpha", "gamma" with "beta" and "alpha" starred
    When the user unstars "beta"
    Then update_tag_orders is called with "beta" positioned after "alpha" in sort_order

  Scenario: No database write occurs when trying to star an unstored tag
    Given "snap1" is an unstored tag
    When the user attempts to star "snap1"
    Then save_tag is NOT called
    And update_tag_orders is NOT called

  # ─── APP STARTUP: RESTORING STATE FROM DATABASE ──────────────────────────────

  Scenario: Starred tags are loaded before unstarred tags on startup
    Given the database contains:
      | name  | starred | sort_order |
      | alpha | false   | 30         |
      | beta  | true    | 10         |
      | gamma | true    | 20         |
    When the app starts and loads the tag list
    Then the display order is: "beta", "gamma", "alpha"
    And "beta" and "gamma" are starred
    And "alpha" is unstarred
    And the three-section invariant holds

  Scenario: Unstored tags from snapshot are inserted at section 1 on load
    Given the database contains stored tags: "alpha" (unstarred), "beta" (starred, sort_order=1)
    And the current file snapshot contains unstored tag "snap1"
    When the tag list is built for the current file
    Then the display order starts with "snap1" (section 1), then "beta" (section 2), then "alpha" (section 3)
    And the three-section invariant holds

  # ─── UI: TAG PANEL ───────────────────────────────────────────────────────────

  Scenario: Starred stored tag renders a filled star in the tag panel
    Given "gamma" is a stored starred tag
    When the tag panel is rendered
    Then the row for "gamma" shows a filled star icon (★)

  Scenario: Unstarred stored tag renders an outline star in the tag panel
    Given "alpha" is a stored unstarred tag
    When the tag panel is rendered
    Then the row for "alpha" shows an outline star icon (☆)

  Scenario: Clicking the star icon dispatches ToggleStar for the correct tag
    When the user clicks the star icon on the "beta" row in the tag panel
    Then the message ToggleStar("beta") is dispatched

  # ─── UI: TAG GRID ────────────────────────────────────────────────────────────

  Scenario: Starred stored tag shows a star overlay in the tag grid
    Given "gamma" is a stored starred tag
    When the tag grid is rendered
    Then the cell for "gamma" shows a star overlay

  Scenario: Unstarred stored tag shows no star overlay in the tag grid
    Given "alpha" is a stored unstarred tag
    When the tag grid is rendered
    Then the cell for "alpha" shows no star overlay

  Scenario: Clicking the star overlay in the tag grid dispatches ToggleStar
    When the user clicks the star overlay on the "delta" cell in the tag grid
    Then the message ToggleStar("delta") is dispatched

  # ─── UI: NO STAR IN FILE-NAME PANEL ─────────────────────────────────────────

  Scenario: File-name panel chips show no star indicator regardless of starred state
    Given "alpha" and "beta" are stored, starred, and checked
    When the file-name panel is rendered
    Then no star icon or star overlay appears on any chip

  Scenario: Starring a tag does not change the visible chip order in the file-name panel
    Given the file-name panel chip order is: "beta", "alpha"
    When the user stars "gamma" (which is unchecked)
    Then the file-name panel chip order is still: "beta", "alpha"
