## Context
List-oriented pages currently share a conceptual structure but implement it independently. This causes repeated markup/state wiring and divergence in interaction details (especially pagination and empty-state rendering).

## Goals / Non-Goals
- Goals:
  - Create reusable list-page structure primitives.
  - Normalize pagination behavior and placement across list pages.
  - Reduce visual nesting noise in list contexts.
  - Keep migration incremental and low-risk.
- Non-Goals:
  - Change backend pagination/filter APIs.
  - Force every page into one rigid component if it reduces page-specific flexibility.

## Decisions

### Decision 1: Shared scaffold with slot-based composition
Use a shared scaffold component (or small component set) that models:
- optional selection toolbar region
- toolbar region (search/filters/actions)
- content region (loading/empty/list)
- footer region (pagination)

This keeps pages composable while removing repeated structural code.

Alternatives considered:
- Hardcode one monolithic `ListPage` component.
  - Rejected because Jobs split-mode and mixed list/detail layout need flexible composition.

### Decision 2: Unify pagination semantics through shared UI wrapper
Create a shared pagination wrapper/contract used by migrated pages:
- standardized props (`page`, `pageSize`, `itemCount`, `pageSizes`, `loading`)
- consistent page reset behavior when filters change
- consistent placement and disabled state handling

Alternatives considered:
- Keep per-page pagination controls and only align styles.
  - Rejected because behavior inconsistency would remain and future drift would continue.

### Decision 3: Context-aware empty-state rendering to remove card-in-card anti-pattern
Extend `AppEmptyState` with explicit variants:
- `card` (default legacy behavior)
- `inset` (framed but not full card)
- `plain` (no extra surface)

Use `plain`/`inset` when empty states are rendered inside existing list cards.

Alternatives considered:
- Keep `AppEmptyState` as-is and manually style each page.
  - Rejected because it repeats style decisions and is easy to regress.

## Migration Plan
1. Add shared scaffold + pagination + empty-state variant support with tests.
2. Migrate Notifications Queue first (lowest coupling).
3. Migrate Agents list page.
4. Migrate Jobs list panel (desktop/mobile code paths).
5. Remove now-redundant per-page wrappers and align tests.

## Risks / Trade-offs
- Risk: over-abstraction may make edge cases harder.
  - Mitigation: keep scaffold slot-based and avoid forcing list/table internals.
- Risk: migration may accidentally change spacing in existing screenshots/tests.
  - Mitigation: page-by-page migration with focused UI tests.
- Risk: pagination reset behavior changes can surprise users.
  - Mitigation: preserve current page-reset-on-filter-change semantics where already present.

## Open Questions
- Whether to keep `AppEmptyState` default as `card` for backwards compatibility or switch to `plain` with explicit opt-in card usage.
