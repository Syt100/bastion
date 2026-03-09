## 1. Spec
- [x] 1.1 Add `web-ui` spec delta for localized run status/target labels and progress help icon redesign
- [x] 1.2 Run `openspec validate update-run-detail-view-product-polish --strict`
- [x] 1.3 Commit the spec proposal (detailed message)

## 2. Implementation
- [x] 2.1 Localize run status labels (Run Detail header + runs list)
- [x] 2.2 Productize target type labels in Run Detail (overview + summary)
- [x] 2.3 Progress panel: help icon + stage stepper layout (less rigid)
- [x] 2.4 Adjust i18n strings (zh-CN/en-US) as needed

## 3. Tests
- [x] 3.1 Update/add unit tests for Run Detail + RunProgressPanel after layout changes

## 4. Validation
- [x] 4.1 Run `npm test --prefix ui`
- [x] 4.2 Run `npm --prefix ui run build`
- [x] 4.3 Run `cargo test -p bastion-http`

## 5. Commits
- [x] 5.1 Commit implementation changes (detailed message with Modules/Tests)
