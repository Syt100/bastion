## Why

The current Jobs experience is split across list modes, detail panes, and a large modal editor. Core controls such as filters and authoring structure are hard to scan, which slows down the product's most important workflow: creating, understanding, and operating backup jobs.

## What Changes

- Rebuild `Jobs` as a first-class workspace with a desktop three-pane model and dedicated mobile list/detail flows.
- Make primary filters and job row actions more visible, and support reusable saved views for common job lists.
- Replace the modal job editor with a full-page stepper flow for create/edit operations.
- Add an aggregated job workspace read model that surfaces operational summary fields such as recent failures, next scheduled run, target health, and restore-readiness indicators.
- Standardize desktop and mobile job authoring around the same step structure, validation model, and review experience.

## Capabilities

### New Capabilities
- `jobs-workspace`: primary Jobs list/detail workspace, filter/view behavior, desktop/mobile layout model, and aggregated job workspace data contract
- `job-editor-flow`: full-page create/edit stepper, validation, draft/resume behavior, and review/risk summary experience

### Modified Capabilities

## Impact

- Affected code:
  - `ui/src/views/jobs`, shared list/layout primitives, router, stores, and i18n
  - job CRUD and job detail APIs in `crates/bastion-http`
  - any storage/query helpers needed to expose workspace-oriented job summary fields
- Affected APIs:
  - aggregated job workspace read model
  - editor-friendly validation and create/edit flows
- Product impact:
  - Jobs becomes the primary operational workspace instead of a secondary list-and-modal combination
