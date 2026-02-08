## ADDED Requirements

### Requirement: CI Enforces UI Static Quality Checks
The project CI workflow SHALL run frontend static quality checks for the `ui/` workspace, including:
- ESLint in non-mutating check mode
- Vue/TypeScript type-check

These checks SHALL fail CI when violations are present.

#### Scenario: UI lint violation fails CI
- **GIVEN** a UI source file contains an ESLint violation
- **WHEN** `scripts/ci.sh` is executed in CI
- **THEN** the UI lint check step fails
- **AND** later build/test steps do not report overall success

#### Scenario: UI type-check violation fails CI
- **GIVEN** a UI source file contains a TypeScript type error
- **WHEN** `scripts/ci.sh` is executed in CI
- **THEN** the UI type-check step fails
- **AND** later build/test steps do not report overall success
