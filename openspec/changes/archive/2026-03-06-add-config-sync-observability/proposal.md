# Change: Add Config Sync Observability (Desired vs Applied)

## Why
Today, the Hub sends agent config snapshots opportunistically (on agent hello and on certain updates), and agents can ACK snapshots.
However, operators cannot answer basic questions from the UI:
- Which nodes are currently out of sync with the desired configuration?
- Which nodes are offline and therefore pending config delivery?
- What was the last sync error for a node?

This change makes configuration delivery observable and operable by persisting “desired vs applied” state and surfacing it in the UI.

## What Changes
- Storage:
  - Persist per-agent config sync state:
    - `desired_config_snapshot_id`
    - `last_applied_config_snapshot_id` (from `ConfigAck`)
    - timestamps + last error summary
- Backend:
  - Update agent websocket handling to persist `ConfigAck` state.
  - When the Hub computes/sends a config snapshot for a node, update that node’s desired snapshot id.
  - Expose sync state in agent list/detail APIs.
  - Provide an operator action “sync config now” (single-node and bulk, via bulk ops).
  - Offline nodes MUST be handled gracefully: mark as pending and deliver on reconnect.
- Web UI:
  - Show config sync status on Agents page (synced / pending / error / offline).
  - Show detailed desired/applied snapshot ids and last error in agent details.
  - Provide “sync now” action (single and bulk).

## Dependencies
- Uses the bulk operations framework from `add-bulk-operations-framework` for the bulk “sync now” operation.

## Decisions
- Sync-now bulk operation continues on failures and records per-node outcomes.
- If a node is offline, the system records that it is pending delivery; the desired snapshot id is still updated so delivery happens automatically when the node reconnects.

## Impact
- Affected specs: `backend`, `web-ui`
- Affected code (expected): `crates/bastion-http` (agent ws + agents APIs), `crates/bastion-storage` (schema + repo), `ui` (Agents page/status + bulk integration)

