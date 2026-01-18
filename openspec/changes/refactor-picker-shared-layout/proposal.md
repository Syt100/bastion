# Change: Refactor picker modals into a shared layout/composable

## Why
Both pickers share a very similar structure (path bar, search, filters, active chips, table with measured height, footer actions).

This duplication makes the code harder to maintain and increases the risk of UX regressions (fixing one modal but forgetting the other).

## What Changes
- Introduce a shared picker layout component/composable that encapsulates:
  - Top bar (path/prefix navigation + refresh/up actions)
  - Search input + filters trigger (desktop popover / mobile drawer)
  - Active filter chips + clear actions
  - Table container height measurement
  - Footer layout (selection summary + primary/secondary actions)
- Migrate:
  - `FsPathPickerModal` (filesystem picker)
  - `RunEntriesPickerModal` (restore entries picker)
  to use the shared building blocks.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/components/fs/FsPathPickerModal.vue`
  - `ui/src/components/jobs/RunEntriesPickerModal.vue`
  - New shared picker layout/composable under `ui/src/components/pickers/`

## Non-Goals
- No intended behavior change (pure refactor + dedup). Any behavior change should be done in follow-up changes with explicit specs.
