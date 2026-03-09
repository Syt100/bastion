## ADDED Requirements

### Requirement: Backup Events and Metrics MUST Include Driver Dimensions
Backup observability data MUST include source/target driver identity and planner mode dimensions for
both success and failure paths.

#### Scenario: Run events include planner and driver fields
- **WHEN** a run transitions through planning, packaging, upload, and completion/failure stages
- **THEN** emitted events include `source_driver`, `target_driver`, and `plan_mode`
- **AND** fallback events include machine-readable fallback reason fields

### Requirement: Driver Implementations MUST Pass Contract Test Suites
Each source and target driver implementation MUST pass shared contract test suites before release.

#### Scenario: New target driver fails contract tests
- **WHEN** a new target driver implementation violates lifecycle idempotency or cleanup contract
- **THEN** CI contract test suite fails and blocks merge

### Requirement: Cross-Driver Matrix Tests MUST Cover Critical Combinations
CI MUST run matrix tests across source-target-format combinations that are marked as supported.

#### Scenario: Supported combination regresses
- **WHEN** a supported source-target-format combination regresses in planner or runtime behavior
- **THEN** matrix tests fail with the offending combination surfaced in test output
