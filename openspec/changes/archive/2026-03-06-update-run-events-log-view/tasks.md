## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for Run Events log viewer UX
- [x] 1.2 Run `openspec validate update-run-events-log-view --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI
- [x] 2.1 Add 2-chip “field summary” rendering from `event.fields` (+ relative time formatting)
- [x] 2.2 Implement follow auto-disable on scroll + new-events counter + “Latest” action
- [x] 2.3 Implement WS auto-reconnect (default on) + manual reconnect + reconnect countdown
- [x] 2.4 Update row layout for desktop (single-line) and mobile (two-line) + sticky header on mobile
- [x] 2.5 Implement details UX: keep Details button; row click opens details; mobile uses bottom drawer (~70vh)
- [x] 2.6 Update/extend unit tests

## 3. Validation
- [x] 3.1 Run `npm run lint --prefix ui`
- [x] 3.2 Run `npm test --prefix ui`
- [x] 3.3 Run `npm run type-check --prefix ui`
- [x] 3.4 Run `npm run build --prefix ui`

## 4. Commits
- [x] 4.1 Commit the UI changes (detailed message with Modules/Tests)
