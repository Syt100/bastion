## ADDED Requirements

### Requirement: Runs and Operations Support Terminal Canceled Status
The backend SHALL support a terminal `canceled` lifecycle status for both runs and operations.

#### Scenario: Queued run is canceled before execution
- **GIVEN** a run is `queued`
- **WHEN** the operator requests cancellation
- **THEN** the run transitions to terminal `canceled`
- **AND** the scheduler MUST NOT start execution for that run

#### Scenario: Running operation transitions to canceled after graceful stop
- **GIVEN** an operation is `running`
- **WHEN** cancellation is requested and execution reaches a cancellation checkpoint
- **THEN** the operation performs cleanup and transitions to terminal `canceled`

### Requirement: Cancel Requests Are Persisted and Idempotent
The backend SHALL persist cancel-request metadata for runs and operations, and cancel APIs SHALL be idempotent.

#### Scenario: Repeated cancel requests return stable state
- **GIVEN** an operator has already requested cancel for a run
- **WHEN** the cancel API is called again for the same run
- **THEN** the backend returns the current status without creating conflicting transitions

### Requirement: Authenticated Cancel APIs Are Available For Runs and Operations
The backend SHALL provide authenticated mutation APIs to request cancellation for runs and operations.

#### Scenario: Operator cancels a running run
- **WHEN** an authenticated operator calls `POST /api/runs/{id}/cancel`
- **THEN** the backend records cancel intent and signals active execution to stop cooperatively

#### Scenario: Operator cancels a running restore/verify operation
- **WHEN** an authenticated operator calls `POST /api/operations/{id}/cancel`
- **THEN** the backend records cancel intent and signals active execution to stop cooperatively

### Requirement: Terminalization Is Race-Safe Against Late Results
The backend SHALL guard terminal status writes so that late success/failure results cannot overwrite `canceled`.

#### Scenario: Late success result arrives after cancellation
- **GIVEN** a run has reached terminal `canceled`
- **WHEN** a delayed success result for the same run is processed
- **THEN** the backend ignores the stale terminalization attempt
- **AND** the run remains `canceled`

### Requirement: Long-Running Work Checks Cooperative Cancellation Points
Long-running backup/restore/verify execution paths SHALL check cancellation at bounded checkpoints and exit via a cleanup-safe canceled path.

#### Scenario: Backup upload loop observes cancellation
- **GIVEN** a backup run is uploading data
- **WHEN** cancellation is requested before the next upload-part checkpoint
- **THEN** the worker exits via canceled flow after required cleanup
