# Change: Update list filters and state consistency

## Why
Filter entry points and list state feedback remain uneven across pages: some views expose all filters inline, others split interactions between popovers/drawers, and loading/empty/no-result states are not consistently structured. This causes discoverability and comprehension friction.

## What Changes
- Add shared filter-layout primitives to unify filter label/control structure in inline and stacked modes.
- Standardize filter summary visibility (result count + active chips) across list pages.
- Add a shared list-state presenter to unify loading, base-empty, and no-result rendering patterns.
- Migrate Jobs and Agents list screens to shared filter and state components.
- Keep existing filter business logic and backend query behavior unchanged.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/components/list/*`
  - `ui/src/views/jobs/JobsFiltersPanel.vue`
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/views/AgentsView.vue`

## Non-Goals
- Introducing new filter dimensions or changing filter semantics.
- Changing pagination API contracts.
