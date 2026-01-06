## MODIFIED Requirements

### Requirement: Job Editor Wizard
The Web UI SHALL support creating and editing filesystem jobs with multi-path sources (files and directories) and SHALL provide a browse UI to pick paths from the selected node.

#### Scenario: Pick multiple filesystem sources
- **WHEN** the user edits a filesystem job and selects multiple source paths
- **THEN** the job spec is saved with all selected paths

### Requirement: Restore Wizard
The Web UI SHALL allow browsing the archived paths of a completed run and selecting a subset of files/directories to restore.

#### Scenario: Restore a directory subtree
- **WHEN** the user selects an archived directory in the restore wizard
- **THEN** the restore operation restores that directory and all of its descendants

