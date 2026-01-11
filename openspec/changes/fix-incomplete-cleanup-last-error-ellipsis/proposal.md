# Change: Fix incomplete cleanup “最近错误” truncation (single-line + full text access)

## Why
The incomplete cleanup list is an operational page. Long “最近错误” strings currently expand the table width and can introduce horizontal scrolling. The “最近错误” cell should stay single-line (not increase row height) while still allowing operators to quickly view the full error.

## What Changes
- Render the “最近错误” column as a single-line truncated summary that does not grow row height.
- Provide access to the full error via hover (tooltip) and click (open details).
- Prevent long error strings from forcing the table to grow wider than intended.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/settings/maintenance/MaintenanceCleanupView.vue`

## Compatibility / Non-Goals
- No backend API changes.
- No changes to the existing error recording format.

