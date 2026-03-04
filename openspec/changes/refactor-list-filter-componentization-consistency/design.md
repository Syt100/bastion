## Context
Jobs already centralizes filter chips/count/clear behavior in a dedicated composable and reusable panel. Other list pages still compute active filters and render filter selects/chips independently, producing repeated code and subtle UX differences.

## Goals / Non-Goals
- Goals:
  - Establish one reusable model for list-filter active-state/chips/reset logic.
  - Establish reusable filter field and active-chip row wrappers to reduce style drift.
  - Apply the shared model to Agents, Notifications Queue, Maintenance Cleanup, and Job Snapshots.
- Non-Goals:
  - Change backend endpoints or route query contracts.
  - Introduce new domain filter semantics.

## Decisions
- Decision: add `ui/src/lib/listFilters.ts` with a schema-like field builder API (`text/single/multi`) and a `useListFilters` aggregator.
  - Rationale: avoids repeating `hasActiveFilters`, chip mapping, clear/reset behavior across pages while preserving page ownership of filter refs/options.
- Decision: add `ListFilterSelectField` and `ListActiveFiltersRow` under `ui/src/components/list`.
  - Rationale: shared wrappers standardize width/layout/clear affordance and reduce repeated utility classes.
- Decision: migrate the four target pages incrementally without changing request payload semantics.
  - Rationale: low-risk refactor with observable consistency gain and minimal behavior regression risk.
- Decision: apply the same shared filter model to picker modals with repeated active-chip/count logic (`RunEntriesPickerModal`, `PathPickerModal`).
  - Rationale: picker modals are high-frequency surfaces that had duplicated filter-chip/count/clear logic and benefit from the same consistency guarantees.

## Risks / Trade-offs
- Generic filter utilities can over-abstract quickly.
  - Mitigation: keep the API intentionally small (text/single/multi only) and allow page-local overrides where needed.
- Chips for multi-select filters may become verbose with many values.
  - Mitigation: preserve per-value close behavior for discoverability and only show chips when active.

## Migration Plan
1. Add shared filter utilities and wrappers.
2. Migrate Notifications Queue and Maintenance Cleanup (most similar structures).
3. Migrate Agents and Job Snapshots.
4. Migrate RunEntries/Path picker modals to shared filter model + active-filter row wrapper.
5. Update/add regression tests for chip visibility and clear behavior.
6. Run UI tests and CI.

## Open Questions
- None; existing list pages already expose enough label/value data to drive shared chip generation.
