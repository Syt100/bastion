## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for workflow_dispatch preview version labels
- [x] 1.2 Run `openspec validate update-workflow-dispatch-version-label --strict`

## 2. Implementation
- [x] 2.1 Update release workflow to resolve tag/manual version metadata centrally
- [x] 2.2 Apply resolved version label to manual artifact naming while keeping tag behavior unchanged
- [x] 2.3 Use numeric package version derivation for packagers/MSI in both trigger modes
- [x] 2.4 Update `CHANGELOG.md` `Unreleased` to describe the manual versioning improvement

## 3. Validation
- [x] 3.1 Run `openspec validate update-workflow-dispatch-version-label --strict`
- [x] 3.2 Run `bash scripts/changelog.sh check`
- [x] 3.3 Parse `.github/workflows/release.yml` as YAML
