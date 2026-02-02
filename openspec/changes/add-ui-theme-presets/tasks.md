## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for: 6 theme presets, Mint Teal default, theme-specific aurora background, no custom editing, Naive UI integration, and theme-color meta behavior
- [x] 1.2 Run `openspec validate add-ui-theme-presets --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI - Theme Tokens and Application
- [x] 2.1 Add `themeId` preference to `ui` store with persistence + fallback to `mint-teal`
- [x] 2.2 Apply `data-theme` to `<html>` (and keep `.dark` behavior consistent across `<html>/<body>`)
- [x] 2.3 Define 6 preset theme token sets (light + dark), including `--app-bg-solid` and `--app-bg` aurora layers per theme
- [x] 2.4 Update `meta[name="theme-color"]` to follow the active theme (light: accent, dark: bg solid)
- [x] 2.5 Commit theme token + application changes (detailed message)

## 3. Web UI - Settings Appearance UI (Mobile-Friendly)
- [x] 3.1 Add an Appearance section in Settings with a theme picker (cards with swatches + name)
- [x] 3.2 Ensure responsive layout (mobile single-column; desktop multi-column)
- [x] 3.3 Add i18n strings for theme names and Appearance section labels (zh-CN + en-US at minimum)
- [x] 3.4 Commit Settings appearance UI (detailed message)

## 4. Web UI - Naive UI Overrides Integration
- [x] 4.1 Ensure Naive UI themeOverrides recompute on theme change (no stale colors)
- [x] 4.2 Keep the "no var(...)" guarantee for overrides
- [x] 4.3 Commit theme override integration updates (detailed message)

## 5. Tests
- [x] 5.1 Add store tests for theme persistence and fallback behavior
- [x] 5.2 Add a UI test to verify theme switching applies the expected `data-theme` and updates derived tokens
- [x] 5.3 Commit tests (detailed message)

## 6. Validation
- [x] 6.1 Run `npm test --prefix ui`
- [x] 6.2 Run `npm run lint --prefix ui`
- [x] 6.3 Run `npm run build --prefix ui`
- [x] 6.4 Run `bash scripts/ci.sh`
