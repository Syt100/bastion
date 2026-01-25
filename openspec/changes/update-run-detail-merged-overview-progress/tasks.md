## 1. Spec
- [x] 1.1 Add web-ui spec delta for merged Run Detail summary card and restore final speed display
- [x] 1.2 Run `openspec validate update-run-detail-merged-overview-progress --strict`
- [ ] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [ ] 2.1 Run Detail: merge Overview + Progress into a single summary card (desktop default expanded)
- [ ] 2.2 Run Progress: adjust layout to match merged summary card (compact spacing, no regressions)
- [ ] 2.3 OperationModal: compute/display final speed for completed restore ops

## 3. Tests
- [ ] 3.1 Add/update unit tests for OperationModal final-speed behavior

## 4. Validation
- [ ] 4.1 Run `npm test --prefix ui`
- [ ] 4.2 Run `npm --prefix ui run build`
- [ ] 4.3 Run `cargo test --workspace`

## 5. Commits
- [ ] 5.1 Commit implementation changes (detailed message with Modules/Tests)
