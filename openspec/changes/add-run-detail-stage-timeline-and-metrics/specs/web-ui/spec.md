## ADDED Requirements

### Requirement: Run Detail Shows Stage Timeline and Durations
The Run Detail page SHALL present a stage timeline for Scan / Build / Upload with durations.

#### Scenario: Completed run shows per-stage durations
- **GIVEN** a run has ended
- **THEN** the UI shows durations for Scan, Build, and Upload when those stages occurred
- **AND** the UI shows total duration

#### Scenario: Running run shows partial stage timing
- **GIVEN** a run is still running
- **THEN** the UI shows elapsed time for the current stage

### Requirement: Run Detail Preserves Transfer Metrics After Completion
The Run Detail page SHALL preserve meaningful transfer metrics after a run completes.

#### Scenario: Completed run shows final average transfer rate
- **GIVEN** a run has ended
- **THEN** the UI shows a final transfer rate value instead of replacing it with a placeholder

### Requirement: Run Detail Indicates Failure Stage
The Run Detail page SHALL indicate the stage in which a run failed when the information can be determined.

#### Scenario: Failed run shows failure stage
- **GIVEN** a run ends with status `failed` or `rejected`
- **WHEN** the UI can determine the last active stage
- **THEN** the UI displays that stage as the failure stage
