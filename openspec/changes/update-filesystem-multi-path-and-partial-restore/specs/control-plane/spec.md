## ADDED Requirements

### Requirement: Node-Scoped Filesystem Browsing API
The system SHALL provide a node-scoped API to list filesystem directory entries to support multi-select backup sources in the Web UI.

#### Scenario: List a directory on a node
- **WHEN** the user requests `GET /api/nodes/<node_id>/fs/list?path=<dir>`
- **THEN** the API returns the directory entries (files and subdirectories) with basic metadata

### Requirement: Run Entries Browsing API
The system SHALL provide an API to browse archived paths for a completed run to support restore selection, with optional server-side filtering suitable for a “search + filters” UI.

#### Scenario: Browse run entries by prefix
- **WHEN** the user requests `GET /api/runs/<run_id>/entries` with an optional `prefix` and pagination parameters (`cursor`, `limit`)
- **THEN** the API returns the run’s archive-path children under that prefix with pagination

#### Scenario: Filter run entries (search + kind + hide dotfiles)
- **WHEN** the user requests `GET /api/runs/<run_id>/entries` with optional filters:
  - `q` (search query; matched against the entry name for the current prefix level, case-insensitive),
  - `kind` (one of `file|dir|symlink`),
  - `hide_dotfiles` (boolean; hides names starting with `.`),
  - `min_size_bytes` / `max_size_bytes` (u64; filters file-like entries by size range; directories remain browsable),
  - `type_sort` (one of `dir_first|file_first`; symlinks are treated as files for type sorting)
- **THEN** the API returns only children that match the applied filters
- **AND** pagination (`cursor`, `limit`) is applied after filtering and sorting so results are stable

### Requirement: Partial Restore API
The system SHALL allow starting a restore operation with an optional selection of archive paths to restore.

#### Scenario: Restore only selected paths
- **WHEN** the user starts restore with a list of selected files and directories
- **THEN** only the selected archive paths (and directory subtrees) are restored to the destination directory
