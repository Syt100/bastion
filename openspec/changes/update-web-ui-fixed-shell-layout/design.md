## Context
This change targets a common admin/workbench UX failure mode: navigation and context UI scroll away on long pages.

## Goals / Non-Goals
- Goals:
  - Keep primary navigation and top bar accessible on desktop while viewing long content.
  - In the Jobs workspace, allow long History/Data content without losing the jobs list or the current job context.
  - Preserve good mobile ergonomics (single-column, drawer-based navigation).
- Non-Goals:
  - Pixel-perfect redesign of the overall visual system.
  - Introducing a new layout framework.

## Decisions
- Decision: Prefer **fixed shell + internal scroll containers** over body scrolling.
  - Rationale: This prevents layout “context loss” and matches operator workflows (scan content while keeping navigation/actions available).

- Decision: In the Jobs workspace (desktop), use **two independent scroll panes**.
  - Rationale: Jobs list and job detail serve different purposes; tying them to a single scroll causes unnecessary navigation friction.

- Decision: Use **sticky/pinned sub-headers** inside scroll panes.
  - Jobs list: sticky filter/search/sort.
  - Job workspace: sticky job header and section tabs.

## Risks / Trade-offs
- Nested scroll regions can create scroll chaining/scroll trapping if not implemented carefully.
  - Mitigation: ensure the shell disables body scroll on desktop and uses a single primary content scroller, and ensure pane scrollers use `min-height: 0` / `min-width: 0` constraints to avoid overflow bugs.

- Sticky positioning can fail when ancestors create new scrolling contexts or set `overflow: hidden` unintentionally.
  - Mitigation: explicitly control which containers scroll and keep sticky elements inside those containers.

## Migration Plan
- Update layout components; visually verify the following:
  - Header and sider remain visible during scroll.
  - Jobs list pane remains accessible while scrolling job details.
  - Job header/tabs remain visible while scrolling section content.
  - Mobile remains single-column and scrolls naturally.

## Open Questions
- Should the fixed shell scrolling pattern apply to *all* pages immediately, or only to Jobs first and expand later?
