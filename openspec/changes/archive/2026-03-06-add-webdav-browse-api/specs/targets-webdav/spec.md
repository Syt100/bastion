## ADDED Requirements

### Requirement: WebDAV Client Supports Directory Listing via PROPFIND
The WebDAV client helpers SHALL support listing a directory via PROPFIND (Depth: 1) and return normalized entries suitable for picker UIs.

#### Scenario: List direct children
- **WHEN** a WebDAV directory is listed
- **THEN** the client returns direct child entries with name, kind (dir/file), and best-effort size/mtime

