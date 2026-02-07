## ADDED Requirements

### Requirement: Job Spec Stores WebDAV Raw-Tree Direct Upload Settings
Jobs with WebDAV targets and raw-tree pipelines SHALL persist explicit direct upload settings at the job/pipeline level.

Direct upload settings SHALL include:
- `mode: off|auto|on` (default: `off`)
- optional WebDAV request limits (concurrency + rate limits)

#### Scenario: Explicit configuration is persisted
- **WHEN** a user saves a filesystem job with `pipeline.webdav.raw_tree_direct.mode="auto"`
- **THEN** the job spec persists the mode and limits (if provided)

#### Scenario: Engine does not auto-enable without opt-in
- **WHEN** a filesystem job does not configure direct upload (`mode="off"` or missing)
- **THEN** the engine does not use WebDAV direct upload for raw-tree backups

