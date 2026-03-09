## Why
The `v0.2.0` Windows MSI asset is abnormally small and likely missing embedded payload data. This creates a broken installer experience for Windows users.

Also, `workflow_dispatch` release builds currently upload combined multi-file artifacts (notably Linux GNU tar/deb/rpm in one artifact), which differs from the per-file structure of published GitHub release assets and makes manual verification less clear.

## What Changes
- Ensure Windows MSI embeds CAB payload data so the MSI contains the installed binary.
- Add MSI package sanity checks in the release workflow so invalid tiny MSI outputs fail fast.
- Adjust build artifact uploads so manual release workflow runs publish per-file artifacts matching release asset granularity.

## Impact
- Affected specs: `dev-workflow`
- Affected code: `packaging/windows/bastion.wxs`, `.github/workflows/release.yml`, `CHANGELOG.md`
