# Change: Refactor shared UI primitives for route/copy/a11y/modal consistency

## Why
Node-scoped route builders, clipboard-feedback handlers, and icon-only action buttons are still duplicated in multiple pages. Remaining picker modals also keep a partial path outside the shared modal shell.

## What Changes
- Add shared node route helpers and replace duplicated `/n/${id}/...` string construction in key views/layouts.
- Add a shared clipboard feedback helper/composable and migrate repeated copy+toast handlers.
- Add a shared icon-only action button component that requires an accessible label.
- Extend/align modal wrapper usage so picker modal card/confirm dialogs follow the shared modal shell conventions.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/lib/nodeRoute.ts` (new)
  - `ui/src/layouts/AppShell.vue`
  - `ui/src/views/AgentsView.vue`
  - `ui/src/lib/clipboardFeedback.ts` (new)
  - `ui/src/views/settings/SettingsStorageView.vue`
  - `ui/src/views/settings/notifications/NotificationsDestinationsView.vue`
  - `ui/src/views/settings/maintenance/MaintenanceCleanupView.vue`
  - `ui/src/components/IconActionButton.vue` (new)
  - `ui/src/components/runs/RunProgressPanel.vue`
  - `ui/src/components/pickers/PickerModalCard.vue`
  - `ui/src/components/pickers/pathPicker/PathPickerModal.vue`

## Non-Goals
- Introducing new navigation destinations or changing route semantics.
- Changing business workflows for copy actions or modal submit/cancel actions.
