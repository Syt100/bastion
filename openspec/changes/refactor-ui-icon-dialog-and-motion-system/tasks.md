## 1. Spec
- [x] 1.1 Draft proposal/design/spec delta for icon/dialog/motion refactor
- [x] 1.2 Run `openspec validate refactor-ui-icon-dialog-and-motion-system --strict`

## 2. Shared Primitives
- [x] 2.1 Add shared icon wrapper with normalized size and semantic tone options
- [x] 2.2 Add shared modal shell component with consistent structure and sizing
- [x] 2.3 Add shared motion utility classes/tokens for rows/cards/buttons and reduced-motion fallback

## 3. Page Migration
- [x] 3.1 Migrate filter trigger and row action icons to shared icon wrapper
- [x] 3.2 Migrate Jobs/Agents critical modals to shared modal shell
- [x] 3.3 Apply shared motion classes to list rows/cards and key interactive elements

## 4. Validation
- [x] 4.1 Add/update component tests for icon/modal primitives
- [x] 4.2 Run `npm run type-check --prefix ui`
- [x] 4.3 Run `npm test --prefix ui -- --run`
