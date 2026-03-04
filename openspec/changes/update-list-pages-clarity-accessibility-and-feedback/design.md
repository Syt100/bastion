## Context
This change follows the recent shared scaffold/pagination migration. The remaining issues are mostly interaction semantics and information hierarchy across the same three list-heavy pages: Jobs, Agents, and Notifications Queue.

Constraints:
- Keep existing API contracts unchanged.
- Keep desktop/table/list mode behavior stable.
- Avoid high-risk rendering changes (for example, virtualization) in this iteration.

## Goals / Non-Goals
- Goals:
  - Make list-state feedback (result metrics, active filters, pagination ranges, empty states) explicit.
  - Improve accessibility semantics for Jobs row activation and nested action boundaries.
  - Reduce mobile scan noise while preserving action completeness.
  - Provide consistent in-flight affordances for key async row actions.
  - Standardize search debounce timing across relevant list pages.
- Non-Goals:
  - Virtual scrolling.
  - API/schema updates.

## Decisions
- Decision: represent pagination context via a shared, translated range label (`start-end / total`) passed through `AppPagination.totalLabel`.
  - Why: keeps a single footer component while avoiding component-level i18n coupling.
- Decision: Jobs row activation becomes explicit via dedicated row-main buttons instead of whole-row pseudo-buttons.
  - Why: avoids nested interactive-role conflicts and improves keyboard semantics.
- Decision: keep async feedback local to row-level controls with per-row busy maps.
  - Why: prevents duplicate requests without blocking unrelated list interactions.
- Decision: add a shared debounce constant for list search refresh cadence.
  - Why: preserves consistency and avoids divergent "feel" between Jobs and Agents.
- Decision: use progressive disclosure for secondary mobile metadata in Agents cards via native details/summary.
  - Why: lower visual noise with minimal state complexity.

## Risks / Trade-offs
- Moving from whole-row click targets to row-main buttons slightly changes hit-area expectations.
  - Mitigation: keep button area broad and preserve row layout cues.
- Additional i18n keys require locale parity.
  - Mitigation: update both `en-US` and `zh-CN` together and cover with existing locale tests.

## Migration Plan
1. Update Jobs/Agents/Notifications list views and shared helpers.
2. Update tests for row interaction boundaries, empty states, and pagination labels.
3. Validate with UI tests + full CI.

## Open Questions
- None for this implementation pass.
