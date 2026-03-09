# Change: Picker keyboard shortcuts and accessibility improvements

## Why
Picker modals are frequently used for navigation and selection, but today they are not fully keyboard-friendly and can be improved for accessibility.

## What Changes
- Add keyboard shortcuts for picker modals:
  - `Enter`: enter/open the currently active directory (when applicable)
  - `Backspace`: go to parent (when not typing in an input/textarea)
  - `Ctrl/Cmd+L`: focus the path/prefix editor
  - `Esc`: close the modal
- Improve focus order and visible focus states to ensure smooth keyboard navigation.
- Add missing `aria-label` / `title` to icon-only buttons and interactive controls.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - `ui/src/components/jobs/RunEntriesPickerModal.vue`
  - Shared picker components under `ui/src/components/pickers/`

## Non-Goals
- Full screen-reader optimization for the data table beyond practical aria labels and focus order.
