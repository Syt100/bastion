## ADDED Requirements

### Requirement: Job Editor Exposes Consistency Policy Controls
The Web UI job editor SHALL expose consistency policy controls for supported sources.

#### Scenario: User can set fail policy
- **WHEN** a user edits a filesystem job
- **THEN** the UI allows selecting `warn|fail|ignore`
- **AND** configuring a threshold and upload behavior when applicable

