## MODIFIED Requirements

### Requirement: Filesystem Source with Filters
The system SHALL support filesystem backups whose source is a set of selected paths (files and directories) and SHALL support include/exclude filtering against the archive path.

#### Scenario: Multi-path source includes files and directories
- **WHEN** a filesystem job is configured with multiple `filesystem.source.paths`
- **THEN** files and directories from all selected paths are included in the tar archive
- **AND** archive paths preserve the original path structure (as defined by the archive path mapping)

#### Scenario: Include/exclude rules match archive paths
- **WHEN** a filesystem source has include/exclude rules
- **THEN** rules are evaluated against the archive path (tar-internal path) of each entry

#### Scenario: Overlapping selections are deduplicated
- **WHEN** a filesystem source selects a directory and also selects a descendant path under that directory
- **THEN** the system deduplicates the selection
- **AND** records a warning summary describing what was deduplicated

