# Change: Improve Bulk Operations UX (auto-refresh, failure focus)

## Why
Bulk operations are designed for fleet management, but the current UX requires manual refresh and makes it harder to focus on failures.

We want a smoother operational experience:
- running operations update automatically
- failures are easy to spot, filter, and retry

## What Changes
- Add auto-refresh for the Bulk Operation detail modal while the operation is running.
- Add quick filters in the detail view (e.g. show only failed items).
- Keep existing “retry failed” and “cancel” actions prominent.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/settings/BulkOperationsView.vue`

## Non-Goals
- Backend changes or new filtering APIs.

