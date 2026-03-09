## Context
The Hub relies on multiple long-running Tokio tasks to provide scheduling, background maintenance, and notifications. These tasks are currently spawned in a detached way, so unexpected panics can terminate functionality silently.

## Goals / Non-Goals
- Goals:
  - Detect unexpected panics in critical background tasks.
  - Emit a clear log entry identifying the failed task.
  - Trigger graceful shutdown so operators can restart with confidence that the system is healthy.
- Non-Goals:
  - Restarting failed tasks in-place (we prefer fail-fast to avoid unknown partial state).
  - Changing public APIs or hub/agent protocol semantics.

## Decisions
- Decision: Wrap critical long-running tasks in a supervision helper that awaits the task join handle and cancels the shared shutdown token on panic.
- Alternatives considered:
  - `JoinSet`-based supervisor: cleaner, but requires more refactoring to thread ownership and lifetimes through multiple modules.
  - Let tasks panic and rely on process-level crash: may be acceptable in some deployments, but today a panic can be isolated to a task and leave the process alive in a degraded state.

## Risks / Trade-offs
- Fail-fast on panic will turn some latent panics into a full Hub shutdown. This is intentional; the mitigation is to treat a panic as a bug and fix it, rather than running in a degraded mode.

## Migration Plan
- Roll out supervision for the most critical loops first (scheduler/notifications/bulk/maintenance).
- Ensure tests cover the supervision helper behavior and that normal shutdown remains graceful.

## Open Questions
- Should we log an operator-facing hint (e.g., "restart required") in addition to the panic info?

