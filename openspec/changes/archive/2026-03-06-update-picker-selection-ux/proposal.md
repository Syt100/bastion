# Change: Picker selection UX (select all/invert, shift-range, selected preview)

## Why
Selecting many entries in a paged table is currently tedious.

Improving selection UX reduces clicks and makes bulk selection practical on both desktop and mobile.

## What Changes
- Add selection helpers to picker modals:
  - Select all (applies to currently loaded rows)
  - Invert selection (applies to currently loaded rows)
  - Clear selection
  - Shift-range selection within the current visible list
- Add a selected-items preview:
  - Show a compact “selected count”
  - Allow expanding to preview selected items and quickly clear
- Define pagination semantics explicitly: “select all” operates on loaded rows; users can load more and repeat as needed.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - `ui/src/components/jobs/RunEntriesPickerModal.vue`

## Non-Goals
- A true “select all matches across the entire directory” without loading pages.
