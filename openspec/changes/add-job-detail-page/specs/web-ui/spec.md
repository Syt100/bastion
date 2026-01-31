## ADDED Requirements

### Requirement: Node-Scoped Job Detail Page
The Web UI SHALL provide a node-scoped Job Detail page for a specific job at `/n/:nodeId/jobs/:jobId`.

#### Scenario: Job Detail is accessible from Jobs list
- **GIVEN** the user is on `/n/:nodeId/jobs`
- **WHEN** the user opens a job
- **THEN** the UI navigates to `/n/:nodeId/jobs/:jobId`

#### Scenario: Job Detail provides Runs and Snapshots views
- **WHEN** the user is on the Job Detail page
- **THEN** the user can view job runs
- **AND** the user can view job snapshots

#### Scenario: Runs list links to Run Detail
- **GIVEN** the job has a run in the Runs tab
- **WHEN** the user clicks the run
- **THEN** the UI navigates to `/n/:nodeId/runs/:runId`

### Requirement: Jobs List Action Simplification
The Jobs list SHALL prioritize primary actions and move secondary actions into a compact overflow menu.

#### Scenario: Jobs list keeps primary actions visible
- **WHEN** the user views the Jobs list
- **THEN** the primary action “Run now” is visible
- **AND** secondary actions (edit, deploy, archive/delete) are available via a “More” menu

