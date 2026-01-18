# Change: Fix picker table fill height (avoid double scroll and mobile blank space)

## Why
`FsPathPickerModal` and `RunEntriesPickerModal` currently hard-code `n-data-table` `max-height` values. This creates two UX problems:

- Desktop: the modal content can scroll while the table also scrolls (double scrollbars).
- Mobile: the hard-coded calc can under-estimate the available height and leave a large blank gap below the table.

## What Changes
- Replace hard-coded `max-height` values with a "fill remaining space" layout:
  - The modal content becomes a flex column (`overflow: hidden`).
  - The table sits in a `flex-1 min-h-0` container and is the single scroll region for long lists.
  - A `ResizeObserver` measures the table container height and binds the pixel value to `n-data-table` `:max-height`.
- Apply to:
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - `ui/src/components/jobs/RunEntriesPickerModal.vue`

## Impact
- Affected specs: `web-ui`
- Affected behavior: picker modal scroll behavior only (no selection/filtering semantics changes).

## Compatibility / Non-Goals
- No changes to list fetching, selection semantics, or filter logic.

