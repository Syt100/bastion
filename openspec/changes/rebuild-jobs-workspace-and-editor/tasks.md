## 1. Spec Foundation

- [x] 1.1 Finalize proposal, design, and spec deltas for `jobs-workspace` and `job-editor-flow`
- [x] 1.2 Run `openspec validate rebuild-jobs-workspace-and-editor --strict`

## 2. Jobs Workspace Data Model

- [x] 2.1 Define and implement the job workspace read model for list rows, detail summaries, and supporting panes
- [x] 2.2 Add backend tests for workspace-oriented job list/detail responses, including empty and degraded states

## 3. Jobs Workspace UI

- [x] 3.1 Build the desktop three-pane Jobs workspace with persistent filters and responsive collapse behavior
- [x] 3.2 Build the mobile Jobs list/detail flow with clear filter and action affordances
- [x] 3.3 Add reusable saved views or an equivalent persisted view-state model for common job slices

## 4. Job Editor Flow

- [x] 4.1 Add full-page create/edit routes and stepper containers for the job editor
- [x] 4.2 Implement step-level validation, server-assisted checks, and draft persistence/resume behavior
- [x] 4.3 Add the review/risk summary step and remove dependence on the modal editor once parity is reached

## 5. Validation

- [x] 5.1 Update and add UI tests for list filters, list selection, desktop/mobile workspace behavior, and editor navigation
- [x] 5.2 Run targeted backend and UI tests plus broader verification for the Jobs workspace transition
