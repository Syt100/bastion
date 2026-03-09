# Change: Update Web UI Frontend Optimizations (Code Splitting, Modularity, Auth UX)

## Why
The current Web UI works well, but a few frontend non-functional issues remain:
- The initial JS bundle is large because all page views are eagerly imported.
- The Dashboard includes the chart library in the main bundle.
- `JobsView.vue` is very large, making it harder to maintain and extend safely.
- Common utilities (date formatting, clipboard copy, CSRF token usage) are duplicated across pages/stores.
- When sessions expire, 401s can degrade UX without a clear redirect back to login.
- Some demo chart strings are not localized.
- When no theme preference is stored, the UI does not follow the system theme.

## What Changes
- Route-level code splitting: lazy-load view components with dynamic imports.
- Defer ECharts loading: load chart components only when needed.
- Refactor the Jobs page into smaller components/composables while keeping behavior unchanged.
- Centralize common utilities: timestamp formatting, clipboard copy, CSRF token injection, and unauthorized handling.
- Localize demo chart labels for `zh-CN` and `en-US`.
- Default theme mode follows `prefers-color-scheme` when no explicit preference is stored.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui` (router, dashboard, jobs view/components, stores, i18n)

