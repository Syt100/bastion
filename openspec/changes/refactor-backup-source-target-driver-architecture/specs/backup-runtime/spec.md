## ADDED Requirements

### Requirement: Backup Runtime MUST Resolve Drivers from a Registry
Backup execution MUST resolve source and target implementations from a driver registry by
`driver_id` and `version`, rather than hard-coded type branches.

#### Scenario: Driver registry resolves execution components
- **WHEN** a run starts with canonical source/target driver identifiers
- **THEN** runtime obtains both drivers from the registry
- **AND** runtime fails with an `unsupported_driver` error if either driver is not available

### Requirement: Runtime MUST Use Capability-Based Execution Planning
Backup execution mode MUST be chosen by a planner using source capabilities, target capabilities,
and pipeline requirements.

#### Scenario: Planner selects direct strategy when both drivers support it
- **WHEN** source and target drivers advertise direct-upload-compatible capabilities
- **AND** pipeline requirements allow direct mode
- **THEN** planner selects a direct execution mode
- **AND** run events include structured planner decision fields

#### Scenario: Planner falls back to staged strategy
- **WHEN** required direct-upload capabilities are not jointly available
- **THEN** planner selects a staged fallback strategy
- **AND** run events include a machine-readable fallback reason

### Requirement: Planner Decisions MUST Be Deterministic
Planner output MUST be deterministic for the same inputs to ensure reproducibility and debuggable
operations.

#### Scenario: Same inputs produce same plan
- **WHEN** planner is invoked repeatedly with identical spec, capabilities, and policy inputs
- **THEN** generated execution plans are identical
