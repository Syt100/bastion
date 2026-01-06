## ADDED Requirements

### Requirement: Deterministic Config Snapshot IDs
The Hub SHALL compute `ConfigSnapshot.snapshot_id` deterministically from the snapshot content so identical snapshots yield identical IDs.

#### Scenario: Same snapshot content yields same snapshot_id
- **WHEN** the Hub computes two configuration snapshots with identical job content for the same Agent
- **THEN** the two snapshots have the same `snapshot_id`

### Requirement: Hub Avoids Re-sending Unchanged Snapshots
The Hub SHALL avoid re-sending an unchanged configuration snapshot to a connected Agent.

#### Scenario: Unchanged snapshot is not re-sent
- **WHEN** a configuration change handler is triggered but the computed snapshot content is unchanged
- **THEN** the Hub does not send a redundant `ConfigSnapshot` to the Agent

### Requirement: Agent Offline Persistence Avoids Blocking the Async Runtime
When persisting offline run state and events, the Agent SHALL avoid blocking the async runtime hot path (e.g., by using async I/O or a dedicated writer task).

#### Scenario: Offline run event persistence does not block scheduling
- **WHEN** an offline run emits run events rapidly
- **THEN** the Agent persists events without blocking the async scheduler/task execution loop

