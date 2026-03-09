## ADDED Requirements

### Requirement: Asynchronous Snapshot Deletion
The system SHALL delete snapshots asynchronously using a persistent task queue, and SHALL NOT require long-running HTTP requests to complete deletion.

#### Scenario: User requests deletion
- **WHEN** a user requests deletion for a snapshot
- **THEN** the system enqueues a delete task and returns immediately

### Requirement: Idempotent Deletion
Snapshot deletion SHALL be idempotent.

#### Scenario: Snapshot is already missing
- **WHEN** a delete task runs and the snapshot does not exist in the target
- **THEN** the task completes successfully

#### Scenario: User requests deletion twice
- **WHEN** a user requests deletion for the same snapshot multiple times
- **THEN** the system keeps a single delete task and does not create conflicting work

### Requirement: Retry / Backoff and Error Classification
The system SHALL retry failed deletions with backoff and SHALL classify errors into actionable kinds (e.g., config/auth/network/http/unknown).

#### Scenario: Target is temporarily unreachable
- **WHEN** a delete attempt fails due to a network error
- **THEN** the task transitions to a retrying state with a future `next_attempt_at`

### Requirement: Operator Controls and Observability
The system SHALL record an event log for deletion attempts and operator actions, and SHALL expose the task state and events to the Web UI.

#### Scenario: View delete events
- **WHEN** a user opens snapshot deletion details
- **THEN** the UI can show attempts, state transitions, and recent errors from the event log

### Requirement: Supported Targets (Hub-Executed)
The system SHALL support Hub-executed snapshot deletion for:
- WebDAV targets
- Local directory targets on the Hub node

#### Scenario: Delete a WebDAV snapshot
- **WHEN** a WebDAV snapshot delete task runs
- **THEN** the system deletes the corresponding remote run directory and marks the task done

