## 1. Spec
- [ ] 1.1 Add Web UI spec deltas for Settings mobile TopBar (fixed layout, centered title, route-meta driven)
- [ ] 1.2 Run `openspec validate update-settings-mobile-topbar --strict`

## 2. Web UI
- [ ] 2.1 Add shared `MobileTopBar` component with fixed 3-region layout (left/back, center/title, right/empty)
- [ ] 2.2 Router: add `meta.mobileTopBar` for `/settings/**` routes with `titleKey` and optional `backTo`
- [ ] 2.3 SettingsShell: render TopBar on mobile using deepest matched meta; hide PageHeader subtitle on mobile
- [ ] 2.4 Remove any per-subpage mobile back bars under `/settings/**` (ensure a single consistent pattern)
- [ ] 2.5 Add/adjust unit tests for navigation/back behavior if needed

## 3. Validation
- [ ] 3.1 Run `npm test` (ui)
- [ ] 3.2 Run `npm run build` (ui)
- [ ] 3.3 Run `npm run lint` (ui)

## 4. Commits
- [ ] 4.1 Commit the spec proposal (detailed message)
- [ ] 4.2 Commit Web UI changes (detailed message with Modules/Tests)

