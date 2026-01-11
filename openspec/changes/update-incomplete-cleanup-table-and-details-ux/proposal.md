# Change: Improve incomplete cleanup table + details UX (compact + responsive)

## Why
The incomplete cleanup list is an operational page. On desktop, the table currently shows too many columns and long error strings dominate the layout, reducing scanability. Also, the “更多” details view should be optimized per device: a modal dialog on desktop and a bottom drawer on mobile.

## What Changes
- Desktop table:
  - Reduce default columns to improve scanability.
  - Show “最近错误” as **error type + short summary**, with truncation (full details in “更多”).
- Details view:
  - Desktop: modal dialog.
  - Mobile: bottom drawer.
  - Provide copy actions for key identifiers and error text.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/settings/maintenance/MaintenanceCleanupView.vue`

## Compatibility / Non-Goals
- No changes to backend APIs or filter semantics.

