# backup-format Specification

## Purpose
TBD - created by archiving change update-archive-v1-streaming-upload. Update Purpose after archive.
## Requirements
### Requirement: Archive Parts Can Be Stored Incrementally
For `archive_v1`, the system SHALL support storing each finalized `payload.part*` to the configured target as soon as it is finalized, without waiting for packaging to finish.

After a part is stored successfully, the system SHALL remove the local staging file for that part.

#### Scenario: A finalized part is stored and deleted before packaging completes
- **GIVEN** an `archive_v1` filesystem backup run produces multiple `payload.part*` files
- **WHEN** the run finalizes `payload.part000001`
- **THEN** the system stores `payload.part000001` to the target
- **AND** the local staging file for `payload.part000001` is removed even if later parts are still being packaged

### Requirement: Rolling Part Storage Applies Backpressure
When storing finalized parts incrementally, the system SHALL apply backpressure so completed parts do not accumulate unboundedly on local disk.

#### Scenario: Target is slower than packaging
- **GIVEN** an `archive_v1` filesystem backup run is packaging data faster than the target can accept parts
- **WHEN** the internal queue of finalized parts is full
- **THEN** packaging waits before finalizing additional parts

### Requirement: Packaging Pipeline
The system SHALL package backups as tar (PAX) compressed with zstd, and SHALL support optional age encryption, producing a byte stream that can be split into parts.

#### Scenario: Create a tar+zstd artifact
- **WHEN** a run executes with compression enabled and encryption disabled
- **THEN** the system produces a tar(PAX)+zstd artifact stream

#### Scenario: Create a tar+zstd+age artifact
- **WHEN** a run executes with compression enabled and age encryption enabled
- **THEN** the system produces a tar(PAX)+zstd+age artifact stream

### Requirement: Default Compression Settings
The system SHALL default zstd compression level to 3 and SHALL default compression threads to automatic (based on available CPU).

#### Scenario: Defaults are applied
- **WHEN** a user does not specify zstd settings
- **THEN** the system uses level 3 and auto threads

### Requirement: Split Parts and Atomic Completion
The system SHALL split artifacts into configurable part sizes and SHALL only consider a backup complete when `complete.json` exists alongside `manifest.json`.

#### Scenario: Incomplete run is excluded
- **WHEN** parts are present but `complete.json` is missing
- **THEN** the run is not selectable as a restore point

### Requirement: Manifest v1
The system SHALL write a versioned `manifest.json` that includes pipeline settings, a list of artifact parts with sizes and hashes, and a reference to an entries index.

#### Scenario: Manifest is generated
- **WHEN** a run completes successfully
- **THEN** a `manifest.json` with `format_version=1` is written and uploaded

### Requirement: Encryption Key Reference in Manifest
When encryption is enabled, the system SHALL record which encryption key name was used in `manifest.json` so restores can locate the correct key.

#### Scenario: Age encryption key name is recorded
- **WHEN** a run executes with age encryption enabled using key name `K`
- **THEN** `manifest.pipeline.encryption=age` and `manifest.pipeline.encryption_key=K`

#### Scenario: No encryption key name when disabled
- **WHEN** a run executes with encryption disabled
- **THEN** `manifest.pipeline.encryption=none` and `manifest.pipeline.encryption_key` is omitted

### Requirement: Per-File Content Hash Index
The system SHALL generate an entries index that includes a content hash for each regular file and SHALL use it to support restore drill verification.

#### Scenario: Restore drill verifies file hashes
- **WHEN** a restore drill is executed
- **THEN** restored files are hashed and compared against the entries index

### Requirement: Artifact Format Is Recorded in the Manifest
The system SHALL record the artifact format used for a run in `manifest.json` so restore/verify can select the correct implementation.

Supported formats SHALL include:
- `archive_v1` (existing tar-based format)
- `raw_tree_v1` (raw file tree + metadata index)

#### Scenario: Restore selects implementation by manifest format
- **GIVEN** a completed run has `manifest.pipeline.format = raw_tree_v1`
- **WHEN** restore is started for that run
- **THEN** the system restores by reading `data/<path>` entries rather than attempting tar extraction

### Requirement: Raw Tree Stores Content Under a Dedicated Data Prefix
For `raw_tree_v1`, the system SHALL store file content under `data/<relative_path>` and SHALL keep run metadata files (`manifest.json`, `entries.jsonl.zst`, `complete.json`) at the run root.

#### Scenario: User file names do not collide with run metadata files
- **WHEN** the source contains a file named `manifest.json`
- **THEN** the raw-tree content is stored as `data/.../manifest.json`
- **AND** the run metadata file `manifest.json` remains at the run root

### Requirement: Entries Index Includes Best-Effort Metadata
For `raw_tree_v1`, the entries index SHALL include best-effort filesystem metadata (mtime, permissions, ownership, xattrs, symlink targets, and hardlink grouping) in addition to existing fields.

#### Scenario: Metadata is preserved for filesystem restore
- **WHEN** a raw-tree run is restored to a filesystem destination
- **THEN** the system applies recorded metadata when supported by the destination platform

