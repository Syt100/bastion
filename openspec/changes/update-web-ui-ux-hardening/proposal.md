# Change: Update Web UI UX Hardening (Titles, A11y Focus, Empty/Loading, Request Cancellation)

## Why
Even with a responsive layout, a few UX details can still degrade perceived quality:
- Browser chrome details (document title/theme-color) do not reflect the current page/theme.
- Keyboard focus visibility is inconsistent when Tailwind preflight is disabled.
- Some screens can feel blank during loading or unclear when empty.
- Rapid filter/page changes can cause overlapping requests and stale UI updates.

We want a hardened baseline so future feature work inherits good UX by default.

## What Changes
- Add route-driven document titles (localized) and update theme-color for light/dark.
- Add a global, accessible focus-visible outline style and respect `prefers-reduced-motion`.
- Introduce a shared empty-state component and apply it to list pages where applicable.
- Add a “latest request wins” helper based on `AbortController` and apply it to pages that trigger rapid refreshes (e.g. notification queue filtering/paging).

## Impact
- Affected specs: `web-ui`
- Affected code: `ui` app shell, styles, views, i18n, and unit tests

