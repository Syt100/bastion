## MODIFIED Requirements

### Requirement: Filesystem Picker Header/Footer Layout Is Clear
The web UI SHALL render the filesystem file/directory picker with a clear layout:
- Up/refresh actions are icon-only and placed adjacent to the current-path input.
- The selected-count indicator is shown in the footer (left side) next to the confirm actions.
- The “Select current directory” action is located in the footer alongside the “Add selected” confirm button.

#### Scenario: Selected count is visible near confirm actions
- **GIVEN** the user has selected items in the filesystem picker
- **WHEN** the user views the picker footer
- **THEN** the footer shows the selected-count indicator on the left
- **AND** the confirm actions are grouped on the right

### Requirement: Selecting Current Directory Confirms with Optional Merge
The web UI SHALL implement “Select current directory” as a confirm-style action:
- If no items are selected, it confirms immediately and returns the current directory.
- If items are selected, it prompts the user to either:
  - select only the current directory (primary/default), or
  - select the current directory plus the already selected items.

The confirmation prompt SHALL show:
- the current directory, and
- the list of already selected items.

#### Scenario: Confirmation appears when items are already selected
- **GIVEN** the user has selected at least one item in the filesystem picker
- **WHEN** the user clicks “Select current directory”
- **THEN** the UI shows a confirmation prompt listing the current directory and the selected items
- **AND** the primary/default action is “Only select current directory”

