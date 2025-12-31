## ADDED Requirements

### Requirement: WebDAV Upload of Parts
The system SHALL support uploading split artifact parts to a WebDAV target using HTTP PUT.

#### Scenario: Upload all parts
- **WHEN** a run produces N parts
- **THEN** the system uploads all N parts to the WebDAV destination

### Requirement: Resume by Existing Part Size
The system SHALL support resuming interrupted uploads by detecting already-present parts on the destination and skipping parts that match expected size.

#### Scenario: Resume after network interruption
- **WHEN** uploading fails mid-run and the run is retried
- **THEN** parts already present with matching size are skipped

### Requirement: Upload Order and Atomic Completion
The system SHALL upload `manifest.json` and `complete.json` only after all parts are uploaded successfully.

#### Scenario: Completion marker uploaded last
- **WHEN** the final part upload succeeds
- **THEN** `manifest.json` is uploaded and then `complete.json` is uploaded

### Requirement: Incomplete-Run Cleanup (WebDAV)
The system SHALL periodically clean up incomplete run directories (missing `complete.json`) older than a configurable threshold.

#### Scenario: Stale incomplete run is removed
- **WHEN** a run directory exists under `<base_url>/<job_id>/<run_id>/` without `complete.json` and is older than the configured threshold
- **THEN** the system deletes the run directory and its contents
