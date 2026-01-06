## 1. Spec
- [x] 1.1 Add `web-ui` spec deltas for run events viewer performance (after-seq, follow mode, details on demand)
- [x] 1.2 Add `backend` spec deltas for ingest hardening (body limits, validation, upsert semantics)
- [x] 1.3 Add `hub-agent` spec deltas for deterministic snapshot IDs + deduped snapshot sending
- [x] 1.4 Run `openspec validate update-run-events-and-agent-hardening --strict`
- [x] 1.5 Commit the spec proposal (detailed message)

## 2. Web UI - Run Events Viewer
- [x] 2.1 Connect run events WebSocket with `after_seq` (avoid duplicate catch-up events)
- [x] 2.2 Add follow-mode toggle that preserves scroll position when disabled
- [x] 2.3 Optimize rendering for large event counts (virtual list / fixed-height rows)
- [x] 2.4 Show event JSON fields on demand (details modal/drawer) rather than inline for every row
- [x] 2.5 Update/extend unit tests for the viewer behavior
- [x] 2.6 Run `npm run lint --prefix ui`, `npm test --prefix ui`, `npm run build --prefix ui`
- [x] 2.7 Commit UI changes (detailed message)

## 3. Backend - Agent Run Ingest Hardening
- [x] 3.1 Add request body size limits (global default + override for ingest)
- [x] 3.2 Tighten validation for `/agent/runs/ingest` (timestamps, strings) and make runs upsert on conflict
- [x] 3.3 Add/extend unit tests for ingest behavior (validation + upsert)
- [x] 3.4 Run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- [x] 3.5 Commit backend changes (detailed message)

## 4. Hub ↔ Agent - Config Snapshot Dedupe
- [x] 4.1 Make `ConfigSnapshot.snapshot_id` deterministic for identical snapshot content
- [x] 4.2 Hub avoids re-sending unchanged snapshots to a connected Agent
- [x] 4.3 Add/extend unit tests where appropriate
- [x] 4.4 Commit hub/agent sync improvements (detailed message)

## 5. Agent - Non-Blocking Offline Run I/O
- [ ] 5.1 Ensure offline run persistence/sync does not block the async runtime hot path
- [ ] 5.2 Add/extend unit tests where appropriate
- [ ] 5.3 Commit agent I/O improvements (detailed message)

## 6. Refactor - Agent Client Modularity
- [ ] 6.1 Split Agent client code into focused modules (ws, managed cache, offline scheduler, persistence, sync)
- [ ] 6.2 Keep behavior unchanged and keep tests passing
- [ ] 6.3 Commit refactor (detailed message)

## 7. Final Validation
- [ ] 7.1 Re-run backend checks
- [ ] 7.2 Re-run UI checks
- [ ] 7.3 Mark this checklist complete
