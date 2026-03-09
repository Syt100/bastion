---
## ADDED Requirements

### Requirement: Filesystem Picker Explains Empty and Error States
The filesystem picker SHALL clearly distinguish empty states and common error states, and provide contextual recovery actions.

#### Scenario: Empty directory vs no matches
- **GIVEN** the filesystem picker is open
- **WHEN** no filters/search are active and the directory contains no entries
- **THEN** the UI shows an “empty directory” state
- **WHEN** filters/search are active and the filtered result is empty
- **THEN** the UI shows a “no matches” state and suggests clearing filters

#### Scenario: Agent offline recovery
- **GIVEN** the user is browsing a non-Hub node
- **WHEN** the agent is offline
- **THEN** the UI shows an “agent offline” error state
- **AND** provides a “retry” action

### Requirement: Filesystem Picker Persists Per-Node Filters
The filesystem picker SHALL persist per-node filter state and restore it when the picker is opened again for the same node.

#### Scenario: Filters are restored on next open
- **GIVEN** the user applied filters in the filesystem picker for a node
- **WHEN** the user closes and reopens the picker for the same node
- **THEN** the previously applied filters are restored
