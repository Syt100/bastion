## ADDED Requirements

### Requirement: Job Editor Can Select Artifact Format
The Web UI SHALL allow selecting an artifact format for a job:
- `archive_v1` (default)
- `raw_tree_v1`

#### Scenario: Raw-tree disables encryption controls
- **WHEN** the user selects artifact format `raw_tree_v1`
- **THEN** encryption controls are disabled or hidden
- **AND** the UI explains that raw-tree does not support encryption

