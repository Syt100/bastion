## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for Run Events timestamp no-wrap + responsive format
- [x] 1.2 Run `openspec validate update-run-events-timestamp-layout --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI
- [ ] 2.1 Prevent timestamp wrapping in Run Events rows (fixed-height virtual list)
- [ ] 2.2 Make timestamp display responsive (desktop: date+time, mobile: `HH:mm`)
- [ ] 2.3 Ensure full timestamp remains accessible (details view / title)
- [ ] 2.4 Update/extend unit tests as needed

## 3. Validation
- [ ] 3.1 Run `npm run lint --prefix ui`
- [ ] 3.2 Run `npm run test --prefix ui`
- [ ] 3.3 Run `npm run build --prefix ui`

## 4. Commits
- [ ] 4.1 Commit the UI changes (detailed message with Modules/Tests)
