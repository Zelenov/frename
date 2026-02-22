Feature: Starred tags — pin tags to the top of the display list

  Starring a tag moves it to the very top of the tag panel and tag grid display order.
  Unstarring moves it to immediately after the last remaining starred tag.
  The order invariant is: all starred tags always appear before all unstarred tags.
  The starred flag and the resulting display order are both persisted to the database.
  The star indicator is shown in the tag panel row and tag grid cell,
  but NOT in the file-name panel (chips panel).

  Background:
    Given a tag list with the following stored tags in display order:
      | name    | starred |
      | alpha   | false   |
      | beta    | false   |
      | gamma   | false   |
      | delta   | false   |

  # ─── STARRING ───────────────────────────────────────────────────────────────

  Scenario: Starring a tag moves it to position 0 in the display list
    When the user stars "gamma"
    Then the display order is: "gamma", "alpha", "beta", "delta"
    And "gamma" has starred = true

  Scenario: Starring a second tag places it above the previously starred tag
    Given the user has already starred "gamma"
    When the user stars "beta"
    Then the display order is: "beta", "gamma", "alpha", "delta"
    And "beta" has starred = true
    And "gamma" has starred = true

  Scenario: Starring a third tag places it at position 0 above all starred tags
    Given the user has already starred "gamma" then "beta"
    When the user stars "alpha"
    Then the display order is: "alpha", "beta", "gamma", "delta"
    And "alpha", "beta", "gamma" all have starred = true

  Scenario: Starring a tag that is already at position 0 leaves the order unchanged
    Given the display order is: "alpha", "beta", "gamma", "delta"
    And "alpha" is already starred
    When the user stars "alpha" again
    Then the display order is: "alpha", "beta", "gamma", "delta"

  Scenario: Starring saves the starred flag to the database
    When the user stars "delta"
    Then save_tag is called for "delta" with starred = true

  Scenario: Starring persists the new display order to the database
    When the user stars "delta"
    Then update_tag_orders is called with "delta" at a sort_order smaller than "alpha"

  # ─── UNSTARRING ─────────────────────────────────────────────────────────────

  Scenario: Unstarring moves the tag to immediately after the last starred tag
    Given the display order is: "beta", "gamma", "alpha", "delta"
    And "beta" and "gamma" are starred; "alpha" and "delta" are unstarred
    When the user unstars "beta"
    Then the display order is: "gamma", "beta", "alpha", "delta"
    And "beta" has starred = false
    And "gamma" has starred = true

  Scenario: Unstarring the only starred tag moves it to position 0
    Given the display order is: "gamma", "alpha", "beta", "delta"
    And only "gamma" is starred
    When the user unstars "gamma"
    Then the display order starts with "gamma" at position 0
    And no tags are starred
    And "gamma" has starred = false

  Scenario: Unstarring a tag that is already at the boundary position leaves order unchanged
    Given the display order is: "gamma", "alpha", "beta", "delta"
    And only "gamma" is starred
    When the user unstars "gamma"
    Then "gamma" is at position 0
    And the display order of unstarred tags is preserved: "alpha", "beta", "delta"

  Scenario: Unstarring saves the starred flag to the database
    Given only "gamma" is starred
    When the user unstars "gamma"
    Then save_tag is called for "gamma" with starred = false

  Scenario: Unstarring persists the new display order to the database
    Given the display order is: "beta", "gamma", "alpha", "delta"
    And "beta" and "gamma" are starred
    When the user unstars "beta"
    Then update_tag_orders is called reflecting "beta" positioned after "gamma"

  # ─── ORDER INVARIANT ────────────────────────────────────────────────────────

  Scenario: After starring, no unstarred tag appears before a starred tag
    When the user stars "delta"
    Then every starred tag in the display list comes before every unstarred tag

  Scenario: After unstarring, no unstarred tag appears before a starred tag
    Given "beta" and "delta" are starred
    When the user unstars "beta"
    Then every starred tag in the display list comes before every unstarred tag

  Scenario Outline: Invariant holds after any sequence of star/unstar operations
    Given the initial display order is: "alpha", "beta", "gamma", "delta"
    When the following operations are performed in order:
      | operation | tag   |
      | star      | delta |
      | star      | alpha |
      | unstar    | delta |
      | star      | beta  |
      | unstar    | alpha |
    Then every starred tag in the display list comes before every unstarred tag

  # ─── DISPLAY: TAG PANEL ─────────────────────────────────────────────────────

  Scenario: Tag panel row shows a filled star for starred tags
    Given "gamma" is starred
    When the tag panel is rendered
    Then the row for "gamma" shows a filled star icon (★)

  Scenario: Tag panel row shows an outline star for unstarred tags
    Given no tags are starred
    When the tag panel is rendered
    Then every row shows an outline star icon (☆)

  Scenario: Clicking the star icon in the tag panel row dispatches ToggleStar
    When the user clicks the star icon on the "beta" row in the tag panel
    Then the message ToggleStar("beta") is dispatched

  # ─── DISPLAY: TAG GRID ──────────────────────────────────────────────────────

  Scenario: Tag grid cell shows a star overlay for starred tags
    Given "gamma" is starred
    When the tag grid is rendered
    Then the cell for "gamma" shows a star overlay

  Scenario: Tag grid cell has no star overlay for unstarred tags
    Given no tags are starred
    When the tag grid is rendered
    Then no cell shows a star overlay

  Scenario: Clicking the star overlay in the tag grid dispatches ToggleStar
    When the user clicks the star overlay on the "delta" cell in the tag grid
    Then the message ToggleStar("delta") is dispatched

  # ─── NO STAR IN FILE-NAME PANEL ─────────────────────────────────────────────

  Scenario: The file-name panel chips do not show a star indicator
    Given "alpha" and "beta" are starred and checked
    When the file-name panel is rendered
    Then no star icon or overlay appears on any chip

  Scenario: Starring a tag does not change the chip order in the file-name panel
    Given the file-name panel chip order is: "beta", "alpha"
    When the user stars "gamma"
    Then the file-name panel chip order is still: "beta", "alpha"

  # ─── PERSISTENCE ON LOAD ────────────────────────────────────────────────────

  Scenario: Starred flags are restored from the database on app startup
    Given the database contains "gamma" with starred = true and "beta" with starred = true
    And the database sort_order places "gamma" before "beta", both before "alpha" and "delta"
    When the app starts and loads the tag list
    Then the display order is: "gamma", "beta", "alpha", "delta"
    And "gamma" and "beta" have starred = true
    And the order invariant holds

  Scenario: Unstarred tags are loaded in their stored sort_order after all starred tags
    Given the database contains "delta" with starred = true (sort_order = 1)
    And "alpha", "beta", "gamma" with starred = false (sort_order = 2, 3, 4)
    When the app starts and loads the tag list
    Then the display order is: "delta", "alpha", "beta", "gamma"
    And the order invariant holds
