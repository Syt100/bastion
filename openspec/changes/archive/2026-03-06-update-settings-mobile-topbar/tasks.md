## 1. Spec
- [x] 1.1 Add Web UI spec deltas for Settings mobile TopBar (fixed layout, centered title, route-meta driven)
- [x] 1.2 Run `openspec validate update-settings-mobile-topbar --strict`

## 2. Web UI
- [x] 2.1 Add shared `MobileTopBar` component with fixed 3-region layout (left/back, center/title, right/empty)
- [x] 2.2 Router: add `meta.mobileTopBar` for `/settings/**` routes with `titleKey` and optional `backTo`
- [x] 2.3 SettingsShell: render TopBar on mobile using deepest matched meta; hide PageHeader subtitle on mobile
- [x] 2.4 Remove any per-subpage mobile back bars under `/settings/**` (ensure a single consistent pattern)
- [x] 2.5 Add/adjust unit tests for navigation/back behavior if needed

## 3. Validation
- [x] 3.1 Run `npm test` (ui)
- [x] 3.2 Run `npm run build` (ui)
- [x] 3.3 Run `npm run lint` (ui)

## 4. Commits
- [x] 4.1 Commit the spec proposal (detailed message)
- [x] 4.2 Commit Web UI changes (detailed message with Modules/Tests)
