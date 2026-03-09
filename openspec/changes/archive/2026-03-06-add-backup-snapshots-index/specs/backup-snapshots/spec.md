## ADDED Requirements

### Requirement: Snapshot Index Row Per Successful Run
The system SHALL persist a snapshot index record for each successful backup run.

The snapshot index record SHALL:
- be uniquely identified by `run_id`
- store `job_id`
- store the run-time target snapshot (from `runs.target_snapshot_json`)
- store the artifact format (e.g., `archive_v1`, `raw_tree_v1`)
- store best-effort metrics (counts/sizes) when available

#### Scenario: Successful run creates a snapshot index record
- **WHEN** a run completes with status `success`
- **THEN** a snapshot index record exists for that `run_id`
- **AND** the record contains `job_id` and the run-time target snapshot

### Requirement: Snapshot Listing API (Per Job)
The system SHALL provide a read-only API to list snapshots for a job, with filtering and pagination.

#### Scenario: List snapshots for a job
- **WHEN** a user requests the snapshots list for a job
- **THEN** the system returns the most recent snapshots first

### Requirement: Snapshot Detail API
The system SHALL provide a read-only API to fetch snapshot details for a job/run.

#### Scenario: Get snapshot details
- **WHEN** a user requests snapshot details for `job_id` and `run_id`
- **THEN** the system returns the snapshot record and related metadata

### Requirement: Dedicated Web UI Page (Login Required)
The system SHALL provide a dedicated Web UI page to browse snapshots for a job.

#### Scenario: Navigate to the snapshots page
- **WHEN** a logged-in user opens the snapshots page for a job
- **THEN** the UI shows a list of snapshots with key metadata and links to run details

