# Change: Update component modal-shell consistency

## Why
Several reusable Jobs/Run components still render dialogs directly with `NModal` while page-level dialogs already share `AppModalShell`. This leaves component-level spacing, footer layout, and slot wiring inconsistent.

## What Changes
- Migrate remaining reusable Jobs/Run dialogs to `AppModalShell`.
- Extend shared shell usage for component needs (for example optional non-scroll body mode) while preserving current modal workflows.
- Keep dialog action semantics, payloads, and emit contracts unchanged.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/AppModalShell.vue`
  - `ui/src/components/jobs/JobDeployModal.vue`
  - `ui/src/components/jobs/JobEditorModal.vue`
  - `ui/src/components/jobs/JobRunsModal.vue`
  - `ui/src/components/jobs/RestoreWizardModal.vue`
  - `ui/src/components/jobs/RunEventsModal.vue`
  - `ui/src/components/jobs/VerifyWizardModal.vue`
  - `ui/src/components/runs/RunEventDetailDialog.vue`
  - `CHANGELOG.md`

## Non-Goals
- Changing picker-only modal wrappers.
- Changing business rules, API calls, or emitted events for these dialogs.
