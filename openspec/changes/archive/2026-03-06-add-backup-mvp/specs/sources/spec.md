## ADDED Requirements

### Requirement: Filesystem Source with Filters
The system SHALL support filesystem backups with include/exclude filtering.

#### Scenario: Excluded paths are skipped
- **WHEN** a path matches an exclude rule
- **THEN** it is not included in the backup entries index

### Requirement: Filesystem Semantics Policies
For filesystem backups, the system SHALL allow configuring symlink handling, hardlink handling, and per-file error handling.

#### Scenario: Symlink policy keep
- **WHEN** a filesystem backup is configured with `symlink_policy=keep`
- **THEN** symlinks are stored as symlinks in the tar archive

#### Scenario: Symlink policy follow
- **WHEN** a filesystem backup is configured with `symlink_policy=follow`
- **THEN** symlinks are dereferenced and stored as regular files/directories in the tar archive

#### Scenario: Symlink policy skip
- **WHEN** a filesystem backup is configured with `symlink_policy=skip`
- **THEN** symlink entries are skipped and recorded as issues

#### Scenario: Hardlink policy keep
- **WHEN** a filesystem backup is configured with `hardlink_policy=keep`
- **THEN** hardlinks are preserved in the tar archive when supported by the platform

#### Scenario: Hardlink policy copy
- **WHEN** a filesystem backup is configured with `hardlink_policy=copy`
- **THEN** hardlinks are stored as independent regular files in the tar archive

#### Scenario: Error policy fail_fast
- **WHEN** a filesystem backup encounters a per-file error and `error_policy=fail_fast`
- **THEN** the run fails immediately

#### Scenario: Error policy skip_fail
- **WHEN** a filesystem backup encounters per-file errors and `error_policy=skip_fail`
- **THEN** the run completes the archive but is marked failed, with issues recorded

#### Scenario: Error policy skip_ok
- **WHEN** a filesystem backup encounters per-file errors and `error_policy=skip_ok`
- **THEN** the run completes successfully, with issues recorded

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
