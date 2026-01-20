## ADDED Requirements

### Requirement: Node-Scoped Run Detail Page
The Web UI SHALL provide a node-scoped Run Detail page at `/n/:nodeId/runs/:runId`.

#### Scenario: User opens Run Detail from the runs list
- **GIVEN** a user is viewing a job’s run list
- **WHEN** the user selects a run
- **THEN** the UI navigates to the Run Detail page for that run

### Requirement: Run Detail Shows Events and Linked Operations
The Run Detail page SHALL show live run events and a sub-list of linked operations (restore/verify) started from the run.

#### Scenario: Restore operation remains visible after closing dialogs
- **WHEN** a user starts a restore from Run Detail
- **THEN** the resulting restore operation appears in the operations sub-list for that run

