# Change: Refactor run-request race safety and shared stream flow

## Why
Run-related dialogs still have request race windows and duplicated WebSocket stream logic. This can cause stale modal data and drift in reconnect/dedup behavior across pages.

## What Changes
- Add latest-request/abort semantics for job run list loading in `JobRunsModal` and `jobs.listRuns`.
- Introduce a shared run-events stream controller utility for WebSocket lifecycle, reconnect backoff, and seq dedupe.
- Migrate `RunEventsModal` and `RunDetailPanel` to the shared stream controller without changing product behavior.
- Add regression tests for stale-request prevention and stream controller behavior.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/jobs/JobRunsModal.vue`
  - `ui/src/stores/jobs.ts`
  - `ui/src/lib/api.ts`
  - `ui/src/lib/latest.ts`
  - `ui/src/lib/runEventsStream.ts` (new)
  - `ui/src/components/jobs/RunEventsModal.vue`
  - `ui/src/components/runs/RunDetailPanel.vue`
  - `ui/src/components/jobs/JobRunsModal.spec.ts` (new)
  - `ui/src/lib/runEventsStream.spec.ts` (new)

## Non-Goals
- Changing run events UI layout, copy, or visual style.
- Changing run-events server payload schema.
