# Change: Add Linux/Windows installers and macOS release artifacts

## Why
Docker is the primary deployment path, but individuals/families/small teams also want “download and install” options.

For an open-source project, publishing official installers per platform reduces friction and makes releases easier to consume.

## What Changes
- Extend GitHub Releases to publish additional installable artifacts:
  - Linux: `.tar.gz` (keep) + `.deb` + `.rpm`
  - Windows: `.zip` (keep) + `.msi` (WiX)
  - macOS: add release archives for x64 and arm64
  - Add `sha256sums.txt` for all assets
- Keep packages minimal and non-invasive:
  - Linux packages install only the `bastion` binary (no systemd unit/user by default)
  - Windows MSI does **not** add Bastion to `PATH` by default
- Add packaging configuration/metadata to the repository and update docs with installation instructions.

## Impact
- Affected specs: `dev-workflow`
- Affected code:
  - `.github/workflows/release.yml`
  - `crates/bastion/Cargo.toml` (packaging metadata)
  - Docs install pages under `docs/` (VitePress)

## Non-Goals
- Code signing, notarization, or signed package repositories (apt/yum/homebrew).
- Shipping system services by default (systemd/launchd/Windows service).
