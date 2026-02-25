## Context
Runs (backup) and operations (restore/verify) can execute for a long time and currently lack a first-class cancellation lifecycle. Existing completion paths can finalize success/failure directly, which introduces a race if cancellation is requested while work is still producing results.

The implementation spans storage, scheduler/worker execution, backup/restore internals, Hub↔Agent protocol, and Web UI behavior.

## Goals / Non-Goals
- Goals:
  - Provide an operator-facing, idempotent cancel capability for queued and running runs/operations.
  - Stop work cooperatively at safe boundaries and complete cleanup before terminalizing.
  - Make cancellation race-safe so late results cannot overwrite `canceled`.
  - Support both local execution and agent-executed tasks.
- Non-Goals:
  - Hard kill/forced thread termination in initial delivery.
  - Redesigning run/operation scheduling semantics beyond cancellation behavior.

## Decisions

- Decision: Two-step cancellation state model
  - Add terminal status `canceled` for runs and operations.
  - Record cancel intent (`cancel_requested_at`, `cancel_requested_by_user_id`, optional reason).
  - Keep active task status as `running` while cleanup completes; transition to `canceled` at checkpoint exit.

- Decision: Persist intent + in-memory signal
  - Cancel API writes cancel-request metadata in storage (source of truth).
  - Active task receives an immediate in-memory cancellation signal (token/registry) for low-latency reaction.

- Decision: Cooperative checkpoints for graceful interruption
  - Checkpoints are inserted at natural boundaries (scan chunk, package batch, upload part, restore entry, verify chunk).
  - On cancellation, execution returns a dedicated canceled error path that performs cleanup and then terminalizes.

- Decision: CAS-style terminalization guards
  - Success/failure completion writes become conditional (only when current state permits).
  - Cancellation terminalization also uses conditional writes to avoid duplicate/flip-flop transitions.
  - Hub result ingestion continues to reject stale terminalization attempts when status is no longer `running`.

- Decision: Explicit Hub↔Agent cancel messages
  - Add protocol messages for canceling run tasks and operation tasks.
  - Agent maps task IDs to local cancellation tokens and exits cooperatively.
  - Agent completion may explicitly report canceled outcome.

## Risks / Trade-offs
- Deep `spawn_blocking` and third-party backup internals may reduce cancellation responsiveness.
  - Mitigation: start with boundary checkpoints and extend into deeper loops in phase 2.
- Cancellation adds more state transitions and race surfaces.
  - Mitigation: repository-level CAS helpers and focused race regression tests.
- Protocol/version compatibility between Hub and Agent.
  - Mitigation: additive protocol fields/messages and feature-gated fallback behavior.

## Migration Plan
1. Add schema/model changes for statuses and cancel metadata.
2. Add cancel APIs + repository primitives + local cancellation registry.
3. Add cooperative checkpoints for local run/operation execution.
4. Add agent protocol cancel messages and agent-side handling.
5. Update UI and docs; then run full CI.

## Open Questions
- Whether canceling a run should emit final notification by default or only record in run history.
- Whether to expose optional cancel reason in initial API/UI or defer to later enhancement.
- Whether to provide an operator-only "force stop" after configurable timeout in the same change or a follow-up change.
