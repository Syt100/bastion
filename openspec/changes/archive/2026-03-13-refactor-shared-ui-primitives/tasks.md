## 1. Spec
- [x] 1.1 Draft proposal/spec delta for shared route/copy/a11y/modal primitives
- [x] 1.2 Run `openspec validate refactor-shared-ui-primitives --strict`

## 2. Implementation
- [x] 2.1 Add `nodeRoute` helper and migrate duplicated node-scoped pushes/parsing
- [x] 2.2 Add shared clipboard feedback helper and migrate repeated handlers
- [x] 2.3 Add shared icon-only action button and migrate existing icon-only help buttons
- [x] 2.4 Align picker modal wrappers/confirm modal with shared modal shell behavior

## 3. Validation
- [x] 3.1 Run `npm run type-check --prefix ui`
- [x] 3.2 Run targeted UI tests for new shared helpers/components
