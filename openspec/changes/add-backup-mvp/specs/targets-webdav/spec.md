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

