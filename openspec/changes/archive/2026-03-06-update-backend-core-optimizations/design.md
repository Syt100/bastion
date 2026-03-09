# Design: Backend core optimizations

## 1. WebSocket run events: push + durable catch-up

### Goals
- Reduce SQLite load by eliminating tight per-connection polling loops for run events.
- Preserve existing semantics for the Web UI:
  - Clients still receive ordered events per run.
  - Clients can reconnect and resume from the last observed `seq`.
- Handle backpressure (slow clients) safely without unbounded memory growth.

### Approach
- Keep SQLite `run_events` as the durable source of truth.
- Introduce an in-process run-event bus:
  - A per-`run_id` `tokio::sync::broadcast` channel.
  - Broadcasting occurs immediately after successfully inserting a run event into SQLite.
- WebSocket handler behavior:
  1. Subscribe to the bus for the `run_id`.
  2. Query SQLite for events `seq > after_seq` and send them first.
  3. Forward live events received from the bus, deduplicating by `seq` (`<= last_seq` is ignored).
  4. If the receiver reports lag (dropped messages), automatically resync by fetching events from SQLite after `last_seq`.

### Resource management
- Maintain a small in-memory map of `run_id -> broadcaster` and prune idle entries when:
  - `receiver_count == 0` and the entry has been unused beyond a short TTL (e.g., 10 minutes).
- Broadcast capacity is bounded (e.g., 1024). Slow clients may experience lag and trigger resync.

## 2. Database maintenance

### Goals
- Prevent unbounded growth for non-run tables that naturally expire over time.

### Approach
- Add a periodic maintenance loop that:
  - Deletes expired `sessions` (`expires_at < now`).
  - Deletes expired `enrollment_tokens` (`expires_at < now`).
  - Deletes stale `login_throttle` rows where `last_failed_at` is older than a conservative retention window.
- Add missing indexes for hot queries:
  - `runs(status, started_at)` for queue claiming and incomplete cleanup scans.
  - `runs(ended_at)` for retention pruning.

## 3. HTTP module refactor

### Goals
- Improve maintainability by splitting the monolithic HTTP module into smaller, logically grouped modules.

### Approach
- Keep the router shape and behavior unchanged.
- Move route definitions, handlers, middleware, and shared types/errors into dedicated modules.
- Avoid functional changes while refactoring; rely on tests and compiler checks.

## 4. Scheduler wakeups + graceful shutdown

### Goals
- Reduce idle DB polling and improve efficiency.
- Allow clean shutdown of background tasks when the HTTP server stops.

### Approach
- Introduce a `Notify`-style wakeup triggered when a run is enqueued.
- Worker loop:
  - Attempts to claim work; if none, waits on notify (with a long timeout as a safety net).
- Add a cancellation signal (e.g., `CancellationToken` or equivalent) shared by background tasks.
- Wire server shutdown (Ctrl-C / termination) to trigger cancellation and graceful task exit.

