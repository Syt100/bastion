# Change: Update backend runtime optimizations (SQLite, scheduler, notifications, static assets, observability, agent reliability)

## Why
The backend is stable for MVP usage, but several runtime behaviors can be improved to reduce load and improve debuggability:
- SQLite can return transient `database is locked` failures under concurrent writes without a busy timeout and tuned pool behavior.
- The scheduler and notifications loops still rely on short-interval polling even when no work is due.
- The Hub’s Web UI static asset serving can be more efficient (streaming, cache headers, ETag) to improve UX and reduce bandwidth.
- HTTP request logs lack request correlation IDs, making production debugging harder.
- Agent reconnect/heartbeat behavior can be more resilient (jittered backoff, clearer timeout behavior).

## What Changes
- SQLite: configure busy timeout and explicit pool options; add/adjust indexes where needed for hot queries.
- Scheduler: compute next cron trigger time(s) and `sleep_until` instead of fixed polling; avoid repeated cron parsing where possible.
- Notifications: sleep until the next due notification instead of polling every second; wake promptly when new notifications are enqueued.
- Static assets: improve UI serving with streaming in non-embed mode, add cache headers (index no-cache, hashed assets long-cache) and ETag support.
- Observability: add request IDs to inbound HTTP requests and correlate logs/spans with request/run/job identifiers.
- Agent: add reconnect backoff jitter and clearer heartbeat/pong timeout behavior; clarify ACK/retry boundaries without increasing duplicate work.

## Impact
- Affected specs: `backend`, `observability`, `hub-agent`
- Affected code:
  - `crates/bastion-storage/*` (SQLite init/pool; indexes if needed)
  - `crates/bastion-engine/*` (scheduler + notifications sleeping behavior)
  - `crates/bastion-http/*` (static asset serving + request-id)
  - `crates/bastion/*` (agent reconnect/heartbeat)

## Compatibility / Non-Goals
- No breaking API route or JSON schema changes.
- No changes to backup formats or target protocols.
- No TLS changes (still delegated to reverse proxy).
