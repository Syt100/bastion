## 1. Spec
- [ ] 1.1 Add `web-ui` spec delta for shared UI surface styles, i18n document-lang syncing, i18n key parity tests, icon-button a11y labels, and dashboard chart loading fallback
- [ ] 1.2 Run `openspec validate update-web-ui-quality-pass --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI - Shared Styles
- [ ] 2.1 Add reusable surface utility classes (e.g. `app-card`) in UI styles
- [ ] 2.2 Replace duplicated card/panel class strings across views/components with the shared utilities
- [ ] 2.3 Commit shared-style refactor (detailed message)

## 3. Web UI - i18n Guardrails
- [ ] 3.1 Sync `<html lang>` with the active UI locale
- [ ] 3.2 Add unit test enforcing i18n key parity between `zh-CN` and `en-US`
- [ ] 3.3 Commit i18n guardrails (detailed message)

## 4. Web UI - Accessibility
- [ ] 4.1 Add `aria-label` for icon-only navigation/header buttons and localize labels
- [ ] 4.2 Commit a11y label improvements (detailed message)

## 5. Web UI - Dashboard Loading Polish
- [ ] 5.1 Add a Dashboard chart fallback (skeleton/placeholder) while the async chart component loads
- [ ] 5.2 Commit dashboard loading polish (detailed message)

## 6. Validation
- [ ] 6.1 Run `npm test --prefix ui`
- [ ] 6.2 Run `npm run lint --prefix ui`
- [ ] 6.3 Run `npm run build --prefix ui`

