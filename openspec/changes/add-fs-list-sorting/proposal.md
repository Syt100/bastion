# Change: Add filesystem list sorting (name/mtime/size) with stable pagination

## Why
After introducing server-side pagination for huge directories, the filesystem picker effectively only supports name-based ordering.

Users need the ability to sort by:
- name,
- modified time,
- size,
while keeping the “directories first / files first” option, and while preserving stable cursor-based pagination.

## What Changes
- Extend filesystem list APIs (Hub local, Hub→Agent) to support sorting:
  - Add `sort_by` (`name|mtime|size`) and `sort_dir` (`asc|desc`) parameters.
  - Keep existing `type_sort` (`dir_first|file_first`) behavior as a separate option.
- Update cursor semantics to remain stable for the chosen sort order:
  - Cursor MUST include the primary sort key + tie-breakers so page boundaries are deterministic.
- Update the filesystem picker UI:
  - Allow choosing sort field and direction.
  - Display the current sort state.

## Impact
- Affected specs: `web-ui`, `hub-agent-protocol`
- Affected code:
  - HTTP: `crates/bastion-http/src/http/fs.rs`
  - Protocol: `crates/bastion-core/src/agent_protocol.rs`
  - Hub/Agent listing implementations and UI picker.

## Non-Goals
- Snapshot consistency across directory mutations; paging is best-effort and users can refresh.
