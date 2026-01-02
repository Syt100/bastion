## 1. Spec
- [ ] 1.1 Add spec deltas for: SQLite busy timeout + pool tuning; scheduler/notifications sleep-until; static asset caching/ETag; request-id observability; agent reconnect/heartbeat expectations
- [ ] 1.2 Run `openspec validate update-backend-runtime-optimizations --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - SQLite busy timeout + pool tuning
- [ ] 2.1 Add SQLite busy timeout and explicit pool options (max/min connections, acquire timeout)
- [ ] 2.2 Add/adjust indexes for any newly-identified hot query paths (only if needed)
- [ ] 2.3 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] 2.4 Commit SQLite runtime optimization changes (detailed message)

## 3. Backend - Scheduler cron sleep-until
- [ ] 3.1 Cache cron parsing where feasible and compute the next trigger time per job
- [ ] 3.2 Replace fixed-interval polling with `sleep_until` to the next trigger time (with shutdown support)
- [ ] 3.3 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] 3.4 Commit scheduler runtime optimization changes (detailed message)

## 4. Backend - Notifications sleep-until next due
- [ ] 4.1 Add a DB query helper to fetch the next due timestamp (if any)
- [ ] 4.2 Replace 1s polling with `sleep_until(next_due)` and a wakeup when notifications are enqueued
- [ ] 4.3 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] 4.4 Commit notification runtime optimization changes (detailed message)

## 5. Backend - HTTP static assets caching + streaming
- [ ] 5.1 Serve UI assets via streaming in non-embed mode
- [ ] 5.2 Add cache headers and ETag support; keep SPA fallback behavior correct
- [ ] 5.3 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] 5.4 Commit HTTP static asset optimization changes (detailed message)

## 6. Observability - Request ID + spans
- [ ] 6.1 Add request-id generation + propagation on HTTP responses
- [ ] 6.2 Correlate key logs/spans with request/run/job identifiers without increasing log noise
- [ ] 6.3 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] 6.4 Commit request-id/observability changes (detailed message)

## 7. Hub/Agent - Reconnect and heartbeat reliability
- [ ] 7.1 Add reconnect backoff jitter to avoid synchronized reconnect storms
- [ ] 7.2 Add clearer pong timeout behavior for heartbeat failures
- [ ] 7.3 Clarify ACK/retry boundaries to reduce duplicate task work across reconnects
- [ ] 7.4 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [ ] 7.5 Commit agent reliability changes (detailed message)

## 8. Quality gate
- [ ] 8.1 Final verification: `cargo fmt`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
