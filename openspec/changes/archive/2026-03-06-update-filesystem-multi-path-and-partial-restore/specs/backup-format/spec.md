## MODIFIED Requirements

### Requirement: Per-File Content Hash Index
The system SHALL generate an entries index that includes a content hash for each regular file, and the recorded `path` SHALL be the archive path used inside the tar payload.

#### Scenario: Entries index paths match tar paths
- **WHEN** a file is archived at a given tar path
- **THEN** the entries index records the same archive path for that file

