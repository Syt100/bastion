# Change: Harden UI quality gates and list refresh stability

## Why
The current project checks are strong on Rust and end-to-end behavior, but there are still two practical gaps in day-to-day frontend quality:

- `scripts/ci.sh` does not run UI lint or TypeScript type-check, so static regressions can slip through while tests still pass.
- The Jobs/Agents list stores can accept out-of-order refresh responses, which may briefly show stale results when users change filters quickly.

These issues increase regression risk and create avoidable operator confusion in high-frequency list workflows.

## What Changes
- Add explicit UI static checks to CI workflow:
  - run a non-mutating ESLint check for `ui/`
  - run Vue/TypeScript type-check for `ui/`
- Keep a local autofix lint command for developer convenience, while adding a CI-safe lint check command.
- Fix existing frontend lint/type-check violations so the new CI gates are green.
- Update `agents` and `jobs` list store refresh behavior to be "latest request wins":
  - stale refresh responses MUST NOT overwrite newer results
  - stale refresh failures MUST NOT replace newer successful state
- Add regression unit tests for stale refresh ordering in affected stores.

## Impact
- Affected specs: `dev-workflow`, `web-ui`
- Affected areas (representative):
  - `scripts/ci.sh`
  - `ui/package.json`
  - `ui/src/stores/agents.ts`
  - `ui/src/stores/jobs.ts`
  - `ui/src/stores/agents.spec.ts`
  - `ui/src/stores/jobs.spec.ts`
  - `ui/src/views/jobs/JobsWorkspaceShellView.vue`
  - `ui/src/views/jobs/JobsWorkspaceShellView.spec.ts`
  - `ui/src/AppTheme.spec.ts`

## Non-Goals
- No backend API schema or endpoint changes.
- No redesign of list filtering UX.
- No virtualization or server-side pagination work in this change.
