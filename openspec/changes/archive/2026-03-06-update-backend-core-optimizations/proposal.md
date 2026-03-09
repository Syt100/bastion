# Change: Update backend core optimizations (WS events, DB maintenance, modularity, scheduler)

## Why
The backend currently works well for MVP usage, but several internal patterns will become bottlenecks as usage grows:
- WebSocket run event streaming currently polls SQLite frequently per connection, which scales poorly and adds unnecessary DB load.
- Some SQLite tables (sessions, enrollment tokens, login throttling) can grow without automated cleanup.
- The HTTP layer is implemented as a large monolithic module, making it harder to evolve safely.
- The scheduler relies on short-interval polling even when idle and lacks a clean shutdown path.
- We want stronger automated coverage around these runtime behaviors to reduce regressions.

## What Changes
- Replace per-connection DB polling for run events with an in-process push model (broadcast) with DB catch-up for reconnects.
- Add periodic database maintenance to prune expired/old rows and add missing indexes for hot queries.
- Refactor the HTTP module into smaller modules (routes/handlers/middleware/types) while keeping behavior/API compatible.
- Reduce scheduler idle polling by using explicit wakeups and add graceful shutdown for background tasks.
- Add/extend tests and CI checks to validate these behaviors.

## Impact
- Affected specs: `backend`
- Affected code:
  - `crates/bastion/src/http/*` (WS + routing structure)
  - `crates/bastion/src/runs_repo.rs` (event append + queries)
  - `crates/bastion/src/scheduler.rs` (wakeups + shutdown)
  - `crates/bastion/src/auth.rs` (session maintenance)
  - `crates/bastion/migrations/*` (new indexes)
  - backend tests

## Compatibility / Non-Goals
- No API route changes and no JSON schema changes for existing clients.
- No changes to authentication/authorization model beyond maintenance cleanup.
- No changes to backup formats or target protocols.

