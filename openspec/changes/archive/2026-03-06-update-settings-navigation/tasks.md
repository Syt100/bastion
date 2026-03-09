## 1. Spec
- [x] 1.1 Add Web UI spec deltas for Settings sidebar submenu + overview + mobile list-first navigation
- [x] 1.2 Run `openspec validate update-settings-navigation --strict`

## 2. Web UI
- [x] 2.1 Router: add Settings overview and Notifications index routes (remove redirects)
- [x] 2.2 Layout: desktop sidebar uses Settings submenu with Overview/Storage/Notifications; Settings parent does not navigate
- [x] 2.3 Views: Settings overview list page; Notifications index list page; mobile list-first navigation
- [x] 2.4 Update i18n strings for new labels/descriptions (zh-CN default + en-US)
- [x] 2.5 Add/adjust unit tests (Vitest) for key navigation flows

## 3. Validation
- [x] 3.1 Run `npm test` (ui)
- [x] 3.2 Run `npm run build` (ui)
- [x] 3.3 Run `npm run lint` (ui)

## 4. Commits
- [x] 4.1 Commit the spec proposal (detailed message)
- [x] 4.2 Commit Web UI changes (detailed message with Modules/Tests)
