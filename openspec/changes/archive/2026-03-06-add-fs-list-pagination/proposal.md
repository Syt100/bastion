# Change: Add filesystem list pagination for huge directories

## Why
The file/directory picker currently lists a directory by fetching **all** entries at once (`/api/nodes/{node_id}/fs/list?path=...`) and then filters/sorts on the client.

This breaks down when a directory contains a very large number of entries:
- response payload can be huge and slow/unreliable,
- both Hub and Agent may spend excessive CPU/memory building the full list,
- the UI may freeze due to rendering/sorting a very large table.

## What Changes
- Add server-side pagination + filtering for filesystem listing:
  - HTTP: extend `/api/nodes/{node_id}/fs/list` with `cursor` + `limit` and filter params.
  - Agent protocol: extend `fs_list` request/response to support the same pagination/filter params so the Hub does not need to pull full lists from Agents.
- Update the filesystem picker modal:
  - fetches the first page with a safe default limit,
  - shows a “加载更多” control when `next_cursor` exists,
  - applies filters by re-fetching (server-side), instead of only filtering the currently loaded rows.

## Impact
- Affected specs: `web-ui`, `hub-agent-protocol`
- Affected code:
  - `crates/bastion-http/src/http/fs.rs`
  - `crates/bastion-core/src/agent_protocol.rs`
  - `crates/bastion-engine/src/agent_manager.rs`
  - `crates/bastion/src/agent_client/fs_list.rs`
  - `crates/bastion/src/agent_client/connect/{mod.rs,handlers.rs}`
  - `crates/bastion-http/src/http/agents/ws.rs`
  - `ui/src/components/fs/FsPathPickerModal.vue`

## Compatibility / Non-Goals
- No wire-compat requirements (Hub/Agent are expected to be upgraded together), but the protocol changes are implemented as additive optional fields to keep future compatibility simpler.
- No attempt to provide a fully consistent snapshot across pages if the underlying directory changes mid-browse; paging is “best effort” and users can refresh.

