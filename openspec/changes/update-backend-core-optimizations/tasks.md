## 1. Spec
- [x] 1.1 Add `backend` spec delta for: WS push-based events + resumable catch-up, DB maintenance + indexes, HTTP modularity, scheduler wakeups + graceful shutdown, and test expectations
- [x] 1.2 Run `openspec validate update-backend-core-optimizations --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - WebSocket run events (push model)
- [ ] 2.1 Implement an in-process run events bus (bounded broadcast + idle pruning)
- [ ] 2.2 Write a single helper to append run events that also broadcasts after DB insert
- [ ] 2.3 Update the WS handler to: subscribe first, DB catch-up, live-forward, dedupe by seq, and resync on lag
- [ ] 2.4 Add/adjust tests for event ordering, reconnect catch-up, and lag resync behavior
- [ ] 2.5 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] 2.6 Commit WS event optimization changes (detailed message)

## 3. Backend - Database maintenance + indexes
- [ ] 3.1 Add a periodic DB maintenance loop to prune expired sessions/tokens and stale login_throttle entries
- [ ] 3.2 Add a migration for missing indexes on hot query paths (runs queue + retention)
- [ ] 3.3 Add/adjust tests for prune queries/migrations where feasible
- [ ] 3.4 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] 3.5 Commit DB maintenance/index changes (detailed message)

## 4. Backend - HTTP modularity refactor (no behavior change)
- [ ] 4.1 Split `crates/bastion/src/http/mod.rs` into smaller modules (routes/handlers/middleware/types/errors)
- [ ] 4.2 Keep routes/API/behavior compatible (no changes to clients)
- [ ] 4.3 Add/adjust tests if needed
- [ ] 4.4 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] 4.5 Commit HTTP refactor changes (detailed message)

## 5. Backend - Scheduler wakeups + graceful shutdown
- [ ] 5.1 Add run-enqueue wakeups for the worker loop (Notify + timeout fallback)
- [ ] 5.2 Add cancellation for scheduler/maintenance loops and wire HTTP server shutdown to trigger it
- [ ] 5.3 Add/adjust tests for shutdown/wakeup behavior where feasible
- [ ] 5.4 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] 5.5 Commit scheduler shutdown/wakeup changes (detailed message)

## 6. Quality gate
- [ ] 6.1 Ensure CI/dev workflow includes `fmt`/`clippy`/`test` for backend runtime changes
- [ ] 6.2 Final verification: `cargo fmt`, `cargo clippy`, `cargo test`
