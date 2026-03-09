## 1. Spec
- [x] 1.1 Add `dev-workflow` spec delta for Windows tray subcommand and startup behavior
- [x] 1.2 Run `openspec validate add-windows-tray-autostart-subcommand --strict`

## 2. Implementation
- [x] 2.1 Implement Windows-only `bastion tray run` subcommand and tray runtime/menu behavior
- [x] 2.2 Update MSI authoring to auto-start service on boot and add startup shortcut for tray launch
- [x] 2.3 Update user-facing docs and changelog for new Windows startup/tray behavior

## 3. Validation
- [x] 3.1 Run `openspec validate add-windows-tray-autostart-subcommand --strict`
- [x] 3.2 Run focused Rust checks for modified `bastion` package
- [x] 3.3 Parse `packaging/windows/bastion.wxs` as XML for syntax sanity
