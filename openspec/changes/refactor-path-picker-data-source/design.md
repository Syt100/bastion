## Context
We need a single picker UI that can browse hierarchical "paths" across multiple backends (local filesystem, agent filesystem, WebDAV, S3) while keeping UX consistent.

## Goals
- Make the picker UI reusable across backends by introducing a data-source interface.
- Avoid feature drift by gating UI features through capabilities declared by the data source.
- Preserve current filesystem picker API and behavior.

## Non-Goals
- Adding new backend browse APIs (WebDAV/S3) in this change.
- Unifying snapshot/run-entries browsing with filesystem browsing (different domain semantics).

## Decisions
### Decision: Data source interface + capability declaration
- Introduce `PickerDataSource` with:
  - `list({ path, cursor, limit, filters, sort }) -> { path, entries, nextCursor?, total? }`
  - `normalizePath`, `parentPath`, `joinPath` (path rules belong to the data source)
  - `mapError(e) -> { kind, message }` (enables consistent empty/error states)
- Introduce `capabilities` so the UI can:
  - Hide unsupported filter controls
  - Hide unsupported sort options
  - Hide columns that have no backing fields
  - Hide pagination controls if not applicable

### Decision: Wrapper components, not a big "one picker for everything"
- Keep `FsPathPickerModal` as a wrapper that wires the FS data source.
- Keep other domain pickers (run entries) as separate data sources only if/when their semantics align.

## Risks / Trade-offs
- Risk: Over-generalization creates complexity.
  - Mitigation: Start with FS as the only data source; keep the interface minimal and extend only when a real new data source is implemented.
- Risk: Persistence keys collide across pickers.
  - Mitigation: Require an explicit `persistenceKey` derived from (dataSourceId + context).
