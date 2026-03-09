## ADDED Requirements

### Requirement: Persist Latest Progress Snapshot For Runs
The backend SHALL persist the latest progress snapshot for a running backup run and expose it via authenticated run APIs.

#### Scenario: Run progress is readable while running
- **GIVEN** a run is `running`
- **WHEN** the system updates its progress snapshot
- **THEN** subsequent reads of the run include the latest `progress` snapshot

### Requirement: Persist Latest Progress Snapshot For Operations
The backend SHALL persist the latest progress snapshot for a running operation (restore/verify) and expose it via authenticated operation APIs.

#### Scenario: Operation progress is readable while running
- **GIVEN** an operation is `running`
- **WHEN** the system updates its progress snapshot
- **THEN** subsequent reads of the operation include the latest `progress` snapshot

### Requirement: Filesystem Backup Supports Optional Pre-Scan
Filesystem backup jobs SHALL support an optional pre-scan stage (`filesystem.source.pre_scan`) to compute totals for progress and ETA. The default value SHALL be `true`.

#### Scenario: Pre-scan computes totals
- **GIVEN** a filesystem job has `source.pre_scan = true`
- **WHEN** a run starts
- **THEN** the run enters a scan stage and computes totals before packaging/upload

#### Scenario: Pre-scan can be disabled
- **GIVEN** a filesystem job has `source.pre_scan = false`
- **WHEN** a run starts
- **THEN** the backend proceeds without requiring totals to be present in the progress snapshot

### Requirement: Progress Updates Are Throttled
The backend SHALL throttle progress snapshot writes and broadcasts to avoid excessive database writes and event volume while still remaining responsive.

#### Scenario: Throttling limits update frequency
- **WHEN** a long-running backup or restore is executing
- **THEN** progress updates are emitted no more than once per second per run/operation (except for stage changes)

