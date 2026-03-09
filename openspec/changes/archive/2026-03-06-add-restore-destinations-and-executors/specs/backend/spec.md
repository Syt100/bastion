---
## ADDED Requirements

### Requirement: Restore Supports Multiple Destinations via Sinks
The backend SHALL support restoring run contents to multiple destination backends by implementing restore sinks (local filesystem and WebDAV).

#### Scenario: WebDAV destination uses a prefix
- **WHEN** restore is started with destination type `webdav` and a `prefix`
- **THEN** the backend writes restored data under the prefix and does not overwrite sibling paths outside the prefix

### Requirement: WebDAV Restore Preserves Metadata via Sidecar
When restoring to WebDAV, the backend SHALL create a `.bastion-meta/` sidecar under the destination prefix that records metadata needed to restore back to a filesystem later.

#### Scenario: Sidecar is created
- **WHEN** restore writes to WebDAV
- **THEN** `.bastion-meta/restore/<op_id>/...` exists under the destination prefix

