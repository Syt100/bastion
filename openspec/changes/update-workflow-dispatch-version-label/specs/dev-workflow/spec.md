## ADDED Requirements

### Requirement: Manual Release Builds Use Tag-Based Preview Version Labels
The release workflow SHALL derive manual `workflow_dispatch` build version labels from the latest repository release tag and the current commit short hash.

#### Scenario: workflow_dispatch computes preview build version
- **WHEN** the release workflow runs via `workflow_dispatch`
- **THEN** it resolves the latest tag matching `v*`
- **AND** computes a preview label in the format `<tag-without-v>-dh<short-hash>`
- **AND** uses that label for build metadata and packaged asset naming in manual build artifacts

### Requirement: Tag-Triggered Release Versioning Remains Stable
Tag-triggered release workflow behavior SHALL remain unchanged for release labels and package version derivation.

#### Scenario: tag push keeps existing release label behavior
- **WHEN** a tag matching `v*` triggers the release workflow
- **THEN** release labels keep using the pushed tag name
- **AND** package semantic version values are derived from that same tag
