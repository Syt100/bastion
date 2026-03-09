# Change: Refactor Web UI Navigation Lists (Notifications + Language Options)

## Why
The Web UI currently repeats “the same list” in multiple places:
- Notifications settings subpages are defined independently in the mobile index list, the desktop tab bar, and the router.
- Language dropdown options are defined independently in AppShell and AuthLayout.

This duplication is easy to forget when adding or removing entries, which can lead to UI drift (missing links, missing tabs, or inconsistent labels).

## What Changes
- Notifications navigation:
  - Introduce a single shared Notifications navigation config.
  - Refactor Notifications index list and desktop tabs to render from that config.
  - Add unit tests to ensure all configured subpages resolve in the router and remain consistent.
- Language options:
  - Introduce a single shared locale options helper derived from `supportedLocales`.
  - Refactor AppShell/AuthLayout (and Naive UI locale mapping) to use that helper.
  - Add unit tests to ensure all supported locales have dropdown labels and Naive UI locale mappings.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui/src/views/settings/notifications/**`, `ui/src/layouts/**`, `ui/src/components/**`, `ui/src/i18n/**`

