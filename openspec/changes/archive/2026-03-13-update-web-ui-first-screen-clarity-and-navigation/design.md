## Context
The current UI is functional and visually consistent enough to operate, but several key journeys still feel dense or under-guided: landing on the dashboard, switching between nodes on mobile, entering the Jobs workspace, onboarding the first agent, and logging into the product. The requested change is intentionally cross-cutting and mostly front-end only.

## Goals / Non-Goals
- Goals:
  - Make first-screen pages easier to scan and act on.
  - Reduce redundant or competing top-level controls.
  - Unify global/mobile navigation and action discoverability.
  - Increase perceived trust and guidance on the login and empty states.
  - Improve accessibility semantics while preserving current routes and behavior.
- Non-Goals:
  - Rebuilding page flows from scratch.
  - Changing persisted data models or server contracts.
  - Adding a new design system beyond targeted token/layout adjustments.

## Decisions
- Decision: Keep existing routes and core shared components, but adjust shell composition and page ordering.
  - Rationale: Minimizes risk and preserves existing user muscle memory while still improving readability.
- Decision: Simplify Jobs workspace by presenting a single primary “workspace” path on desktop and demoting redundant layout distinctions.
  - Rationale: Users care about completing job tasks, not choosing between multiple structural modes.
- Decision: Keep dashboard trend visualization, but move it behind higher-priority health and recent activity surfaces.
  - Rationale: Trend is useful context but rarely the first action driver.
- Decision: Treat empty states as onboarding/action surfaces rather than passive placeholders.
  - Rationale: Empty pages are common in first-run environments and should actively guide setup.

## Risks / Trade-offs
- Shell/navigation changes can affect multiple pages unexpectedly.
  - Mitigation: keep route behavior unchanged and add targeted tests around shell semantics and visible actions.
- Jobs workspace simplification may conflict with existing layout-mode persistence expectations.
  - Mitigation: map old persisted states to the new presentation instead of breaking stored preferences.
- Stronger first-screen hierarchy may reduce density for power users.
  - Mitigation: preserve key quick actions while improving grouping rather than removing core capabilities.

## Migration Plan
1. Add OpenSpec proposal/tasks/spec delta.
2. Update shared shell and styling tokens/classes.
3. Refactor dashboard and jobs page hierarchy.
4. Refine agents and auth empty/trust states.
5. Update tests and changelog.

## Open Questions
- Whether desktop topbar should eventually expose a user/settings menu instead of direct action buttons. This change keeps the existing actions but groups them more cleanly.
