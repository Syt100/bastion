# Change: Update Settings Mobile TopBar (Route-Meta Driven, Center Title, No Layout Shift)

## Why
The Settings area currently shows a mobile “back” UI that appears/disappears depending on the page.
This causes visible layout shifts:
- The Settings title moves vertically when a separate back row is inserted/removed.
- The Settings title moves horizontally when a back button is conditionally rendered inside the header row.

These shifts make the UI feel unstable on mobile.

Also, as Settings grows (more categories and nested subpages), we need a consistent, scalable pattern that:
- works for any `/settings/**` page without bespoke per-page back bars,
- keeps the title centered, and
- supports future right-side actions while reserving layout space now.

## What Changes
- Introduce a dedicated mobile TopBar for the Settings route tree (`/settings/**`).
- The mobile TopBar:
  - has a fixed height (no vertical shift),
  - uses fixed left/right regions so the centered title never moves horizontally,
  - shows a back button when `backTo` exists; otherwise the left region remains reserved,
  - reserves a right-side actions region but leaves it empty for now (per product decision).
- TopBar configuration is derived from route metadata:
  - `titleKey`: i18n key for the title to display (centered)
  - `backTo`: optional route path to navigate back to
  - The deepest matched route provides the effective TopBar config.
- Mobile Settings pages do not display the desktop-style subtitle under the title.
  Any explanations are provided as regular in-page gray helper text where needed.

## Impact
- Affected specs: `web-ui`
- Affected code: `ui` (router meta, Settings shell layout, new shared component), i18n, and unit tests

