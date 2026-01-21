## ADDED Requirements

### Requirement: Backup Progress Snapshots Include Stable Source Totals
The backend SHALL include stable SOURCE totals (files, dirs, bytes) for filesystem backup runs in the run progress snapshot detail when the totals are known, and SHALL keep these totals visible across stage transitions.

#### Scenario: Source totals remain visible after entering upload
- **GIVEN** a filesystem backup run computes SOURCE totals during scan or packaging
- **WHEN** the run transitions into the upload stage
- **THEN** subsequent run progress snapshots include the previously computed SOURCE totals

### Requirement: raw_tree_v1 Upload Reports A Stable Transfer Total
For filesystem backups using raw_tree_v1, the backend SHALL expose a stable TRANSFER total bytes during upload so the UI can compute a meaningful percentage.

#### Scenario: Upload total bytes is stable for raw_tree_v1
- **GIVEN** a filesystem backup run uses raw_tree_v1
- **WHEN** the run is uploading artifacts to the target
- **THEN** the run progress snapshot includes TRANSFER total bytes that do not grow during the upload

### Requirement: Backup Upload Snapshots Include Transfer Metrics
During upload, the backend SHALL include TRANSFER done bytes and TRANSFER total bytes in the run progress snapshot detail for filesystem backup runs.

#### Scenario: Transfer done bytes increases during upload
- **GIVEN** a filesystem backup run is in the upload stage
- **WHEN** the system uploads additional data to the target
- **THEN** TRANSFER done bytes in the run progress snapshot increases until it reaches TRANSFER total bytes
