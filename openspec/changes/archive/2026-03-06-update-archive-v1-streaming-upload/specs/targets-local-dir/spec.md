## ADDED Requirements

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
