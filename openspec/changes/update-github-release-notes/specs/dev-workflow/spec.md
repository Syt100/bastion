## ADDED Requirements

### Requirement: GitHub Release Includes Changelog Body
Each GitHub Release SHALL include a human-readable changelog section in the release body.

#### Scenario: Tag push generates release notes from git history
- **WHEN** a tag matching `v*` is pushed
- **THEN** the release workflow generates release notes from git commits since the previous tag (or from repository start for the first tag)
- **AND** the release body contains a changelog section and a link to the full compare view
