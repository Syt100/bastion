## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for the redesigned Run Detail layout (hero first screen + details tabs)
- [x] 1.2 Run `openspec validate update-run-detail-view-redesign --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Rework Run Detail layout: denser overview + progress first screen (desktop 2-col / mobile 1-col)
- [ ] 2.2 Consolidate Events / Operations / Summary into a tabbed Details area
- [ ] 2.3 Summary: hide empty blocks; keep raw JSON accessible with copy affordance
- [ ] 2.4 Adjust i18n strings (zh-CN/en-US) as needed

## 3. Tests
- [ ] 3.1 Update/add unit tests for the Run Detail redesign (tabs + empty states + summary rendering)

## 4. Validation
- [ ] 4.1 Run `npm test --prefix ui`
- [ ] 4.2 Run `npm --prefix ui run build`
- [ ] 4.3 Run `cargo test -p bastion-http`

## 5. Commits
- [ ] 5.1 Commit implementation changes (detailed message with Modules/Tests)
