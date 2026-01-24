## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for Run Detail visual hierarchy/layout improvements
- [x] 1.2 Run `openspec validate update-run-detail-view-visual-polish --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Run Detail header: status badge, run id copy, primary/secondary actions grouping
- [ ] 2.2 Run Detail overview card: key/value layout, duration, target summary, warnings/errors, error placement
- [ ] 2.3 Layout: desktop 2-col + mobile 1-col spacing and consistent card sizing
- [ ] 2.4 Operations section: compact empty state, tighten table layout when present
- [ ] 2.5 Events section: timeline list + details (desktop modal / mobile bottom drawer), reuse styling from RunEventsModal where possible
- [ ] 2.6 Summary section: structured highlights + raw JSON collapse + copy affordance
- [ ] 2.7 Add/adjust i18n strings as needed (zh-CN/en-US)

## 3. Tests
- [ ] 3.1 Add/adjust unit tests for Run Detail view components (layout/content/empty states)
- [ ] 3.2 Ensure existing RunEventsModal tests remain valid (if shared components are introduced)

## 4. Validation
- [ ] 4.1 Run `npm test --prefix ui`
- [ ] 4.2 Run `cargo test -p bastion-http` (ensure the UI route still builds in embedded UI)

## 5. Commits
- [ ] 5.1 Commit implementation changes (detailed message with Modules/Tests)
