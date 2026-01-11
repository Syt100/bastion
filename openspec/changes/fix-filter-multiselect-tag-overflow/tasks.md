## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for compact multi-select tag display
- [x] 1.2 Run `openspec validate fix-filter-multiselect-tag-overflow --strict`

## 2. UI
- [ ] 2.1 Apply `max-tag-count=\"responsive\"` (or equivalent) on multi-select filters
- [ ] 2.2 Confirm control height remains compact with many selections

## 3. Verification
- [ ] 3.1 Verify visually (desktop + mobile breakpoints)
- [ ] 3.2 Run `cd ui && npm test` and `npm run type-check`
