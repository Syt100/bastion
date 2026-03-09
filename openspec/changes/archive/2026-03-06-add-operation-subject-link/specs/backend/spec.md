## ADDED Requirements

### Requirement: Operations May Reference a Subject Entity
The backend SHALL support linking an operation to a domain entity via a subject reference (`subject_kind`, `subject_id`).

#### Scenario: Operation has a run subject
- **WHEN** an operation is created for a run-scoped action
- **THEN** the operation stores `subject_kind = "run"` and `subject_id = <run_id>`

### Requirement: Restore and Verify Operations Link Back to the Run
When a restore or verify operation is started from a run, the backend SHALL create the operation linked to the run.

#### Scenario: Restore started from a successful run
- **GIVEN** a run with `status = success`
- **WHEN** the user starts a restore operation for the run
- **THEN** the created operation is linked to the run subject

#### Scenario: Verify started from a successful run
- **GIVEN** a run with `status = success`
- **WHEN** the user starts a verify operation for the run
- **THEN** the created operation is linked to the run subject

### Requirement: Run-Scoped Operations Listing API
The backend SHALL provide an API to list operations linked to a run.

#### Scenario: List operations for a run
- **GIVEN** a run with linked operations
- **WHEN** the user requests `GET /api/runs/{run_id}/operations`
- **THEN** the backend returns operations linked to the run ordered by `started_at` descending

