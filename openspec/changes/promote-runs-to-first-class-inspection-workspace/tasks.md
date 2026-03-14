## 1. Spec Foundation

- [ ] 1.1 Finalize proposal, design, and spec deltas for `runs-workspace` and `run-diagnostics`
- [ ] 1.2 Run `openspec validate promote-runs-to-first-class-inspection-workspace --strict`

## 2. Run Workspace Read Models

- [ ] 2.1 Add the top-level Runs list and dedicated run-detail view models, including structured root-cause fields
- [ ] 2.2 Add backend tests for cross-job/node filtering, run-detail response shape, and structured diagnostics fallbacks

## 3. Run Event Console Contract

- [ ] 3.1 Implement server-driven event filtering, pagination/cursor semantics, and first-error location support
- [ ] 3.2 Add API and backend tests for event-console query behavior and authenticated access

## 4. Runs UI

- [ ] 4.1 Build the top-level Runs index on desktop and mobile
- [ ] 4.2 Build the dedicated run-detail workspace with summary-first diagnostics and direct restore/verify/cancel actions
- [ ] 4.3 Update Jobs and Command Center deep links to open the dedicated run routes

## 5. Validation

- [ ] 5.1 Add or update UI tests for run list filters, run-detail rendering, mobile layout, and event-console behavior
- [ ] 5.2 Run targeted backend and UI tests plus broader verification for temporary run-path alias coverage and eventual canonical-route cleanup
