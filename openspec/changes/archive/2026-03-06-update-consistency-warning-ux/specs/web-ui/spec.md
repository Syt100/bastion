## MODIFIED Requirements

### Requirement: Run Detail Shows Actionable Consistency Warnings
When a run includes source consistency warnings, the Web UI SHALL show:
- the warning total
- a breakdown (changed/replaced/deleted/read_error)
- a capped sample list of affected paths

#### Scenario: Run detail shows breakdown and samples
- **WHEN** a user opens a run detail page whose summary contains a consistency report
- **THEN** the UI displays the breakdown counts
- **AND** the UI displays sample paths and reasons (bounded)

#### Scenario: Run detail can jump to filtered events
- **WHEN** a user clicks “view consistency events”
- **THEN** the UI switches to the Events tab
- **AND** applies an event kind filter for `source_consistency`

### Requirement: Job Runs List Shows Warning For Running Runs
The job runs list SHALL show a warning tag when `consistency_changed_total > 0`, including for runs still in progress.

#### Scenario: Running run shows warning tag
- **WHEN** a run is in `running` status and has `consistency_changed_total > 0`
- **THEN** the runs list shows a warning tag in the status column

