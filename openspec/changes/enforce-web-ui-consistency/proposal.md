# Change: Enforce Web UI visual consistency (design system rules)

## Why

The Web UI already has a token-based theme system (CSS variables + Naive UI overrides), but visual decisions can still drift page-by-page when contributors use ad-hoc patterns (e.g., mixed card borders, non-token colors, inconsistent “muted” text).

This change establishes an authoritative, long-lived visual rule set and applies a small set of mechanical refactors so that the consistent choice is also the easiest choice.

## What Changes

- Add an authoritative Web UI style guide (English + Chinese) for contributors.
- Surface the style guide in the docs site navigation.
- Standardize the “content card” pattern:
  - Any `n-card` using the shared `app-card` class MUST be `:bordered="false"` (elevation is from `app-card`).
- Add a regression test to prevent reintroducing inconsistent `app-card` usage.

## Impact

- Affected spec capability: `web-ui-design-system`
- Affected docs:
  - `docs/dev/design/web-ui-style-guide.md`
  - `docs/zh/dev/design/web-ui-style-guide.md`
  - VitePress nav (`docs/.vitepress/config.ts`)
- Affected UI code: multiple `.vue` files in `ui/src/` using `n-card` + `app-card`

