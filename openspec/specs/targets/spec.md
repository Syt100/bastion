# targets Specification

## Purpose
TBD - created by archiving change refactor-restore-target-reader-final-decoupling. Update Purpose after archive.
## Requirements
### Requirement: Target Drivers MUST Own Reader Construction
Target drivers MUST implement reader construction (`open_reader`) and provide `TargetRunReader`
behavior for run-relative artifact access.

#### Scenario: Registry delegates open_reader to target driver
- **WHEN** runtime opens a target reader for a run
- **THEN** registry resolves the driver and calls the driver `open_reader`
- **AND** registry does not branch on concrete target kinds to construct readers

### Requirement: Runtime Target Config Mapping MUST Be Shared
Driver input mapping (`DriverId + config`) MUST be built through shared helper functions for job
spec targets and agent-resolved targets.

#### Scenario: Hub and Agent use same mapping helpers
- **WHEN** hub/agent code needs target runtime config
- **THEN** it calls shared resolver helpers
- **AND** webdav credentials/secret metadata validation is enforced consistently

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

