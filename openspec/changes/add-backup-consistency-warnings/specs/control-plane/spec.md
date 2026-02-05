## MODIFIED Requirements

### Requirement: Job Run List Includes Consistency Warning Signal
The job run list endpoint (`GET /api/jobs/:id/runs`) SHALL include enough information for the Web UI to display a "source changed during backup" warning without fetching each run detail.

#### Scenario: Runs list includes changed count
- **WHEN** the UI requests a job's run list
- **THEN** each run item includes `consistency_changed_total` (default `0` when absent)
- **AND** the value is derived from the run summary produced by the executor (Hub or Agent)

