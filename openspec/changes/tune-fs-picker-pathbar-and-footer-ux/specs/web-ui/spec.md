## MODIFIED Requirements

### Requirement: Filesystem Picker Focus and Path Bar UX
The web UI SHALL improve the filesystem picker path bar UX:
- On open, the path input SHALL receive focus to avoid misleading focus states on toolbar buttons.
- Up/refresh actions SHOULD be displayed inline with the path input (e.g., as prefix actions) to avoid an abrupt multi-row layout on mobile.
- Icon-only path actions SHOULD use a softened visual weight (thinner stroke / less harsh appearance).
- The “Current path” label SHOULD be omitted in favor of a placeholder and/or accessibility label.

#### Scenario: Opening the picker focuses the path input
- **WHEN** the user opens the filesystem picker
- **THEN** the current-path input receives focus

#### Scenario: Mobile path bar stays visually unified
- **GIVEN** the user is on a mobile-sized screen
- **WHEN** the user views the filesystem picker path bar
- **THEN** the up/refresh actions appear inline with the path input

### Requirement: Mobile Footer Selected Count Does Not Compete with Actions
The web UI SHALL keep the filesystem picker footer usable on narrow screens:
- On desktop, the selected-count indicator MAY be shown as text on the left side of the footer.
- On mobile, the selected-count indicator SHOULD be rendered as a badge on the “Add selected” confirm action (not as separate text), so the footer remains on a single row.

#### Scenario: Mobile selected count is shown as a badge
- **GIVEN** the user is on a mobile-sized screen
- **AND** the user has selected one or more items
- **WHEN** the user views the picker footer
- **THEN** the “Add selected” action displays the selected count as a badge

