# Design

## Filesystem Source Schema
- Replace the single `filesystem.source.root` with `filesystem.source.paths: string[]`.
- For development convenience, accept legacy `root` as an input alias and treat it as `paths=[root]` (UI will only write `paths`).

## Archive Path Mapping
Each selected source path is mapped to an **archive path prefix**:
- Normalize separators to `/`.
- Ensure archive paths are **relative** (no leading `/`) and contain no `..` segments.
- Unix absolute path `/var/log/syslog` → `var/log/syslog`
- Windows absolute path `C:\Windows\System32` → `C/Windows/System32`
- Windows UNC path `\\server\share\dir` → `UNC/server/share/dir`

For directory sources, all descendants are archived under `<archive_prefix>/<relative_to_source_dir>`.
For file sources, the file is archived at `<archive_prefix>` (no extra nesting).

## Include / Exclude Semantics
- `include`/`exclude` glob rules are matched against the **archive path**.
- If `include` is non-empty: only regular files whose archive paths match at least one include are archived.
- `exclude` applies to files and directories:
  - if a directory matches exclude, it is skipped recursively.
  - directory matching considers both `path` and `path/` for ergonomics.

## Deduplication
Deduplicate in two layers:
1) **Source selection dedupe**:
   - remove exact duplicates,
   - remove paths already covered by a selected parent directory,
   - record a single warning summary (sample-limited).
2) **Archive path dedupe (safety net)**:
   - when two different sources still map to the same archive path, keep the first and warn (sample-limited).

## Restore: Browse + Partial Restore
- Expose the entries index to the UI for browsing archived paths.
- Allow starting a restore with an optional selection filter:
  - Selecting a **file** restores only that path.
  - Selecting a **directory** restores the entire subtree (`dir/**` by prefix match).
- Restore still streams the tar.zst; non-selected entries are skipped.

## Node-Scoped Filesystem Browsing
- Add a node-scoped API: `GET /api/nodes/:node_id/fs/list?path=...`
- For `node_id=hub`: list local filesystem directly.
- For Agent nodes:
  - forward the request to the Agent over the existing websocket control channel,
  - await a response with a short timeout,
  - if the Agent is offline, return a clear error suitable for UI inline display.

