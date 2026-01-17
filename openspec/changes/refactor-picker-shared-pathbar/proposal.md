# Change: Refactor picker modals to share a common path bar component

## Why
The filesystem picker (`FsPathPickerModal`) and run entries picker (`RunEntriesPickerModal`) both implement a “path/prefix input + up/refresh actions” UI. When these modals evolve independently, small UX improvements (focus behavior, icon styling, compact spacing, mobile layout) easily diverge and become inconsistent.

## What Changes
- Extract a shared `PickerPathBarInput` component that encapsulates:
  - the input field,
  - icon-only up/refresh actions rendered as input prefix buttons,
  - consistent compact spacing and softened icon styling,
  - an exposed `focus()` method for modal open autofocus.
- Refactor:
  - `FsPathPickerModal` to use `PickerPathBarInput` for the current-path input.
  - `RunEntriesPickerModal` to use `PickerPathBarInput` for the current-prefix input.
- Keep behavior unchanged except where explicitly required by existing UX specs (e.g., focusing the input on open, mobile count badge).

## Impact
- Affected specs: `web-ui`
- Affected code:
  - (new) `ui/src/components/pickers/PickerPathBarInput.vue`
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - `ui/src/components/jobs/RunEntriesPickerModal.vue`

## Compatibility / Non-Goals
- No backend API changes.
- No changes to selection semantics beyond aligning presentation and focus behavior across the two pickers.

