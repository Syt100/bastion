# Change: Refactor UI icon, dialog, and motion system

## Why
Icon rendering, modal shell styling, and micro-interaction feedback still vary by page. This leads to inconsistent perceived polish even when behavior is correct.

## What Changes
- Add a shared icon wrapper API to normalize icon size and semantic tone usage.
- Add a shared modal shell component that standardizes title/body/footer spacing, scroll behavior, and width presets.
- Add shared micro-interaction classes/tokens for rows, cards, and action controls.
- Migrate key pages/components (Jobs, Agents, shared picker/filter actions) to the new icon/dialog/motion primitives.
- Preserve existing business logic and accessibility constraints, including reduced-motion handling.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/components/AppIcon.vue` (new)
  - `ui/src/components/AppModalShell.vue` (new)
  - `ui/src/components/pickers/PickerFiltersPopoverDrawer.vue`
  - `ui/src/views/jobs/JobsListRowItem.vue`
  - `ui/src/views/AgentsView.vue`
  - `ui/src/styles/main.css`

## Non-Goals
- Replacing the existing icon library.
- Reworking domain-specific modal content logic.
