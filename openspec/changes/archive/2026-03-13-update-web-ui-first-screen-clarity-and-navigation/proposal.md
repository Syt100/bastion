# Change: Improve web UI first-screen clarity and navigation flow

## Why
Recent UI foundation work improved consistency, but the current first-screen experience still makes users work too hard to understand status, navigation context, and next actions. The dashboard prioritizes statistics over actionable signals, the Jobs workspace exposes too many top-level mode switches, and mobile/global navigation splits key actions across separate menus. The login and empty states also feel sparse for a backup product that should communicate trust and guidance quickly.

## What Changes
- Rebalance global shell chrome so the content area reads as primary and desktop/mobile global actions are grouped more cleanly.
- Rework dashboard first-screen information architecture to prioritize actionable status and recent activity before lower-priority trend detail.
- Simplify Jobs workspace top-level controls by collapsing redundant layout/view choices into a clearer primary workflow.
- Improve Agents empty-state onboarding and filter/action hierarchy so first-time setup and ongoing management are easier to understand.
- Strengthen login-page trust cues and guidance without changing authentication behavior.
- Add accessibility refinements for main landmarks and navigation/action grouping.

## Impact
- Affected specs: `web-ui`
- Affected code (representative):
  - `ui/src/layouts/AppShell.vue`
  - `ui/src/views/DashboardView.vue`
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/views/AgentsView.vue`
  - `ui/src/views/LoginView.vue`
  - `ui/src/components/AuthLayout.vue`
  - `ui/src/styles/main.css`
  - `ui/src/i18n/locales/en-US.ts`
  - `ui/src/i18n/locales/zh-CN.ts`
  - `CHANGELOG.md`

## Non-Goals
- No backend API or schema changes.
- No new backup/runtime capabilities.
- No replacement of the existing theme system or navigation structure with a new framework.
