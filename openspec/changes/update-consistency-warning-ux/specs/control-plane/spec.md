## MODIFIED Requirements

### Requirement: Job Runs List Shows Consistency Warnings Early
The job run list endpoint (`GET /api/jobs/:id/runs`) SHALL provide a consistency warning signal even when a run is still running and `summary_json` is not present yet.

#### Scenario: Running run derives warning from event
- **WHEN** a run is running and has emitted a `source_consistency` run event
- **AND** the run has no `summary_json` yet
- **THEN** the run item includes `consistency_changed_total > 0`

#### Scenario: Summary takes precedence over event
- **WHEN** a run has a `summary_json` consistency report
- **THEN** the run item’s `consistency_changed_total` is derived from `summary_json` (not from events)

