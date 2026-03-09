---
## ADDED Requirements

### Requirement: Incomplete Cleanup Uses a Persistent Queue
The backend SHALL track stale incomplete run cleanup via a persistent task queue with retry scheduling to avoid tight loops and log spam.

#### Scenario: Unreachable targets do not cause tight loops
- **GIVEN** a run with an unreachable WebDAV target
- **WHEN** the cleanup worker processes due tasks
- **THEN** the task is retried with backoff and does not repeatedly emit warnings in a tight loop

### Requirement: Cleanup Tasks Are Observable and Operable
The backend SHALL expose authenticated APIs to list cleanup tasks, view attempt history, and perform operator actions (retry/ignore/unignore).

#### Scenario: Operator can retry now
- **WHEN** a user triggers “retry now” for a cleanup task
- **THEN** the task becomes due immediately and the cleanup worker attempts it again

### Requirement: Runs Store Target Snapshots
The backend SHALL persist a per-run target snapshot for maintenance workflows.

#### Scenario: Cleanup uses the run snapshot
- **GIVEN** a job target configuration changes after a run starts
- **WHEN** maintenance performs cleanup for that run
- **THEN** maintenance uses the stored run target snapshot rather than the job’s current target

### Requirement: Jobs Support Archive (Soft Delete)
The backend SHALL allow jobs to be archived (soft-deleted) so history remains, while permanent deletion remains available and cascades by default.

#### Scenario: Archive keeps history
- **WHEN** a user archives a job
- **THEN** the job is hidden from default listings and no new runs are scheduled, but past runs remain queryable

