# Change: Update filesystem picker layout + confirm when picking current directory

## Why
In the filesystem file/directory picker, the “Select current directory” action currently appears in the header area and only toggles an internal selection. Because the table typically does not render the current directory itself as a row, clicking the button appears to do nothing.

Additionally, the header layout is noisy (multiple text buttons) and the selected count is visually detached from the primary confirm actions.

## What Changes
- Layout:
  - Replace the “Up” and “Refresh” text buttons with icon-only buttons placed next to the current-path input.
  - Move “selected count” (`已选 x 项`) into the modal footer (left side).
  - Move “Select current directory” into the footer alongside “Add selected”.
  - Keep mobile usability in mind: avoid making the current-path input too narrow on small screens.
- Behavior:
  - “Select current directory” is a confirm-style action:
    - If no items are selected, it immediately confirms and returns the current directory.
    - If items are already selected, it prompts for confirmation and shows:
      - the current directory, and
      - the list of selected items.
    - The confirmation dialog’s primary/default action is “Only select current directory”.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - `ui/src/i18n/locales/zh-CN.ts`
  - `ui/src/i18n/locales/en-US.ts`
  - `ui/src/components/fs/FsPathPickerModal.spec.ts`

## Compatibility / Non-Goals
- No backend API changes.
- No change to restore/archive picker behavior (out of scope).

