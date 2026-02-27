## Context
`JobsWorkspaceShellView.vue` currently duplicates filter-control rendering across multiple branches (desktop split, desktop list, mobile). Row actions in list view are partially hidden behind hover overlays, reducing affordance clarity.

## Goals / Non-Goals
- Goals:
  - Single source of truth for Jobs filters.
  - Consistent filter behavior across split/list/mobile containers.
  - Primary row actions discoverable without hover.
  - Maintain support for keyboard and touch interactions.
- Non-Goals:
  - Add new filter types.
  - Change Jobs API contracts.

## Decisions

### Decision 1: Consolidate filter state/actions into composable
Create a `useJobsFilters` composable that owns filter refs, derived active-count/chips, clear/reset behavior, and normalization.

Alternatives considered:
- Keep state in page component and only extract UI blocks.
  - Rejected because logic duplication would remain and layout branches could still diverge.

### Decision 2: Single filter panel component with multiple containers
Create a reusable filter panel content component (e.g., `JobsFiltersPanel`) and render it via:
- inline region (wide desktop list)
- popover (split mode)
- drawer (mobile)

Control surface differs by breakpoint/layout, but control content and semantics stay identical.

Alternatives considered:
- Build separate desktop/mobile filter components.
  - Rejected due to high long-term drift risk.

### Decision 3: Replace hover-only action reveal with always-visible action rail
Refactor row action area so at least core actions are always visible (for example: Run Now + More), while secondary actions remain in overflow.

Alternatives considered:
- Keep hover-only actions and add helper text.
  - Rejected because discoverability issue remains for touch and keyboard users.

### Decision 4: Explicit interaction boundaries for row click vs action click
Ensure row click opens detail but action buttons do not trigger row navigation. This is implemented with clear event-boundary handling and tested behavior.

## Migration Plan
1. Extract filter state/computed helpers into composable.
2. Extract filter panel UI into reusable component.
3. Replace existing split/list/mobile filter rendering blocks with shared component instances.
4. Refactor row action area to always-visible primary actions + overflow.
5. Update and expand tests for filter parity and action accessibility.

## Risks / Trade-offs
- Risk: users accustomed to hover-only compact rows may perceive denser rows.
  - Mitigation: keep only 1-2 primary actions always visible and move the rest to overflow.
- Risk: behavior regressions when switching between layout modes.
  - Mitigation: canonical filter state in composable + targeted tests for mode switching.

## Open Questions
- Whether filter state should synchronize with route query params as part of this change or follow-up work.
