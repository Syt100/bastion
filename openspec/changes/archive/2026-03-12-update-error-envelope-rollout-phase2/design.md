## Context
Phase 1 introduced canonical `error_envelope` and made Run Events UI envelope-first for core failure paths. Remaining gaps are concentrated in:
- Agent-to-Hub bridged failure events (especially snapshot delete result/status paths).
- Execute-stage failures/warnings in filesystem/sqlite/vaultwarden pipelines.
- Maintenance/snapshot management pages that still primarily show legacy `last_error_kind/last_error`.

## Goals
- Ensure remaining backend event producers emit `fields.error_envelope` in a consistent shape.
- Ensure UI diagnostics outside Run Events can consume envelope data when present.
- Preserve compatibility by keeping legacy fields and fallback rendering paths.

## Non-Goals
- Replace HTTP API `AppError` payload schema in this change.
- Remove legacy task-level `last_error*` storage columns in this change.

## Decisions

### 1) Canonical field location and compatibility
- Keep canonical diagnostics at `fields.error_envelope`.
- Continue writing existing legacy fields in parallel (`error_kind`, `hint`, `last_error_kind`, etc.).

### 2) Agent-bridged failure envelope policy
- In `agents/ws` handlers, when incoming event/result has envelope data, pass through after validation.
- If incoming payload lacks envelope, synthesize one from available status/error/error_kind context.
- Use stable namespaces for code families:
  - `agent.snapshot_delete.*`
  - `agent.task_result.*`

### 3) Execute-stage envelope policy
- For execute-stage warnings/failures (`snapshot_cleanup_failed`, direct path unavailable, integrity check failure, etc.), emit envelope with:
  - stable `code` family under `scheduler.execute.*`,
  - `origin` pointing to concrete execute component,
  - transport selected by effective target/protocol when meaningful (`file`, `http`, `internal`, `unknown`).

### 4) Maintenance/Snapshot UI rendering policy
- Keep current task list rendering for compatibility.
- In task detail dialogs/panels, prefer envelope-derived diagnostics when latest related event includes envelope.
- Fallback chain:
  1) envelope localized message/hint;
  2) legacy `last_error_kind/last_error`;
  3) generic localized fallback.

## Risks / Trade-offs
- Temporary duplication of diagnostics data (envelope + legacy fields) increases payload size.
- Bridged event synthesis can misclassify edge-case unknown errors if upstream metadata is poor.
- UI might need extra API fetches for event details in maintenance/snapshot screens.

## Validation
- Backend regression tests:
  - Agent snapshot-delete/task-result envelope emission and fallback synthesis.
  - Execute-stage failure/warn envelope presence and field semantics.
- UI tests:
  - Maintenance/snapshot detail rendering envelope-first + fallback.
- Full CI and changelog update for user-visible diagnostics changes.
