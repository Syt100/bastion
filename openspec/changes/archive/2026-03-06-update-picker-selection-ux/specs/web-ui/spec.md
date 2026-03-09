---
## ADDED Requirements

### Requirement: Picker Modals Provide Selection Helpers
The web UI picker modals SHALL provide selection helpers for efficient bulk selection in a paged table.

#### Scenario: Select a range with Shift
- **GIVEN** a picker modal displays a list of entries
- **WHEN** the user selects one row and shift-selects another row
- **THEN** all rows in the range become selected

#### Scenario: Select all loaded rows
- **GIVEN** a picker modal shows a paged listing
- **WHEN** the user clicks “Select all”
- **THEN** all currently loaded rows are selected
- **AND** the UI indicates that selection applies to loaded rows (not the entire directory)
