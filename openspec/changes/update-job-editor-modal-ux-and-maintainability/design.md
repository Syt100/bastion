# Design: Job Editor Modal Refactor & UX Enhancements

## Goals
- Reduce the surface area of `JobEditorModal.vue` by isolating responsibilities:
  - **Mapping** (API ↔ form)
  - **Validation** (per-step, deterministic)
  - **UI** (per-step components)
  - **Orchestration** (load/save, step navigation, error routing)
- Improve user guidance on validation failures (scroll/focus first invalid field).
- Improve mobile usability by keeping actions visible.

## Proposed Structure

### Files
- `ui/src/components/jobs/editor/types.ts`
  - `JobEditorForm`, `JobEditorField`, policy enums, constants.
- `ui/src/components/jobs/editor/form.ts`
  - `createInitialJobEditorForm()` and helpers to reset/normalize.
- `ui/src/components/jobs/editor/mapping.ts`
  - `jobDetailToEditorForm(job)` and `editorFormToRequest(form)`.
  - Handles legacy filesystem `root` → `paths` fallback.
- `ui/src/components/jobs/editor/validation.ts`
  - `validateStep(step, form, t)` and helpers.
  - Includes lightweight cron validation (5 fields + allowed characters).
- `ui/src/components/jobs/editor/steps/*`
  - One component per step: Basics, Source, Target, Security, Notifications, Review.

### Data Flow
- **Edit mode**
  - Fetch `JobDetail` → `jobDetailToEditorForm` → bind into reactive form.
- **Create / Preview / Save**
  - Reactive form → `editorFormToRequest` → preview JSON and submit payload.

## Validation & Field Focus
- Validation runs on:
  - Next-step navigation (`validateStep(currentStep)`).
  - Save (`validateAllRequiredSteps`).
  - Step clicking: backward always allowed; forward requires validating all prior steps.
- On failure:
  - Update `fieldErrors` with messages.
  - Determine the first invalid field for the relevant step order.
  - Scroll the field wrapper into view and focus the first input/textarea within.

## UX Enhancements
- Use the modal footer for the action bar so buttons remain accessible without scrolling.
- Add “common cron presets” (e.g., hourly, daily midnight) that fills the schedule input.
- Add “manage” shortcuts:
  - WebDAV secrets: node-scoped storage settings (`/n/:nodeId/settings/storage`).
  - Notification destinations: global settings (`/settings/notifications/destinations`).
  - Open in a new tab to preserve the current draft in the modal.

