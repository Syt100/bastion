## ADDED Requirements

### Requirement: Job Spec Stores Snapshot Settings
Jobs with filesystem sources SHALL persist snapshot settings in the job spec and apply them consistently in both Hub and Agent execution paths.

#### Scenario: Snapshot settings are persisted
- **WHEN** a user saves a filesystem job with `snapshot_mode="auto"`
- **THEN** the job spec stores that mode and provider override (if set)

