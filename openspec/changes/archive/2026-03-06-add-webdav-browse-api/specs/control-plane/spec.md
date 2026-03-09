## ADDED Requirements

### Requirement: Node-Scoped WebDAV Browsing API
The system SHALL provide a node-scoped API to list WebDAV directory entries to support selecting WebDAV destination prefixes in the Web UI.

#### Scenario: List a WebDAV directory on a node
- **WHEN** the user requests `GET/POST /api/nodes/<node_id>/webdav/list` with `{ base_url, secret_name, path }`
- **THEN** the API returns the directory entries (files and subdirectories) with basic metadata

