## ADDED Requirements

### Requirement: Job Can Configure Consistency Policy
Backup jobs whose sources support best-effort consistency detection SHALL allow configuring how detected changes affect the run outcome.

The job spec SHALL support:
- `consistency_policy`: `warn|fail|ignore`
- `consistency_fail_threshold` (optional)
- `upload_on_consistency_failure` (optional)

#### Scenario: Policy is configured in job spec
- **WHEN** a user saves a job with `consistency_policy="fail"`
- **THEN** the job spec persists that policy and associated parameters

## MODIFIED Requirements

### Requirement: Consistency Warnings Can Fail A Run
When `consistency_policy="fail"`, the system SHALL fail the run if the consistency warning total exceeds the configured threshold.

#### Scenario: Fail policy fails run when warnings exceed threshold
- **WHEN** a run completes packaging with `total > consistency_fail_threshold`
- **AND** the policy is `fail`
- **THEN** the run status is `failed`
- **AND** the run records `error_code="source_consistency"`

