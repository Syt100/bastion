## 1. Spec Foundation

- [x] 1.1 Finalize proposal, design, and spec deltas for `runs-workspace` and `run-diagnostics`
- [x] 1.2 Run `openspec validate promote-runs-to-first-class-inspection-workspace --strict`

## 2. Run Workspace Read Models

- [x] 2.1 Add the top-level Runs list and dedicated run-detail view models, including structured root-cause fields
- [x] 2.2 Add backend tests for cross-job/node filtering, run-detail response shape, and structured diagnostics fallbacks

## 3. Run Event Console Contract

- [x] 3.1 Implement server-driven event filtering, pagination/cursor semantics, and first-error location support
- [x] 3.2 Add API and backend tests for event-console query behavior and authenticated access

## 4. Runs UI

- [x] 4.1 Build the top-level Runs index on desktop and mobile
- [x] 4.2 Build the dedicated run-detail workspace with summary-first diagnostics and direct restore/verify/cancel actions
- [x] 4.3 Update Jobs and Command Center deep links to open the dedicated run routes

## 5. Validation

- [x] 5.1 Add or update UI tests for run list filters, run-detail rendering, mobile layout, and event-console behavior
- [x] 5.2 Run targeted backend and UI tests plus broader verification for temporary run-path alias coverage and eventual canonical-route cleanup
