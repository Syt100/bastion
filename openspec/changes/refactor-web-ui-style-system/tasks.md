## 1. Spec
- [ ] 1.1 Add `web-ui` spec delta for shared style utilities and removing unused legacy views
- [ ] 1.2 Run `openspec validate refactor-web-ui-style-system --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI - Style Utilities
- [ ] 2.1 Add shared utility classes to UI styles (glass surface, list row, muted helper, icon tile)
- [ ] 2.2 Refactor layout/views to use the shared utilities where applicable
- [ ] 2.3 Commit style system refactor (detailed message)

## 3. Web UI - Remove Legacy Views
- [ ] 3.1 Remove unused legacy views not referenced by the router/tests
- [ ] 3.2 Commit legacy cleanup (detailed message)

## 4. Validation
- [ ] 4.1 Run `npm run lint --prefix ui`
- [ ] 4.2 Run `npm test --prefix ui`
- [ ] 4.3 Run `npm run build --prefix ui`

