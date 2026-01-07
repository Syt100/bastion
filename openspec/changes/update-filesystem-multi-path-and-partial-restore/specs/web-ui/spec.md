## ADDED Requirements

### Requirement: Filesystem / Archive Browser Modals
The Web UI SHALL provide reusable browser modals for:
- browsing a node’s filesystem to pick backup source paths, and
- browsing a completed run’s archive paths to pick a subset for restore.

The browser modals SHALL support:
- multi-select (files and directories),
- search and filters (type filter; hide dotfiles),
- a mobile-friendly layout.

#### Scenario: Mobile browser modal is full-screen
- **WHEN** the user opens a browser modal on a small screen
- **THEN** the modal uses a full-screen layout with scrollable content and a fixed action bar

#### Scenario: Filesystem browser remembers the last browsed directory per node
- **WHEN** the user browses directories on node `<node_id>` and closes the modal
- **AND** later re-opens the filesystem browser for the same node
- **THEN** the modal opens at the last successfully listed directory for that node (not derived from previously selected file paths)

#### Scenario: Restore browser search is explicit
- **WHEN** the user types a search query in the restore browser
- **THEN** the query is applied only when the user clicks “Search” or presses Enter
- **AND** changing filters like type / hide-dotfiles applies immediately

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
