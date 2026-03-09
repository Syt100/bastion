## ADDED Requirements

### Requirement: Agent-Executed Deletion for Agent-Local Snapshots
For snapshots stored on an Agent node (e.g., `local_dir` target), the system SHALL execute snapshot deletion on that Agent.

#### Scenario: Delete an Agent-local snapshot
- **GIVEN** a snapshot index record indicates the snapshot is stored on Agent `A`
- **WHEN** a user requests deletion for that snapshot
- **THEN** the Hub dispatches a delete task to Agent `A`
- **AND** the snapshot is removed from Agent `A`'s filesystem

### Requirement: Offline Tolerance
If the owning Agent is offline, the system SHALL retry deletion with backoff and SHALL eventually execute when the Agent reconnects.

#### Scenario: Agent is offline during deletion
- **WHEN** the Hub attempts to dispatch a delete task and the Agent is offline
- **THEN** the task transitions to `retrying` with a future `next_attempt_at`

### Requirement: Idempotency on Agent
Agent-executed deletion SHALL be idempotent.

#### Scenario: Directory already deleted
- **WHEN** the Agent executes deletion and the snapshot directory does not exist
- **THEN** the Agent reports success

### Requirement: Safety Checks Before Deletion
The Agent SHALL validate that the deletion target path looks like a Bastion snapshot directory before performing recursive deletion.

#### Scenario: Reject unsafe delete path
- **WHEN** a delete task payload does not resolve to a safe Bastion snapshot path
- **THEN** the Agent rejects the task and reports a configuration error

### Requirement: Observable Progress and Results
The system SHALL provide an event log and a final result for Agent-executed deletions.

#### Scenario: View Agent deletion events
- **WHEN** a user opens deletion details for a snapshot
- **THEN** the UI can display Agent-emitted events and the final result

