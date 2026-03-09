## ADDED Requirements

### Requirement: Web UI Exposes Cancel Actions For Runs and Operations
The Web UI SHALL provide cancel actions for eligible run/operation states and call backend cancel APIs.

#### Scenario: Cancel button is available for queued/running run
- **GIVEN** a run is `queued` or `running`
- **WHEN** the operator opens run details or list actions
- **THEN** the UI shows a cancel action and triggers run cancel API on confirmation

#### Scenario: Cancel button is available for running restore/verify operation
- **GIVEN** an operation is `running`
- **WHEN** the operator opens operation details
- **THEN** the UI shows a cancel action and triggers operation cancel API on confirmation

### Requirement: Web UI Shows Cancel-In-Progress and Canceled Terminal States
The Web UI SHALL represent cancellation lifecycle clearly for both runs and operations.

#### Scenario: Cancel requested while task still running
- **GIVEN** cancel has been requested for a running run/operation
- **WHEN** terminal `canceled` has not yet been reached
- **THEN** the UI displays a cancel-in-progress state and disables duplicate action spam

#### Scenario: Task reaches terminal canceled
- **WHEN** run/operation status becomes `canceled`
- **THEN** the UI renders terminal canceled badge/status and hides actions that require active execution

### Requirement: Cancel Mutation Handling Is Idempotent In UI State Stores
Web UI stores SHALL handle repeated cancel clicks and repeated cancel responses without inconsistent local state.

#### Scenario: User clicks cancel multiple times
- **WHEN** the user clicks cancel repeatedly before the first response returns
- **THEN** only one effective cancel mutation is processed and local status remains consistent
