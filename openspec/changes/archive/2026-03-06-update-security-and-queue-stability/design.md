## Context
The project already addressed Hub/Agent websocket backpressure and snapshot keyset paging. The next stability batch targets adjacent risk areas that remained after that pass: a Rust advisory from a target-specific tray dependency graph, unbounded in-process channels for offline execution paths, and OFFSET-based notification queue pagination.

## Goals / Non-Goals
- Goals:
  - Remove the dependency chain that introduces vulnerable `glib` versions into the lockfile.
  - Bound offline queue memory usage without regressing completion semantics.
  - Provide stable notifications queue pagination under concurrent status changes.
  - Improve query/index fit for keyset scans.
  - Remove UI locale switch race under rapid user toggles.
- Non-Goals:
  - Full cursor-only migration for all list endpoints.
  - Broad dashboard architecture rewrites.

## Decisions
- Decision: replace `tray-icon` with a Windows-only compatible crate (`tray-icon-win`) using dependency renaming to keep import paths stable.
  - Rationale: eliminate Linux GTK/GLib transitive graph from lockfile while keeping Windows tray code shape unchanged.
- Decision: use bounded `tokio::mpsc::channel` for offline scheduler queue and offline writer command queue.
  - Rationale: cap memory growth and preserve ordered delivery for queued commands.
- Decision: add optional opaque keyset cursor (`next_cursor`) for notifications queue listing, while preserving existing page/page_size inputs for compatibility.
  - Rationale: allow incremental rollout without breaking current clients.
- Decision: add composite indexes matching list ordering/filter patterns.
  - Rationale: reduce scan cost and keep keyset pagination latency predictable at scale.
- Decision: guard locale switch application with request sequencing and only prefetch dashboard desktop table on desktop.
  - Rationale: avoid stale async locale load overriding newer user intent and avoid unnecessary mobile prefetch work.

## Risks / Trade-offs
- Replacing tray dependency may require small API compatibility adjustments in Windows tray code.
  - Mitigation: run targeted build checks and keep dependency aliasing to preserve imports.
- Bounded channels may drop or fail writes when downstream is blocked.
  - Mitigation: explicit error handling + tests for full/closed channel behavior.
- Dual pagination mode (page + cursor) increases temporary complexity.
  - Mitigation: strict query precedence and regression tests for both paths.

## Migration Plan
1. Apply dependency graph remediation and regenerate lockfile.
2. Update offline scheduler/writer queue implementations and tests.
3. Implement notifications keyset cursor (storage + HTTP) and tests.
4. Add migration for missing composite indexes.
5. Apply UI race/perf adjustments and tests.
6. Run full checks and finalize changelog/spec tasks.

## Open Questions
- None for this batch.
