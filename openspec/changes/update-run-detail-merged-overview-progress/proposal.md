# Change: Merge Run Detail Overview + Progress into a single summary card

## Why
The Run Detail page currently renders Overview and Progress as two separate cards side-by-side on desktop. When Progress is taller than Overview, the layout leaves a large visible blank area under Overview, and the page feels less compact/productized.

Additionally, restore operations often end with `rate_bps` missing, so the Operation details modal shows speed as "-" after completion even though an average speed can be derived.

## What Changes
- Run Detail: replace the two-card Overview/Progress row with a single "Summary" card that contains both Overview and Progress sections.
  - Desktop: default expanded (show full Overview + full Progress + Source/Transfer stats).
  - Mobile: same information, stacked with compact spacing.
- Run Progress: preserve completion behavior (Upload at 100% renders finished) and keep help accessible.
- OperationModal (Restore): after completion, show a computed final average speed when live `rate_bps` is missing.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/RunDetailView.vue`
  - `ui/src/components/runs/RunProgressPanel.vue`
  - `ui/src/components/jobs/OperationModal.vue`

## Non-Goals
- Changing backend progress payloads.
- Adding pagination/virtualization changes to events.
