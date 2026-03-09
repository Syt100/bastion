## ADDED Requirements

### Requirement: GitHub Releases Publish Installers And macOS Binaries
The project SHALL publish platform-appropriate release artifacts on GitHub Releases for each `v*` tag.

#### Scenario: Tag push creates a GitHub Release with installers and checksums
- **WHEN** a tag matching `v*` is pushed
- **THEN** GitHub Actions builds `bastion` with embedded UI+docs for:
  - Linux x64
  - Windows x64
  - macOS x64
  - macOS arm64
- **AND** the workflow publishes a GitHub Release containing:
  - Linux: `tar.gz`, `.deb`, `.rpm`
  - Windows: `.zip`, `.msi` (MSI MUST NOT add Bastion to `PATH` by default)
  - macOS: archives for x64 and arm64
  - `sha256sums.txt` covering all uploaded artifacts
