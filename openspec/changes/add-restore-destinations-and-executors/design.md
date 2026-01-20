# Design: Restore Executors, Destinations, and Hub Relay

## Goals
- Support arbitrary combinations of:
  - backup target (where run artifacts live) and
  - restore destination (where restored files are written),
  - with restore execution on Hub or Agents.
- Keep the Hub as the control-plane authority:
  - the Hub owns operation lifecycle, audit, and UI-facing events,
  - Agents execute restore work when required (local fs destination).

## Restore Model

### Executor
The executor is the node that performs the restore engine:
- `hub`
- an `agent_id`

Rules:
- If destination is `local_fs` on an Agent, executor MUST be that Agent.
- If destination is `local_fs` on Hub, executor MUST be Hub.
- If destination is `webdav`, executor MAY be Hub or an Agent (selected by UI / defaults to Hub).

### Destination types
1) `local_fs`:
   - `node_id` (hub/agent)
   - `directory` (absolute path on that node)
2) `webdav`:
   - `base_url`, `secret_name`
   - `prefix` (directory-like path under `base_url`, user-selected)
   - `.bastion-meta/` sidecar is created under the prefix for metadata preservation

## Hub Relay
When an Agent executor needs artifacts not available locally (e.g. artifacts stored on Hub-local LocalDir, or on WebDAV reachable from Hub, or on another Agent-local LocalDir), the Hub relays data:

1) Agent requests an artifact byte stream from Hub (by logical artifact name and offset/range).
2) Hub opens the upstream artifact source:
   - local filesystem on Hub
   - WebDAV client
   - remote Agent-local filesystem via the same protocol (Agent → Hub streaming)
3) Hub streams bytes to the executor Agent with flow control.

This design avoids any Agent↔Agent direct connections and keeps security boundaries centralized.

## Protocol Shape (high level)
The protocol adds:
- Restore task dispatch: Hub → Agent
- Operation events/results: Agent → Hub
- Artifact stream RPC: bidirectional streaming over the existing WS connection

Flow control MUST ensure bounded memory on both ends (pull-based reads or windowed push).

## Metadata Preservation for WebDAV Destinations
When restoring to WebDAV, the destination cannot faithfully represent POSIX metadata. To preserve round-trip fidelity, the restore writes:
- `<prefix>/.bastion-meta/restore/<op_id>/...` metadata artifacts

These artifacts are used when restoring from WebDAV back to a local filesystem later.

