## ADDED Requirements

### Requirement: GitHub Releases Publish Prebuilt Binaries
The project SHALL provide a GitHub Actions release workflow that publishes prebuilt binaries for Linux x64 and Windows x64.

#### Scenario: Tag push creates a GitHub Release with binaries
- **WHEN** a tag matching `v*` is pushed
- **THEN** GitHub Actions builds `bastion` for Linux x64 and Windows x64
- **AND** the workflow publishes a GitHub Release containing the two artifacts
