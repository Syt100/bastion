---
## ADDED Requirements

### Requirement: Filesystem Picker Supports Sorting
The web UI filesystem picker SHALL allow sorting by name, modified time, and size, while preserving stable pagination.

#### Scenario: User sorts by modified time descending
- **GIVEN** the filesystem picker is open for a directory
- **WHEN** the user chooses sorting by “modified time” in descending order
- **THEN** the picker refreshes from the first page using server-side sorting
- **AND** the displayed sort state matches the active sort selection
