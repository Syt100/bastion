# Change: Optimize run-event detail dialog content hierarchy

## Why
The event-detail dialog layout is now stable, but the content is still dense when payloads are long. Users must scan large raw blocks before finding actionable diagnostics.

## What Changes
- Refine the shared `RunEventDetailContent` renderer to use progressive disclosure for verbose sections.
- Keep concise, actionable diagnostics in the first screen (message/hint/diagnostics/context), and move raw payload behind explicit expand/collapse actions.
- Add dedicated rendering for long `error_chain` content so users can read key failures without immediately loading the full raw JSON block.
- Keep backend/API schema unchanged; this is a web-ui presentation-only improvement.
- Add regression tests for the new dialog content behavior.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/runs/RunEventDetailContent.vue`
  - `ui/src/components/runs/RunDetailDetailsTabs.vue`
  - `ui/src/components/jobs/RunEventsModal.vue`
  - `ui/src/i18n/locales/*`
  - related UI tests
