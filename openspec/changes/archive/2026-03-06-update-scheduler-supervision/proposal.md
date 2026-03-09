# Change: Supervise critical background tasks and fail fast on unexpected panics

## Why
The Hub spawns multiple long-running background loops (scheduler, notifications, bulk operations, maintenance). If one of these tasks panics, it can silently stop working while the HTTP server continues running, which is hard to diagnose and can lead to partial outages.

## What Changes
- Introduce a lightweight supervision pattern for critical long-running background tasks.
- When a supervised task panics unexpectedly, log a clear error and trigger graceful shutdown (via the shared `CancellationToken`).
- Apply supervision to critical loops in the scheduler and other engine background workers.

## Impact
- Affected specs: `backend`, `observability`
- Affected code: engine/task spawning entrypoints (`crates/bastion-engine/src/scheduler/mod.rs`, `crates/bastion-engine/src/notifications/loop.rs`, `crates/bastion-engine/src/bulk_operations.rs`, `crates/bastion-engine/src/maintenance.rs`)

