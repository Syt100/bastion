## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for Windows launch sequencing and explicit data-removal uninstall entry
- [x] 1.2 Run `openspec validate update-windows-uninstall-data-entrypoint-and-launch-flow --strict`

## 2. Implementation
- [x] 2.1 Update WiX install completion action to avoid blocking finish UI while still opening Web UI after readiness
- [x] 2.2 Start Bastion service during install execution instead of relying on finish-button custom action context
- [x] 2.3 Replace uninstall dialog dependency with an explicit Start Menu "uninstall and remove data" entry, keeping default uninstall data-preserving
- [x] 2.4 Update `CHANGELOG.md` `Unreleased` with user-visible behavior changes

## 3. Validation
- [x] 3.1 Run `openspec validate update-windows-uninstall-data-entrypoint-and-launch-flow --strict`
- [x] 3.2 Run `bash scripts/changelog.sh check`
- [x] 3.3 Parse `packaging/windows/bastion.wxs` as XML for syntax sanity
