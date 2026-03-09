# Change: Update Job Editor Modal UX and Maintainability

## Why
The job create/edit modal is a core workflow, but it is currently difficult to evolve safely:
- `JobEditorModal.vue` mixes state, API ↔ form mapping, validation, and UI for 6 steps in a single large file, increasing change risk.
- Validation feedback is coarse (toast + inline message) and does not guide the user to the first problem.
- Mobile usability can be improved by keeping the action bar persistently visible.

## What Changes
- Refactor `JobEditorModal` into a small orchestration component + per-step subcomponents.
- Extract pure mapping utilities for `JobDetail` → editor form and editor form → `CreateOrUpdateJobRequest`.
- Add unit tests for the mapping layer (including legacy/compat fields) to reduce regression risk.
- Rework validation into per-step functions (including cron format validation when provided).
- When validation fails, automatically scroll/focus the first invalid field in the modal.
- UX improvements:
  - Persistent action bar (modal footer) so actions remain accessible on mobile.
  - Allow clicking steps to navigate to completed steps; prevent skipping ahead when prior steps are invalid.
  - Add quick links to manage WebDAV secrets (node-scoped) and notification destinations (open in new tab to preserve draft).
  - Add a lightweight “common cron presets” picker for faster scheduling.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/jobs/JobEditorModal.vue`
  - (new) `ui/src/components/jobs/editor/*` for types, mapping, validation, and step components
  - `ui/src/i18n/locales/*` for new labels/help text

## Compatibility / Non-Goals
- No backend API changes.
- No changes to job execution semantics.
- No new job types or targets.

