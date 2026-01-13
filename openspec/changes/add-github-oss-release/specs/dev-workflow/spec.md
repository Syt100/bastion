## ADDED Requirements

### Requirement: Repository Is Published With License And CI
The project SHALL be publishable as an open-source GitHub repository with a clear OSI-approved license and a baseline CI workflow that runs the project checks.

#### Scenario: GitHub Actions runs repo CI script
- **WHEN** a commit is pushed or a pull request is opened
- **THEN** GitHub Actions runs `bash scripts/ci.sh`
- **AND** the job fails on formatting, lint, tests, or secret scanning failures

