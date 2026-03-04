## Context
While major UX behavior is already aligned, polished consistency depends on shared visual language for iconography, modal shells, and micro-motion. These currently remain partly page-local.

## Goals / Non-Goals
- Goals:
  - Normalize icon size/tone across common interactive controls.
  - Make modal scaffolding visually and structurally consistent.
  - Apply motion feedback uniformly while honoring reduced-motion preferences.
- Non-Goals:
  - No icon library migration.
  - No animation-heavy redesign.

## Decisions
- Decision: Add `AppIcon` wrapper to centralize size/tone semantics over `NIcon`.
- Decision: Add `AppModalShell` wrapper over `NModal` preset card to standardize spacing, footer alignment, and scroll containment.
- Decision: Add app-scoped motion utilities in `main.css` and consume through shared/list components.

## Risks / Trade-offs
- Risk: Wrapper components may reduce flexibility for edge cases.
  - Mitigation: keep wrappers slot-driven and allow opt-in custom widths/content classes.

## Migration Plan
1. Introduce `AppIcon` and `AppModalShell` components.
2. Migrate high-traffic controls and modals (Jobs/Agents/filter trigger).
3. Add/apply shared motion utilities.
4. Validate with unit tests and full frontend test suite.
