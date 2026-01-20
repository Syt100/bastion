## MODIFIED Requirements

### Requirement: Partial Restore API
The system SHALL support starting a restore operation with an optional selection of archived paths to restore, and SHALL support restoring to multiple destination types.

The restore start request SHALL include:
- `destination` (typed):
  - `local_fs`: `{ node_id, directory }`
  - `webdav`: `{ base_url, secret_name, prefix }`
- `executor` (optional): `{ node_id }`
- `conflict_policy`
- optional `selection`

#### Scenario: Restore to an Agent local directory
- **GIVEN** an Agent is connected
- **WHEN** the user starts restore with destination `local_fs` on that Agent
- **THEN** the restore operation executes on that Agent and writes into the requested directory

#### Scenario: Restore to WebDAV prefix
- **WHEN** the user starts restore with destination `webdav` and a `prefix`
- **THEN** the restore operation writes restored paths under that prefix
- **AND** `.bastion-meta/` is written under the prefix to preserve metadata

