## ADDED Requirements

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
