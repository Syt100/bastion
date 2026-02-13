## Why
Windows MSI users currently lack a persistent tray entry for quick Hub access/control, and startup behavior still depends on manual service/tray launching.

## What Changes
- Add a Windows-only `bastion tray run` subcommand to host the tray icon and menu actions.
- Add tray menu actions to open Web UI and control the Bastion Windows service.
- Update MSI packaging to auto-start Bastion service at boot and install a startup shortcut that launches `bastion tray run` at user logon.
- Keep uninstall behavior/data-retention semantics unchanged.

## Impact
- Affected specs: `dev-workflow`
- Affected code: `crates/bastion/src/config.rs`, `crates/bastion/src/main.rs`, `crates/bastion/src/win_tray.rs`, `packaging/windows/bastion.wxs`, `docs/user/getting-started.md`, `docs/zh/user/getting-started.md`, `CHANGELOG.md`
