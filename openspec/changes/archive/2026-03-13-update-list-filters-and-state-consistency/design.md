## Context
The project has shared list scaffolding, but filter field presentation and list state feedback still have page-specific branching, especially between desktop and mobile modes.

## Goals / Non-Goals
- Goals:
  - Present filter controls with consistent structure in inline and stacked contexts.
  - Keep result visibility and active filter feedback predictable.
  - Remove repeated loading/empty/no-result template blocks.
- Non-Goals:
  - No changes to filtering semantics.
  - No changes to remote pagination contracts.

## Decisions
- Decision: Introduce composable shared UI primitives in `components/list`:
  - filter field row wrapper,
  - unified filter summary row,
  - unified list-state block.
- Decision: Keep existing data stores and filter composables; only migrate rendering and layout wiring.
- Decision: Prefer progressive migration by updating Jobs and Agents first as baseline pages.

## Risks / Trade-offs
- Risk: Shared state component may not cover every edge case.
  - Mitigation: keep API flexible with slots/actions and explicitly support base-empty vs filtered-empty.

## Migration Plan
1. Add new shared list filter/state components.
2. Migrate `JobsFiltersPanel` + Jobs workspace to shared components.
3. Migrate Agents list view to shared components.
4. Validate behavior via existing view tests.
