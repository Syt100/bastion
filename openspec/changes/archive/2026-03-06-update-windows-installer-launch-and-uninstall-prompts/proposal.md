## Why
Windows installer UX still misses two key flows users expect: a post-install option to launch Bastion Web UI, and a reliable uninstall-time prompt for optional data removal across interactive uninstall entry points.

## What Changes
- Add an install-completion checkbox (default checked) to start Bastion and open the Web UI.
- Enforce launch ordering by attempting service start first, then opening Web UI only after local endpoint readiness is confirmed.
- Adjust uninstall dialog flow so the data-retention choice is shown before remove confirmation even when uninstall is invoked via direct remove entry points.

## Impact
- Affected specs: `dev-workflow`
- Affected code: `packaging/windows/bastion.wxs`, `CHANGELOG.md`
