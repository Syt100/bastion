# Change: Improve dashboard loading and refresh cancellation behavior

## Why
The current UI has three practical quality issues that impact responsiveness and development signal quality:

- Dashboard loads chart dependencies eagerly, producing an oversized route chunk and slower first render on dashboard entry.
- Jobs/Agents stores now ignore stale refresh results, but superseded requests still keep running, wasting network and backend resources.
- `AgentsView` unit test currently emits avoidable Vue prop warnings from a stubbed input component, adding noise in CI output.

## What Changes
- Defer loading of Dashboard chart visualization via async component loading with a loading fallback.
- Upgrade list refresh behavior in Jobs/Agents stores from “ignore stale response” to “cancel stale request + latest request wins”.
- Add regression tests covering canceled stale refresh requests in both stores.
- Fix the `AgentsView` test stub so it does not forward invalid native input props and keep the test warning-clean.

## Impact
- Affected specs: `web-ui`, `dev-workflow`
- Affected areas (representative):
  - `ui/src/views/DashboardView.vue`
  - `ui/src/stores/agents.ts`
  - `ui/src/stores/jobs.ts`
  - `ui/src/stores/agents.spec.ts`
  - `ui/src/stores/jobs.spec.ts`
  - `ui/src/views/AgentsView.spec.ts`

## Non-Goals
- No dependency security governance changes in this proposal.
- No backend API schema or endpoint changes.
- No redesign of dashboard information architecture.
