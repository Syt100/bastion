## 1. Spec
- [x] 1.1 Add spec deltas for: SQLite busy timeout + pool tuning; scheduler/notifications sleep-until; static asset caching/ETag; request-id observability; agent reconnect/heartbeat expectations
- [x] 1.2 Run `openspec validate update-backend-runtime-optimizations --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Backend - SQLite busy timeout + pool tuning
- [x] 2.1 Add SQLite busy timeout and explicit pool options (max/min connections, acquire timeout)
- [x] 2.2 Add/adjust indexes for any newly-identified hot query paths (only if needed)
- [x] 2.3 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [x] 2.4 Commit SQLite runtime optimization changes (detailed message)

## 3. Backend - Scheduler cron sleep-until
- [x] 3.1 Cache cron parsing where feasible and compute the next trigger time per job
- [x] 3.2 Replace fixed-interval polling with `sleep_until` to the next trigger time (with shutdown support)
- [x] 3.3 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [x] 3.4 Commit scheduler runtime optimization changes (detailed message)

## 4. Backend - Notifications sleep-until next due
- [x] 4.1 Add a DB query helper to fetch the next due timestamp (if any)
- [x] 4.2 Replace 1s polling with `sleep_until(next_due)` and a wakeup when notifications are enqueued
- [x] 4.3 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [x] 4.4 Commit notification runtime optimization changes (detailed message)

## 5. Backend - HTTP static assets caching + streaming
- [x] 5.1 Serve UI assets via streaming in non-embed mode
- [x] 5.2 Add cache headers and ETag support; keep SPA fallback behavior correct
- [x] 5.3 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [x] 5.4 Commit HTTP static asset optimization changes (detailed message)

## 6. Observability - Request ID + spans
- [x] 6.1 Add request-id generation + propagation on HTTP responses
- [x] 6.2 Correlate key logs/spans with request/run/job identifiers without increasing log noise
- [x] 6.3 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [x] 6.4 Commit request-id/observability changes (detailed message)

## 7. Hub/Agent - Reconnect and heartbeat reliability
- [x] 7.1 Add reconnect backoff jitter to avoid synchronized reconnect storms
- [x] 7.2 Add clearer pong timeout behavior for heartbeat failures
- [x] 7.3 Clarify ACK/retry boundaries to reduce duplicate task work across reconnects
- [x] 7.4 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [x] 7.5 Commit agent reliability changes (detailed message)

## 8. Quality gate
- [x] 8.1 Final verification: `cargo fmt`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
