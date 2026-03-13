## 1. Spec
- [x] 1.1 Draft proposal and spec delta for modal shell layout contract
- [x] 1.2 Get proposal approval before implementation
- [x] 1.3 Run `openspec validate update-modal-shell-layout-contract --strict`

## 2. Implementation
- [x] 2.1 Refactor `AppModalShell` to expose explicit container/body layout responsibilities
- [x] 2.2 Align `.app-modal-shell__body-plain` with bounded-height and overflow expectations
- [x] 2.3 Migrate `JobEditorModal` to use container-level height constraints under the new contract
- [x] 2.4 Preserve existing modal title/footer/actions and mobile behavior

## 3. Validation
- [x] 3.1 Add/extend unit tests for `AppModalShell` layout contract behaviors
- [x] 3.2 Add/extend unit tests for `JobEditorModal` height-limit behavior
- [x] 3.3 Run `npm run test:unit --prefix ui` for touched test suites
- [x] 3.4 Run `npm run type-check --prefix ui`
