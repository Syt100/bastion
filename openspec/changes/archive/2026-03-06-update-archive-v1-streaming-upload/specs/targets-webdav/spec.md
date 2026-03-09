## ADDED Requirements

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
