## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for MSI payload integrity and manual artifact granularity
- [x] 1.2 Run `openspec validate fix-release-windows-msi-and-manual-artifacts --strict`

## 2. Implementation
- [x] 2.1 Update WiX packaging config to embed CAB payload into MSI
- [x] 2.2 Update release workflow MSI packaging step with sanity validation
- [x] 2.3 Update release workflow artifact uploads to per-file granularity
- [x] 2.4 Update `CHANGELOG.md` `Unreleased` for these user-visible release workflow fixes

## 3. Validation
- [x] 3.1 Run `openspec validate fix-release-windows-msi-and-manual-artifacts --strict`
- [x] 3.2 Run `bash scripts/changelog.sh check`
