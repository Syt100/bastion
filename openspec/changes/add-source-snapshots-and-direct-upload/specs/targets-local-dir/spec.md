## MODIFIED Requirements

### Requirement: Raw-Tree Can Avoid Duplicate Local Staging (LocalDir)
For `raw_tree_v1` with `local_dir` targets, the system SHALL support a mode that avoids duplicating the staged data tree locally by writing raw-tree data directly into the target run directory before the completion marker is written.

#### Scenario: Direct data path writes into target before completion
- **WHEN** a raw-tree run is configured with direct-to-target mode
- **THEN** raw-tree payload files appear under the target run directory during packaging
- **AND** the `complete` marker is written last

