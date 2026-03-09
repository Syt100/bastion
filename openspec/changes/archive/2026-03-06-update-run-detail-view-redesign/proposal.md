# Change: Redesign Run Detail layout for density and readability

## Why
The current Run Detail page is functionally correct, but it still reads like a stack of generic cards: there is excessive vertical whitespace (especially for empty Operations/Summary), the page becomes long on desktop, and the user’s primary scanning flow (status → progress → events) is visually fragmented.

## What Changes
- Introduce a more cohesive first screen:
  - keep a clear header (status + run id + actions)
  - increase information density in the overview area (key facts + key counters)
  - keep Progress visible in the first screen (desktop: alongside overview; mobile: directly below)
- Consolidate secondary sections into a single “Details” area:
  - move Events / Operations / Summary into tabs to reduce long scrolling and empty card blocks
  - make empty states compact (no large empty table/card)
- Improve Summary presentation:
  - hide empty “More” panels; only render detail blocks when there is content
  - keep raw JSON accessible via collapse or modal (with copy)

## Impact
- Affected specs: `web-ui`
- Affected code:
  - `ui/src/views/RunDetailView.vue`
  - `ui/src/components/runs/RunProgressPanel.vue` (if embedded mode is introduced)

## Non-Goals
- Backend API/schema changes.
- Changing run/operation semantics.
