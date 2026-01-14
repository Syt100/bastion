# Change: Bulk Distribute WebDAV Credentials to Nodes

## Why
WebDAV targets are commonly shared across multiple agents.
Today, WebDAV credentials are managed per node, which makes it tedious and error-prone to keep dozens of nodes consistent.

This change introduces a safe bulk distribution flow that:
- Targets nodes via label selectors.
- Defaults to **skip** when the credential already exists on a node.
- Supports an explicit “overwrite” option when needed.
- Provides a preview before execution.

## What Changes
- Backend:
  - Add a bulk operation action to distribute a WebDAV secret to selected nodes.
  - Re-encrypt secrets per node when copying.
  - Record per-node outcomes (skipped/updated/failed).
  - Ensure affected nodes receive updated config snapshots (or are marked pending if offline).
- Web UI:
  - Add an operator flow (Hub context) to select a WebDAV credential and distribute it to nodes selected by labels.
  - Provide a preview list of impacted nodes (will skip / will update).
  - Reuse the bulk operations panel for execution status and per-node errors.

## Dependencies
- Uses the bulk operations framework from `add-bulk-operations-framework`.
- Uses agent labels from `add-agent-labels`.

## Decisions
- If the credential already exists on a target node, the default behavior is **skip**.
- Overwrite requires explicit operator selection.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code (expected): `crates/bastion-http` (bulk action + secrets), `crates/bastion-storage` (bulk records), `ui` (storage UI + bulk panel)

