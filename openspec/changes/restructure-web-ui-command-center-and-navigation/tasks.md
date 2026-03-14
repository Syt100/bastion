## 1. Spec Foundation

- [x] 1.1 Finalize proposal, design, and spec deltas for `command-center`, `app-shell-navigation`, and `web-ui-design-system`
- [x] 1.2 Run `openspec validate restructure-web-ui-command-center-and-navigation --strict`

## 2. Shell And Route Model

- [x] 2.1 Add the new top-level route map and navigation metadata for `Command Center`, `Jobs`, `Runs`, `Fleet`, `Integrations`, and `System`
- [x] 2.2 Implement shell-level scope selection state and contextual secondary navigation behavior
- [x] 2.3 Add only the minimal temporary client-side aliases needed for current internal node-scoped entry points, and plan their removal

## 3. Command Center Data Model

- [x] 3.1 Define and implement the aggregated `GET /api/command-center` read model with attention, activity, readiness, and watchlist sections
- [x] 3.2 Add backend tests for authenticated access, scope handling, and healthy/empty/degraded response shapes

## 4. Command Center UI

- [x] 4.1 Replace the current Dashboard landing page with the Command Center layout on desktop and mobile
- [x] 4.2 Add direct-action cards/rails for attention items, recent critical activity, and recovery-readiness summaries
- [x] 4.3 Update i18n, navigation copy, and shell affordances to match the new information architecture

## 5. Design-System And Validation

- [x] 5.1 Add shared shell/panel/rail primitives or recipes required by the new console hierarchy
- [x] 5.2 Add or update UI tests for desktop/mobile navigation and Command Center rendering
- [x] 5.3 Run targeted backend/UI tests plus any broader validation needed after the shell migration
