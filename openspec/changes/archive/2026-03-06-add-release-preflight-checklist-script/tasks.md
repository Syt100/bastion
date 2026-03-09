## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for release preflight script
- [x] 1.2 Run `openspec validate add-release-preflight-checklist-script --strict`

## 2. Implementation
- [x] 2.1 Add `scripts/release-preflight.sh`
- [x] 2.2 Add regression tests for the preflight script
- [x] 2.3 Update release workflow to call preflight script
- [x] 2.4 Update README and skill reference docs with preflight usage

## 3. Validation
- [x] 3.1 Run `bash scripts/release-preflight.sh --tag v0.1.0 --output /tmp/release-notes.md`
- [x] 3.2 Run `bash scripts/release-preflight_test.sh`
- [x] 3.3 Run `openspec validate add-release-preflight-checklist-script --strict`
