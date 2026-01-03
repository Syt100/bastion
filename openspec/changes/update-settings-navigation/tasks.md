## 1. Spec
- [ ] 1.1 Add Web UI spec deltas for Settings sidebar submenu + overview + mobile list-first navigation
- [ ] 1.2 Run `openspec validate update-settings-navigation --strict`

## 2. Web UI
- [ ] 2.1 Router: add Settings overview and Notifications index routes (remove redirects)
- [ ] 2.2 Layout: desktop sidebar uses Settings submenu with Overview/Storage/Notifications; Settings parent does not navigate
- [ ] 2.3 Views: Settings overview list page; Notifications index list page; mobile list-first navigation
- [ ] 2.4 Update i18n strings for new labels/descriptions (zh-CN default + en-US)
- [ ] 2.5 Add/adjust unit tests (Vitest) for key navigation flows

## 3. Validation
- [ ] 3.1 Run `npm test` (ui)
- [ ] 3.2 Run `npm run build` (ui)
- [ ] 3.3 Run `npm run lint` (ui)

## 4. Commits
- [ ] 4.1 Commit the spec proposal (detailed message)
- [ ] 4.2 Commit Web UI changes (detailed message with Modules/Tests)

