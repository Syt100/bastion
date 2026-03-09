# Change: Improve first-paint UX and list scalability in Web UI

## Why
Recent improvements made list refresh semantics safer, but there are still user-visible and maintainability gaps:

- Dashboard still has a heavy chart payload path and lacks dedicated first-paint skeletons for key cards.
- Refresh cancellation logic is duplicated across stores, increasing future drift risk.
- Large list rendering currently paints full filtered sets in both jobs and agents views, which scales poorly.
- Filter-driven refresh triggers are immediate and can issue redundant requests during rapid changes.
- Several UI tests duplicate Naive UI stubs and route meta config has repeated blocks, raising maintenance cost.

## What Changes
- Add dashboard first-paint skeleton loading UI and defer trend chart mount until chart section enters viewport.
- Add Vite `manualChunks` rules to isolate heavy chart dependencies from main entry chunks.
- Introduce a shared latest-request cancellation helper and apply it to `agents`, `jobs`, and `dashboard` stores.
- Add/adjust store regression tests for cancellation and stale-order semantics, including dashboard refresh overlap.
- Add client-side pagination in jobs and agents list views to reduce DOM/render pressure on large result sets.
- Debounce high-frequency filter-triggered refresh on agents view and parallelize mount-time refreshes where safe.
- Create shared Naive UI test-stub helpers and migrate repeated view specs to use them.
- Refactor router route meta duplication via small shared meta helper factories.

## Impact
- Affected specs: `web-ui`, `dev-workflow`
- Affected areas (representative):
  - `ui/src/views/DashboardView.vue`
  - `ui/vite.config.ts`
  - `ui/src/lib/latest.ts`
  - `ui/src/stores/agents.ts`
  - `ui/src/stores/jobs.ts`
  - `ui/src/stores/dashboard.ts`
  - `ui/src/views/AgentsView.vue`
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/router/index.ts`
  - `ui/src/test-utils/naiveUiStubs.ts`

## Non-Goals
- No backend API schema or endpoint changes.
- No dependency vulnerability governance changes.
- No server-side pagination implementation in this proposal.
