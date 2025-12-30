## ADDED Requirements

### Requirement: Filesystem Source with Filters
The system SHALL support filesystem backups with include/exclude filtering.

#### Scenario: Excluded paths are skipped
- **WHEN** a path matches an exclude rule
- **THEN** it is not included in the backup entries index

### Requirement: SQLite Online Backup (No Downtime)
The system SHALL support backing up SQLite databases without downtime using an online backup mechanism and SHALL produce a consistent snapshot file.

#### Scenario: SQLite backup while service is running
- **WHEN** the SQLite database is actively used
- **THEN** the system can still create a consistent snapshot without stopping the service

### Requirement: SQLite Integrity Check (Optional)
The system SHALL optionally run `PRAGMA integrity_check` against the backed-up SQLite snapshot and record results.

#### Scenario: Integrity check failure is recorded
- **WHEN** the snapshot fails integrity check
- **THEN** the run records a verification failure and triggers failure notifications

