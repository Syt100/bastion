## Context
The UI already has theme presets and shared list scaffolds, but visual hierarchy and density rules are not fully codified. As a result, text levels, card emphasis, and vertical rhythm vary between pages.

## Goals / Non-Goals
- Goals:
  - Make hierarchy explicit and reusable.
  - Reduce decorative noise while preserving theme identity.
  - Keep dense list data readable on desktop and mobile.
- Non-Goals:
  - No API changes.
  - No redesign of IA/navigation.

## Decisions
- Decision: Introduce shared utility classes in `main.css` for hierarchy and metadata, then consume from shared components (`PageHeader`, `ListPageScaffold`, `ListToolbar`, `AppPagination`) instead of page-local class tuning.
- Decision: Keep existing theme IDs and semantics; only tune intensity and chrome emphasis.
- Decision: Express spacing rhythm through shared component-level class updates to avoid per-page divergence.

## Risks / Trade-offs
- Risk: Global style changes can unintentionally impact unrelated views.
  - Mitigation: limit new classes to prefixed app utilities and add CSS regression tests.

## Migration Plan
1. Add visual foundation classes/tokens in `main.css`.
2. Migrate shared list/page shell components first.
3. Migrate Jobs/Agents row and table secondary metadata styles.
4. Validate with existing unit/regression suite.
