# Change: Fix Run Detail header/progress polish regressions and tighten layout

## Why
Recent UI polish introduced regressions and rough edges:
- Stage help for Scan/Packaging is no longer easily accessible.
- The status badge placement and some labels still feel non-productized.
- Upload stage at 100% still looks like an in-progress stage.
- Long target paths are truncated, preventing users from reading the full destination.
- Overview/Progress panels can be more compact.

## What Changes
- Restore access to Scan/Packaging help content via a compact help icon in the Progress “Stages” header.
- Move the run status badge to the right side of the Run Detail header.
- When Upload stage reaches 100%, render it as a finished stage (no stage progress bar).
- Render the Overview target path without ellipsis (wrap to multiple lines).
- Reduce vertical spacing in Overview/Progress panels for a denser layout.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/RunDetailView.vue`
  - `ui/src/components/runs/RunProgressPanel.vue`

## Non-Goals
- Changing backend progress semantics or API payloads.
