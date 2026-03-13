# Change: Update Agents modal shell consistency

## Why
Agents page still had several legacy card modals implemented directly with `NModal` while other key pages were already migrated to the shared modal shell. This caused subtle spacing/footer/scroll inconsistencies.

## What Changes
- Migrate remaining Agents page management modals (labels, bulk sync, bulk labels) to `AppModalShell`.
- Keep existing modal business logic, validation, and action semantics unchanged.

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/AgentsView.vue`
  - `CHANGELOG.md`

## Non-Goals
- Changing modal workflows or payload contracts.
