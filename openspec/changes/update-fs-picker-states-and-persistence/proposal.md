# Change: Filesystem picker empty/error states and filter persistence

## Why
When browsing nodes and directories, users need clearer feedback and recovery actions:
- distinguish “empty directory” vs “no matches”,
- clearly surface permission/offline errors,
- retry quickly without losing context.

Additionally, remembering per-node filter state improves usability for repeated browsing.

## What Changes
- Improve empty states:
  - “Directory is empty” when no filters are active and no entries exist.
  - “No matches” when filters/search are active and results are empty.
- Improve error states:
  - Distinguish common errors (offline, permission denied, not found, invalid cursor).
  - Provide contextual actions: retry / go up / copy path / clear filters.
- Persist per-node picker filter state in `localStorage`:
  - search term, kind filter, hide dotfiles, type sort, size filter values.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - Shared picker components/utilities (as needed)

## Non-Goals
- Persist selections across modal sessions.
