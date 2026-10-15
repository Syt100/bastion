## ADDED Requirements

### Requirement: Group Compatible Security Updates Separately From Version Updates
Dependabot SHALL define a dedicated group with `applies-to: security-updates`, `patterns: ["*"]`, and `update-types: ["minor", "patch"]` for each configured ecosystem/directory entry.

#### Scenario: Minor or patch security updates are available
- **WHEN** multiple eligible security updates exist within one configured ecosystem/directory
- **THEN** Dependabot groups them according to that entry's security-update rule
- **AND** existing version-update groups, schedules, and pull-request limits remain unchanged

#### Scenario: A security update requires a major upgrade
- **WHEN** a security update is classified by Dependabot as a major upgrade
- **THEN** it remains eligible for an individual pull request rather than being included in the minor/patch security group

#### Scenario: Individual security pull requests already exist
- **WHEN** the grouping configuration is merged
- **THEN** this change does not manually close or merge those pull requests
