# targets-webdav Specification

## Purpose
TBD - created by archiving change update-archive-v1-streaming-upload. Update Purpose after archive.
## Requirements
### Requirement: WebDAV Rolling Part Upload Deletes Local Parts
When storing an `archive_v1` run to a WebDAV target, the system SHALL upload each finalized `payload.part*` as soon as it is finalized.

After a part upload succeeds, the system SHALL delete the local staging file for that part.

#### Scenario: Part upload frees local disk
- **GIVEN** an `archive_v1` filesystem backup run produces multiple part files
- **WHEN** `payload.part000001` is uploaded successfully to the WebDAV destination
- **THEN** the local staging file `payload.part000001` is deleted

### Requirement: WebDAV Rolling Upload Preserves Atomic Completion
The system SHALL still upload `manifest.json` and `complete.json` only after all parts and `entries.jsonl.zst` are uploaded successfully.

#### Scenario: Completion marker remains last with rolling upload
- **GIVEN** an `archive_v1` run is uploading parts incrementally
- **WHEN** the last part and `entries.jsonl.zst` have been uploaded successfully
- **THEN** `manifest.json` is uploaded and then `complete.json` is uploaded

### Requirement: WebDAV Client Supports Directory Listing via PROPFIND
The WebDAV client helpers SHALL support listing a directory via PROPFIND (Depth: 1) and return normalized entries suitable for picker UIs.

#### Scenario: List direct children
- **WHEN** a WebDAV directory is listed
- **THEN** the client returns direct child entries with name, kind (dir/file), and best-effort size/mtime

### Requirement: WebDAV Targets Are Node-Scoped
WebDAV targets and their referenced credentials SHALL be node-scoped.

#### Scenario: WebDAV credential belongs to a node
- **WHEN** a WebDAV target is created for a node
- **THEN** its credential material is stored and referenced within that node scope only

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

