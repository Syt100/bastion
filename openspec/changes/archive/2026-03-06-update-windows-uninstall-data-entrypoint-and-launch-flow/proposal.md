## Why
Current Windows MSI behavior still has two experience gaps: the post-install launch action can block the finish flow without reliably starting the service, and uninstall invoked from Windows Settings does not show custom MSI dialogs for data-removal selection.

## What Changes
- Make post-install launch UX non-blocking and rely on installer service-control actions for service startup.
- Keep default uninstall behavior as data-preserving for standard Settings/App uninstall paths.
- Add an explicit Start Menu entry to uninstall Bastion and remove `C:\ProgramData\bastion` data via MSI property override.

## Impact
- Affected specs: `dev-workflow`
- Affected code: `packaging/windows/bastion.wxs`, `CHANGELOG.md`
