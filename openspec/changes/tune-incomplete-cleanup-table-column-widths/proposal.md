# Change: Tune incomplete cleanup desktop table column widths (compact enums, wider “最近错误”)

## Why
The incomplete cleanup list is an operational table. After adding single-line truncation to “最近错误”, the current column sizing makes enum columns (target/status) appear too wide while the long-text “最近错误” column is too narrow, reducing scanability.

## What Changes
- Make enum columns (目标/状态) compact on desktop.
- Give the “最近错误” column more width so the summary is meaningful, while still keeping it single-line truncated and accessible via hover/click.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/settings/maintenance/MaintenanceCleanupView.vue`

## Compatibility / Non-Goals
- No backend changes.
- No change to labels or status names.

