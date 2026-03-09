# Design: WebDAV listing (PROPFIND) and picker API shape

## API Shape
The WebDAV list API mirrors the filesystem list API so the UI can reuse the same picker primitives:
- request: `path`, optional `cursor`, `limit`, filters (`q`, `kind`, `hide_dotfiles`, sorting)
- response: `{ path, entries, next_cursor?, total? }`

## WebDAV Listing Implementation
- Use `PROPFIND` with `Depth: 1` to list direct children.
- Parse:
  - href → child path
  - `resourcetype` → `dir` vs `file`
  - `getcontentlength` → size
  - `getlastmodified` → mtime (best-effort)
- Normalize paths so the picker can navigate consistently (`/`-style).

## Node-scoped Execution
- For `node_id=hub`: Hub performs PROPFIND directly using Hub-scoped WebDAV credentials.
- For `node_id=<agent_id>`: Hub forwards the list request to the Agent and returns the Agent’s result.

This keeps “browse what the executor can reach” aligned with restore execution.

## Error Mapping
Return stable error codes for the picker:
- `agent_offline`
- `permission_denied`
- `path_not_found`
- `not_directory`
- `invalid_cursor`

