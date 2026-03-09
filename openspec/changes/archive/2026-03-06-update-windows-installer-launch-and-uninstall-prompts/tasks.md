## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for post-install launch option sequencing and uninstall prompt coverage
- [x] 1.2 Run `openspec validate update-windows-installer-launch-and-uninstall-prompts --strict`

## 2. Implementation
- [x] 2.1 Add post-install launch checkbox (default checked) and ordered launch behavior in WiX UI flow
- [x] 2.2 Ensure uninstall data-removal prompt appears before remove confirmation for interactive uninstall entry paths
- [x] 2.3 Update `CHANGELOG.md` `Unreleased` with user-visible Windows installer flow changes

## 3. Validation
- [x] 3.1 Run `openspec validate update-windows-installer-launch-and-uninstall-prompts --strict`
- [x] 3.2 Run `bash scripts/changelog.sh check`
- [x] 3.3 Parse `packaging/windows/bastion.wxs` as XML for syntax sanity
