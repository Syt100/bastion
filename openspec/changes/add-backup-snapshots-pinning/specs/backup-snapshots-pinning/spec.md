## ADDED Requirements

### Requirement: Pin/Protect a Snapshot
The system SHALL allow users to pin (protect) and unpin snapshots.

#### Scenario: Pin a snapshot
- **WHEN** a user pins a snapshot
- **THEN** the snapshot is marked as pinned and is clearly visible as protected in the UI

### Requirement: Retention Excludes Pinned Snapshots
Retention policy selection SHALL exclude pinned snapshots.

#### Scenario: Pinned snapshot is not selected by retention
- **GIVEN** retention policy is enabled
- **AND** a snapshot is pinned
- **WHEN** retention selection runs
- **THEN** the pinned snapshot is not selected for deletion

### Requirement: Safer Manual Deletion
The system SHALL prevent accidental deletion of pinned snapshots by requiring an explicit force deletion.

#### Scenario: Attempt to delete a pinned snapshot
- **WHEN** a user requests deletion for a pinned snapshot without force
- **THEN** the request is rejected with a clear error

