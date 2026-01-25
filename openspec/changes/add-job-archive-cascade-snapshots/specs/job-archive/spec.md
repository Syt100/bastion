## ADDED Requirements

### Requirement: Archive Job With Optional Cascade Snapshot Deletion
When archiving a job, the system SHALL allow the user to optionally cascade delete snapshots belonging to that job.

#### Scenario: Archive job without cascade deletion
- **WHEN** a user archives a job without cascade deletion
- **THEN** the job stops scheduling future runs
- **AND** existing snapshots remain available

#### Scenario: Archive job with cascade deletion
- **WHEN** a user archives a job with cascade deletion enabled
- **THEN** the system enqueues deletion tasks for eligible snapshots of that job
- **AND** deletion proceeds asynchronously and is observable via task status/events

