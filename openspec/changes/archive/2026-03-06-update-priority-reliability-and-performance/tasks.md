## 1. Spec
- [x] 1.1 Create proposal/design/spec deltas for prioritized reliability + performance optimizations
- [x] 1.2 Run `openspec validate update-priority-reliability-and-performance --strict`

## 2. Docs test locking + CI gate (P0)
- [x] 2.1 Refactor docs fs-mode tests to avoid holding a sync mutex guard across any `await`
- [x] 2.2 Keep test isolation guarantees for shared docs directory state
- [x] 2.3 Update CI script/lint workflow so default-feature clippy path is checked in addition to existing strict checks
- [x] 2.4 Run focused backend tests and clippy checks

## 3. Agent WS backpressure + heartbeat write optimization (P1)
- [x] 3.1 Replace unbounded WS outbox channels with bounded channels in Hub and Agent critical paths
- [x] 3.2 Add resilient handling for bounded send failures / disconnections
- [x] 3.3 Throttle `agents.last_seen_at` updates to avoid per-message write amplification
- [x] 3.4 Add or update regression tests for disconnect/backpressure behavior

## 4. Snapshot keyset pagination (P1)
- [x] 4.1 Replace snapshot listing OFFSET query with keyset query in storage/http layers
- [x] 4.2 Ensure cursor contract remains stable and deterministic under concurrent status updates
- [x] 4.3 Add/update API tests to cover pagination continuity and cursor roundtrip

## 5. Frontend bundle optimization (P2)
- [x] 5.1 Convert i18n locale messages to lazy loading (load selected locale before mount)
- [x] 5.2 Preserve locale resolution and persistence behavior
- [x] 5.3 Add/update UI tests for locale initialization path and run UI build checks

## 6. Targeted maintainability cleanup (P2)
- [x] 6.1 Reduce argument fanout in Agent WS handler with context grouping
- [x] 6.2 Remove at least one targeted `#[allow(clippy::too_many_arguments)]` in the optimized path
- [x] 6.3 Run full project checks (`scripts/ci.sh` when feasible) and finalize checklist
