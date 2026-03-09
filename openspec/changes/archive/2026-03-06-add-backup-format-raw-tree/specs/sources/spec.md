## MODIFIED Requirements

### Requirement: Filesystem Source with Filters
The system SHALL support filesystem backups whose source is a set of selected paths (files and directories) and SHALL support include/exclude filtering against the recorded archive path.

The system SHALL support emitting filesystem backups into different artifact formats (archive vs raw-tree) while keeping path mapping consistent.

#### Scenario: Raw-tree paths match archive-path mapping
- **WHEN** a filesystem job runs with artifact format `raw_tree_v1`
- **THEN** the entries index `path` field matches the same archive-path mapping used by the tar-based format

#### Scenario: Symlink and hardlink policies are reflected in raw-tree output
- **WHEN** a filesystem job is configured with symlink/hardlink policies
- **THEN** the raw-tree output records symlink targets and hardlink grouping metadata (when applicable)
- **AND** the restore engine uses that metadata to recreate links best-effort

