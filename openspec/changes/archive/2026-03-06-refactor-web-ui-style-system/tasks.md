## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for shared style utilities and removing unused legacy views
- [x] 1.2 Run `openspec validate refactor-web-ui-style-system --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Web UI - Style Utilities
- [x] 2.1 Add shared utility classes to UI styles (glass surface, list row, muted helper, icon tile)
- [x] 2.2 Refactor layout/views to use the shared utilities where applicable
- [x] 2.3 Commit style system refactor (detailed message)

## 3. Web UI - Remove Legacy Views
- [x] 3.1 Remove unused legacy views not referenced by the router/tests
- [x] 3.2 Commit legacy cleanup (detailed message)

## 4. Validation
- [x] 4.1 Run `npm run lint --prefix ui`
- [x] 4.2 Run `npm test --prefix ui`
- [x] 4.3 Run `npm run build --prefix ui`
