## MODIFIED Requirements

### Requirement: Best-Effort Source Consistency Detection (Filesystem)
When building a filesystem backup, the system SHALL perform a best-effort detection of source changes during packaging and SHALL record "consistency warnings" when a file changes while being read.

Consistency warnings MUST be best-effort and SHALL NOT be treated as a hard correctness guarantee.

#### Scenario: File changes during packaging is detected
- **WHEN** a filesystem backup reads a file that changes while being packaged
- **THEN** the run records a consistency warning for that archive path
- **AND** the run remains eligible to succeed (subject to existing error policy)

### Requirement: Best-Effort Source Consistency Detection (Raw-Tree)
When building a filesystem backup in `raw_tree_v1` mode, the system SHALL perform best-effort change detection around the copy+hash operation and SHALL record consistency warnings when the source changes during the copy.

#### Scenario: Raw-tree file changes during copy is detected
- **WHEN** a raw-tree filesystem backup copies a file that changes while being copied
- **THEN** the run records a consistency warning for that archive path

### Requirement: Best-Effort Source Consistency Detection (Vaultwarden)
When building a Vaultwarden backup, the system SHALL perform best-effort change detection for files included from the Vaultwarden data directory (excluding the live SQLite db file which is replaced by an online snapshot) and SHALL record consistency warnings when a file changes while being packaged.

#### Scenario: Vaultwarden data file changes during packaging is detected
- **WHEN** a Vaultwarden backup packages a file that changes while being read
- **THEN** the run records a consistency warning for that archive path

### Requirement: Archive Hashes Match Archived Bytes (Single-Read Hashing)
For `archive_v1` packaging, the system SHALL compute each file's recorded content hash from the same bytes written into the archive output.

#### Scenario: Hashes match archived bytes even when source changes
- **WHEN** a file changes during packaging
- **THEN** the recorded hash corresponds to the archived bytes for that entry
- **AND** a consistency warning is recorded for that entry

