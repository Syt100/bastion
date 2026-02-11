## ADDED Requirements

### Requirement: Target Drivers MUST Implement a Unified Lifecycle
Each target driver MUST implement a single lifecycle contract that supports backup writes, restore
reads, incomplete-run cleanup, and redacted snapshot serialization.

#### Scenario: Backup path uses lifecycle writer API
- **WHEN** backup runtime stores artifacts to a target driver
- **THEN** it uses target lifecycle writer methods (`open_writer`, upload operations, `finalize`)
- **AND** uses `abort` on failed runs

#### Scenario: Restore path uses lifecycle reader API
- **WHEN** restore or artifact stream accesses run data from a target
- **THEN** it uses target lifecycle reader methods (`open_reader` and typed artifact reads)

### Requirement: Incomplete Cleanup MUST Delegate to Target Driver
Incomplete-run cleanup MUST call target driver cleanup logic instead of hard-coded target-kind
branches in scheduler code.

#### Scenario: Cleanup uses target lifecycle cleanup API
- **WHEN** an incomplete cleanup task is processed
- **THEN** scheduler invokes `cleanup_run` on the resolved target driver
- **AND** retry classification is driven by standardized driver error categories

### Requirement: Run Target Snapshot MUST Be Driver-Owned and Redacted
The stored run target snapshot MUST be produced by target drivers and MUST contain only redacted,
non-secret fields.

#### Scenario: Persist redacted snapshot from driver
- **WHEN** run artifacts metadata is upserted after a successful run
- **THEN** snapshot payload is emitted by `snapshot_redacted`
- **AND** snapshot does not include credential material
