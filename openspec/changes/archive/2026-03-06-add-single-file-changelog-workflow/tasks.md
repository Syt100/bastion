## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for single-file changelog workflow
- [x] 1.2 Run `openspec validate add-single-file-changelog-workflow --strict`

## 2. Implementation
- [x] 2.1 Add root `CHANGELOG.md` with `Unreleased` and initial `v0.1.0` section
- [x] 2.2 Add `scripts/changelog.sh` to validate format and extract release notes by tag
- [x] 2.3 Add regression test script for changelog tooling
- [x] 2.4 Update release workflow to use changelog extraction for release body
- [x] 2.5 Integrate changelog checks into `scripts/ci.sh`
- [x] 2.6 Document changelog contribution/release expectations in `README.md`

## 3. Validation
- [x] 3.1 Run `openspec validate add-single-file-changelog-workflow --strict`
- [x] 3.2 Run `bash scripts/changelog.sh check`
- [x] 3.3 Run `bash scripts/changelog_test.sh`
