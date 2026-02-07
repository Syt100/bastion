## MODIFIED Requirements

### Requirement: Job Runs List Returns High-Signal Warning Digest
The job runs list endpoint (`GET /api/jobs/:id/runs`) SHALL provide a compact warning digest suitable for runs list display.

At minimum, each run item SHALL include:
- filesystem issues totals (`issues_errors_total`, `issues_warnings_total`) when present
- a source consistency breakdown total (`consistency_total`)
- a high-signal consistency total (`consistency_signal_total`), derived from replaced/deleted/read_error counts

#### Scenario: Summary takes precedence
- **WHEN** a run has a `summary_json` containing filesystem issues and/or a consistency report
- **THEN** the runs list fields are derived from `summary_json`

#### Scenario: Running run derives early digest from events
- **WHEN** a run is running and has emitted relevant run events (e.g. `fs_issues`, `source_consistency`)
- **AND** the run has no `summary_json` yet
- **THEN** the runs list includes best-effort digest fields derived from the latest events

