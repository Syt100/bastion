# Change: Add Web UI background style options (aurora/solid/plain)

## Why

The current Web UI themes always render an aurora gradient background. Some environments prefer a more neutral backdrop:
- **Solid**: keep the theme’s solid base color, but remove the gradient.
- **Plain**: remove the gradient *and* remove theme-tinted background color, using a neutral white/black base instead.

This improves accessibility and reduces visual noise while keeping theme accents (primary colors, surfaces) intact.

## What Changes

- Add a persisted “Background style” setting, independent from `themeId`.
- Support 3 styles:
  - `aurora` (current default)
  - `solid` (no gradient)
  - `plain` (no gradient + neutral base)
- Implement via document-level data attributes and token overrides:
  - `data-bg="..."` controls `--app-bg` and `--app-bg-solid`.
- In `plain` mode, also neutralize UI surfaces/chrome so cards and navigation do not inherit theme-tinted dark surfaces.
- Update mobile browser chrome color (`meta[name="theme-color"]`) so `plain` does not tint the address bar.
- Add regression tests for the new setting and the document data attribute behavior.

## Impact

- Affected spec capability: `web-ui-design-system`
- Affected UI code:
  - Theme bootstrap: `ui/src/theme/bootstrap.ts`
  - App root: `ui/src/App.vue`
  - Theme tokens: `ui/src/styles/main.css`
  - Appearance settings UI: `ui/src/views/settings/AppearanceView.vue`
  - UI store persistence: `ui/src/stores/ui.ts`
