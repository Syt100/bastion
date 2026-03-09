# Change: Update priority reliability and performance optimizations

## Why
A quick project health check surfaced several high-impact optimization opportunities: default-feature clippy gaps in docs tests, unbounded WebSocket outbox memory risk, snapshot pagination instability under concurrent updates, and frontend bundle bloat from eager locale loading.

## What Changes
1. Fix docs filesystem-mode tests so no sync `MutexGuard` is held across `await`, and harden CI/lint commands to cover the default-feature path.
2. Introduce bounded backpressure for Hub↔Agent WebSocket outboxes and throttle high-frequency `last_seen_at` updates.
3. Replace job snapshot listing `OFFSET` pagination with keyset pagination to avoid skipped/duplicated rows during concurrent status transitions.
4. Reduce frontend initial bundle size by lazily loading i18n locale messages and preserving locale preference behavior.
5. Start reducing high-arity function complexity in the Agent WS path by introducing request context grouping and removing targeted `too_many_arguments` suppression.

## Impact
- Affected specs: `backend`, `ui`, `ci`
- Affected code:
  - `crates/bastion-http/src/http/docs.rs`
  - `crates/bastion-http/src/http/agents/ws.rs`
  - `crates/bastion-engine/src/agent_manager.rs`
  - `crates/bastion-storage/src/run_artifacts_repo.rs`
  - `crates/bastion-http/src/http/jobs/snapshots.rs`
  - `scripts/ci.sh`
  - `ui/src/i18n/index.ts`
  - `ui/src/main.ts`

## Non-Goals
- No API contract break for existing clients.
- No broad architecture rewrite of scheduler/backup internals in this change.
