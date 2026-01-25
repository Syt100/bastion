# Change: Make Run Detail more compact and keep transfer speed visible after completion

## Why
The Run Detail page still feels visually loose:
- The Overview card stretches to match the Progress card height, leaving large blank areas.
- Spacing in Overview/Progress panels can be tighter for better information density.
- After backup/restore completes, transfer speed is often shown as "-" (missing), which makes the final result less informative.

## What Changes
- Run Detail layout: reduce whitespace and prevent card stretching so the Overview/Progress panels are more compact.
- Progress panel: when a run/operation is finished and live `rate_bps` is missing, display a computed final transfer speed (average over the upload/restore stage when timestamps are available; otherwise fall back to overall duration).
- Minor visual polish to make the page feel more productized (alignment, typography, compact spacing).

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/RunDetailView.vue`
  - `ui/src/components/runs/RunProgressPanel.vue`

## Non-Goals
- Redefining backend progress semantics or adding new API fields.
