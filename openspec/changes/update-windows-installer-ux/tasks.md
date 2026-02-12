## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for Windows installer/uninstaller UX and metadata expectations
- [x] 1.2 Run `openspec validate update-windows-installer-ux --strict`

## 2. Implementation
- [x] 2.1 Update WiX authoring for x64 install location and component architecture
- [x] 2.2 Add installer UI + Start Menu shortcuts to improve Windows install UX
- [x] 2.3 Add uninstall-time checkbox option to optionally remove `C:\ProgramData\bastion` (default keep)
- [x] 2.4 Update release workflow Windows MSI build flags/extensions for the new WiX authoring
- [x] 2.5 Update `CHANGELOG.md` `Unreleased` with user-visible Windows installer changes

## 3. Validation
- [x] 3.1 Run `openspec validate update-windows-installer-ux --strict`
- [x] 3.2 Run `bash scripts/changelog.sh check`
