## Context
The current Web UI already has shared primitives (`useListFilters`, shared list chip rows, and latest-request helpers), but key flows still mix shared and custom implementations:
- `RunEntriesPickerModal` lacks stale-response protection.
- Picker selection and request query translation are duplicated.
- Jobs filter chips/count/clear logic lives in a custom composable.
- Abort-error checks and debounced refresh scheduling are duplicated in list pages.

These gaps increase maintenance cost and can cause subtle behavior drift across comparable screens.

## Goals / Non-Goals
- Goals:
  - Standardize filter state modeling on shared infrastructure.
  - Prevent stale list responses from overriding newer UI state.
  - Remove duplicated selection/query/debounce/abort helper logic.
  - Keep current API contracts and UX semantics unchanged.
- Non-Goals:
  - Changing backend list/filter API contracts.
  - Introducing virtual list/windowing behavior.
  - Introducing new bundle chunk strategies or lazy-load tuning.

## Decisions
- Decision: use latest-request style stale guards in `RunEntriesPickerModal` for refresh and load-more.
  - Rationale: consistent with existing picker modal request safety; lowest behavior risk.
- Decision: extract picker loaded-row selection operations into a reusable composable.
  - Rationale: both picker modals already share near-identical logic and keyboard range semantics.
- Decision: migrate Jobs filters to `useListFilters` and shared active-filter row component.
  - Rationale: enforce one filter modeling pattern across list pages.
- Decision: move filter chip type definitions from picker component exports to `lib` type module.
  - Rationale: remove cross-layer coupling (lib depending on component module type export).
- Decision: centralize list query parameter building for picker-like requests.
  - Rationale: avoid repeated ad-hoc param mapping for size/sort/search filters.
- Decision: centralize debounce + abort-error utilities in `lib`.
  - Rationale: reduce repeated timeout and abort detection code across list pages.

## Risks / Trade-offs
- Shared helper extraction can unintentionally alter serialization defaults.
  - Mitigation: add/keep request URL assertions in existing picker tests.
- Jobs filter migration can alter chip ordering/count semantics.
  - Mitigation: keep existing defaults and add regression tests for active count + chips + clear.
- Selection composable migration can regress shift-range behavior.
  - Mitigation: preserve current row order semantics and port existing tests.

## Migration Plan
1. Add shared helper/type/composable modules with focused unit tests.
2. Migrate `RunEntriesPickerModal` and `PathPickerModal` to shared selection/query helpers + stale guards.
3. Migrate Jobs filter modeling/chip rendering to shared filter model.
4. Migrate duplicated debounce/abort helpers in list views.
5. Run targeted UI tests, then full `scripts/ci.sh`.

## Open Questions
- None.
