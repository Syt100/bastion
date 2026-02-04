## 1. Implementation

- [x] 1.1 Add `backgroundStyle` to `useUiStore` and persist to localStorage.
- [x] 1.2 Apply `data-bg` in early bootstrap (`ui/src/theme/bootstrap.ts`) to avoid background flashes.
- [x] 1.3 Add background style token overrides in `ui/src/styles/main.css` (solid/plain + neutral base).
- [x] 1.4 Update `ui/src/App.vue` to keep `data-bg` and `meta[name="theme-color"]` in sync at runtime.
- [x] 1.5 Add background style selector UI in Appearance settings and ensure preview respects it.
- [x] 1.6 Add/update tests (store defaults + App dataset behavior) and run `npm test` + `scripts/ci.sh`.
- [x] 1.7 Fix dark-mode CSS override specificity so solid/plain work in dark themes.
- [x] 1.8 In `plain` mode, neutralize UI surfaces/chrome (cards + glass navigation) to avoid theme-tinted dark surfaces.
- [x] 1.9 Add/update regression tests for `plain` surface/chrome overrides.
