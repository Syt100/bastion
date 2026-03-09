## ADDED Requirements

### Requirement: Repository Maintains A Single Changelog File
The repository SHALL keep a root-level `CHANGELOG.md` as the authoritative changelog for user-facing product changes.

#### Scenario: User-visible changes are recorded in Unreleased
- **WHEN** a change affects end-user behavior, UX, compatibility, or operational guidance
- **THEN** contributors add an entry under `## [Unreleased]` in `CHANGELOG.md`
- **AND** entries are grouped into standard categories (`Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`)
- **AND** internal-only maintenance work (for example CI/spec chores without user impact) MAY be omitted

### Requirement: Release Notes Come From Versioned Changelog Sections
The release workflow SHALL generate GitHub Release notes from the matching version section in `CHANGELOG.md`.

#### Scenario: Tag push publishes matching changelog section
- **WHEN** a tag matching `v*` is pushed
- **THEN** the workflow extracts the section `## [<tag>]` (or equivalent semver heading) from `CHANGELOG.md`
- **AND** publishes that extracted content as the GitHub Release body

#### Scenario: Missing changelog section blocks release publication
- **WHEN** a release tag does not have a matching version section in `CHANGELOG.md`
- **THEN** release notes generation fails
- **AND** the release publish job fails before creating/updating the GitHub Release

### Requirement: CI Validates Changelog Tooling
Project CI SHALL validate changelog format and extraction behavior.

#### Scenario: Changelog structure is invalid
- **WHEN** changelog validation runs in CI
- **THEN** CI fails if `CHANGELOG.md` is missing required top-level sections or uses unsupported category headings

#### Scenario: Extraction regression is introduced
- **WHEN** changelog extraction regression tests run in CI
- **THEN** CI fails if extraction no longer returns the expected version section output
