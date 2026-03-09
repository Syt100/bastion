# targets-local-dir Specification

## Purpose
TBD - created by archiving change update-archive-v1-streaming-upload. Update Purpose after archive.
## Requirements
### Requirement: Local Directory Rolling Part Storage Deletes Local Parts
When storing an `archive_v1` run to a local directory target, the system SHALL store each finalized `payload.part*` as soon as it is finalized.

After a part is stored successfully, the system SHALL delete the local staging file for that part.

#### Scenario: Part storage frees local disk
- **GIVEN** an `archive_v1` filesystem backup run produces multiple part files
- **WHEN** `payload.part000001` is written successfully under `<base_dir>/<job_id>/<run_id>/`
- **THEN** the local staging file `payload.part000001` is deleted

### Requirement: Local Rolling Storage Preserves Atomic Completion
The system SHALL still write `manifest.json` and `complete.json` only after all parts and `entries.jsonl.zst` are written successfully, and SHALL write `complete.json` last.

#### Scenario: Completion marker remains last with rolling storage
- **GIVEN** an `archive_v1` run is writing parts incrementally
- **WHEN** the last part and `entries.jsonl.zst` have been written successfully
- **THEN** `manifest.json` is written and then `complete.json` is written

### Requirement: Local Directory Targets Are Node-Scoped
Local directory targets SHALL be node-scoped and SHALL be configurable per node.

#### Scenario: Local target differs per node
- **WHEN** node A and node B each configure a local directory target
- **THEN** the configured base directories may differ and remain isolated per node

### Requirement: Local Directory Target
The system SHALL support storing backup artifacts to a local directory target on the same machine.

#### Scenario: Store a run to local directory
- **WHEN** a run produces split parts and metadata files
- **THEN** the system writes them under the configured base directory

### Requirement: Local Directory Layout
The system SHALL store runs using a deterministic layout: `<base_dir>/<job_id>/<run_id>/`.

#### Scenario: Layout is stable
- **WHEN** the same job runs multiple times
- **THEN** each run is written to a separate directory under the job directory

### Requirement: Resume by Existing File Size (Local)
The system SHALL support resuming interrupted local writes by skipping files that already exist with matching size.

#### Scenario: Resume after interruption
- **WHEN** writing files is interrupted and the run is retried
- **THEN** files already present with matching size are skipped

### Requirement: Write Order and Atomic Completion (Local)
The system SHALL write `manifest.json` and `complete.json` only after all parts and the entries index are written successfully, and SHALL write `complete.json` last.

#### Scenario: Completion marker written last
- **WHEN** all parts and `entries.jsonl.zst` are written
- **THEN** `manifest.json` is written and then `complete.json` is written

### Requirement: Incomplete-Run Cleanup (Local)
The system SHALL periodically clean up incomplete run directories (missing `complete.json`) older than a configurable threshold.

#### Scenario: Stale incomplete run is removed
- **WHEN** a run directory exists under `<base_dir>/<job_id>/<run_id>/` without `complete.json` and is older than the configured threshold
- **THEN** the system deletes the run directory and its contents

