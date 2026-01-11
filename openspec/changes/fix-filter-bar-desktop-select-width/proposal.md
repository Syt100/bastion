# Change: Fix desktop filter bar select width (avoid full-width dropdowns)

## Why
On desktop screens, list-page filter bars should be compact. However, Naive UI's `n-select` root element defaults to `width: 100%`, which causes each select to span the full row. This also makes the dropdown menu excessively wide (it follows the trigger width by default).

## What Changes
- Constrain filter `n-select` width on desktop by wrapping selects in fixed-width containers.
- Keep mobile behavior unchanged (full-width controls).
- Apply to:
  - Settings → Maintenance → Incomplete cleanup
  - Settings → Notifications → Queue

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/settings/maintenance/MaintenanceCleanupView.vue`
  - `ui/src/views/settings/notifications/NotificationsQueueView.vue`

## Compatibility / Non-Goals
- No functional changes to filtering; only layout/UX.

