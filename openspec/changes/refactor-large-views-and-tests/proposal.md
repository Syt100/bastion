# Change: Refactor large frontend views and expand modal regression tests

## Why
Several key frontend views/components remain very large and mix data orchestration with presentation, which slows iteration and increases regression risk. Recently migrated jobs modals also need stronger direct test coverage.

## What Changes
- Split large views/components into smaller composables and presentational pieces while preserving behavior.
- Move complex action/query/watch logic into dedicated composables where appropriate.
- Add direct component tests for jobs modal flows that currently have sparse coverage.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/pickers/pathPicker/PathPickerModal.vue`
  - `ui/src/views/AgentsView.vue`
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/views/JobSnapshotsView.vue`
  - `ui/src/components/jobs/RunEntriesPickerModal.vue`
  - `ui/src/components/jobs/JobEditorModal.spec.ts` (new)
  - `ui/src/components/jobs/JobDeployModal.spec.ts` (new)
  - `ui/src/components/jobs/JobRunsModal.spec.ts`
  - `ui/src/components/jobs/RestoreWizardModal.spec.ts` (new)
  - `ui/src/components/jobs/VerifyWizardModal.spec.ts` (new)

## Non-Goals
- Redesigning UI layout or visual style.
- Introducing new product capabilities unrelated to maintainability and regression coverage.
