# Design: Better consistency warnings UX

## Data Model

### SourceConsistencyReport
We reuse the existing structured consistency report stored in `summary_json` and emitted in run events:
- `changed_total`
- `replaced_total`
- `deleted_total`
- `read_error_total`
- `sample_truncated`
- `sample[]` (capped)

### Derived total
Define:
`consistency_changed_total = changed_total + replaced_total + deleted_total + read_error_total`

## Control-plane: Early signal for running runs

### Problem
`GET /api/jobs/:id/runs` currently derives `consistency_changed_total` from `summary_json`, but:
- `summary_json` is typically written at completion
- so running runs can’t show the warning until late

### Approach
In the job runs list endpoint, compute `consistency_changed_total` as:
1) If `summary_json` contains a report, use its total.
2) Else, look up the latest `run_event(kind='source_consistency')` for the run and derive the total from event fields.
3) Else, return `0`.

Implementation notes:
- Must avoid N+1 queries: for up to 50 runs, fetch their IDs and fetch latest events in a single query.
- If multiple `source_consistency` events exist, use the one with the highest `seq`.

## Web UI: Actionable warning

### Run detail
Add a “Consistency” highlight block:
- Show breakdown and sample paths (first N, show truncation state).
- Provide “View event details” action that switches to the Events tab and sets `kindFilter='source_consistency'`.

### Job runs list
Continue to show a warning tag in the status column; because the API now has an early signal, running runs will show it as soon as the event is emitted.

## Notifications
When a run completes with `consistency_changed_total > 0`, include a concise line:
- `Source changed during backup: {count}`
and recommend checking the run events for samples.

## Testing
- HTTP unit/integration tests for `GET /api/jobs/:id/runs`:
  - when only event exists (running run), `consistency_changed_total` is non-zero
  - when summary exists, summary takes precedence
- UI unit tests:
  - run detail renders breakdown + sample when present
  - “jump to events” sets the filter and switches tabs
- Notifications template tests:
  - includes the line when `consistency_changed_total > 0`

