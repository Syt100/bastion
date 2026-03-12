## Context

`crates/bastion-http` currently writes `state.config.debug_errors` into a process-global atomic flag during router construction, and `AppError::from(anyhow::Error)` reads that flag later when building internal-error responses. This creates hidden coupling between routers that happen to live in the same process. Separately, `crates/bastion-backup` reads `BASTION_FS_SNAPSHOT_*` variables from inside `source_snapshot` helpers, even though the callers already know when a given task starts and can capture runtime inputs once.

## Goals / Non-Goals

**Goals:**
- Make HTTP internal-error rendering depend on request-scoped runtime options rather than process-global mutable state.
- Make filesystem snapshot resolution depend on explicit caller-provided runtime settings rather than helper-local environment reads.
- Add regression tests that validate scoped isolation and explicit-input behavior.

**Non-Goals:**
- Changing whether `debug_errors` exists or how operators configure it.
- Changing snapshot provider behavior, allowlist semantics, or user-visible policy precedence.
- Redesigning the full HTTP error handling stack.

## Decisions

- Decision: keep `debug_errors` in `AppState`, but bind it to request execution via middleware-scoped render options instead of a global atomic.
  - Rationale: handlers already carry `AppState`; middleware is the narrowest place that can associate a request with its router configuration without touching every handler.
  - Alternative considered: mapping every handler error with explicit config. Rejected because it would require wide call-site churn and be easy to regress.
- Decision: store internal debug payload inside `AppError`, and decide whether to expose it only at response-render time.
  - Rationale: classification remains local to `AppError`, while exposure policy becomes request-scoped.
- Decision: make snapshot settings capture explicit and public at the backup crate boundary, then thread the resulting value through run execution entry points.
  - Rationale: the backup helper remains pure with respect to process environment, while callers still capture current runtime settings exactly once per execution path.
  - Alternative considered: keep a convenience wrapper that still reads env inside `source_snapshot`. Rejected because it preserves the hidden-input pattern we are trying to remove.

## Risks / Trade-offs

- [Task-local or request-scoped render context can be bypassed if future code constructs responses outside the scoped path] → Centralize router middleware wiring and add regression coverage for concurrent scoped rendering.
- [Exposing snapshot settings at the crate boundary adds some call-site churn] → Keep the settings object minimal and derive `Clone` so callers can pass it into blocking work without bespoke plumbing.
- [Always collecting internal debug payloads adds small overhead on internal errors] → Internal errors are exceptional paths, and the trade-off removes hidden global behavior.

## Migration Plan

1. Introduce request-scoped HTTP error render options and remove the process-global debug flag.
2. Capture snapshot settings at filesystem execution entry points and pass them into `source_snapshot`.
3. Replace or extend regression tests to cover concurrent request scoping and explicit snapshot settings.
4. Run targeted tests, then `scripts/ci.sh`.

## Open Questions

- None.
