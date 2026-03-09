# Change: Improve dashboard first-screen performance and loading experience

## Why
After moving agents/jobs lists to server-side pagination, dashboard is still the first page most users land on and remains a critical performance path. There are still practical gaps:

- Desktop recent-runs rendering depends on `NDataTable` and is eagerly coupled into the dashboard route path.
- Lower-priority dashboard sections do not uniformly defer heavy UI modules until users are likely to view them.
- Deferred sections currently rely on plain skeleton blocks; adding a lightweight loading animation can improve perceived responsiveness without adding heavy assets.
- Viewport-lazy logic is duplicated and ad-hoc, making future maintenance riskier.

## What Changes
- Defer desktop recent-runs table into an async component and activate it only when the recent section reaches viewport proximity.
- Keep trend chart viewport-lazy behavior and migrate dashboard lazy-section activation to a shared helper.
- Add lightweight animated loading indicator(s) for deferred dashboard sections.
- Improve dashboard refresh UX by wiring refresh button loading state to avoid repeated blind clicks.
- Add regression tests for the shared viewport-lazy helper and dashboard deferred-loading behavior touchpoints.

## Impact
- Affected specs: `web-ui`
- Affected areas (representative):
  - `ui/src/views/DashboardView.vue`
  - `ui/src/components/dashboard/DashboardRecentRunsDesktopTable.vue`
  - `ui/src/components/loading/InlineLoadingDots.vue`
  - `ui/src/lib/viewportLazyReady.ts`
  - `ui/src/lib/viewportLazyReady.spec.ts`
  - `ui/src/i18n/locales/en-US.ts`
  - `ui/src/i18n/locales/zh-CN.ts`

## Non-Goals
- No dependency vulnerability governance changes.
- No backend API contract/schema changes.
- No redesign of dashboard information architecture.
