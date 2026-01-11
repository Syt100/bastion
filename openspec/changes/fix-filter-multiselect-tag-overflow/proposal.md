# Change: Keep multi-select filter controls compact when many values are selected

## Why
Multi-select filters are useful, but when many values are selected the select trigger can grow vertically (wrapping tags). This pushes the list content down and makes the filter bar visually noisy.

## What Changes
- For multi-select filter controls on list pages, limit displayed tags in the trigger so the control height stays compact.
- Use an overflow indicator (e.g. `+N`) with a popover to view all selected values.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/settings/maintenance/MaintenanceCleanupView.vue`
  - `ui/src/views/settings/notifications/NotificationsQueueView.vue`

## Compatibility / Non-Goals
- No changes to filter semantics or API calls.

