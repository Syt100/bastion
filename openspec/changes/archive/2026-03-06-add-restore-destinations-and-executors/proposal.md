# Change: Add restore destinations and executors (Hub / Agent) with Hub relay

## Why
Restore is currently limited to:
- executing on the Hub, and
- restoring into a Hub-local `destination_dir`.

This prevents:
- restoring to an Agent’s local filesystem,
- restoring to WebDAV (e.g. to a chosen prefix),
- future restore targets (S3, etc.), and
- arbitrary combinations of backup target ↔ restore destination.

## What Changes
- Add a restore **executor** concept: restore runs can execute on the Hub or on a selected Agent.
- Add restore **destination** types:
  - `local_fs` (Hub or Agent local directory)
  - `webdav` (write into a user-selected WebDAV prefix)
- Use the Hub as a **relay** when the executor needs to read artifacts from another node or write to a destination not directly available from the artifact source node.
- Extend Hub↔Agent protocol to:
  - dispatch restore tasks to Agents (reconnect-safe),
  - stream run artifacts across the WebSocket connection (Hub relay),
  - report operation events and completion from Agent → Hub.
- Update the Web UI restore wizard to select:
  - destination type,
  - target node (when destination is `local_fs`),
  - destination directory / WebDAV prefix,
  - conflict policy and optional selection.
- When restoring to WebDAV, the system will create a `.bastion-meta/` sidecar directory under the destination prefix to preserve metadata for future restores.

## Impact
- Affected specs: `control-plane`, `hub-agent`, `hub-agent-protocol`, `backend`, `web-ui`
- Affected code:
  - API: `crates/bastion-http/src/http/operations.rs` (restore start request)
  - Restore orchestration: `crates/bastion-backup/src/restore/*`
  - Hub relay + agent tasks: `crates/bastion-engine/src/agent_manager/*`, `crates/bastion-engine/src/scheduler/*`, `crates/bastion-storage/src/agent_tasks_repo/*`
  - Agent task execution: `crates/bastion/src/agent_client/tasks/*`
  - UI restore flow: `ui/src/components/*restore*`, `ui/src/stores/operations.ts`

## Compatibility / Non-Goals
- WebDAV directory browsing (PROPFIND-backed picker) is handled in a separate change; this change supports manual prefix entry.
- Raw-tree (“no tar”) backup format is handled in a separate change.

