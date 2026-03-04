# Change: Update page modal-shell consistency

## Why
Several page-level management dialogs still use raw `NModal` cards while newer surfaces already use `AppModalShell`. This keeps spacing/footer alignment/scroll containment inconsistent across core pages.

## What Changes
- Migrate remaining page-level dialogs in Jobs/Settings/Snapshots views to `AppModalShell`.
- Extend shared modal shell usage where needed (for example header extra actions) so existing dialog affordances are preserved.
- Keep existing dialog workflows, form payloads, and action semantics unchanged.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/AppModalShell.vue`
  - `ui/src/views/JobSnapshotsView.vue`
  - `ui/src/views/jobs/JobWorkspaceView.vue`
  - `ui/src/views/settings/BulkOperationsView.vue`
  - `ui/src/views/settings/SettingsStorageView.vue`
  - `ui/src/views/settings/maintenance/MaintenanceCleanupView.vue`
  - `ui/src/views/settings/notifications/NotificationsDestinationsView.vue`
  - `CHANGELOG.md`

## Non-Goals
- Changing modal business rules, API payloads, or submit/cancel side effects.
