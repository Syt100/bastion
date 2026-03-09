## MODIFIED Requirements

### Requirement: Restore Wizard
The Web UI SHALL allow selecting a restore point and restoring into different destination types, with a mobile-friendly workflow.

The restore wizard SHALL support:
- selecting a destination type:
  - local filesystem (Hub or Agent),
  - WebDAV (base_url + secret + destination prefix),
- selecting conflict strategy,
- optionally selecting a subset of paths to restore.

#### Scenario: Restore to an Agent local directory
- **WHEN** the user selects destination type "Local filesystem" and chooses Agent `<node_id>`
- **AND** selects a destination directory on that Agent
- **THEN** the restore request is executed on that Agent

#### Scenario: Restore to WebDAV writes metadata sidecar
- **WHEN** the user selects destination type "WebDAV" and chooses a prefix
- **THEN** the UI indicates that `.bastion-meta/` will be created under the prefix for metadata preservation

