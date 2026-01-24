# Change: Product polish for Run Detail header, target labels, and progress panel

## Why
The Run Detail page still exposes internal identifiers in the UI (e.g. `success`, `local_dir`) and the Progress panel visual rhythm is too rigid (oversized help buttons, repetitive stage rows).

## What Changes
- Localize run statuses in the Run Detail header and runs list views (CN shows Chinese labels).
- Productize target type labels (e.g. `local_dir` -> “本地目录”) while keeping raw values as fallbacks.
- Progress panel polish:
  - replace oversized `?` buttons with a small help-circle icon
  - reduce visual rigidity by using a stage stepper + a single “current stage” progress row

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/RunDetailView.vue`
  - `ui/src/components/runs/RunProgressPanel.vue`
  - `ui/src/components/jobs/JobRunsModal.vue`
  - `ui/src/i18n/locales/*`

## Non-Goals
- Backend API/schema changes.
