## ADDED Requirements

### Requirement: Consistency Failure Uses Stable Error Code
When a run fails due to consistency policy enforcement, the system SHALL store `error_code="source_consistency"` for that run.

#### Scenario: Run exposes source_consistency error code
- **WHEN** a run fails due to consistency policy
- **THEN** APIs that return run details include `error_code="source_consistency"` (where applicable)

