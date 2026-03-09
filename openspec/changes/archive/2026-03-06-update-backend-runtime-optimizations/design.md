# Design: backend runtime optimizations

## SQLite busy timeout and pool tuning
- Enable a non-zero SQLite busy timeout to reduce transient `database is locked` failures under concurrent access.
- Use explicit `SqlitePoolOptions` to avoid depending on defaults and to keep concurrency bounded.

## Scheduler (cron) sleep-until
- Replace short-interval polling with `sleep_until(next_due)` where `next_due` is computed from parsed cron schedules.
- Avoid repeated cron parsing by caching parsed schedules per unique cron string (and/or per job).
- Keep shutdown cancellation responsive.

## Notifications sleep-until
- When no notifications are immediately due, query for the earliest future `due_at` and sleep until then.
- Wake early when new notifications are enqueued (so inserting a sooner `due_at` does not require waiting for the prior sleep deadline).

## Static UI asset serving
- Non-embed mode SHOULD use a streaming file service instead of reading entire files into memory.
- Cache headers:
  - `index.html` and SPA fallback responses: `Cache-Control: no-cache` (or stricter) to ensure updates are picked up.
  - Hashed build assets (e.g., `ui/dist/assets/*`): long-lived cache (`max-age=31536000, immutable`).
- Add ETag support for conditional requests to reduce bandwidth.
- Preserve correct SPA behavior: unknown paths should return `index.html` (not prompt a download).

## Observability: request-id
- Generate a request ID for each inbound HTTP request when absent.
- Propagate it to responses using a stable header (e.g., `X-Request-Id`) and include it in request logs/spans.
- Keep logs low-noise; do not log request/response bodies or secrets.

## Agent reliability
- Add jitter to reconnect backoff to avoid reconnect storms when many agents restart simultaneously.
- Track heartbeat liveness using ping/pong timing; reconnect when pong is not observed within a configured timeout window.
- Reduce duplicate work by clarifying when tasks are resent and how duplicate deliveries are handled (ACK/resend boundaries).
