# Change: Add systemd service (Linux) and Windows Service support

## Why
For individuals/families/small teams, “install and run as a service” is a common expectation for OS-native packages.

Running as a service also improves reliability (auto-restart, central logs) and provides a consistent operational model for non-Docker deployments.

## What Changes
- Linux `.deb` / `.rpm` packages install a `systemd` unit (`bastion.service`) for the Hub.
  - The unit is installed but **NOT** started automatically.
  - Documentation explains how to reload systemd and start/enable the service.
- Windows MSI installs a Windows Service entry for Bastion Hub.
  - The service is installed but **NOT** started automatically.
  - Bastion implements a Windows Service entrypoint to support running under SCM.
- Hub shutdown is graceful under service managers:
  - Linux: handle `SIGTERM` (systemd stop) in addition to Ctrl-C.
  - Windows: handle service stop/shutdown control signals.

## Impact
- Affected specs: `dev-workflow`
- Affected code:
  - `crates/bastion/src/main.rs` (shutdown handling + service entrypoint wiring)
  - `crates/bastion/src/config.rs` (CLI surface for Windows service run mode)
  - `packaging/windows/bastion.wxs` (MSI installs service, but does not start)
  - `packaging/linux/*` (new systemd unit + env)
  - `crates/bastion/Cargo.toml` (package assets for `.deb`/`.rpm`)
  - Docs under `docs/` (install/run instructions, EN/ZH)

## Non-Goals
- Auto-starting services on install.
- System users/groups management beyond `systemd` (Linux) and LocalSystem defaults (Windows).
- Code signing/notarization for installers.
