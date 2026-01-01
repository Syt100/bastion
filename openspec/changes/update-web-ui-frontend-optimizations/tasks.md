## 1. Spec
- [ ] 1.1 Add `web-ui` spec delta for: route-level code splitting, deferred chart loading, Jobs modularization, shared utilities, 401 redirect behavior, chart i18n, and system-theme default
- [ ] 1.2 Run `openspec validate update-web-ui-frontend-optimizations --strict`

## 2. Web UI - Performance
- [ ] 2.1 Convert router view imports to dynamic imports (code-splitting)
- [ ] 2.2 Defer ECharts/chart component loading on Dashboard
- [ ] 2.3 Update/extend unit tests as needed
- [ ] 2.4 Run `npm test` and `npm run build` (ui)
- [ ] 2.5 Commit UI performance changes (detailed message)

## 3. Web UI - Jobs Modularity
- [ ] 3.1 Extract Jobs page sub-areas into dedicated components/composables (editor, runs, events, restore/verify, operation)
- [ ] 3.2 Keep current behavior (WS updates, polling, validation, i18n) unchanged
- [ ] 3.3 Update/extend unit tests for the refactor
- [ ] 3.4 Run `npm test` and `npm run build` (ui)
- [ ] 3.5 Commit Jobs refactor changes (detailed message)

## 4. Web UI - Shared Utilities & Auth UX
- [ ] 4.1 Extract shared helpers: unix timestamp formatting and clipboard copy
- [ ] 4.2 Centralize CSRF token acquisition and remove per-store duplication
- [ ] 4.3 On API 401, transition to anonymous + redirect to `/login` (single place)
- [ ] 4.4 Default theme follows system preference if no stored preference exists (with unit test)
- [ ] 4.5 Move chart labels to i18n (`zh-CN`/`en-US`)
- [ ] 4.6 Run `npm test` and `npm run build` (ui)
- [ ] 4.7 Commit utilities/auth/theme/i18n changes (detailed message)

## 5. Commits
- [ ] 5.1 Commit the spec proposal (detailed message)

