## Why
Windows MSI installation and uninstall behavior still feels rough for end users: install path defaults can be confusing on x64 hosts, Start Menu integration is missing, uninstall cannot ask whether to clear persisted data, and package metadata/polish is incomplete.

## What Changes
- Make Windows MSI explicitly x64 so installs land under 64-bit Program Files for x86_64 releases.
- Improve MSI UX with a standard interactive installer flow and Start Menu shortcuts.
- Add uninstall-time optional data cleanup for `C:\ProgramData\bastion`, defaulting to keep data.
- Improve Add/Remove Programs metadata and release workflow wiring required by the updated WiX authoring.

## Impact
- Affected specs: `dev-workflow`
- Affected code: `packaging/windows/bastion.wxs`, `.github/workflows/release.yml`, `CHANGELOG.md`
