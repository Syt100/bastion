## 1. Spec
- [x] 1.1 Draft proposal/design/spec delta for first-screen clarity and navigation improvements
- [x] 1.2 Run `openspec validate update-web-ui-first-screen-clarity-and-navigation --strict`

## 2. Shell + Dashboard
- [x] 2.1 Rebalance global shell chrome and mobile/global action grouping
- [x] 2.2 Reorder dashboard first-screen sections to prioritize actionable status and recent activity
- [x] 2.3 Reduce dashboard perceived loading friction where possible without backend changes

## 3. Jobs + Agents + Auth
- [x] 3.1 Simplify Jobs workspace top-level mode controls and empty-state guidance
- [x] 3.2 Improve Agents onboarding/empty-state and filter/action hierarchy
- [x] 3.3 Strengthen login-page guidance and trust cues

## 4. Validation
- [x] 4.1 Add/update focused frontend regression tests for shell/page hierarchy changes
- [x] 4.2 Update `CHANGELOG.md` for user-visible UI improvements
- [x] 4.3 Run `npm --prefix ui run lint:check`
- [x] 4.4 Run `npm --prefix ui run type-check`
- [x] 4.5 Run `npm --prefix ui run test -- --run`
