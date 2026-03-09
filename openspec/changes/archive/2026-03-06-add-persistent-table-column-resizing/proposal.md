# Change: Add persistent manual table column resizing (per list)

## Why
Operational list pages benefit from user-controlled column widths. Some columns (e.g. enum/status) should remain compact while long-text columns (e.g. “最近错误”) need more room depending on the operator’s workflow. Column resize preferences should persist across refresh and be isolated per list page.

## What Changes
- Add manual column resizing on desktop data tables.
- Persist resized column widths across refresh using per-page storage keys (tables do not affect each other).
- Apply to:
  - Settings → 运维 → 不完整运行清理
  - Settings → 通知 → 队列

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/settings/maintenance/MaintenanceCleanupView.vue`
  - `ui/src/views/settings/notifications/NotificationsQueueView.vue`
  - (new) shared utility for column width persistence

## Compatibility / Non-Goals
- No backend changes.
- No change to existing filtering, paging, or actions behavior.

