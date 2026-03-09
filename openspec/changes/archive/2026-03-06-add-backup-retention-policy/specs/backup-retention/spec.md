## ADDED Requirements

### Requirement: Per-Job Retention Policy
The system SHALL support configuring a retention policy per job.

The policy SHALL support at least:
- keep last N snapshots
- keep snapshots within the last D days

#### Scenario: Configure retention on a job
- **WHEN** a user enables retention on a job and sets keep-last and keep-days values
- **THEN** the configuration is persisted and used for future retention enforcement

### Requirement: Preview (Simulate) Retention Selection
The system SHALL provide a preview endpoint that simulates retention selection without deleting data.

#### Scenario: Preview retention
- **WHEN** a user requests a retention preview for a job
- **THEN** the system returns which snapshots would be kept and which would be deleted, with reasons

### Requirement: Server-Enforced Retention Loop
The system SHALL periodically enforce retention on the server by enqueueing deletion tasks for eligible snapshots.

#### Scenario: Retention loop enqueues deletions
- **GIVEN** retention is enabled for a job
- **WHEN** the retention loop runs
- **THEN** eligible snapshots are enqueued for deletion

### Requirement: Safety Limits
The system SHALL provide safety limits to bound deletion work (e.g., max deletes per tick/day).

#### Scenario: Deletion is bounded
- **WHEN** more snapshots are eligible for deletion than the safety limit
- **THEN** only up to the limit are enqueued and the rest remain for future ticks

### Requirement: Pinned Snapshots Are Excluded
Pinned snapshots SHALL NOT be selected for deletion by retention.

#### Scenario: Pinned snapshots remain
- **GIVEN** a snapshot is pinned
- **WHEN** retention selection runs
- **THEN** the snapshot is not selected for deletion

