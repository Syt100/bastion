# Change: Refactor picker open/refresh pipeline for smoother dialog entry

## Why
Directory picker dialogs currently trigger open transition, data refresh, and table-height measurements in the same critical window. On heavier lists this can make the path list appear to animate slowly when opening or refreshing.

## What Changes
- Split picker session opening into staged phases so dialog enter transition and first data refresh do not compete on the same frame.
- Simplify table-body height measurement lifecycle to avoid redundant frame-chained re-measures on open.
- Keep existing picker capabilities/filters/paging semantics unchanged while reducing layout thrash during open and refresh.
- Add unit tests for session open sequencing and measurement lifecycle expectations.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/pickers/usePickerSessionState.ts`
  - `ui/src/components/pickers/usePickerTableBodyMaxHeightPx.ts`
  - `ui/src/components/pickers/pathPicker/PathPickerModal.vue`
  - `ui/src/components/pickers/usePickerSessionState.spec.ts`
  - `ui/src/components/fs/FsPathPickerModal.spec.ts`

## Non-Goals
- Introducing virtual scrolling or large data rendering architecture changes.
- Changing picker business logic (selected paths, filter semantics, API request contract).
