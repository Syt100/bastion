# Change: Add breadcrumb-style path navigation to picker path bars

## Why
The picker path/prefix bar is currently a plain text input. When the current path is long, it becomes hard to read and hard to navigate quickly.

Users expect a Windows Explorer-like experience where each path segment is clickable to jump directly to that directory, while still retaining the ability to paste/type a path.

## What Changes
- Update the shared picker path bar component (`PickerPathBarInput`) to support a hybrid UI:
  - **Breadcrumb mode (default):** renders the current path/prefix as clickable segments.
  - **Edit mode:** provides a normal text input for manual entry.
- Add long-path handling in breadcrumb mode:
  - middle segments collapse into an `…` item when the path has many segments,
  - desktop shows a popover menu for collapsed segments,
  - mobile shows a bottom drawer for collapsed segments.
- Keep existing affordances:
  - icon-only up/refresh actions,
  - `focus()` API for modals to autofocus the path bar.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/pickers/PickerPathBarInput.vue`
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - `ui/src/components/jobs/RunEntriesPickerModal.vue`

## Compatibility / Non-Goals
- No backend API changes.
- No changes to selection semantics in pickers (only navigation / path bar presentation changes).

