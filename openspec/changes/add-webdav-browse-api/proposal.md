# Change: Add WebDAV browsing API (PROPFIND) and UI picker integration

## Why
To make WebDAV restore destinations ergonomic and less error-prone, users should be able to browse/select a WebDAV prefix (directory) instead of manually typing paths.

The Web UI already has a reusable path picker abstraction; we need a backend API that can list WebDAV directory entries (via PROPFIND) so the picker can work for WebDAV as a data source.

## What Changes
- Add a node-scoped WebDAV browsing API:
  - `GET/POST /api/nodes/{node_id}/webdav/list` (shape aligned with filesystem list responses)
  - The Hub executes the list for `node_id=hub`, or forwards the request to the Agent for `node_id=<agent_id>`.
- Implement a WebDAV directory listing client using PROPFIND (depth=1) and parse common WebDAV properties (name, resource type, size, mtime).
- Add a WebDAV picker data source in the Web UI and use it to “Browse” a destination prefix in the restore wizard.

## Impact
- Affected specs: `control-plane`, `hub-agent-protocol`, `backend`, `web-ui`, `targets-webdav`
- Affected code:
  - WebDAV client: `crates/bastion-targets/src/webdav_client.rs`
  - HTTP API: `crates/bastion-http/src/http/*`
  - Hub↔Agent forwarding (for node-scoped requests): `crates/bastion-engine/src/agent_manager/*`
  - UI: `ui/src/components/*picker*`, restore wizard

## Compatibility / Non-Goals
- This change focuses on browsing and selection. Restore-to-WebDAV execution is handled in a separate change.

