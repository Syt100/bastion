## ADDED Requirements

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
