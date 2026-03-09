## Context
The optimization sequence spans backend runtime behavior, storage query semantics, CI quality gates, and UI loading strategy. The work should be delivered incrementally in the same priority order to reduce risk and keep each improvement independently verifiable.

## Goals / Non-Goals
- Goals:
  - Eliminate known default-feature clippy failures in docs test code.
  - Add practical backpressure controls to agent messaging paths.
  - Ensure snapshot listing remains stable while rows mutate.
  - Reduce initial frontend payload without changing UX behavior.
  - Start targeted maintainability cleanup on high-arity handlers.
- Non-Goals:
  - Full migration of all pagination endpoints to keyset.
  - Full removal of every `too_many_arguments` allowance in the workspace.

## Decisions
- Decision: Keep docs test mutual exclusion but avoid holding a sync lock across async operations.
  - Rationale: Preserve test isolation while satisfying clippy safety rules.
- Decision: Use bounded Tokio mpsc channels for WS outboxes.
  - Rationale: Prevent unbounded memory growth under slow/broken peers while retaining async behavior.
- Decision: Throttle `last_seen_at` writes per connection.
  - Rationale: Reduce SQLite write amplification from chatty message streams.
- Decision: Convert snapshot listing to keyset cursor (`ended_at`, `run_id`).
  - Rationale: Stable iteration during state transitions (e.g., `present` -> `deleting`).
- Decision: Lazy-load locale message bundles before app mount.
  - Rationale: Remove eager inclusion of both locale dictionaries from initial chunk while preserving current locale resolution logic.
- Decision: Introduce context struct in Agent WS path for high-arity handler.
  - Rationale: Reduce argument fanout and begin incremental maintainability cleanup.

## Risks / Trade-offs
- Bounded channels can drop or block if not tuned correctly.
  - Mitigation: choose conservative capacities and handle send failures as reconnect/offline events.
- Keyset cursor migration can break clients if response shape changes unexpectedly.
  - Mitigation: keep backward compatibility where possible and add tests for cursor semantics.
- Lazy locale load can delay first render.
  - Mitigation: load only one locale before mount and keep fallback behavior deterministic.

## Migration Plan
1. Land docs/CI gate change and keep tests green.
2. Land WS backpressure + heartbeat write throttling with backend tests.
3. Land snapshot keyset pagination with API tests.
4. Land UI locale lazy-loading with unit tests/build verification.
5. Land targeted complexity cleanup in Agent WS path.

## Open Questions
- Should we deprecate old numeric snapshot cursors immediately or keep dual-accept parsing for one release window?
