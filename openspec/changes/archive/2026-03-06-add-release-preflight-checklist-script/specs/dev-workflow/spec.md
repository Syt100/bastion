## ADDED Requirements

### Requirement: Release Preflight Script Validates Changelog Readiness
The project SHALL provide a release preflight script that validates changelog readiness for a target release tag.

#### Scenario: Maintainer runs preflight for a release tag
- **WHEN** a maintainer runs the preflight script with `--tag vX.Y.Z`
- **THEN** the script runs changelog structure checks
- **AND** the script runs changelog regression tests
- **AND** the script extracts release notes for the target tag into an output file

#### Scenario: Invalid tag or missing changelog section blocks preflight
- **WHEN** the preflight script receives an invalid tag format or a tag without matching changelog section
- **THEN** the script exits with a non-zero status
- **AND** prints a clear error message describing the failure

### Requirement: Release Workflow Uses Preflight Script
The GitHub release workflow SHALL use the release preflight script for changelog validation and release-note generation.

#### Scenario: Tag push runs preflight before publishing release
- **WHEN** a `v*` tag triggers the release workflow
- **THEN** the workflow runs the release preflight script with the workflow tag
- **AND** uses the preflight-generated notes file as the GitHub Release body
