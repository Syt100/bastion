## ADDED Requirements

### Requirement: Restore Runtime MUST Use Target Driver Readers
Restore runtime MUST obtain run artifacts through target driver reader contracts instead of direct
target-type branches.

#### Scenario: Restore resolves reader from target driver
- **WHEN** a restore operation starts for a successful run
- **THEN** runtime resolves the run target driver and opens a target reader
- **AND** manifest/index/payload access all flow through that reader contract

### Requirement: Artifact Streaming MUST Share Restore Reader Contracts
Hub artifact stream APIs MUST reuse the same target reader contract as restore runtime to avoid
inconsistent behavior.

#### Scenario: Artifact stream and restore read the same way
- **WHEN** a client opens artifact stream for a run
- **THEN** stream path uses the same target reader contract used by restore
- **AND** validation rules for complete markers and missing artifacts remain consistent
