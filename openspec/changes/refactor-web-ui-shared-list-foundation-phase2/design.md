## Context
Multiple UI surfaces already moved to shared filter chips and pagination components, but code duplication persists at the composable/model layer (route query parsing, busy maps, picker reset blocks, query serialization). This phase focuses on reducing repeat logic and lowering drift risk while preserving existing UX and backend contracts.

## Goals / Non-Goals
- Goals:
  - Reduce repeated logic for list/picker state management.
  - Keep user-visible behavior stable while standardizing implementation paths.
  - Improve maintainability by splitting oversized page components into smaller reusable units.
- Non-Goals:
  - No API contract changes.
  - No bundle-splitting / lazy loading / virtual scrolling work.

## Decisions
- Decision: Add narrowly scoped shared helpers/composables (`route query parse`, `id busy`, `picker reset`, `query serialization`) rather than introducing a broad generic framework.
  - Alternatives considered: keep local page logic and patch incrementally; rejected because drift reappears quickly.
- Decision: Extract Jobs row/table rendering into dedicated child components while keeping existing page-level orchestration in `JobsWorkspaceShellView`.
  - Alternatives considered: fully rewrite Jobs workspace structure; rejected to reduce regression risk.
- Decision: Standardize pagination options and behavior through shared constants and `AppPagination` in list pages that already use server pagination.
  - Alternatives considered: leave page-local pagination where currently functional; rejected due interaction inconsistency.

## Risks / Trade-offs
- Risk: Component extraction can break interaction wiring (selection, row actions, accessibility labels).
  - Mitigation: keep public props/events explicit; add focused regression tests around row actions and selection.
- Risk: Shared query helpers may accidentally alter request parameter names/default behavior.
  - Mitigation: preserve existing key names and add unit tests on serialized params.
- Risk: Route-query parsing unification can change edge-case fallback behavior.
  - Mitigation: encode previous valid-value allowlists and add tests for unknown/invalid query values.

## Migration Plan
1. Add shared helper/composable modules with tests.
2. Migrate pages/components one by one and keep existing semantics.
3. Run UI tests + full `scripts/ci.sh` before push.

## Open Questions
- None at proposal time.
