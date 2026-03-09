# Change: Picker visual polish (table density, separators, hover/selected)

## Why
The picker tables can feel visually “too heavy” and the hover/selected states can be clearer, especially on mobile.

## What Changes
- Soften table separators and reduce “hard” borders where appropriate.
- Improve row hover and selected styling for better scanability.
- Tune mobile row density (spacing + information hierarchy) for both pickers.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - `ui/src/components/jobs/RunEntriesPickerModal.vue`
  - Shared picker styles/utilities (as needed)
