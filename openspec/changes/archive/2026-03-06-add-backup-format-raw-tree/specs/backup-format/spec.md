## ADDED Requirements

### Requirement: Artifact Format Is Recorded in the Manifest
The system SHALL record the artifact format used for a run in `manifest.json` so restore/verify can select the correct implementation.

Supported formats SHALL include:
- `archive_v1` (existing tar-based format)
- `raw_tree_v1` (raw file tree + metadata index)

#### Scenario: Restore selects implementation by manifest format
- **GIVEN** a completed run has `manifest.pipeline.format = raw_tree_v1`
- **WHEN** restore is started for that run
- **THEN** the system restores by reading `data/<path>` entries rather than attempting tar extraction

### Requirement: Raw Tree Stores Content Under a Dedicated Data Prefix
For `raw_tree_v1`, the system SHALL store file content under `data/<relative_path>` and SHALL keep run metadata files (`manifest.json`, `entries.jsonl.zst`, `complete.json`) at the run root.

#### Scenario: User file names do not collide with run metadata files
- **WHEN** the source contains a file named `manifest.json`
- **THEN** the raw-tree content is stored as `data/.../manifest.json`
- **AND** the run metadata file `manifest.json` remains at the run root

### Requirement: Entries Index Includes Best-Effort Metadata
For `raw_tree_v1`, the entries index SHALL include best-effort filesystem metadata (mtime, permissions, ownership, xattrs, symlink targets, and hardlink grouping) in addition to existing fields.

#### Scenario: Metadata is preserved for filesystem restore
- **WHEN** a raw-tree run is restored to a filesystem destination
- **THEN** the system applies recorded metadata when supported by the destination platform

