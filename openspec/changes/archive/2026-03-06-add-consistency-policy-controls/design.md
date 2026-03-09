# Design: Consistency policy enforcement

## Policy fields
For sources that support consistency detection (filesystem, vaultwarden):
- `consistency_policy: "warn" | "fail" | "ignore"`
- `consistency_fail_threshold: u64` (optional; default depends on policy)
- `upload_on_consistency_failure: bool` (optional; default `false`)

## Enforcement timing
Enforce policy after packaging (when the report totals are known), before the upload stage:
- If `policy=warn`: emit warning event and proceed.
- If `policy=ignore`: skip emitting the warning event and do not show UI tags (still may store the report for debugging, based on a final decision).
- If `policy=fail`:
  - If `total > threshold`:
    - Emit the warning event.
    - Mark run failed with `error_code="source_consistency"`.
    - If `upload_on_consistency_failure=false`, do not upload artifacts and best-effort cleanup local stage.

## UI/Copy
Job editor should explain:
- `warn`: run succeeds, but results may be inconsistent.
- `fail`: prevents retaining potentially inconsistent backups (recommended for strict workloads).
- Recommend enabling snapshots where available.

## Testing
- Executor tests for:
  - `warn` does not fail run
  - `fail` fails run when total exceeds threshold
  - `fail` does not upload when configured
- UI tests for editor field round-trip.

